//! Native Popups und SMTP laufen im Backend, unabhängig vom sichtbaren Fenster.

use crate::store;
use chrono::{Local, NaiveTime, Utc};
use lettre::{
    Message, SmtpTransport, Transport, message::Mailbox,
    transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_notification::{NotificationExt, PermissionState};
use todo_core::notifications::{self as reminders, Channel, Reminder};

const KEY_SETTINGS: &str = "notifications.settings";
const KEYRING_SERVICE: &str = "de.ptr-digital.todosidebar.smtp";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Security {
    StartTls,
    Tls,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub security: Security,
    pub username: String,
    pub from: String,
    pub to: String,
    /// Nur Metadaten. Das Passwort wird nie in SQLite oder an die UI geschrieben.
    #[serde(default)]
    pub has_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationSettings {
    pub popup_enabled: bool,
    pub email_enabled: bool,
    pub reminder_time: String,
    pub email: Option<EmailConfig>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            popup_enabled: false,
            email_enabled: false,
            reminder_time: "09:00".into(),
            email: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatus {
    pub popup_error: Option<String>,
    pub email_error: Option<String>,
    pub last_email_sent_at: Option<String>,
}

#[derive(Default)]
pub struct NotificationState {
    // Serialisiert Versenden und Änderungen, damit kein alter Entwurf nach dem
    // Ausschalten eines Kanals oder Ändern des Empfängers versendet wird.
    gate: Mutex<()>,
    status: Mutex<DeliveryStatus>,
    email_retry_after: Mutex<Option<Instant>>,
}

pub fn load(conn: &todo_core::rusqlite::Connection) -> NotificationSettings {
    todo_core::settings::get_or(conn, KEY_SETTINGS, NotificationSettings::default())
}

fn save(app: &AppHandle<Wry>, settings: &NotificationSettings) -> Result<(), String> {
    store::with(app, |c| todo_core::settings::set(c, KEY_SETTINGS, settings))
        .map_err(|e| e.to_string())
}

fn parse_time(value: &str) -> Result<NaiveTime, String> {
    if value.len() != 5
        || value.as_bytes()[2] != b':'
        || !value
            .bytes()
            .enumerate()
            .all(|(i, c)| i == 2 || c.is_ascii_digit())
    {
        return Err("Uhrzeit muss im Format HH:MM angegeben werden.".into());
    }
    NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|_| "Gültige Uhrzeit zwischen 00:00 und 23:59 eingeben.".into())
}

fn validate_email(config: &mut EmailConfig) -> Result<(), String> {
    config.host = config.host.trim().to_lowercase();
    config.username = config.username.trim().into();
    config.from = config.from.trim().into();
    config.to = config.to.trim().into();
    if config.host.is_empty()
        || config
            .host
            .chars()
            .any(|c| c.is_whitespace() || "/@?#\\".contains(c))
    {
        return Err("SMTP-Host ohne Protokoll oder Pfad eingeben, z. B. smtp.example.com.".into());
    }
    if config.port == 0 {
        return Err("SMTP-Port muss zwischen 1 und 65535 liegen.".into());
    }
    if config.username.is_empty() || config.username.chars().any(char::is_control) {
        return Err("SMTP-Benutzername eingeben.".into());
    }
    for (label, address) in [("Absender", &config.from), ("Empfänger", &config.to)] {
        if address.contains(['\r', '\n']) || address.parse::<Mailbox>().is_err() {
            return Err(format!("Gültige E-Mail-Adresse für {label} eingeben."));
        }
    }
    Ok(())
}

fn credential_id(config: &EmailConfig) -> String {
    // JSON statt Trennzeichen: auch Benutzernamen mit @ bleiben eindeutig.
    serde_json::to_string(&(&config.host, config.port, &config.username)).unwrap()
}

fn entry(config: &EmailConfig) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, &credential_id(config))
        .map_err(|_| "System-Schlüsselbund ist nicht verfügbar.".into())
}

fn password(config: &EmailConfig) -> Result<String, String> {
    entry(config)?.get_password().map_err(|_| {
        "SMTP-Passwort im System-Schlüsselbund fehlt oder ist gesperrt. Bitte erneut speichern."
            .into()
    })
}

