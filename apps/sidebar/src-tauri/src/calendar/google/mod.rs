//! Google Calendar API v3 provider with the app's own OAuth connection.
//! Works on every platform and without macOS internet accounts.
pub mod credentials;
pub mod mapping;
pub mod oauth;

use super::provider::{CalendarInfo, CalendarProvider, EventRef, ProviderKind, Remote, Write};
use mapping::GoogleEvent;
use reqwest::{
    StatusCode, Url,
    blocking::{Client, RequestBuilder, Response},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use todo_core::calendar::CalendarEvent;

/// Namespaces Google calendar identifiers in settings and calendar links.
pub const ID_PREFIX: &str = "google:";
const API: &str = "https://www.googleapis.com/calendar/v3";

/// Non-secret connection details. Secrets: see `credentials`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoogleSettings {
    pub client_id: Option<String>,
    /// Primary calendar of the connected account, normally its e-mail address.
    /// `Some` means connected.
    pub account: Option<String>,
}

#[derive(Default)]
pub struct GoogleState {
    http: OnceLock<Client>,
    token: Mutex<Option<(String, Instant)>>,
}

impl GoogleState {
    pub fn http(&self) -> &Client {
        self.http.get_or_init(|| {
            Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(concat!("todoSideBar/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("HTTP client")
        })
    }

    fn cache(&self, token: &oauth::TokenResponse) {
        let expires = Instant::now() + Duration::from_secs(token.expires_in);
        *self.token.lock().unwrap() = Some((token.access_token.clone(), expires));
    }

    fn reset(&self) {
        *self.token.lock().unwrap() = None;
    }
}

/// Runs the browser login and stores the refresh token. Returns the account.
pub fn connect(
    state: &GoogleState,
    client_id: &str,
    client_secret: &str,
    open: impl FnOnce(&str) -> Result<(), String>,
) -> Result<String, String> {
    let token = oauth::authorize(state.http(), client_id, client_secret, open)?;
    credentials::save(&credentials::Credentials {
        client_id: client_id.into(),
        client_secret: client_secret.into(),
        refresh_token: token.refresh_token.clone().unwrap_or_default(),
    })?;
    state.cache(&token);
    let google = Google {
        state,
        account: None,
    };
    let primary: CalendarListEntry =
        google.get_json(api(&["users", "me", "calendarList", "primary"]), &[])?;
    Ok(primary.id)
}

pub fn disconnect(state: &GoogleState) -> Result<(), String> {
    if let Some(credentials) = credentials::load().ok().flatten() {
        oauth::revoke(state.http(), &credentials.refresh_token);
    }
    state.reset();
    credentials::clear()
}

pub struct Google<'a> {
    state: &'a GoogleState,
    account: Option<String>,
}

impl<'a> Google<'a> {
    pub fn new(state: &'a GoogleState, settings: &GoogleSettings) -> Self {
        Self {
            state,
            account: settings.account.clone(),
        }
    }

    fn access_token(&self) -> Result<String, String> {
        if let Some((token, expires)) = self.state.token.lock().unwrap().as_ref()
            && *expires > Instant::now() + Duration::from_secs(60)
        {
            return Ok(token.clone());
        }
        let credentials = credentials::load()?.ok_or(
            "Google Kalender ist nicht verbunden. Unter Kalendersync mit Google verbinden.",
        )?;
        let token = oauth::refresh(
            self.state.http(),
            &credentials.client_id,
            &credentials.client_secret,
            &credentials.refresh_token,
        )
        .map_err(|e| match e {
            oauth::RefreshError::Revoked => "Google-Zugriff wurde widerrufen oder ist abgelaufen. Unter Kalendersync Google trennen und neu verbinden.".to_owned(),
            oauth::RefreshError::Other(e) => e,
        })?;
        self.state.cache(&token);
        Ok(token.access_token)
    }

    /// Sends with a bearer token and retries once with a fresh token on 401.
    fn send(&self, build: impl Fn(&Client) -> RequestBuilder) -> Result<Response, String> {
        for attempt in 0..2 {
            let response = build(self.state.http())
                .bearer_auth(self.access_token()?)
                .send()
                .map_err(|e| format!("Google Kalender nicht erreichbar: {e}"))?;
            if response.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                self.state.reset();
                continue;
            }
            return Ok(response);
        }
        unreachable!()
    }

    fn get_json<T: DeserializeOwned>(&self, url: Url, query: &[(&str, &str)]) -> Result<T, String> {
        let response = self.send(|http| http.get(url.clone()).query(query))?;
        parse(ok(response)?)
    }

