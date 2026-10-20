//! Provider-independent two-way sync pass. Conflict rules live in
//! `todo_core::calendar`; providers only read and write events.
use super::provider::{EventRef, Providers, Write};
use crate::store;
use chrono::Utc;
use std::collections::{BTreeSet, HashMap};
use tauri::{AppHandle, Emitter, Wry};
use todo_core::{calendar as core, models::TaskView};

pub struct Outcome {
    pub synced_tasks: usize,
    pub warnings: Vec<String>,
}

fn reference(
    task_id: &str,
    calendar_id: &str,
    link: Option<&core::CalendarLink>,
    desired: Option<&core::Projection>,
) -> EventRef {
    let search_dates = link
        .map(|l| l.snapshot.data.date)
        .into_iter()
        .chain(link.and_then(|l| l.pending.as_ref()).map(|p| p.data.date))
        .chain(desired.map(|d| d.data.date))
        .collect();
    EventRef {
        task_id: task_id.into(),
        calendar_id: calendar_id.into(),
        event_id: link.and_then(|l| l.event_id.clone()),
        external_id: link.and_then(|l| l.external_id.clone()),
        search_dates,
    }
}

fn write(
    mut reference: EventRef,
    event: Option<&core::CalendarEvent>,
    data: Option<&core::EventData>,
) -> Write {
    // Prefer identifiers from this read, which may differ after account sync.
    if let Some(event) = event {
        reference.event_id = Some(event.event_id.clone());
        reference.external_id = event.external_id.clone();
    }
    Write {
        reference,
        expected_modified_at: event.map(|e| e.modified_at),
        data: data.cloned(),
    }
}

pub fn synchronize(
    app: &AppHandle<Wry>,
    providers: &Providers,
    calendar_ids: &[String],
) -> Result<Outcome, String> {
    let (tasks, links) = store::with(app, |c| -> todo_core::Result<_> {
        Ok((
            todo_core::repo::list_tasks(c, &TaskView::All, true)?,
            core::links(c)?,
        ))
    })
    .map_err(|e| e.to_string())?;
    let links: HashMap<(String, String), core::CalendarLink> = links
        .into_iter()
        .filter(|link| calendar_ids.contains(&link.calendar_id))
        .map(|link| ((link.task_id.clone(), link.calendar_id.clone()), link))
        .collect();
    let ids: BTreeSet<String> = tasks
        .iter()
        .filter(|task| core::projection(task).is_some())
        .map(|task| task.id.clone())
        .chain(links.keys().map(|(id, _)| id.clone()))
        .collect();
    let task_map: HashMap<_, _> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let refs: Vec<EventRef> = ids
        .iter()
        .flat_map(|id| {
            calendar_ids.iter().map(|calendar_id| {
                let desired = task_map
                    .get(id.as_str())
                    .and_then(|task| core::projection(task));
                reference(
                    id,
                    calendar_id,
                    links.get(&(id.clone(), calendar_id.clone())),
                    desired.as_ref(),
                )
            })
        })
        .collect();
    let remote = providers.read(calendar_ids, &refs)?;
    // Read failures never mean deletion. Skip all replicas of that task.
    let skipped: BTreeSet<_> = remote
        .iter()
        .filter(|entry| entry.error.is_some())
        .map(|entry| entry.task_id.clone())
        .collect();
    let mut warnings: BTreeSet<String> = remote
        .iter()
        .filter_map(|entry| entry.error.clone())
        .collect();
    let remote: HashMap<_, _> = remote
        .into_iter()
        .map(|entry| ((entry.task_id, entry.calendar_id), entry.event))
        .collect();
    let mut synced = BTreeSet::new();
    for id in ids {
        if skipped.contains(&id) {
            continue;
        }
        let initial =
            store::with(app, |c| core::current_task(c, &id)).map_err(|e| e.to_string())?;
        if let Some(task) = initial.as_ref() {
            let replicas: Vec<_> = calendar_ids
                .iter()
                .filter_map(|calendar_id| {
                    let key = (id.clone(), calendar_id.clone());
                    Some((links.get(&key)?, remote.get(&key)?.as_ref()?))
                })
                .collect();
            if let Some((_, event)) = core::remote_winner(task, &replicas) {
                let field = core::projection(task).unwrap().date_field;
                let imported =
                    match store::with(app, |c| core::import_event(c, task, field, &event.data)) {
                        Ok(imported) => imported,
                        Err(error) => {
                            warnings.insert(format!(
                                "„{}“: {}. Aufgabe wird übersprungen.",
                                task.title, error
                            ));
                            continue;
                        }
                    };
                if !imported {
                    continue;
                }
                app.emit("tasks://changed", ()).map_err(|e| e.to_string())?;
            }
        }
        let current =
            store::with(app, |c| core::current_task(c, &id)).map_err(|e| e.to_string())?;
        let desired = current.as_ref().and_then(core::projection);
        for calendar_id in calendar_ids {
            let provider = providers.for_calendar(calendar_id);
            let key = (id.clone(), calendar_id.clone());
            let link = links.get(&key);
            let event = remote
                .get(&key)
                .ok_or("Unvollständige Kalenderantwort. Erneut versuchen.")?
                .as_ref();
            let reference = reference(&id, calendar_id, link, desired.as_ref());
            let Some(desired) = desired.as_ref() else {
                if let Some(event) = event {
                    provider.delete(&write(reference, Some(event), None))?;
                }
                store::with(app, |c| core::remove_link(c, &id, calendar_id))
                    .map_err(|e| e.to_string())?;
                continue;
            };
            if event.is_none()
                && let Some(link) =
                    link.filter(|link| link.snapshot == *desired && link.pending.is_none())
            {
                // Deletion does not delete an app task. Remember it until
                // exported task content changes, avoiding recreation loops.
                let mut missing = link.clone();
                if missing.event_id.is_some() {
                    core::confirm_missing(&mut missing, Utc::now());
                    store::with(app, |c| core::save_link(c, &missing))
                        .map_err(|e| e.to_string())?;
                }
                continue;
            }
            let saved = if let Some(event) = event.filter(|event| event.data == desired.data) {
                event.clone()
            } else {
                // Persist the intent before a provider write. Marker recovery
                // can then find the event even if its identifiers were never
                // committed, including after the task is deleted meanwhile.
                let mut journal = link.cloned().unwrap_or_else(|| core::CalendarLink {
                    task_id: id.clone(),
                    calendar_id: calendar_id.clone(),
                    event_id: None,
                    external_id: None,
                    snapshot: desired.clone(),
                    missing_since: None,
                    pending: None,
                    schedule_version: 1,
                });
                journal.pending = Some(desired.clone());
                journal.schedule_version = 1;
                store::with(app, |c| core::save_link(c, &journal)).map_err(|e| e.to_string())?;
                provider.save(&write(reference, event, Some(&desired.data)))?
            };
            let link = core::CalendarLink {
                task_id: id.clone(),
                calendar_id: calendar_id.clone(),
                event_id: Some(saved.event_id),
                external_id: saved.external_id,
                snapshot: desired.clone(),
                missing_since: None,
                pending: None,
                schedule_version: 1,
            };
            store::with(app, |c| core::save_link(c, &link)).map_err(|e| e.to_string())?;
            synced.insert(id.clone());
        }
    }
    Ok(Outcome {
        synced_tasks: synced.len(),
        warnings: warnings.into_iter().collect(),
    })
}
