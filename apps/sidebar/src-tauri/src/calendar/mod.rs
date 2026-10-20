//! Opt-in task/calendar sync. Tauri glue only: commands, settings, status and
//! the background loop. Sync rules: `engine`; backends: `eventkit`, `google`.
mod engine;
mod eventkit;
pub mod google;
mod provider;

use crate::store;
use chrono::Utc;
use google::{Google, GoogleSettings, GoogleState};
use provider::{CalendarInfo, ProviderKind, Providers};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Mutex, mpsc},
    time::Duration,
};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_opener::OpenerExt;

const KEY: &str = "calendar.sync";
const KEY_GOOGLE: &str = "calendar.google";
const MAX_CALENDARS: usize = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CalendarSettings {
    pub enabled: bool,
    pub calendar_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarStatus {
    /// EventKit availability. Google works on every platform.
    pub supported: bool,
    pub permission: String,
    pub syncing: bool,
    pub last_synced_at: Option<String>,
    pub synced_tasks: usize,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub google: GoogleSettings,
}

impl Default for CalendarStatus {
    fn default() -> Self {
        Self {
            supported: eventkit::supported(),
            permission: "notDetermined".into(),
            syncing: false,
            last_synced_at: None,
            synced_tasks: 0,
            error: None,
            warnings: Vec::new(),
            google: GoogleSettings::default(),
        }
    }
}

#[derive(Default)]
pub struct CalendarState {
    /// Serializes sync passes and every EventKit call.
    gate: Mutex<()>,
    /// Only one browser login at a time; never held together with `gate`.
    connecting: Mutex<()>,
    status: Mutex<CalendarStatus>,
    wake: Mutex<Option<mpsc::SyncSender<()>>>,
    google: GoogleState,
}

pub fn load(conn: &todo_core::rusqlite::Connection) -> CalendarSettings {
    todo_core::settings::get_or(conn, KEY, CalendarSettings::default())
}

fn load_google(app: &AppHandle<Wry>) -> GoogleSettings {
    store::with(app, |c| {
        todo_core::settings::get_or(c, KEY_GOOGLE, GoogleSettings::default())
    })
}

fn save_google(app: &AppHandle<Wry>, settings: &GoogleSettings) -> Result<(), String> {
    store::with(app, |c| todo_core::settings::set(c, KEY_GOOGLE, settings))
        .map_err(|e| e.to_string())
}

/// Calls `work` with both providers wired to the current settings.
fn with_providers<T>(app: &AppHandle<Wry>, work: impl FnOnce(&Providers) -> T) -> T {
    let state = app.state::<CalendarState>();
    let google = Google::new(&state.google, &load_google(app));
    work(&Providers {
        apple: &eventkit::EventKit,
        google: &google,
    })
}

fn apple_ready(state: &CalendarState) -> bool {
    eventkit::supported() && state.status.lock().unwrap().permission == "authorized"
}

fn snapshot(app: &AppHandle<Wry>) -> CalendarStatus {
    let mut status = app.state::<CalendarState>().status.lock().unwrap().clone();
    status.google = load_google(app);
    status
}

async fn worker<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn calendar_status(app: AppHandle<Wry>) -> Result<CalendarStatus, String> {
    worker(move || {
        let state = app.state::<CalendarState>();
        if eventkit::supported()
            && let Ok(_gate) = state.gate.try_lock()
        {
            state.status.lock().unwrap().permission = eventkit::permission()?;
        }
        Ok(snapshot(&app))
    })
    .await
}

#[tauri::command]
pub async fn request_calendar_access(app: AppHandle<Wry>) -> Result<CalendarStatus, String> {
    worker(move || {
        let state = app.state::<CalendarState>();
        let _gate = state.gate.lock().unwrap();
        state.status.lock().unwrap().permission = eventkit::authorize()?;
        Ok(snapshot(&app))
    })
    .await
}

#[tauri::command]
pub async fn list_calendars(app: AppHandle<Wry>) -> Result<Vec<CalendarInfo>, String> {
    worker(move || {
        let state = app.state::<CalendarState>();
        let _gate = state.gate.lock().unwrap();
        let mut calendars = Vec::new();
        with_providers(&app, |providers| -> Result<(), String> {
            if apple_ready(&state) {
                calendars.extend(providers.apple.calendars()?);
            }
            calendars.extend(providers.google.calendars()?);
            Ok(())
        })?;
        calendars.sort_by(|left, right| {
            (&left.account, &left.title, &left.id).cmp(&(&right.account, &right.title, &right.id))
        });
        Ok(calendars)
    })
    .await
}

#[tauri::command]
pub async fn set_calendar_sync(
    app: AppHandle<Wry>,
    config: CalendarSettings,
) -> Result<CalendarSettings, String> {
    worker(move || {
        let state = app.state::<CalendarState>();
        let _gate = state.gate.lock().unwrap();
        let mut config = config;
        config.calendar_ids.sort();
        config.calendar_ids.dedup();
        if config.calendar_ids.len() > MAX_CALENDARS {
            return Err("Höchstens zehn Zielkalender auswählen.".into());
        }
        if config.enabled {
            if config.calendar_ids.is_empty() {
                return Err("Mindestens einen Zielkalender auswählen.".into());
            }
            with_providers(&app, |providers| -> Result<(), String> {
                for kind in [ProviderKind::Apple, ProviderKind::Google] {
                    let selected: Vec<_> = config
                        .calendar_ids
                        .iter()
                        .filter(|id| ProviderKind::of(id) == kind)
                        .collect();
                    if selected.is_empty() {
                        continue;
                    }
                    let calendars = providers.get(kind).calendars()?;
                    if selected
                        .iter()
                        .any(|id| !calendars.iter().any(|c| c.id == **id && c.writable))
                    {
                        return Err("Kalenderauswahl fehlt oder ist schreibgeschützt.".into());
                    }
                }
                Ok(())
            })?;
        }
        store::with(&app, |c| todo_core::settings::set(c, KEY, &config))
            .map_err(|e| e.to_string())?;
        wake(&app);
        Ok(config)
    })
    .await
}

#[tauri::command]
pub async fn sync_calendars_now(app: AppHandle<Wry>) -> Result<CalendarStatus, String> {
    worker(move || {
        if !store::with(&app, load).enabled {
            return Err("Kalendersync ist ausgeschaltet.".into());
        }
        run(&app)?;
        Ok(snapshot(&app))
    })
    .await
}

#[tauri::command]
pub async fn google_connect(
    app: AppHandle<Wry>,
    client_id: String,
    client_secret: String,
) -> Result<CalendarStatus, String> {
    worker(move || {
        let (client_id, client_secret) = (client_id.trim(), client_secret.trim());
        if !client_id.ends_with(".apps.googleusercontent.com") {
            return Err("Client-ID endet auf .apps.googleusercontent.com. Wert aus der Google Cloud Console kopieren.".into());
        }
        if client_secret.is_empty() {
            return Err("Clientschlüssel fehlt.".into());
        }
        let state = app.state::<CalendarState>();
        let Ok(_login) = state.connecting.try_lock() else {
            return Err("Google-Anmeldung läuft bereits im Browser.".into());
        };
        let mut settings = load_google(&app);
        settings.client_id = Some(client_id.into());
        save_google(&app, &settings)?;
        let account = google::connect(&state.google, client_id, client_secret, |url| {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| format!("Browser konnte nicht geöffnet werden: {e}"))
        })?;
        settings.account = Some(account);
        save_google(&app, &settings)?;
        wake(&app);
        Ok(snapshot(&app))
    })
    .await
}