    fn list_events(
        &self,
        calendar: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<GoogleEvent>, String> {
        let mut events = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut query = query.to_vec();
            query.extend([("showDeleted", "false"), ("maxResults", "2500")]);
            if let Some(page) = page.as_deref() {
                query.push(("pageToken", page));
            }
            let list: Page<GoogleEvent> =
                self.get_json(api(&["calendars", calendar, "events"]), &query)?;
            events.extend(list.items);
            match list.next_page_token {
                Some(next) => page = Some(next),
                None => return Ok(events),
            }
        }
    }

    fn require_writable(&self, calendar: &str) -> Result<(), String> {
        let response =
            self.send(|http| http.get(api(&["users", "me", "calendarList", calendar])))?;
        let missing = "Ein ausgewählter Google-Kalender fehlt oder ist schreibgeschützt. Kalenderauswahl prüfen.";
        if response.status() == StatusCode::NOT_FOUND {
            return Err(missing.into());
        }
        let entry: CalendarListEntry = parse(ok(response)?)?;
        if !entry.writable() {
            return Err(missing.into());
        }
        Ok(())
    }

    /// The live event for a reference: by stored identifier, else by marker.
    fn current(&self, reference: &EventRef) -> Result<Option<GoogleEvent>, String> {
        let calendar = raw_id(&reference.calendar_id);
        if let Some(event_id) = reference.event_id.as_deref() {
            let response =
                self.send(|http| http.get(api(&["calendars", calendar, "events", event_id])))?;
            if !matches!(response.status(), StatusCode::NOT_FOUND | StatusCode::GONE) {
                let event: GoogleEvent = parse(ok(response)?)?;
                if !event.cancelled() {
                    return Ok(Some(event));
                }
            }
        }
        let marker = format!("{}={}", mapping::TASK_KEY, reference.task_id);
        Ok(self
            .list_events(calendar, &[("privateExtendedProperty", &marker)])?
            .into_iter()
            .next())
    }

    fn modify(
        &self,
        build: impl Fn(&Client) -> RequestBuilder,
    ) -> Result<Option<Response>, String> {
        let response = self.send(build)?;
        match response.status() {
            StatusCode::PRECONDITION_FAILED => Err(CHANGED.into()),
            StatusCode::NOT_FOUND | StatusCode::GONE => Ok(None),
            _ => ok(response).map(Some),
        }
    }
}

const CHANGED: &str = "Kalendertermin wurde während der Synchronisierung geändert. Beim nächsten Durchlauf erneut versuchen.";

/// Same optimistic concurrency rule as the EventKit bridge.
fn check_revision(event: Option<&GoogleEvent>, expected: Option<f64>) -> Result<(), String> {
    match (event, expected) {
        (Some(event), Some(expected)) if (event.modified_at() - expected).abs() <= 0.001 => Ok(()),
        (None, None) => Ok(()),
        (Some(_), None) => Err(
            "Kalendertermin ist inzwischen erschienen. Beim nächsten Durchlauf erneut prüfen."
                .into(),
        ),
        _ => Err(CHANGED.into()),
    }
}

