//! IPC für Aufgaben und Projekte. Dünne Hülle um `todo_core::repo` -
//! jede Logik gehört in den Kern, damit die spätere Voll-App sie mitbenutzt.

use crate::store;
use tauri::{AppHandle, Wry};
use std::collections::HashMap;
use todo_core::models::{
    ListQuery, NewTask, Project, ProjectFields, SortSpec, Status, Task, TaskFields, TaskFilter,
    TaskView,
};
use todo_core::repo;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn changed<T>(app: &AppHandle<Wry>, result: todo_core::Result<T>) -> Result<T, String> {
    let value = result.map_err(err)?;
    crate::calendar::wake(app);
    Ok(value)
}

/// `filter` und `sort` sind optional: das Detailfenster holt sich die volle
/// Liste ohne beides, und der Aufruf soll deswegen nicht laenger werden.
#[tauri::command]
pub fn list_tasks(
    app: AppHandle<Wry>,
    view: TaskView,
    include_subtasks: bool,
    filter: Option<TaskFilter>,
    sort: Option<SortSpec>,
) -> Result<Vec<Task>, String> {
    let query = ListQuery {
        view,
        include_subtasks,
        filter: filter.unwrap_or_default(),
        sort,
    };
    store::with(&app, |c| repo::query_tasks(c, &query)).map_err(err)
}

#[tauri::command]
pub fn list_subtasks(app: AppHandle<Wry>, parent_id: String) -> Result<Vec<Task>, String> {
    store::with(&app, |c| repo::list_subtasks(c, &parent_id)).map_err(err)
}

#[tauri::command]
pub fn create_task(app: AppHandle<Wry>, task: NewTask) -> Result<Task, String> {
    changed(&app, store::with(&app, |c| repo::create_task(c, task)))
}

/// Freitextsuche ueber alle Aufgaben, unabhaengig von der Ansicht.
#[tauri::command]
pub fn search_tasks(app: AppHandle<Wry>, query: String, limit: i64) -> Result<Vec<Task>, String> {
    store::with(&app, |c| repo::search_tasks(c, &query, limit)).map_err(err)
}

/// Kopie samt Unteraufgaben, direkt unter dem Original.
#[tauri::command]
pub fn duplicate_task(app: AppHandle<Wry>, id: String) -> Result<Task, String> {
    changed(&app, store::with(&app, |c| repo::duplicate_task(c, &id)))
}

#[tauri::command]
pub fn update_task(app: AppHandle<Wry>, id: String, fields: TaskFields) -> Result<Task, String> {
    changed(&app, store::with(&app, |c| repo::update_task(c, &id, fields)))
}

#[tauri::command]
pub fn set_task_status(app: AppHandle<Wry>, id: String, done: bool) -> Result<Task, String> {
    let status = if done { Status::Done } else { Status::Open };
    changed(&app, store::with(&app, |c| repo::set_status(c, &id, status)))
}

#[tauri::command]
pub fn delete_task(app: AppHandle<Wry>, id: String) -> Result<(), String> {
    changed(&app, store::with(&app, |c| repo::delete_task(c, &id)))
}

#[tauri::command]
pub fn reorder_task(
    app: AppHandle<Wry>,
    id: String,
    before: Option<String>,
    after: Option<String>,
) -> Result<(), String> {
    store::with(&app, |c| {
        repo::reorder_task(c, &id, before.as_deref(), after.as_deref())
    })
    .map_err(err)
}

#[tauri::command]
pub fn list_projects(app: AppHandle<Wry>, archived: bool) -> Result<Vec<Project>, String> {
    store::with(&app, |c| repo::list_projects(c, archived)).map_err(err)
}

/// Offene Aufgaben je Projekt, fuer die Zahl neben dem Namen.
#[tauri::command]
pub fn project_open_counts(app: AppHandle<Wry>) -> Result<HashMap<String, i64>, String> {
    store::with(&app, repo::open_task_counts).map_err(err)
}

#[tauri::command]
pub fn set_project_archived(
    app: AppHandle<Wry>,
    id: String,
    archived: bool,
) -> Result<(), String> {
    store::with(&app, |c| repo::set_project_archived(c, &id, archived)).map_err(err)
}

#[tauri::command]
pub fn create_project(
    app: AppHandle<Wry>,
    name: String,
    color: Option<String>,
) -> Result<Project, String> {
    store::with(&app, |c| repo::create_project(c, &name, color.as_deref())).map_err(err)
}

#[tauri::command]
pub fn update_project(
    app: AppHandle<Wry>,
    id: String,
    fields: ProjectFields,
) -> Result<(), String> {
    store::with(&app, |c| repo::update_project(c, &id, fields)).map_err(err)
}

#[tauri::command]
pub fn delete_project(app: AppHandle<Wry>, id: String) -> Result<(), String> {
    store::with(&app, |c| repo::delete_project(c, &id)).map_err(err)
}
