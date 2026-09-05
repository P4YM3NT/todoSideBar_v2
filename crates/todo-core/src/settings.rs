//! Einstellungen als Key-Value-Paare in der `settings`-Tabelle.
//!
//! Bewusst im gemeinsamen Kern und nicht in einer App-eigenen Datei: die
//! spätere Voll-App soll dieselben Einstellungen sehen.

use crate::error::Result;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};

pub fn get_raw(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}

pub fn set_raw(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Liest einen Wert; bei fehlendem *oder* unlesbarem Eintrag kommt `default`
/// zurück. Ein kaputter Eintrag darf die App nicht am Starten hindern.
pub fn get_or<T: DeserializeOwned>(conn: &Connection, key: &str, default: T) -> T {
    match get_raw(conn, key) {
        Ok(Some(raw)) => serde_json::from_str(&raw).unwrap_or(default),
        _ => default,
    }
}

pub fn set<T: Serialize>(conn: &Connection, key: &str, value: &T) -> Result<()> {
    let raw = serde_json::to_string(value)
        .map_err(|e| crate::CoreError::Invalid(format!("settings {key}: {e}")))?;
    set_raw(conn, key, &raw)
}
