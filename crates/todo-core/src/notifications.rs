//! Fällige Erinnerungen und persistente Zustellhistorie, unabhängig von der UI.

use crate::Result;
use chrono::{NaiveDate, NaiveTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Popup,
    Email,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Popup => "popup",
            Self::Email => "email",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reminder {
    pub task_id: String,
    pub title: String,
    pub due_on: NaiveDate,
}

/// Die App übergibt bewusst lokale Zeit: ein Datum hat keine UTC-Zeitzone.
/// Überfälliges wird beim nächsten Start nachgeholt, aber nie täglich wiederholt.
pub fn pending(
    conn: &Connection,
    channel: Channel,
    today: NaiveDate,
    now: NaiveTime,
    reminder_time: NaiveTime,
) -> Result<Vec<Reminder>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.due_on FROM tasks t
         WHERE t.status = 'open' AND t.deleted_at IS NULL
           AND (t.due_on < ?1 OR (t.due_on = ?1 AND ?2))
           AND NOT EXISTS (
               SELECT 1 FROM projects p WHERE p.id = t.project_id
               AND (p.archived_at IS NOT NULL OR p.deleted_at IS NOT NULL)
           )
           AND NOT EXISTS (
               SELECT 1 FROM tasks parent WHERE parent.id = t.parent_id
               AND (parent.status = 'done' OR parent.deleted_at IS NOT NULL)
           )
           AND NOT EXISTS (
               SELECT 1 FROM notification_deliveries d
               WHERE d.task_id = t.id AND d.due_on = t.due_on AND d.channel = ?3
           )
         ORDER BY t.due_on, t.sort_order",
    )?;
    let reminders = stmt
        .query_map(
            params![today, now >= reminder_time, channel.as_str()],
            |r| {
                Ok(Reminder {
                    task_id: r.get(0)?,
                    title: r.get(1)?,
                    due_on: r.get(2)?,
                })
            },
        )?
        .collect::<std::result::Result<_, _>>()?;
    Ok(reminders)
}

/// Erst NACH erfolgreicher Übergabe an den jeweiligen Transport aufrufen.
pub fn mark_delivered(conn: &Connection, channel: Channel, reminders: &[Reminder]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    for reminder in reminders {
        tx.execute(
            "INSERT OR IGNORE INTO notification_deliveries
             (task_id, due_on, channel, delivered_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                reminder.task_id,
                reminder.due_on,
                channel.as_str(),
                Utc::now().to_rfc3339()
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}
