mod commands;
mod detail;
mod shortcuts;
mod tasks;
mod store;
mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Ein zweiter Start soll die vorhandene Sidebar zeigen, nicht scheitern.
            let _ = commands::set_expanded(app, true);
        }));
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(window::Sidebar::default())
        .manage(detail::DetailTarget::default())
        .manage(detail::DetailRestore::default())
        .manage(detail::DetailBackground::default())
        .manage(detail::DetailEpoch::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_hotkey,
            commands::set_add_hotkey,
            commands::set_checkbox_color,
            commands::set_theme,
            commands::set_background_alpha,
            commands::set_background_color,
            commands::set_priorities,
            commands::set_show_subtasks,
            commands::sidebar_state,
            commands::sidebar_expand,
            commands::sidebar_collapse,
            commands::sidebar_toggle,
            commands::sidebar_set_width,
            commands::sidebar_commit_width,
            commands::reset_sidebar_width,
            commands::sidebar_set_handle_visible,
            tasks::list_tasks,
            tasks::list_subtasks,
            tasks::create_task,
            tasks::search_tasks,
            tasks::duplicate_task,
            tasks::update_task,
            tasks::set_task_status,
            tasks::delete_task,
            tasks::reorder_task,
            tasks::list_projects,
            tasks::project_open_counts,
            tasks::set_project_archived,
            tasks::create_project,
            tasks::update_project,
            tasks::delete_project,
            detail::open_detail,
            detail::close_detail,
            detail::toggle_detail_maximized,
            detail::set_detail_background,
            detail::detail_target,
            detail::detail_ready,
            detail::notify_tasks_changed,
        ])
        .setup(|app| {
            // Kein Dock-Icon: die Sidebar ist ein Hintergrundwerkzeug, keine App,
            // zwischen die man mit Cmd+Tab wechselt.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(store::open(&app.handle().clone())?);

            let window = app
                .get_webview_window(window::LABEL)
                .expect("sidebar window missing from tauri.conf.json");

            #[cfg(target_os = "macos")]
            window::macos::setup(&window)?;
            #[cfg(target_os = "windows")]
            window::windows::setup(&window)?;

            tray::setup(app)?;
            shortcuts::apply_saved(&app.handle().clone());

            // Gespeicherte Einstellungen in den Fensterzustand übernehmen,
            // bevor der erste Rahmen gesetzt wird.
            let saved = commands::load_settings(&app.handle().clone());
            let mut snapshot = {
                let sidebar = app.state::<window::Sidebar>();
                let mut state = sidebar.state.lock().unwrap();
                state.width = saved.width;
                state.handle_visible = saved.handle_visible;
                *state
            };

            // Gleiches Muster wie in `commands`: kein Lock über Fensteraufrufe.
            window::apply_frame(&window, &mut snapshot)?;
            if snapshot.handle_visible {
                window.show()?;
            }
            *app.state::<window::Sidebar>().state.lock().unwrap() = snapshot;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
