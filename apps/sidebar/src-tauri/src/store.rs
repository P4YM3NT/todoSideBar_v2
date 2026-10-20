//! Datenbankzugriff der App. Öffnet die gemeinsame Datei, die sich die spätere
//! Voll-App mit der Sidebar teilen soll.

use std::sync::Mutex;
use todo_core::rusqlite::Connection;
use tauri::{AppHandle, Manager, Runtime};

pub struct Db(pub Mutex<Connection>);

pub const KEY_WIDTH: &str = "sidebar.width";
pub const KEY_HANDLE: &str = "sidebar.handleVisible";
pub const KEY_HOTKEY: &str = "hotkey.toggle";
pub const KEY_HOTKEY_ADD: &str = "hotkey.add";
pub const KEY_CHECKBOX_COLOR: &str = "appearance.checkboxColor";
pub const KEY_THEME: &str = "appearance.theme";
pub const KEY_BG_ALPHA: &str = "appearance.backgroundAlpha";
pub const KEY_BG_COLOR: &str = "appearance.backgroundColor";
pub const KEY_PRIORITIES: &str = "tasks.priorities";
pub const KEY_FILTER_PRESETS: &str = "tasks.filterPresets";
pub const KEY_SHOW_SUBTASKS: &str = "tasks.showSubtasks";

/// Deckkraft der Sidebar-Flaeche in Prozent.
pub const DEFAULT_BG_ALPHA: f64 = 95.0;

/// "light" | "dark" | "system"
pub const DEFAULT_THEME: &str = "light";

pub const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+T";

/// Oeffnet die Sidebar und setzt den Cursor gleich ins Eingabefeld.
pub const DEFAULT_HOTKEY_ADD: &str = "CmdOrCtrl+Shift+N";

pub fn open<R: Runtime>(app: &AppHandle<R>) -> Result<Db, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    let conn = todo_core::db::open(&dir.join("todo.db"))?;
    Ok(Db(Mutex::new(conn)))
}

/// Kleiner Helfer, damit Aufrufer nicht überall das Mutex entsperren müssen.
pub fn with<R: Runtime, T>(
    app: &AppHandle<R>,
    f: impl FnOnce(&Connection) -> T,
) -> T {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    f(&conn)
}
