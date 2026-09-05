use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Done,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "done" => Status::Done,
            _ => Status::Open,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    /// Einzeilig, erscheint in der Liste unter dem Titel.
    pub description: Option<String>,
    /// Langer Freitext, nur im Detailfenster.
    pub notes: Option<String>,
    pub project_id: Option<String>,
    /// Gesetzt, wenn dies eine Unteraufgabe ist. Unteraufgaben sind vollwertige
    /// Aufgaben - sie koennen eigene Termine, Projekte und Prioritaeten haben.
    pub parent_id: Option<String>,
    pub status: Status,
    /// "Wann arbeite ich dran" - die Planung.
    pub scheduled_on: Option<NaiveDate>,
    /// "Bis wann muss es fertig sein" - die Deadline.
    pub due_on: Option<NaiveDate>,
    pub priority: i64,
    pub estimate_minutes: Option<i64>,
    pub sort_order: f64,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Alles, was beim Anlegen gesetzt werden darf. Nur `title` ist Pflicht -
/// das Schnellanlegen in der Liste schickt genau das und sonst nichts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub scheduled_on: Option<NaiveDate>,
    pub due_on: Option<NaiveDate>,
    pub priority: Option<i64>,
    pub estimate_minutes: Option<i64>,
}

/// Vollständiger Satz editierbarer Felder.
///
/// Bewusst Überschreib- statt Patch-Semantik: `None` heißt "Feld leeren", nicht
/// "unverändert lassen". Sonst bräuchte jedes Feld ein `Option<Option<T>>`, um
/// Leeren von Nichtstun zu unterscheiden - und das Detailfenster schickt beim
/// Speichern ohnehin immer den kompletten Stand.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFields {
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<String>,
    pub scheduled_on: Option<NaiveDate>,
    pub due_on: Option<NaiveDate>,
    pub priority: i64,
    pub estimate_minutes: Option<i64>,
}

/// Die Ansichten der Sidebar.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum TaskView {
    /// Offen, ohne Projekt.
    Inbox,
    /// Offen und für heute oder früher eingeplant oder fällig.
    Today,
    /// Offen mit Termin in der Zukunft.
    Upcoming,
    Project(String),
    Done,
    All,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    /// Name eines Lucide-Symbols; das Frontend loest ihn auf.
    pub icon: Option<String>,
    pub due_on: Option<NaiveDate>,
    pub sort_order: f64,
    /// Ob Unteraufgaben in der Projektliste mit auftauchen.
    pub show_subtasks: bool,
    pub archived: bool,
}

/// Alles, was an einem Projekt geaendert werden darf.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFields {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub due_on: Option<NaiveDate>,
    pub show_subtasks: bool,
}
