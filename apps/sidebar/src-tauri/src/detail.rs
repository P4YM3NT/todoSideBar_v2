//! Detailfenster: ein zweites, normales Fenster für alles, was in einer
//! ~250 px breiten Spalte keinen Platz hat.
//!
//! Bewusst **kein** NSPanel: hier will man echten Tastaturfokus zum Schreiben,
//! und die Space-/Level-Eigenheiten des Panels braucht es nicht.

use crate::window::{self, Sidebar};
use serde::Serialize;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindowBuilder, Wry, window::Color,
};

pub const LABEL: &str = "detail";

/// Luft zwischen vergroessertem Dialog und Sidebar, in logischen Punkten.
/// Ohne sie stossen beide Fenster aneinander und wirken wie eine Flaeche.
const SIDEBAR_GAP: f64 = 24.0;

/// Was das Detailfenster anzeigen soll. `None` heißt "neue Aufgabe".
///
/// Liegt in Rust statt in der URL, damit ein bereits offenes Fenster auf eine
/// andere Aufgabe umschalten kann, ohne neu zu laden.
#[derive(Default)]
pub struct DetailTarget(pub Mutex<Option<String>>);

/// Rahmen vor dem Vergroessern, um ihn wiederherstellen zu koennen.
#[derive(Default)]
pub struct DetailRestore(pub Mutex<Option<(PhysicalPosition<i32>, PhysicalSize<u32>)>>);

/// Zaehlt jedes Oeffnen mit.
///
/// Die Notbremse in `reveal_late` schlaeft anderthalb Sekunden. Ohne diese
/// Marke risse sie ein Fenster wieder auf, das der Nutzer in der Zwischenzeit
/// laengst mit Escape geschlossen hat.
#[derive(Default)]
pub struct DetailEpoch(AtomicU64);

