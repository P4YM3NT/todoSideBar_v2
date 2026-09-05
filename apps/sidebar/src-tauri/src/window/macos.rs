//! macOS: das Fenster in ein nicht-aktivierendes NSPanel verwandeln.
//!
//! Tauris eigenes `focusable: false` ist auf macOS defekt (tauri#14102), deshalb
//! der Umweg über NSPanel. Der entscheidende Schalter ist
//! `becomes_key_only_if_needed`: ein Klick auf eine Checkbox lässt den Fokus
//! beim Editor, ein Klick ins Textfeld holt ihn her.

use tauri::{Manager, WebviewWindow, Wry};
use tauri_nspanel::{
    CollectionBehavior, ManagerExt, PanelLevel, WebviewWindowExt, tauri_panel,
};

tauri_panel! {
    panel!(SidebarPanel {
        config: {
            can_become_main_window: false,
            can_become_key_window: true,
            becomes_key_only_if_needed: true,
            is_floating_panel: true
        }
    })
}

pub fn setup(window: &WebviewWindow<Wry>) -> tauri::Result<()> {
    let panel = window.to_panel::<SidebarPanel>()?;

    apply_behavior(&panel);

    panel.set_has_shadow(false);
    panel.set_opaque(false);
    panel.set_transparent(true);
    panel.set_hides_on_deactivate(false);

    Ok(())
}

/// Setzt Space-Verhalten und Fensterlevel.
///
/// Wird bei jedem Anzeigen aufgerufen, nicht nur beim Einrichten - ein
/// Space-Wechsel setzt das Verhalten sonst zurück.
///
/// BEKANNTE EINSCHRÄNKUNG: Die Sidebar erscheint **nicht** über Vollbild-Apps
/// anderer Programme. Empirisch ausgeschlossen wurden: `collectionBehavior`
/// (wird korrekt gesetzt, verifiziert per Rücklesen), das Flag `stationary`,
/// das Flag `full_screen_auxiliary` und das Fensterlevel (4, 25 und 1000
/// verhalten sich identisch). In allen Fällen bleibt `isOnActiveSpace() ==
/// false`.
///
/// Ursache ist sehr wahrscheinlich das fehlende `NSWindowStyleMaskNonactivatingPanel`:
/// AppKit akzeptiert das Flag nur bei der Fenstererzeugung, und `tauri-nspanel`
/// verwandelt per `object_setClass` ein bereits von tao erzeugtes NSWindow.
/// Ein `setStyleMask:` mit dem Flag wird nachweislich verworfen (Rücklesen
/// ergab 0). Eine Lösung erfordert ein echtes, selbst erzeugtes NSPanel
/// anstelle des Plugins.
fn apply_behavior(panel: &tauri_nspanel::PanelHandle<Wry>) {
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()   // auf jedem Space sichtbar
            .ignores_cycle()         // taucht nicht in Cmd+Tab auf
            .into(),
    );

    // `Floating` (4) reicht, um über normalen Fenstern aller Apps zu liegen,
    // und bleibt unter Menüs und System-Dialogen. Höhere Level (25, 1000)
    // wurden getestet und ändern am Vollbild-Verhalten nichts, verdecken aber
    // zunehmend Systemoberfläche - deshalb der niedrigste passende Wert.
    panel.set_level(PanelLevel::Floating.value());
}

/// Zeigt das Panel, ohne die App in den Vordergrund zu holen.
pub fn show(app: &tauri::AppHandle<Wry>) {
    if let Ok(panel) = app.get_webview_panel(super::LABEL) {
        apply_behavior(&panel);
        panel.order_front_regardless();
    }
}

#[allow(dead_code)] // Diagnosewerkzeug für Panel-Probleme
/// Liest den Zustand direkt aus AppKit - Tauris `is_visible()` spiegelt nach
/// einem Space-Wechsel nicht zwingend wider, was das NSPanel wirklich tut.
pub fn debug_state(app: &tauri::AppHandle<Wry>, tag: &str) {
    let Ok(handle) = app.get_webview_panel(super::LABEL) else {
        eprintln!("[panel:{tag}] kein Panel registriert");
        return;
    };
    let p = handle.as_panel();
    let f = p.frame();
    eprintln!(
        "[panel:{tag}] visible={} onActiveSpace={} level={} behavior={:?} style={:?} frame=({},{},{},{})",
        p.isVisible(),
        p.isOnActiveSpace(),
        p.level(),
        p.collectionBehavior().0,
        p.styleMask().0,
        f.origin.x,
        f.origin.y,
        f.size.width,
        f.size.height
    );
}