fn allow_popup(app: &AppHandle<Wry>) -> Result<(), String> {
    let permission = app
        .notification()
        .request_permission()
        .map_err(|e| e.to_string())?;
    if permission != PermissionState::Granted {
        return Err("Benachrichtigungen bitte in den Systemeinstellungen erlauben.".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn set_notification_channel(
    app: AppHandle<Wry>,
    channel: Channel,
    enabled: bool,
) -> Result<NotificationSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<NotificationState>();
        let _gate = state.gate.lock().unwrap();
        let mut settings = store::with(&app, load);
        match channel {
            Channel::Popup => {
                if enabled {
                    allow_popup(&app)?;
                }
                settings.popup_enabled = enabled;
            }
            Channel::Email => {
                if enabled {
                    let mut config = settings
                        .email
                        .clone()
                        .ok_or("Bitte zuerst E-Mail einrichten.")?;
                    validate_email(&mut config)?;
                    password(&config)?;
                }
                settings.email_enabled = enabled;
            }
        }
        save(&app, &settings)?;
        set_error(&app, channel, None);
        if channel == Channel::Email {
            *state.email_retry_after.lock().unwrap() = None;
        }
        Ok(settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_notification_time(
    app: AppHandle<Wry>,
    time: String,
) -> Result<NotificationSettings, String> {
    parse_time(&time)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<NotificationState>();
        let _gate = state.gate.lock().unwrap();
        let mut settings = store::with(&app, load);
        settings.reminder_time = time;
        save(&app, &settings)?;
        Ok(settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_email_config(
    app: AppHandle<Wry>,
    mut config: EmailConfig,
    password: Option<String>,
) -> Result<NotificationSettings, String> {
    validate_email(&mut config)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<NotificationState>();
        let _gate = state.gate.lock().unwrap();
        let mut settings = store::with(&app, load);
        let old_config = settings.email.clone();
        let credential = entry(&config)?;
        let old_password = match credential.get_password() {
            Ok(value) => Some(value),
            Err(keyring::Error::NoEntry) => None,
            Err(_) => return Err("System-Schlüsselbund ist gesperrt oder nicht verfügbar.".into()),
        };
        if let Some(value) = password.as_ref().filter(|v| !v.is_empty()) {
            credential
                .set_password(value)
                .map_err(|_| "Passwort konnte nicht im System-Schlüsselbund gespeichert werden.")?;
        } else if old_password.is_none() {
            return Err("SMTP-Passwort oder App-Passwort eingeben.".into());
        }
        config.has_password = true;
        settings.email = Some(config.clone());
        if let Err(error) = save(&app, &settings) {
            // Bei fehlgeschlagenem DB-Schreiben bleibt auch der alte Zugang erhalten.
            if let Some(value) = old_password {
                let _ = credential.set_password(&value);
            } else {
                let _ = credential.delete_credential();
            }
            return Err(error);
        }
        if let Some(old) = old_config.filter(|old| credential_id(old) != credential_id(&config)) {
            if let Ok(old_entry) = entry(&old) {
                let _ = old_entry.delete_credential();
            }
        }
        set_error(&app, Channel::Email, None);
        *state.email_retry_after.lock().unwrap() = None;
        Ok(settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn clear_email_config(app: AppHandle<Wry>) -> Result<NotificationSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<NotificationState>();
        let _gate = state.gate.lock().unwrap();
        let mut settings = store::with(&app, load);
        let mut old_password = None;
        if let Some(config) = &settings.email {
            old_password = entry(config)?.get_password().ok();
            match entry(config)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(_) => {
                    return Err(
                        "Passwort konnte nicht aus dem System-Schlüsselbund entfernt werden."
                            .into(),
                    );
                }
            }
        }
        settings.email_enabled = false;
        let old_config = settings.email.take();
        if let Err(error) = save(&app, &settings) {
            if let (Some(config), Some(value)) = (&old_config, old_password) {
                if let Ok(credential) = entry(config) {
                    let _ = credential.set_password(&value);
                }
            }
            return Err(error);
        }
        set_error(&app, Channel::Email, None);
        *state.email_retry_after.lock().unwrap() = None;
        state.status.lock().unwrap().last_email_sent_at = None;
        Ok(settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn notification_status(app: AppHandle<Wry>) -> DeliveryStatus {
    app.state::<NotificationState>()
        .status
        .lock()
        .unwrap()
        .clone()
}

fn set_error(app: &AppHandle<Wry>, channel: Channel, error: Option<String>) {
    let state = app.state::<NotificationState>();
    let mut status = state.status.lock().unwrap();
    match channel {
        Channel::Popup => status.popup_error = error,
        Channel::Email => status.email_error = error,
    }
}

fn show_popup(app: &AppHandle<Wry>, title: &str, body: &str) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|_| "Popup konnte nicht an das Betriebssystem übergeben werden.".into())
}

fn send_email(config: &EmailConfig, subject: &str, body: String) -> Result<(), String> {
    let mut config = config.clone();
    validate_email(&mut config)?;
    let message = Message::builder()
        .from(config.from.parse().map_err(|_| "Ungültiger Absender.")?)
        .to(config.to.parse().map_err(|_| "Ungültiger Empfänger.")?)
        .subject(subject)
        .body(body)
        .map_err(|_| "E-Mail konnte nicht erstellt werden.")?;
    let transport = match config.security {
        Security::StartTls => SmtpTransport::starttls_relay(&config.host),
        Security::Tls => SmtpTransport::relay(&config.host),
    }
    .map_err(|_| "TLS-Verbindung zum SMTP-Host konnte nicht vorbereitet werden.")?
    .port(config.port)
    .credentials(Credentials::new(
        config.username.clone(),
        password(&config)?,
    ))
    .timeout(Some(Duration::from_secs(15)))
    .build();
    transport.send(&message).map_err(|_| {
        "E-Mail konnte nicht versendet werden. SMTP-Zugang, Verschlüsselung und Internetverbindung prüfen.".to_string()
    })?;
    Ok(())
}

#[tauri::command]
pub async fn test_notification(app: AppHandle<Wry>, channel: Channel) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<NotificationState>();
        let _gate = state.gate.lock().unwrap();
        let settings = store::with(&app, load);
        let result = match channel {
            Channel::Popup => allow_popup(&app).and_then(|_| show_popup(&app, "todoSideBar · Test", "Native Benachrichtigungen sind eingerichtet.")),
            Channel::Email => settings.email.as_ref().ok_or("Bitte zuerst E-Mail-Einstellungen speichern.".into())
                .and_then(|config| send_email(config, "todoSideBar · Test-E-Mail", "Die E-Mail-Benachrichtigungen sind eingerichtet.\n\nFällige Aufgaben werden an diese Adresse gesendet, solange todoSideBar läuft.".into())),
        };
        set_error(&app, channel, result.as_ref().err().cloned());
        if channel == Channel::Email {
            *state.email_retry_after.lock().unwrap() = result.as_ref().err()
                .map(|_| Instant::now() + Duration::from_secs(300));
        }
        if result.is_ok() && channel == Channel::Email {
            state.status.lock().unwrap().last_email_sent_at = Some(Utc::now().to_rfc3339());
        }
        result
    }).await.map_err(|e| e.to_string())?
}

fn deliver(
    app: &AppHandle<Wry>,
    channel: Channel,
    config: &NotificationSettings,
    batch: &[Reminder],
) -> Result<(), String> {
    match channel {
        Channel::Popup => {
            let title = if batch.len() == 1 {
                "todoSideBar · Aufgabe fällig".into()
            } else {
                format!("todoSideBar · {} Aufgaben fällig", batch.len())
            };
            let body = batch
                .iter()
                .take(3)
                .map(|r| r.title.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            show_popup(app, &title, &body)
        }
        Channel::Email => {
            let email = config.email.as_ref().ok_or("Bitte E-Mail einrichten.")?;
            let lines = batch
                .iter()
                .map(|r| format!("• {} — fällig am {}", r.title, r.due_on.format("%d.%m.%Y")))
                .collect::<Vec<_>>()
                .join("\n");
            send_email(
                email,
                &format!(
                    "todoSideBar · {} fällige Aufgabe{}",
                    batch.len(),
                    if batch.len() == 1 { "" } else { "n" }
                ),
                format!(
                    "Diese Aufgaben sind noch offen:\n\n{lines}\n\nÖffne todoSideBar, um sie zu bearbeiten."
                ),
            )
        }
    }
}

fn check(app: &AppHandle<Wry>) {
    let state = app.state::<NotificationState>();
    let _gate = state.gate.lock().unwrap();
    let config = store::with(app, load);
    let time = match parse_time(&config.reminder_time) {
        Ok(time) => time,
        Err(error) => {
            set_error(app, Channel::Popup, Some(error.clone()));
            set_error(app, Channel::Email, Some(error));
            return;
        }
    };
    for (channel, enabled) in [
        (Channel::Popup, config.popup_enabled),
        (Channel::Email, config.email_enabled),
    ] {
        if !enabled {
            continue;
        }
        if channel == Channel::Email
            && state
                .email_retry_after
                .lock()
                .unwrap()
                .is_some_and(|retry| Instant::now() < retry)
        {
            continue;
        }
        // Datum und Zeit nach jedem eventuell langsamen SMTP-Versand neu lesen.
        let now = Local::now();
        let result = store::with(app, |c| {
            reminders::pending(c, channel, now.date_naive(), now.time(), time)
        })
        .map_err(|e| e.to_string())
        .and_then(|batch| {
            if batch.is_empty() {
                return Ok(());
            }
            deliver(app, channel, &config, &batch)?;
            if channel == Channel::Email {
                state.status.lock().unwrap().last_email_sent_at = Some(Utc::now().to_rfc3339());
            }
            store::with(app, |c| reminders::mark_delivered(c, channel, &batch))
                .map_err(|e| e.to_string())
        });
        if channel == Channel::Email {
            *state.email_retry_after.lock().unwrap() = result
                .as_ref()
                .err()
                .map(|_| Instant::now() + Duration::from_secs(300));
        }
        set_error(app, channel, result.err());
    }
}

pub fn start(app: AppHandle<Wry>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let handle = app.clone();
            if let Err(error) = tauri::async_runtime::spawn_blocking(move || check(&handle)).await {
                set_error(
                    &app,
                    Channel::Popup,
                    Some(format!(
                        "Erinnerungen konnten nicht geprüft werden: {error}"
                    )),
                );
            }
            // Ein Worker, keine parallelen Zustellungen; nach Fehlern erneut versuchen.
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> EmailConfig {
        EmailConfig {
            host: " SMTP.EXAMPLE.COM ".into(),
            port: 587,
            security: Security::StartTls,
            username: " user ".into(),
            from: "from@example.com".into(),
            to: "to@example.com".into(),
            has_password: false,
        }
    }

    #[test]
    fn validates_time_and_email_before_saving() {
        for valid in ["00:00", "09:00", "23:59"] {
            assert!(parse_time(valid).is_ok());
        }
        for invalid in ["9:00", " 9:00", "24:00", "12:60", "", "12:00:00"] {
            assert!(parse_time(invalid).is_err());
        }
        let mut email = config();
        validate_email(&mut email).unwrap();
        assert_eq!(email.host, "smtp.example.com");
        assert_eq!(email.username, "user");
        email.host = "https://smtp.example.com".into();
        assert!(validate_email(&mut email).is_err());
        email = config();
        email.to = "bad\r\nBcc: other@example.com".into();
        assert!(validate_email(&mut email).is_err());
        email = config();
        email.port = 0;
        assert!(validate_email(&mut email).is_err());
    }

    #[test]
    fn defaults_and_serialization_exclude_credentials() {
        let settings = NotificationSettings::default();
        assert!(!settings.popup_enabled && !settings.email_enabled);
        let json = serde_json::to_value(config()).unwrap();
        assert!(json.get("password").is_none());
        let old: NotificationSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(old.reminder_time, "09:00");
    }
}