impl DetailEpoch {
    fn bump(&self) -> u64 {
        self.0.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn current(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Die Flaechenfarbe, mit der das Fenster erzeugt wird.
///
/// Muss vom Frontend kommen: nur dort ist bekannt, was "system" gerade
/// bedeutet. Die Sidebar meldet sie bei jedem Themenwechsel, damit hier schon
/// vor dem ersten Detailfenster der richtige Ton steht - sonst entstuende es
/// weiss und blitzte im dunklen Theme sichtbar auf.
pub struct DetailBackground(pub Mutex<Color>);

impl Default for DetailBackground {
    fn default() -> Self {
        Self(Mutex::new(Color(255, 255, 255, 255)))
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    /// `None` = neue Aufgabe anlegen.
    pub task_id: Option<String>,
    /// Vorbelegung beim Anlegen aus der Sidebar heraus.
    pub prefill_title: Option<String>,
    pub prefill_project_id: Option<String>,
}

#[tauri::command]
pub fn detail_target(app: AppHandle<Wry>) -> Option<String> {
    app.state::<DetailTarget>().0.lock().unwrap().clone()
}

#[tauri::command]
pub fn open_detail(
    app: AppHandle<Wry>,
    task_id: Option<String>,
    prefill_title: Option<String>,
    prefill_project_id: Option<String>,
) -> Result<(), String> {
    *app.state::<DetailTarget>().0.lock().unwrap() = task_id.clone();
    let epoch = app.state::<DetailEpoch>().bump();

    let target = Target {
        task_id,
        prefill_title,
        prefill_project_id,
    };

    if app.get_webview_window(LABEL).is_some() {
        // Fenster lebt schon - beim Schliessen wird es nur versteckt. Es
        // bleibt versteckt, bis das Frontend die neue Aufgabe geladen hat und
        // `detail_ready` ruft; sonst saehe man kurz die vorige Aufgabe.
        app.emit_to(LABEL, "detail://target", target).map_err(err)?;
        reveal_late(app.clone(), epoch);
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Aufgabe")
        .inner_size(620.0, 540.0)
        .min_inner_size(420.0, 360.0)
        .center()
        .resizable(true)
        // Unsichtbar aufbauen. Ein sichtbares Fenster zeigt erst eine leere
        // Flaeche, dann den Ladekreis und traegt die Felder einzeln nach,
        // waehrend WebView, Bundle und die Abfragen durchlaufen. `detail_ready`
        // blendet es ein, sobald wirklich alles steht.
        .visible(false)
        // Ohne gesetzte Hintergrundfarbe blitzt beim Vergroessern die graue
        // Standardflaeche auf, bevor der WebView den neuen Bereich gezeichnet
        // hat. Der Ton kommt aus dem, was die Sidebar zuletzt gemeldet hat.
        .background_color(*app.state::<DetailBackground>().0.lock().unwrap())
        .build()
        .map_err(err)?;

    // Das frische Fenster holt sein Ziel selbst per `detail_target` ab; das
    // Event hier greift nur für den Fall, dass es schon geladen war.
    let _ = app.emit_to(LABEL, "detail://target", target);

    // Der rote Knopf und ⌘W wuerden das Fenster zerstoeren. Verstecken statt
    // zerstoeren macht jedes weitere Oeffnen sofort - der WebView muss das
    // Bundle dann nicht erneut laden.
    let hidden = window.clone();
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            // Wie in `close_detail`: sonst holt die Notbremse es zurueck.
            handle.state::<DetailEpoch>().bump();
            let _ = hidden.hide();
        }
    });

    reveal_late(app.clone(), epoch);
    Ok(())
}

/// Blendet das Fenster ein, sobald das Frontend fertig geladen hat.
#[tauri::command]
pub fn detail_ready(app: AppHandle<Wry>) -> Result<(), String> {
    // Die Notbremse hat nichts mehr zu tun.
    app.state::<DetailEpoch>().bump();

    if let Some(window) = app.get_webview_window(LABEL) {
        window.show().map_err(err)?;
        window.set_focus().map_err(err)?;
    }
    Ok(())
}

/// Notbremse: zeigt das Fenster auch dann, wenn `detail_ready` ausbleibt.
///
/// Ohne sie bliebe das Fenster nach einem Fehler im Frontend unsichtbar - der
/// Nutzer klickte auf eine Aufgabe und es passierte gar nichts. Lieber ein
/// halb gefuelltes Fenster als keines.
fn reveal_late(app: AppHandle<Wry>, epoch: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Seitdem ist etwas anderes passiert - eingeblendet, geschlossen oder
        // eine andere Aufgabe geoeffnet. Dann gilt das hier nicht mehr.
        if app.state::<DetailEpoch>().current() != epoch {
            return;
        }

        if let Some(window) = app.get_webview_window(LABEL) {
            if !window.is_visible().unwrap_or(true) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    });
}

/// Vergroessert das Detailfenster auf die freie Flaeche und wieder zurueck;
/// liefert den neuen Zustand.
///
/// Bewusst nicht `set_fullscreen`: die App laeuft als
/// `ActivationPolicy::Accessory` und hat keine Menueleiste - echtes
/// macOS-Vollbild verweigert der Fenstermanager dann stillschweigend.
///
/// Und bewusst nicht die volle Arbeitsflaeche: die Sidebar ist ein schwebendes
/// Panel und liegt ueber jedem normalen Fenster. Wuerde der Dialog die ganze
/// Breite nehmen, verschwaende er hinter ihr. Stattdessen endet er an ihrer
/// Kante, sodass beide benutzbar bleiben.
#[tauri::command]
pub fn toggle_detail_maximized(app: AppHandle<Wry>) -> Result<bool, String> {
    let Some(window) = app.get_webview_window(LABEL) else {
        return Ok(false);
    };

    let restore = app.state::<DetailRestore>();
    let previous = *restore.0.lock().unwrap();

    if let Some((pos, size)) = previous {
        window.set_size(size).map_err(err)?;
        window.set_position(pos).map_err(err)?;
        *restore.0.lock().unwrap() = None;
        return Ok(false);
    }

    *restore.0.lock().unwrap() = Some((
        window.outer_position().map_err(err)?,
        window.outer_size().map_err(err)?,
    ));

    fit_beside_sidebar(&app).map_err(err)?;

    Ok(true)
}

/// Setzt die Fenster- und WebView-Hintergrundfarbe auf den Ton des Themes.
///
/// Muss vom Frontend kommen: nur dort ist bekannt, welches Theme nach dem
/// Aufloesen von "system" tatsaechlich gilt. Wird sowohl vom Detailfenster
/// gerufen als auch von der Sidebar bei jedem Themenwechsel - letzteres, damit
/// das naechste Detailfenster gleich in der richtigen Farbe entsteht.
#[tauri::command]
pub fn set_detail_background(app: AppHandle<Wry>, r: u8, g: u8, b: u8) -> Result<(), String> {
    let color = Color(r, g, b, 255);
    *app.state::<DetailBackground>().0.lock().unwrap() = color;

    if let Some(window) = app.get_webview_window(LABEL) {
        window.set_background_color(Some(color)).map_err(err)?;
    }
    Ok(())
}

/// Legt den Dialog neben die Sidebar - volle Arbeitsflaeche minus dem, was die
/// Sidebar belegt, minus `SIDEBAR_GAP`.
fn fit_beside_sidebar(app: &AppHandle<Wry>) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(LABEL) else {
        return Ok(());
    };

    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(());
    };

    let area = monitor.work_area();
    let scale = monitor.scale_factor();

    // Wie viel die Sidebar rechts belegt - ausgeklappt ihre Breite, sonst der
    // Griff, und gar nichts, wenn er abgeschaltet ist.
    let state = *app.state::<Sidebar>().state.lock().unwrap();
    let occupied = if state.expanded {
        state.width
    } else if state.handle_visible {
        window::HANDLE_W
    } else {
        0.0
    };
    let reserved = ((occupied + SIDEBAR_GAP) * scale).round() as u32;

    // Erst an die Ecke setzen, dann aufziehen: umgekehrt wandert das Fenster
    // sichtbar in zwei Schritten.
    window.set_position(PhysicalPosition::new(area.position.x, area.position.y))?;
    window.set_size(PhysicalSize::new(
        area.size.width.saturating_sub(reserved),
        area.size.height,
    ))?;
    Ok(())
}

/// Zieht den Dialog nach, wenn die Sidebar breiter oder schmaler wird.
///
/// Ohne das ueberlappen beide, sobald man die Sidebar aufzieht - der Dialog
/// behaelt sonst die Breite von dem Moment, in dem er vergroessert wurde.
/// Tut nichts, solange der Dialog nicht vergroessert ist.
pub fn refit_if_maximized(app: &AppHandle<Wry>) {
    let is_maximized = app.state::<DetailRestore>().0.lock().unwrap().is_some();
    if is_maximized {
        let _ = fit_beside_sidebar(app);
    }
}

/// Versteckt das Fenster, statt es zu zerstoeren - siehe `open_detail`.
#[tauri::command]
pub fn close_detail(app: AppHandle<Wry>) -> Result<(), String> {
    // Sonst holt die Notbremse das eben Geschlossene zurueck.
    app.state::<DetailEpoch>().bump();

    if let Some(window) = app.get_webview_window(LABEL) {
        window.hide().map_err(err)?;
    }
    Ok(())
}

/// Die Sidebar muss neu laden, wenn im Detailfenster gespeichert wurde.
#[tauri::command]
pub fn notify_tasks_changed(app: AppHandle<Wry>) -> Result<(), String> {
    app.emit("tasks://changed", ()).map_err(err)
}

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}
