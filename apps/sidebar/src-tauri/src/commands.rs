//! IPC-Oberfläche. Bewusst dünn: die Logik liegt in `window`, `shortcuts` und
//! `todo-core`, damit die spätere Voll-App dieselben Bausteine nutzen kann.

use crate::{
    shortcuts, store,
    window::{self, Sidebar, SidebarState},
};
use serde::{Deserialize, Serialize};
use todo_core::models::{FilterPreset, SortSpec, TaskFilter};
use tauri::{AppHandle, Emitter, Manager, Wry};

/// Eine Prioritaetsstufe.
///
/// `id` ist bewusst **stabil** und wird nie neu vergeben: `tasks.priority`
/// zeigt darauf. Waere es die Rangnummer, wuerden beim Loeschen einer Stufe
/// alle darunterliegenden Aufgaben stillschweigend verrutschen. Die Reihenfolge
/// in der Liste bestimmt den Rang.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub id: i64,
    pub label: String,
    /// Beliebige CSS-Farbe.
    pub color: String,
}

/// Ausgeliefert wird genau ein Preset. Weitere Vorgaben muessten auf
/// Prioritaets-IDs zeigen, die jeder Nutzer umbauen darf - das waere von
/// Anfang an bruechig.
fn default_presets() -> Vec<FilterPreset> {
    vec![FilterPreset {
        id: 1,
        name: "Standard".into(),
        filter: TaskFilter::default(),
        sort: SortSpec::default(),
    }]
}

fn default_priorities() -> Vec<Priority> {
    vec![
        Priority { id: 1, label: "Niedrig".into(), color: "#9ca3af".into() },
        Priority { id: 2, label: "Mittel".into(), color: "#f59e0b".into() },
        Priority { id: 3, label: "Hoch".into(), color: "#ef4444".into() },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub handle_visible: bool,
    pub hotkey: String,
    /// Oeffnet die Sidebar und legt sofort eine Aufgabe an.
    pub add_hotkey: String,
    pub width: f64,
    /// Hex-Farbe fuer abgehakte Checkboxen. `None` = Standardfarbe des Themes.
    pub checkbox_color: Option<String>,
    /// "light" | "dark" | "system"
    pub theme: String,
    /// Deckkraft der Sidebar-Flaeche in Prozent (0 = ganz durchsichtig).
    pub background_alpha: f64,
    /// Eigene Hintergrundfarbe (beliebige CSS-Farbe). `None` = Themefarbe.
    pub background_color: Option<String>,
    pub priorities: Vec<Priority>,
    /// Benannte Paare aus Filter und Sortierung, im Filter-Panel waehlbar.
    pub filter_presets: Vec<FilterPreset>,
    /// Ob Unteraufgaben in den festen Ansichten als eigene Zeilen erscheinen.
    /// Projekte koennen das einzeln ueberschreiben.
    pub show_subtasks: bool,
    pub notifications: crate::notifications::NotificationSettings,
    pub calendar_sync: crate::calendar::CalendarSettings,
}

pub fn load_settings(app: &AppHandle<Wry>) -> AppSettings {
    store::with(app, |c| AppSettings {
        handle_visible: todo_core::settings::get_or(c, store::KEY_HANDLE, true),
        hotkey: todo_core::settings::get_or(
            c,
            store::KEY_HOTKEY,
            store::DEFAULT_HOTKEY.to_string(),
        ),
        add_hotkey: todo_core::settings::get_or(
            c,
            store::KEY_HOTKEY_ADD,
            store::DEFAULT_HOTKEY_ADD.to_string(),
        ),
        width: todo_core::settings::get_or(c, store::KEY_WIDTH, 0.0),
        checkbox_color: todo_core::settings::get_or(c, store::KEY_CHECKBOX_COLOR, None),
        theme: todo_core::settings::get_or(
            c,
            store::KEY_THEME,
            store::DEFAULT_THEME.to_string(),
        ),
        background_alpha: todo_core::settings::get_or(
            c,
            store::KEY_BG_ALPHA,
            store::DEFAULT_BG_ALPHA,
        ),
        background_color: todo_core::settings::get_or(c, store::KEY_BG_COLOR, None),
        priorities: todo_core::settings::get_or(
            c,
            store::KEY_PRIORITIES,
            default_priorities(),
        ),
        filter_presets: todo_core::settings::get_or(
            c,
            store::KEY_FILTER_PRESETS,
            default_presets(),
        ),
        show_subtasks: todo_core::settings::get_or(c, store::KEY_SHOW_SUBTASKS, false),
        notifications: crate::notifications::load(c),
        calendar_sync: crate::calendar::load(c),
    })
}

#[tauri::command]
pub fn get_settings(app: AppHandle<Wry>) -> AppSettings {
    load_settings(&app)
}

/// Der Hotkey wird erst gespeichert, wenn er sich tatsächlich registrieren
/// ließ - sonst sperrt sich der Nutzer mit einer belegten Kombination aus.
///
/// Beide Hotkeys werden dabei neu gesetzt: `shortcuts::apply` meldet zuerst
/// alle ab, der jeweils andere muss also mitkommen.
#[tauri::command]
pub fn set_hotkey(app: AppHandle<Wry>, accelerator: String) -> Result<(), String> {
    let add = load_settings(&app).add_hotkey;
    shortcuts::apply(&app, &accelerator, &add)?;
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_HOTKEY, &accelerator)
    })
    .map_err(err)
}

