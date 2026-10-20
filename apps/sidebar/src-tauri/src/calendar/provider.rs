//! Contract between the sync engine and calendar backends. The engine decides
//! what to write; providers only translate references and event data.
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use todo_core::calendar::{CalendarEvent, EventData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    /// macOS EventKit: iCloud, Google and other accounts configured in macOS.
    Apple,
    /// Google Calendar API with the app's own OAuth connection.
    Google,
}

impl ProviderKind {
    pub fn of(calendar_id: &str) -> Self {
        // EventKit identifiers predate providers and stay unprefixed so
        // existing links remain valid.
        if calendar_id.starts_with(crate::calendar::google::ID_PREFIX) {
            Self::Google
        } else {
            Self::Apple
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarInfo {
    pub id: String,
    pub title: String,
    pub account: String,
    pub writable: bool,
    #[serde(default = "apple")]
    pub provider: ProviderKind,
}

fn apple() -> ProviderKind {
    ProviderKind::Apple
}

/// Everything a provider may use to locate one task's event in one calendar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRef {
    pub task_id: String,
    pub calendar_id: String,
    pub event_id: Option<String>,
    pub external_id: Option<String>,
    /// Days around which a provider without marker queries searches for an
    /// event whose identifiers were never recorded.
    pub search_dates: BTreeSet<NaiveDate>,
}

/// A save or delete. `expected_modified_at` is the revision seen during the
/// read pass; `None` means no event existed and none may have appeared since.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Write {
    #[serde(flatten)]
    pub reference: EventRef,
    pub expected_modified_at: Option<f64>,
    pub data: Option<EventData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Remote {
    pub task_id: String,
    pub calendar_id: String,
    pub event: Option<CalendarEvent>,
    /// Per-reference read failure. Never interpreted as a deletion.
    #[serde(default)]
    pub error: Option<String>,
}

pub trait CalendarProvider {
    fn calendars(&self) -> Result<Vec<CalendarInfo>, String>;
    /// Must return exactly one entry per reference. Fails as a whole if any
    /// selected calendar is unavailable, so missing events are trustworthy.
    fn read(&self, calendar_ids: &[String], refs: &[EventRef]) -> Result<Vec<Remote>, String>;
    fn save(&self, write: &Write) -> Result<CalendarEvent, String>;
    fn delete(&self, write: &Write) -> Result<(), String>;
}

/// Routes every call to the provider owning the calendar identifier.
pub struct Providers<'a> {
    pub apple: &'a dyn CalendarProvider,
    pub google: &'a dyn CalendarProvider,
}

impl Providers<'_> {
    pub fn get(&self, kind: ProviderKind) -> &dyn CalendarProvider {
        match kind {
            ProviderKind::Apple => self.apple,
            ProviderKind::Google => self.google,
        }
    }

    pub fn for_calendar(&self, calendar_id: &str) -> &dyn CalendarProvider {
        self.get(ProviderKind::of(calendar_id))
    }

    pub fn read(&self, calendar_ids: &[String], refs: &[EventRef]) -> Result<Vec<Remote>, String> {
        let mut remote = Vec::with_capacity(refs.len());
        for kind in [ProviderKind::Apple, ProviderKind::Google] {
            let ids: Vec<String> = calendar_ids
                .iter()
                .filter(|id| ProviderKind::of(id) == kind)
                .cloned()
                .collect();
            if ids.is_empty() {
                continue;
            }
            let refs: Vec<EventRef> = refs
                .iter()
                .filter(|r| ProviderKind::of(&r.calendar_id) == kind)
                .cloned()
                .collect();
            remote.extend(self.get(kind).read(&ids, &refs)?);
        }
        Ok(remote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_by_identifier_prefix() {
        assert_eq!(ProviderKind::of("6A1F-EVENTKIT-ID"), ProviderKind::Apple);
        assert_eq!(ProviderKind::of("google:primary"), ProviderKind::Google);
    }

    #[test]
    fn legacy_calendar_info_defaults_to_apple() {
        let info: CalendarInfo = serde_json::from_str(
            r#"{"id":"x","title":"Privat","account":"iCloud","writable":true}"#,
        )
        .unwrap();
        assert_eq!(info.provider, ProviderKind::Apple);
    }
}
