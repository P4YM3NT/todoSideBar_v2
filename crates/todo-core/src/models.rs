use chrono::{DateTime, NaiveDate, Utc};
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
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub ends_on: Option<NaiveDate>,
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
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub ends_on: Option<NaiveDate>,
    pub priority: Option<i64>,
    pub estimate_minutes: Option<i64>,
}

/// Vollständiger Satz editierbarer Felder.
///
/// Bewusst Überschreib- statt Patch-Semantik: `None` heißt "Feld leeren", nicht
/// "unverändert lassen". Sonst bräuchte jedes Feld ein `Option<Option<T>>`, um
/// Leeren von Nichtstun zu unterscheiden - und das Detailfenster schickt beim
/// Speichern ohnehin immer den kompletten Stand.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFields {
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<String>,
    pub scheduled_on: Option<NaiveDate>,
    pub due_on: Option<NaiveDate>,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_on: Option<NaiveDate>,
    pub priority: i64,
    pub estimate_minutes: Option<i64>,
}

/// Die Ansichten der Sidebar.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum TaskView {
    /// Offen, ohne Projekt.
    Inbox,
    /// Offen und für heute oder früher eingeplant oder fällig.
    /// Zugleich der Standard - dieselbe Ansicht, mit der die Sidebar startet.
    #[default]
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

// ---------------------------------------------------------------------------
// Sortierung und Filter
//
// Beides liegt hier im Kern und nicht im Frontend: die spaetere Voll-App soll
// dieselben Regeln benutzen, ohne sie ein zweites Mal zu bauen.
// ---------------------------------------------------------------------------

/// Wonach die Liste sortiert wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    /// Die Sortierung, die die Ansicht selbst vorgibt - **nicht** schlicht
    /// `sort_order`. Nur so bleibt "Erledigt" auch hier nach Abschlusszeit
    /// sortiert und der heutige Standard ist ueberhaupt ausdrueckbar.
    Manual,
    DueOn,
    ScheduledOn,
    Priority,
    Title,
    CreatedAt,
    CompletedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn sql(self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSpec {
    pub field: SortField,
    pub direction: SortDirection,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            field: SortField::Manual,
            direction: SortDirection::Asc,
        }
    }
}

impl SortSpec {
    /// Baut den `ORDER BY`-Ausdruck. `view_default` ist die Sortierung, die die
    /// Ansicht mitbringt; sie gilt bei `Manual` und als letztes Kriterium.
    ///
    /// Feld und Richtung kommen ausschliesslich aus `match`-Zweigen ueber die
    /// Enums, nie aus durchgereichtem Text - damit ist Injektion hier
    /// strukturell ausgeschlossen und nicht bloss unwahrscheinlich.
    pub fn order_by(&self, view_default: &str) -> String {
        let dir = self.direction.sql();

        match self.field {
            SortField::Manual => view_default.to_string(),

            // `x IS NULL` zuerst: das sortiert Datenlose immer ans Ende, in
            // beide Richtungen. Eine Aufgabe ohne Termin darf nicht ueber einer
            // ueberfaelligen stehen, nur weil NULL in SQLite klein ist.
            SortField::DueOn => format!("due_on IS NULL, due_on {dir}, sort_order ASC"),
            SortField::ScheduledOn => {
                format!("scheduled_on IS NULL, scheduled_on {dir}, sort_order ASC")
            }
            SortField::CompletedAt => {
                format!("completed_at IS NULL, completed_at {dir}, sort_order ASC")
            }

            // Stufe 0 heisst "ohne Prioritaet". Absteigend landet sie damit
            // unten, wo sie hingehoert.
            SortField::Priority => format!("priority {dir}, sort_order ASC"),

            // NOCASE ist ASCII-only: "Aerger" mit Umlaut sortiert hinter
            // "Zebra". Fuer eine persoenliche Aufgabenliste vertretbar - eine
            // eigene Kollationsfunktion waere hier deutlich zu viel.
            SortField::Title => format!("title COLLATE NOCASE {dir}"),

            SortField::CreatedAt => format!("created_at {dir}"),
        }
    }
}

/// Status-Filter, unabhaengig von dem, was die Ansicht ohnehin vorgibt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusFilter {
    /// Die Ansicht entscheidet - "Erledigt" zeigt Erledigtes, alles andere Offenes.
    #[default]
    Inherit,
    Open,
    Done,
    All,
}

/// Zeitfenster der Faelligkeit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DueFilter {
    #[default]
    Any,
    Overdue,
    Today,
    Week,
    NoDate,
}

/// Was zusaetzlich zur Ansicht herausgefiltert wird.
///
/// Eine leere Liste heisst ueberall "keine Einschraenkung", nicht "nichts
/// zeigen" - abgewaehlt bekommt man alles, nicht das Leere.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskFilter {
    /// `None` als Eintrag meint "ohne Projekt", also die Inbox.
    pub projects: Vec<Option<String>>,
    /// `0` meint "ohne Prioritaet".
    pub priorities: Vec<i64>,
    pub status: StatusFilter,
    pub due: DueFilter,
}

impl TaskFilter {
    /// Ob ueberhaupt etwas eingeschraenkt wird. Das Frontend zeigt danach den
    /// Punkt am Symbol.
    pub fn is_active(&self) -> bool {
        !self.projects.is_empty()
            || !self.priorities.is_empty()
            || self.status != StatusFilter::Inherit
            || self.due != DueFilter::Any
    }
}

/// Eine vollstaendige Listenabfrage.
///
/// Als Struct statt als fuenf Argumente: die naechste Erweiterung (Tags etwa)
/// kommt hier dazu, ohne jede Aufrufstelle anzufassen.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ListQuery {
    pub view: TaskView,
    pub include_subtasks: bool,
    pub filter: TaskFilter,
    /// `None` = die Sortierung der Ansicht.
    pub sort: Option<SortSpec>,
}

/// Ein benanntes Paar aus Filter und Sortierung.
///
/// `id` ist wie bei `Priority` **stabil** und wird nie neu vergeben: die
/// gemerkte Auswahl je Ansicht zeigt darauf, eine recycelte ID wuerde sie
/// stillschweigend auf ein anderes Preset umbiegen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterPreset {
    pub id: i64,
    pub name: String,
    pub filter: TaskFilter,
    pub sort: SortSpec,
}