/// Wie `set_hotkey`, nur für den Hotkey zum Anlegen.
#[tauri::command]
pub fn set_add_hotkey(app: AppHandle<Wry>, accelerator: String) -> Result<(), String> {
    let toggle = load_settings(&app).hotkey;
    shortcuts::apply(&app, &toggle, &accelerator)?;
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_HOTKEY_ADD, &accelerator)
    })
    .map_err(err)
}

/// Sidebar aufklappen und dort das Schnellanlegen öffnen.
///
/// Anders als `toggle` bewusst ohne Umschalten: der Hotkey soll zum Anlegen
/// führen, nicht mal dorthin und mal weg. Steht die Sidebar schon offen,
/// genügt das Ereignis.
pub fn quick_add(app: &AppHandle<Wry>) -> Result<(), String> {
    if !window::state_of(app).expanded {
        set_expanded(app, true)?;
    }
    app.emit("sidebar://quick-add", ()).map_err(err)
}

/// `None` setzt auf die Themefarbe zurueck. Die Farbe wird nicht geprueft -
/// das erledigt das Frontend mit `CSS.supports`, das die echten Regeln des
/// Browsers kennt.
/// Speichert die Stufen und raeumt Aufgaben auf, deren Stufe verschwunden ist.
#[tauri::command]
pub fn set_show_subtasks(app: AppHandle<Wry>, show: bool) -> Result<(), String> {
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_SHOW_SUBTASKS, &show)
    })
    .map_err(err)
}

#[tauri::command]
pub fn set_priorities(app: AppHandle<Wry>, priorities: Vec<Priority>) -> Result<(), String> {
    let before = load_settings(&app).priorities;
    let removed: Vec<i64> = before
        .iter()
        .map(|p| p.id)
        .filter(|id| !priorities.iter().any(|p| p.id == *id))
        .collect();

    // Presets zeigen ebenfalls auf Stufen. Bliebe eine geloeschte ID darin
    // stehen, filterte das Preset stumm ins Leere - man saehe eine leere Liste
    // und faende nirgends den Grund dafuer.
    let presets = load_settings(&app).filter_presets;
    let cleaned: Vec<FilterPreset> = presets
        .iter()
        .map(|preset| {
            let mut next = preset.clone();
            next.filter.priorities.retain(|id| !removed.contains(id));
            next
        })
        .collect();
    let presets_changed = cleaned != presets;

    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_PRIORITIES, &priorities)?;
        // Ohne das zeigen Aufgaben auf eine Stufe, die es nicht mehr gibt.
        for id in &removed {
            todo_core::repo::clear_priority(c, *id)?;
        }
        if presets_changed {
            todo_core::settings::set(c, store::KEY_FILTER_PRESETS, &cleaned)?;
        }
        Ok::<_, todo_core::CoreError>(())
    })
    .map_err(err)?;

    if !removed.is_empty() {
        app.emit("tasks://changed", ()).map_err(err)?;
    }
    Ok(())
}

/// Speichert die Presets. Die IDs vergibt das Frontend und verwendet sie nie
/// erneut - die gemerkte Auswahl je Ansicht zeigt darauf.
#[tauri::command]
pub fn set_filter_presets(
    app: AppHandle<Wry>,
    presets: Vec<FilterPreset>,
) -> Result<(), String> {
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_FILTER_PRESETS, &presets)
    })
    .map_err(err)
}

