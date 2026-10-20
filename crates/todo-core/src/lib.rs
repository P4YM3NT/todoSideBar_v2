//! Datenkern für todoSideBar.
//!
//! Dieses Crate hängt bewusst **nicht** von Tauri ab: die Sidebar und die
//! spätere Voll-App teilen es sich unverändert.

pub mod db;
pub mod calendar;
pub mod error;
pub mod models;
pub mod notifications;
pub mod repo;
pub mod settings;

pub use error::{CoreError, Result};

/// Re-Export, damit Anwender nicht selbst eine - womöglich abweichende -
/// rusqlite-Version einbinden müssen.
pub use rusqlite;
