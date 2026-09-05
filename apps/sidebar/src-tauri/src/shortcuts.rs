//! Globaler Hotkey. Frei belegbar, deshalb wird bei jeder Änderung neu registriert.

use crate::{commands, store};
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Registriert beide globalen Hotkeys.
///
/// Erst alles abmelden, dann neu anmelden - sonst bleibt beim Umstellen der
/// alte Hotkey aktiv und es reagieren zwei Tastenkombinationen. Weil dabei
/// auch der jeweils andere kurz verschwindet, werden immer beide gesetzt.
pub fn apply(app: &AppHandle<Wry>, toggle: &str, add: &str) -> Result<(), String> {
    if toggle.eq_ignore_ascii_case(add) {
        return Err("Beide Hotkeys wären dieselbe Kombination.".into());
    }

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    gs.on_shortcut(toggle, move |app, _shortcut, event| {
        // Ohne diese Prüfung feuert der Hotkey zweimal (Pressed + Released).
        if event.state() == ShortcutState::Pressed {
            let _ = commands::toggle(&app.clone());
        }
    })
    .map_err(|e| failed(toggle, e))?;

    gs.on_shortcut(add, move |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = commands::quick_add(&app.clone());
        }
    })
    .map_err(|e| {
        // Der erste liess sich registrieren, der zweite nicht - sonst haenge
        // die App mit genau einem aktiven Hotkey da, ohne dass es jemand merkt.
        let _ = gs.unregister_all();
        failed(add, e)
    })
}

fn failed(accelerator: &str, e: impl std::fmt::Display) -> String {
    format!("Hotkey '{accelerator}' konnte nicht registriert werden: {e}")
}

/// Beim Start die gespeicherten Hotkeys setzen; fällt auf die Standards
/// zurück, wenn sich die gespeicherten nicht (mehr) registrieren lassen -
/// z. B. weil sie inzwischen eine andere App belegt.
pub fn apply_saved(app: &AppHandle<Wry>) {
    let (toggle, add) = store::with(app, |c| {
        (
            todo_core::settings::get_or(c, store::KEY_HOTKEY, store::DEFAULT_HOTKEY.to_string()),
            todo_core::settings::get_or(
                c,
                store::KEY_HOTKEY_ADD,
                store::DEFAULT_HOTKEY_ADD.to_string(),
            ),
        )
    });

    if let Err(e) = apply(app, &toggle, &add) {
        eprintln!("[hotkey] {e} - falle auf die Standards zurück");
        let _ = apply(app, store::DEFAULT_HOTKEY, store::DEFAULT_HOTKEY_ADD);
    }
}