#[tauri::command]
pub fn set_background_color(app: AppHandle<Wry>, color: Option<String>) -> Result<(), String> {
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_BG_COLOR, &color)
    })
    .map_err(err)
}

#[tauri::command]
pub fn set_background_alpha(app: AppHandle<Wry>, alpha: f64) -> Result<(), String> {
    let alpha = alpha.clamp(0.0, 100.0);
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_BG_ALPHA, &alpha)
    })
    .map_err(err)
}

#[tauri::command]
pub fn set_theme(app: AppHandle<Wry>, theme: String) -> Result<(), String> {
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_THEME, &theme)
    })
    .map_err(err)?;

    // Das Detailfenster wird beim Schliessen nur versteckt und bleibt darum
    // ueber Themenwechsel hinweg am Leben. Ohne diese Nachricht behielte es das
    // Theme von dem Moment, in dem es zum ersten Mal geoeffnet wurde.
    app.emit("theme://changed", theme).map_err(err)
}

/// `None` setzt auf die Themefarbe zurueck.
#[tauri::command]
pub fn set_checkbox_color(app: AppHandle<Wry>, color: Option<String>) -> Result<(), String> {
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_CHECKBOX_COLOR, &color)
    })
    .map_err(err)
}

#[tauri::command]
pub fn sidebar_state(app: AppHandle<Wry>) -> SidebarState {
    window::state_of(&app)
}

#[tauri::command]
pub fn sidebar_expand(app: AppHandle<Wry>) -> Result<(), String> {
    set_expanded(&app, true)
}

#[tauri::command]
pub fn sidebar_collapse(app: AppHandle<Wry>) -> Result<(), String> {
    set_expanded(&app, false)
}

#[tauri::command]
pub fn sidebar_toggle(app: AppHandle<Wry>) -> Result<(), String> {
    toggle(&app)
}

/// Hotkey und Tray rufen das hier direkt auf, ohne Umweg über die IPC-Schicht.
pub fn toggle(app: &AppHandle<Wry>) -> Result<(), String> {
    if !window::state_of(app).expanded {
        return set_expanded(app, true);
    }

    // Erst das Frontend ausblenden lassen; der Rahmen schrumpft danach über
    // `sidebar_collapse`.
    let epoch = app.state::<Sidebar>().bump();
    app.emit("sidebar://close", ()).map_err(err)?;

    // Fallback, falls das Frontend nicht antwortet: ein WebView auf einem
    // inaktiven Space wird angehalten und ruft `sidebar_collapse` nie auf.
    // Ohne diesen Abschluss bliebe der Zustand dauerhaft auf "expanded" - jeder
    // weitere Tastendruck ginge dann wieder nur in diesen Zweig und die Sidebar
    // ließe sich nie wieder öffnen.
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let sidebar = app.state::<Sidebar>();
        // Nur abschließen, wenn seitdem nichts anderes passiert ist.
        if sidebar.epoch() == epoch && window::state_of(&app).expanded {
            let _ = set_expanded(&app, false);
        }
    });
    Ok(())
}

/// Breite waehrend des Ziehens: setzt Zustand und Fenster, **ohne** zu speichern.
///
/// Tauri fuehrt synchrone Kommandos auf einem Thread-Pool aus. Beim Ziehen
/// laufen dutzende Aufrufe pro Sekunde parallel und committen in beliebiger
/// Reihenfolge - in der Datenbank landete dadurch ein zufaelliger Zwischenwert
/// statt der Endbreite. Gespeichert wird deshalb nur einmal, am Ende des
/// Ziehens, ueber `sidebar_commit_width`.
#[tauri::command]
pub fn sidebar_set_width(app: AppHandle<Wry>, width: f64) -> Result<(), String> {
    let window = window(&app)?;

    let mut snapshot = {
        let sidebar = app.state::<Sidebar>();
        let mut state = sidebar.state.lock().unwrap();
        state.width = width;
        *state
    };

    if snapshot.expanded {
        window::apply_frame(&window, &mut snapshot).map_err(err)?;
    }
    commit(&app, snapshot);
    // Erst nach `commit`: `refit_if_maximized` liest denselben Mutex.
    crate::detail::refit_if_maximized(&app);
    Ok(())
}

