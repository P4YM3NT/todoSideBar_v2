//! Google secrets live in the OS keychain, never in SQLite or settings.
use serde::{Deserialize, Serialize};

const SERVICE: &str = "de.ptr-digital.todosidebar.google";
const USER: &str = "oauth";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub client_id: String,
    /// Google's "secret" for desktop clients is not confidential, but it is
    /// still kept beside the refresh token rather than in plain settings.
    pub client_secret: String,
    pub refresh_token: String,
}

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, USER).map_err(|e| format!("Schlüsselbund: {e}"))
}

pub fn load() -> Result<Option<Credentials>, String> {
    match entry()?.get_password() {
        Ok(value) => serde_json::from_str(&value)
            .map(Some)
            .map_err(|_| "Google-Zugangsdaten sind beschädigt. Google neu verbinden.".into()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Schlüsselbund: {e}")),
    }
}

pub fn save(credentials: &Credentials) -> Result<(), String> {
    let value = serde_json::to_string(credentials).map_err(|e| e.to_string())?;
    entry()?
        .set_password(&value)
        .map_err(|e| format!("Schlüsselbund: {e}"))
}

pub fn clear() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Schlüsselbund: {e}")),
    }
}
