//! Provider-independent calendar projections, conflict resolution and durable links.
use crate::{
    CoreError, Result,
    models::{Status, Task},
    repo,
};
use chrono::{DateTime, Local, NaiveDate, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventData {
    pub title: String,
    pub notes: Option<String>,
    pub date: NaiveDate,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_on: Option<NaiveDate>,
}

pub fn validate_schedule(
    date: Option<NaiveDate>,
    ends_on: Option<NaiveDate>,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
) -> Result<()> {
    if ends_on.is_some() || starts_at.is_some() || ends_at.is_some() {
        let date = date.ok_or_else(|| {
            CoreError::Invalid(
                "Für Uhrzeiten oder Enddatum zuerst ein geplantes oder fälliges Datum auswählen."
                    .into(),
            )
        })?;
        if ends_on.is_some_and(|end| end < date) {
            return Err(CoreError::Invalid(
                "Enddatum darf nicht vor dem Startdatum liegen.".into(),
            ));
        }
        match (starts_at, ends_at) {
            (Some(start), Some(end)) => {
                if end <= start {
                    return Err(CoreError::Invalid("Bis muss nach Von liegen. Für einen Termin über Mitternacht das Enddatum auf den Folgetag setzen.".into()));
                }
                if start.with_timezone(&Local).date_naive() != date
                    || end.with_timezone(&Local).date_naive() != ends_on.unwrap_or(date)
                {
                    return Err(CoreError::Invalid("Datum und Uhrzeiten passen nicht zusammen. Termin im Detailfenster prüfen.".into()));
                }
            }
            (None, None) => (),
            _ => {
                return Err(CoreError::Invalid(
                    "Von und Bis müssen beide angegeben werden.".into(),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateField {
    Scheduled,
    Due,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Projection {
    pub data: EventData,
    pub date_field: DateField,
}

pub fn projection(task: &Task) -> Option<Projection> {
    if task.status != Status::Open {
        return None;
    }
    let (date, date_field) = match (task.scheduled_on, task.due_on) {
        (Some(date), _) => (date, DateField::Scheduled),
        (None, Some(date)) => (date, DateField::Due),
        _ => return None,
    };
    Some(Projection {
        data: EventData {
            title: task.title.clone(),
            notes: normalized_notes(task.notes.as_deref()),
            date,
            starts_at: task.starts_at,
            ends_at: task.ends_at,
            ends_on: task.ends_on.filter(|end| *end != date),
        },
        date_field,
    })
}

fn normalized_notes(notes: Option<&str>) -> Option<String> {
    notes.filter(|s| !s.is_empty()).map(str::to_owned)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub event_id: String,
    pub external_id: Option<String>,
    pub data: EventData,
    pub modified_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarLink {
    pub task_id: String,
    pub calendar_id: String,
    /// None marks a deliberately deleted calendar event. It stays suppressed
    /// until the task's exported fields change; an unchanged task cannot revive it.
    pub event_id: Option<String>,
    pub external_id: Option<String>,
    pub snapshot: Projection,
    pub missing_since: Option<String>,
    /// Durable write intent. Recover saves that reached the provider before a
    /// crash or SQLite error, and recognize them as our own writes on restart.
    #[serde(default)]
    pub pending: Option<Projection>,
    /// Version 0 used the following midnight for an all-day end. On macOS
    /// that included an extra day and failed before identifiers were recorded.
    #[serde(default)]
    pub schedule_version: u8,
}

fn interrupted_legacy_all_day(link: &CalendarLink, event: &CalendarEvent) -> bool {
    if link.schedule_version != 0 || link.event_id.is_some() || link.external_id.is_some() {
        return false;
    }
    let Some(pending) = link.pending.as_ref() else {
        return false;
    };
    if pending.data.starts_at.is_some()
        || pending.data.ends_at.is_some()
        || pending.data.ends_on.is_some()
        || event.data.ends_on != pending.data.date.succ_opt()
    {
        return false;
    }
    let mut recovered = event.data.clone();
    recovered.ends_on = None;
    recovered == pending.data
}

pub fn links(conn: &Connection) -> Result<Vec<CalendarLink>> {
    let mut stmt =
        conn.prepare("SELECT value FROM calendar_links ORDER BY task_id, calendar_id")?;
    let values = stmt.query_map([], |row| row.get::<_, String>(0))?;
    values
        .map(|value| {
            serde_json::from_str(&value?)
                .map_err(|e| CoreError::Invalid(format!("calendar link: {e}")))
        })
        .collect()
}

pub fn save_link(conn: &Connection, link: &CalendarLink) -> Result<()> {
    let value = serde_json::to_string(link).map_err(|e| CoreError::Invalid(e.to_string()))?;
    conn.execute(
        "INSERT INTO calendar_links (task_id, calendar_id, value) VALUES (?1, ?2, ?3)
        ON CONFLICT(task_id, calendar_id) DO UPDATE SET value = excluded.value",
        params![link.task_id, link.calendar_id, value],
    )?;
    Ok(())
}

pub fn remove_link(conn: &Connection, task_id: &str, calendar_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM calendar_links WHERE task_id = ?1 AND calendar_id = ?2",
        params![task_id, calendar_id],
    )?;
    Ok(())
}

/// Choose the newest changed replica. Unchanged replicas never win a conflict;
/// if both sides changed, task wins ties. Calendar-only changes win regardless
/// of edits to unrelated task fields (priority, project, estimate, etc.).
pub fn remote_winner<'a>(
    task: &Task,
    replicas: &'a [(&CalendarLink, &CalendarEvent)],
) -> Option<(&'a CalendarLink, &'a CalendarEvent)> {
    let desired = projection(task)?;
    let task_time = DateTime::parse_from_rfc3339(&task.updated_at)
        .ok()
        .map(|ts| ts.timestamp_millis() as f64 / 1000.0)
        .unwrap_or(f64::INFINITY);
    replicas
        .iter()
        .copied()
        .filter(|(link, event)| {
            event.data != link.snapshot.data
                && !interrupted_legacy_all_day(link, event)
                && !link
                    .pending
                    .as_ref()
                    .is_some_and(|pending| event.data == pending.data)
                && !event.data.title.trim().is_empty()
                && event.modified_at.is_finite()
                && (desired == link.snapshot || event.modified_at > task_time)
        })
        .max_by(|(left_link, left), (right_link, right)| {
            left.modified_at
                .total_cmp(&right.modified_at)
                .then_with(|| left_link.calendar_id.cmp(&right_link.calendar_id))
        })
}

/// Only touch synced fields. The revision condition prevents calendar import
/// from overwriting a task edit that arrived while the provider was being read.
pub fn import_event(
    conn: &Connection,
    task: &Task,
    field: DateField,
    event: &EventData,
) -> Result<bool> {
    if event.title.trim().is_empty() {
        return Err(CoreError::Invalid(
            "Kalendertitel darf nicht leer sein".into(),
        ));
    }
    validate_schedule(
        Some(event.date),
        event.ends_on,
        event.starts_at,
        event.ends_at,
    )?;
    let column = match field {
        DateField::Scheduled => "scheduled_on",
        DateField::Due => "due_on",
    };
    let changed = conn.execute(
        &format!(
            "UPDATE tasks SET title = ?2, notes = ?3,
        {column} = ?4, updated_at = ?5, starts_at = ?7, ends_at = ?8, ends_on = ?9
        WHERE id = ?1 AND updated_at = ?6
        AND status = 'open' AND deleted_at IS NULL"
        ),
        params![
            task.id,
            event.title.trim(),
            normalized_notes(event.notes.as_deref()),
            event.date,
            Utc::now().to_rfc3339(),
            task.updated_at,
            event.starts_at,
            event.ends_at,
            event.ends_on.filter(|end| *end != event.date),
        ],
    )?;
    Ok(changed != 0)
}

/// A missing event must be absent in two successful reads at least one minute
/// apart before suppression. This tolerates transient EventKit refreshes.
pub fn confirm_missing(link: &mut CalendarLink, now: DateTime<Utc>) -> bool {
    let confirmed = link
        .missing_since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .is_some_and(|since| now.signed_duration_since(since).num_seconds() >= 60);
    if confirmed {
        link.event_id = None;
        link.external_id = None;
    } else if link.missing_since.is_none() {
        link.missing_since = Some(now.to_rfc3339());
    }
    confirmed
}

pub fn current_task(conn: &Connection, id: &str) -> Result<Option<Task>> {
    match repo::get_task(conn, id) {
        Ok(task) => Ok(Some(task)),
        Err(CoreError::NotFound(_)) => Ok(None),
        Err(error) => Err(error),
    }
}
