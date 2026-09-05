//! Tray-Icon. Der einzige verbleibende Zugang, wenn der Griff ausgeschaltet ist -
//! deshalb bekommt es immer einen Menüeintrag zum Öffnen der Einstellungen.

use crate::commands;
use tauri::{
    App, Emitter,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

pub fn setup(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Sidebar anzeigen", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Einstellungen…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show, &settings, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        // Template-Modus nutzt nur den Alphakanal - unser Platzhalter-Icon ist
        // ein gefülltes Quadrat und würde dadurch als schwarzer Klotz erscheinen.
        // Umstellen, sobald es ein monochromes Menüleisten-Icon gibt.
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = commands::set_expanded(&app.clone(), true);
            }
            "settings" => {
                let _ = commands::set_expanded(&app.clone(), true);
                // Das Frontend entscheidet, welche Ansicht es zeigt.
                let _ = app.emit("sidebar://show-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