/// Endgueltige Breite: wie oben, plus Speichern. Einmal am Ende des Ziehens
/// oder direkt aus dem Eingabefeld in den Einstellungen.
#[tauri::command]
pub fn sidebar_commit_width(app: AppHandle<Wry>, width: f64) -> Result<f64, String> {
    sidebar_set_width(app.clone(), width)?;

    let stored = window::state_of(&app).width;
    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_WIDTH, &stored)
    })
    .map_err(err)?;

    Ok(stored)
}

/// Griff abschaltbar: ohne Griff verschwindet das Fenster im eingeklappten
/// Zustand komplett. Hotkey und Tray bleiben die Trigger - deshalb darf das
/// Tray-Icon nie zusammen mit dem Griff abgeschaltet werden.
#[tauri::command]
pub fn sidebar_set_handle_visible(app: AppHandle<Wry>, visible: bool) -> Result<(), String> {
    let sidebar = app.state::<Sidebar>();
    let mut state = sidebar.state.lock().unwrap();
    state.handle_visible = visible;
    let expanded = state.expanded;
    drop(state);

    if !expanded {
        let window = window(&app)?;
        if visible {
            window.show().map_err(err)?;
        } else {
            window.hide().map_err(err)?;
        }
    }

    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_HANDLE, &visible)
    })
    .map_err(err)
}

/// Setzt die Breite auf den Standard zurueck.
///
/// `0.0` ist der Marker fuer "noch nicht bestimmt" - `apply_frame` rechnet
/// daraus den Anteil des Monitors aus, auf dem die Sidebar gerade steht.
#[tauri::command]
pub fn reset_sidebar_width(app: AppHandle<Wry>) -> Result<f64, String> {
    let window = window(&app)?;

    let mut snapshot = {
        let sidebar = app.state::<Sidebar>();
        let mut state = sidebar.state.lock().unwrap();
        state.width = 0.0;
        *state
    };

    window::apply_frame(&window, &mut snapshot).map_err(err)?;
    commit(&app, snapshot);

    store::with(&app, |c| {
        todo_core::settings::set(c, store::KEY_WIDTH, &snapshot.width)
    })
    .map_err(err)?;

    Ok(snapshot.width)
}

pub fn set_expanded(app: &AppHandle<Wry>, expanded: bool) -> Result<(), String> {
    let window = window(app)?;

    let mut snapshot = {
        let sidebar = app.state::<Sidebar>();
        sidebar.bump();
        let mut state = sidebar.state.lock().unwrap();
        state.expanded = expanded;
        *state
    };

    // Ab hier ohne gehaltenen Lock, siehe `commit`.
    window::apply_frame(&window, &mut snapshot).map_err(err)?;

    if expanded || snapshot.handle_visible {
        window.show().map_err(err)?;
    } else {
        window.hide().map_err(err)?;
    }

    commit(app, snapshot);
    crate::detail::refit_if_maximized(app);

    // Auch beim Einklappen: der Griff muss ebenfalls auf dem aktiven Space
    // liegen, sonst ist er nach einem Vollbild-Wechsel verschwunden.
    #[cfg(target_os = "macos")]
    if expanded || snapshot.handle_visible {
        window::macos::show(app);
    }

    app.emit(
        if expanded {
            "sidebar://open"
        } else {
            "sidebar://closed"
        },
        snapshot,
    )
    .map_err(err)
}

/// Schreibt den Zustand zurück, nachdem alle Fensteraufrufe durch sind.
///
/// Der `Sidebar`-Mutex darf **nie** über einen Fensteraufruf hinweg gehalten
/// werden: `set_size`/`set_position` blockieren, bis der Main-Thread sie
/// abarbeitet. Läuft dort gleichzeitig ein Kommando, das denselben Mutex will,
/// hängen beide Threads dauerhaft.
fn commit(app: &AppHandle<Wry>, snapshot: SidebarState) {
    let sidebar = app.state::<Sidebar>();
    *sidebar.state.lock().unwrap() = snapshot;
}

fn window(app: &AppHandle<Wry>) -> Result<tauri::WebviewWindow<Wry>, String> {
    app.get_webview_window(window::LABEL)
        .ok_or_else(|| "sidebar window not found".to_string())
}

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}
