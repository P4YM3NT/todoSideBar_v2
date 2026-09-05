//! Geometrie und Zustand des Sidebar-Fensters.
//!
//! Ein einziges Fenster wechselt zwischen zwei Rahmen:
//!   * eingeklappt: `HANDLE_W x HANDLE_H`, am rechten Rand vertikal zentriert
//!   * ausgeklappt: `width x Arbeitsflächenhöhe`, rechtsbündig
//!
//! Der Wechsel des Rahmens passiert ohne Animation; die sichtbare Slide-Animation
//! läuft im Frontend per CSS, während der Rahmen bereits die Endgröße hat. Das
//! hält das Hit-Testing in beiden Zuständen korrekt: eingeklappt deckt das
//! Fenster nur den Griff ab, Klicks daneben gehen an die App darunter.

use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub const LABEL: &str = "sidebar";
pub const HANDLE_W: f64 = 14.0;
pub const HANDLE_H: f64 = 100.0;
pub const DEFAULT_WIDTH_RATIO: f64 = 0.15;
pub const MIN_WIDTH: f64 = 220.0;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SidebarState {
    pub expanded: bool,
    pub handle_visible: bool,
    pub width: f64,
}

pub struct Sidebar {
    pub state: Mutex<SidebarState>,
    /// Zählt jeden Zustandswechsel. Verzögerte Aufgaben merken sich den Stand
    /// beim Start und tun nichts mehr, wenn inzwischen etwas anderes passiert ist.
    pub epoch: AtomicU64,
}

impl Sidebar {
    pub fn bump(&self) -> u64 {
        self.epoch.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::SeqCst)
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            state: Mutex::new(SidebarState {
                expanded: false,
                handle_visible: true,
                // 0.0 heißt "noch nicht bestimmt" - beim ersten Ausklappen wird
                // daraus DEFAULT_WIDTH_RATIO des dann aktiven Monitors.
                width: 0.0,
            }),
        }
    }
}

/// Arbeitsfläche des Monitors unter dem Mauszeiger - in **physischen** Pixeln,
/// zusammen mit dem Skalierungsfaktor.
///
/// Bewusst der Monitor unter dem Zeiger und nicht der zuletzt benutzte: die
/// Sidebar soll dort erscheinen, wo gerade gearbeitet wird.
///
/// Gerechnet wird durchgehend physisch, weil `work_area()` bereits physische
/// Werte liefert - so entfällt ein Hin- und Herrechnen, bei dem sich leicht ein
/// Faktor `scale` einschleicht.
///
/// Achtung beim Debuggen: `outer_size()`/`outer_position()` direkt nach
/// `set_size()`/`set_position()` liefern noch die alten Werte, weil Tauri die
/// Fensteroperationen über die Event-Loop zustellt.
struct Area {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    scale: f64,
}

fn work_area<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<Area> {
    let cursor = window
        .app_handle()
        .cursor_position()
        .unwrap_or(PhysicalPosition::new(0.0, 0.0));

    let monitor = window
        .monitor_from_point(cursor.x, cursor.y)?
        .or(window.current_monitor()?)
        .or(window.primary_monitor()?);

    let Some(monitor) = monitor else {
        // Ohne Monitorinformation ist jede Positionierung geraten - lieber ein
        // brauchbarer Standardrahmen als ein Fenster an Position (0,0).
        return Ok(Area { x: 0.0, y: 0.0, w: 1280.0, h: 800.0, scale: 1.0 });
    };

    let area = monitor.work_area();
    Ok(Area {
        x: area.position.x as f64,
        y: area.position.y as f64,
        w: area.size.width as f64,
        h: area.size.height as f64,
        scale: monitor.scale_factor(),
    })
}

/// Setzt Größe und Position passend zum Zustand.
///
/// `state.width` und die Griffmaße sind logisch (Design-Einheiten), die
/// Fensterrahmen werden physisch gesetzt.
pub fn apply_frame<R: Runtime>(
    window: &WebviewWindow<R>,
    state: &mut SidebarState,
) -> tauri::Result<()> {
    let a = work_area(window)?;
    let logical_w = a.w / a.scale;

    if state.width <= 0.0 {
        state.width = (logical_w * DEFAULT_WIDTH_RATIO).max(MIN_WIDTH);
    }
    state.width = state.width.clamp(MIN_WIDTH, logical_w);

    let (size, pos) = if state.expanded {
        let w = (state.width * a.scale).round();
        (
            PhysicalSize::new(w, a.h),
            PhysicalPosition::new(a.x + a.w - w, a.y),
        )
    } else {
        let w = (HANDLE_W * a.scale).round();
        let h = (HANDLE_H * a.scale).round();
        (
            PhysicalSize::new(w, h),
            PhysicalPosition::new(a.x + a.w - w, a.y + (a.h - h) / 2.0),
        )
    };

    // Reihenfolge zählt: erst Größe, dann Position. Umgekehrt springt das
    // Fenster unter macOS kurz, weil dort der Ursprung unten links liegt und
    // eine Größenänderung die Position mitverschiebt.
    window.set_size(size)?;
    window.set_position(pos)?;
    Ok(())
}

pub fn state_of<R: Runtime>(app: &tauri::AppHandle<R>) -> SidebarState {
    *app.state::<Sidebar>().state.lock().unwrap()
}