impl CalendarProvider for Google<'_> {
    fn calendars(&self) -> Result<Vec<CalendarInfo>, String> {
        let account = match self.account.as_deref() {
            Some(account) => format!("Google · {account}"),
            None => return Ok(Vec::new()),
        };
        let mut calendars = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut query = vec![("maxResults", "250")];
            if let Some(page) = page.as_deref() {
                query.push(("pageToken", page));
            }
            let list: Page<CalendarListEntry> =
                self.get_json(api(&["users", "me", "calendarList"]), &query)?;
            calendars.extend(list.items.into_iter().map(|entry| CalendarInfo {
                id: format!("{ID_PREFIX}{}", entry.id),
                writable: entry.writable(),
                title: entry.summary_override.or(entry.summary).unwrap_or(entry.id),
                account: account.clone(),
                provider: ProviderKind::Google,
            }));
            match list.next_page_token {
                Some(next) => page = Some(next),
                None => return Ok(calendars),
            }
        }
    }

    fn read(&self, calendar_ids: &[String], refs: &[EventRef]) -> Result<Vec<Remote>, String> {
        // Resolve all selected calendars before interpreting any missing events.
        let mut by_calendar: HashMap<&str, Vec<GoogleEvent>> = HashMap::new();
        for id in calendar_ids {
            let calendar = raw_id(id);
            self.require_writable(calendar)?;
            let marker = format!("{}=1", mapping::MARKER_KEY);
            by_calendar.insert(
                id.as_str(),
                self.list_events(calendar, &[("privateExtendedProperty", &marker)])?,
            );
        }
        Ok(refs
            .iter()
            .map(|reference| {
                let events = by_calendar
                    .get(reference.calendar_id.as_str())
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let event = reference
                    .event_id
                    .as_deref()
                    .and_then(|id| events.iter().find(|e| e.id == id))
                    .or_else(|| {
                        events
                            .iter()
                            .find(|e| e.task_id() == Some(reference.task_id.as_str()))
                    });
                let (event, error) = match event.map(GoogleEvent::to_calendar_event) {
                    Some(Ok(event)) => (Some(event), None),
                    Some(Err(error)) => (None, Some(error)),
                    None => (None, None),
                };
                Remote {
                    task_id: reference.task_id.clone(),
                    calendar_id: reference.calendar_id.clone(),
                    event,
                    error,
                }
            })
            .collect())
    }

    fn save(&self, write: &Write) -> Result<CalendarEvent, String> {
        let data = write.data.as_ref().ok_or("Termindaten fehlen.")?;
        let reference = &write.reference;
        let calendar = raw_id(&reference.calendar_id);
        let current = self.current(reference)?;
        check_revision(current.as_ref(), write.expected_modified_at)?;
        let body = mapping::event_body(&reference.task_id, data);
        let response = match current {
            Some(event) => self
                .modify(|http| {
                    let request = http
                        .patch(api(&["calendars", calendar, "events", &event.id]))
                        .json(&body);
                    match event.etag.as_deref() {
                        Some(etag) => request.header("If-Match", etag),
                        None => request,
                    }
                })?
                .ok_or(CHANGED)?,
            None => ok(self.send(|http| {
                http.post(api(&["calendars", calendar, "events"]))
                    .json(&body)
            })?)?,
        };
        parse::<GoogleEvent>(response)?.to_calendar_event()
    }

    fn delete(&self, write: &Write) -> Result<(), String> {
        let calendar = raw_id(&write.reference.calendar_id);
        let Some(event) = self.current(&write.reference)? else {
            return Ok(());
        };
        check_revision(Some(&event), write.expected_modified_at)?;
        self.modify(|http| {
            let request = http.delete(api(&["calendars", calendar, "events", &event.id]));
            match event.etag.as_deref() {
                Some(etag) => request.header("If-Match", etag),
                None => request,
            }
        })
        .map(drop)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
    #[serde(default = "Vec::new")]
    items: Vec<T>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarListEntry {
    id: String,
    summary: Option<String>,
    summary_override: Option<String>,
    access_role: Option<String>,
}

impl CalendarListEntry {
    fn writable(&self) -> bool {
        matches!(self.access_role.as_deref(), Some("owner" | "writer"))
    }
}

fn raw_id(calendar_id: &str) -> &str {
    calendar_id.strip_prefix(ID_PREFIX).unwrap_or(calendar_id)
}

/// Builds an API URL with each segment percent-encoded (ids contain `@`, `#`).
fn api(segments: &[&str]) -> Url {
    let mut url = Url::parse(API).expect("API URL");
    url.path_segments_mut().expect("API URL").extend(segments);
    url
}

fn ok(response: Response) -> Result<Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }
    #[derive(Deserialize)]
    struct Body {
        error: Detail,
    }
    #[derive(Deserialize)]
    struct Detail {
        message: String,
    }
    let status = response.status();
    let message = response
        .json::<Body>()
        .map(|body| body.error.message)
        .unwrap_or_else(|_| status.to_string());
    Err(format!("Google Kalender: {message}"))
}

fn parse<T: DeserializeOwned>(response: Response) -> Result<T, String> {
    response
        .json()
        .map_err(|e| format!("Google Kalender: unerwartete Antwort ({e})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_calendar_ids_in_paths() {
        let url = api(&[
            "calendars",
            "abc#holiday@group.v.calendar.google.com",
            "events",
        ]);
        assert_eq!(
            url.as_str(),
            "https://www.googleapis.com/calendar/v3/calendars/abc%23holiday@group.v.calendar.google.com/events"
        );
        assert_eq!(raw_id("google:primary"), "primary");
    }

    #[test]
    fn revision_rules_match_eventkit() {
        let event: GoogleEvent = serde_json::from_value(serde_json::json!({
            "id": "e", "updated": "2026-10-20T08:00:00.000Z"
        }))
        .unwrap();
        let seen = event.modified_at();
        assert!(check_revision(Some(&event), Some(seen)).is_ok());
        assert!(check_revision(Some(&event), Some(seen - 1.0)).is_err());
        assert!(check_revision(Some(&event), None).is_err());
        assert!(check_revision(None, Some(seen)).is_err());
        assert!(check_revision(None, None).is_ok());
    }
}