#[tauri::command]
pub async fn google_disconnect(app: AppHandle<Wry>) -> Result<CalendarStatus, String> {
    worker(move || {
        let state = app.state::<CalendarState>();
        // Waits for a running pass so it never sees half-removed credentials.
        let _gate = state.gate.lock().unwrap();
        google::disconnect(&state.google)?;
        let mut settings = load_google(&app);
        settings.account = None;
        save_google(&app, &settings)?;
        Ok(snapshot(&app))
    })
    .await
}

fn run(app: &AppHandle<Wry>) -> Result<(), String> {
    let state = app.state::<CalendarState>();
    let _gate = state.gate.lock().unwrap();
    let config = store::with(app, load);
    if !config.enabled {
        return Ok(());
    }
    state.status.lock().unwrap().syncing = true;
    let result = (|| {
        if eventkit::supported() {
            state.status.lock().unwrap().permission = eventkit::permission()?;
        }
        with_providers(app, |providers| {
            engine::synchronize(app, providers, &config.calendar_ids)
        })
    })();
    let mut status = state.status.lock().unwrap();
    status.syncing = false;
    match result {
        Ok(outcome) => {
            status.last_synced_at = Some(Utc::now().to_rfc3339());
            status.synced_tasks = outcome.synced_tasks;
            status.error = None;
            status.warnings = outcome.warnings;
            Ok(())
        }
        Err(error) => {
            status.error = Some(error.clone());
            Err(error)
        }
    }
}

pub fn wake(app: &AppHandle<Wry>) {
    if let Some(sender) = app.state::<CalendarState>().wake.lock().unwrap().as_ref() {
        let _ = sender.try_send(());
    }
}

pub fn start(app: AppHandle<Wry>) {
    let (sender, receiver) = mpsc::sync_channel(1);
    *app.state::<CalendarState>().wake.lock().unwrap() = Some(sender);
    std::thread::spawn(move || {
        loop {
            let _ = run(&app);
            if matches!(
                receiver.recv_timeout(Duration::from_secs(60)),
                Err(mpsc::RecvTimeoutError::Disconnected)
            ) {
                break;
            }
        }
    });
}
