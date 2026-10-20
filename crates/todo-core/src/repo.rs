//! Lese- und Schreiboperationen. Kennt Tauri nicht - die spätere Voll-App
//! benutzt exakt dieselben Funktionen.

use crate::error::{CoreError, Result};
use crate::models::{
    DueFilter, ListQuery, NewTask, Project, ProjectFields, Status, StatusFilter, Task,
    TaskFields, TaskFilter, TaskView,
};
use chrono::Utc;
use rusqlite::{Connection, Row, ToSql, params, params_from_iter};
use std::collections::HashMap;

const TASK_COLUMNS: &str = "id, title, description, notes, project_id, parent_id, \
     status, scheduled_on, due_on, starts_at, ends_at, ends_on, priority, estimate_minutes, sort_order, \
     completed_at, created_at, updated_at";

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn new_id() -> String {
    // UUIDv7: zeitsortierbar und geräteübergreifend kollisionsfrei - beides
    // Voraussetzung für spätere Synchronisation.
    uuid::Uuid::now_v7().to_string()
}

fn row_to_task(row: &Row) -> rusqlite::Result<Task> {
    let status: String = row.get("status")?;
    Ok(Task {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        notes: row.get("notes")?,
        project_id: row.get("project_id")?,
        parent_id: row.get("parent_id")?,
        status: Status::parse(&status),
        scheduled_on: row.get("scheduled_on")?,
        due_on: row.get("due_on")?,
        starts_at: row.get("starts_at")?,
        ends_at: row.get("ends_at")?,
        ends_on: row.get("ends_on")?,
        priority: row.get("priority")?,
        estimate_minutes: row.get("estimate_minutes")?,
        sort_order: row.get("sort_order")?,
        completed_at: row.get("completed_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Neue Aufgaben landen oben: kleinster vorhandener Wert minus eins.
fn sort_order_for_top(conn: &Connection) -> Result<f64> {
    let min: Option<f64> = conn.query_row(
        "SELECT MIN(sort_order) FROM tasks WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(min.unwrap_or(0.0) - 1.0)
}

pub fn create_task(conn: &Connection, new: NewTask) -> Result<Task> {
    let title = new.title.trim();
    if title.is_empty() {
        return Err(CoreError::Invalid("Titel darf nicht leer sein".into()));
    }
    crate::calendar::validate_schedule(new.scheduled_on.or(new.due_on), new.ends_on, new.starts_at, new.ends_at)?;

    let id = new_id();
    let ts = now();
    conn.execute(
        "INSERT INTO tasks (id, title, description, notes, project_id, parent_id,
                            status, scheduled_on, due_on, priority,
                            estimate_minutes, sort_order, created_at, updated_at, starts_at, ends_at, ends_on)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', ?7, ?8, ?9, ?10, ?11, ?12, ?12, ?13, ?14, ?15)",
        params![
            id,
            title,
            new.description,
            new.notes,
            new.project_id,
            new.parent_id,
            new.scheduled_on,
            new.due_on,
            new.priority.unwrap_or(0),
            new.estimate_minutes,
            sort_order_for_top(conn)?,
            ts,
            new.starts_at,
            new.ends_at,
            new.ends_on,
        ],
    )?;
    get_task(conn, &id)
}

pub fn get_task(conn: &Connection, id: &str) -> Result<Task> {
    conn.query_row(
        &format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?1 AND deleted_at IS NULL"),
        [id],
        row_to_task,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => CoreError::NotFound(id.to_string()),
        other => other.into(),
    })
}

/// `include_subtasks` steuert, ob Unteraufgaben als eigene Zeilen erscheinen.
/// Standardmaessig nicht - sonst stuende jede Unteraufgabe doppelt in der Liste,
/// einmal fuer sich und einmal unter ihrer Hauptaufgabe.
///
/// Die schlichte Fassung ohne Filter und ohne eigene Sortierung. Bleibt
/// bestehen, weil sie an vielen Stellen so gebraucht wird - und weil die
/// vorhandenen Tests damit weiter genau das Standardverhalten bewachen.
pub fn list_tasks(conn: &Connection, view: &TaskView, include_subtasks: bool) -> Result<Vec<Task>> {
    query_tasks(
        conn,
        &ListQuery {
            view: view.clone(),
            include_subtasks,
            ..Default::default()
        },
    )
}

/// Was eine Ansicht vorgibt.
///
/// Die Statusbedingung steht getrennt, weil der Filter sie **ersetzen** koennen
/// muss. Haenge man sie mit `AND` daneben, liesse sich "alle Zustaende" nie
/// ausdruecken - die Ansicht wuerde immer dazwischenfunken.
struct ViewSql {
    /// `None` = die Ansicht legt sich nicht fest.
    status: Option<&'static str>,
    /// Alles Uebrige. `?` steht fuer heute bzw. die Projekt-ID.
    rest: &'static str,
    order: &'static str,
    /// Wie oft `rest` das heutige Datum bindet.
    date_params: usize,
}

fn view_sql(view: &TaskView) -> ViewSql {
    match view {
        TaskView::Inbox => ViewSql {
            status: Some("open"),
            rest: "project_id IS NULL",
            order: "sort_order ASC",
            date_params: 0,
        },
        TaskView::Today => ViewSql {
            status: Some("open"),
            rest: "(scheduled_on <= ? OR due_on <= ?)",
            order: "due_on IS NULL, due_on ASC, sort_order ASC",
            date_params: 2,
        },
        TaskView::Upcoming => ViewSql {
            status: Some("open"),
            rest: "(scheduled_on > ? OR due_on > ?)",
            order: "COALESCE(scheduled_on, due_on) ASC, sort_order ASC",
            date_params: 2,
        },
        TaskView::Project(_) => ViewSql {
            status: Some("open"),
            rest: "project_id = ?",
            order: "sort_order ASC",
            date_params: 0,
        },
        // Erledigtes nach Abschlusszeit - dort will man das Neueste oben sehen.
        TaskView::Done => ViewSql {
            status: Some("done"),
            rest: "1 = 1",
            order: "completed_at DESC",
            date_params: 0,
        },
        TaskView::All => ViewSql {
            status: None,
            rest: "1 = 1",
            order: "sort_order ASC",
            date_params: 0,
        },
    }
}

/// Baut die Filterbedingungen und sammelt ihre Parameter ein.
fn filter_sql(filter: &TaskFilter, today: &str, out: &mut Vec<Box<dyn ToSql>>) -> Vec<String> {
    let mut clauses = Vec::new();

    if !filter.projects.is_empty() {
        // `IN` trifft NULL in SQL nicht - "ohne Projekt" braucht deshalb einen
        // eigenen Zweig statt eines NULL-Eintrags in der Liste.
        let (ids, inbox): (Vec<_>, Vec<_>) = filter.projects.iter().partition(|p| p.is_some());

        let mut parts = Vec::new();
        if !ids.is_empty() {
            let holes = vec!["?"; ids.len()].join(", ");
            parts.push(format!("project_id IN ({holes})"));
            for id in ids.into_iter().flatten() {
                out.push(Box::new(id.clone()));
            }
        }
        if !inbox.is_empty() {
            parts.push("project_id IS NULL".into());
        }
        clauses.push(format!("({})", parts.join(" OR ")));
    }

    if !filter.priorities.is_empty() {
        let holes = vec!["?"; filter.priorities.len()].join(", ");
        clauses.push(format!("priority IN ({holes})"));
        for p in &filter.priorities {
            out.push(Box::new(*p));
        }
    }

    match filter.due {
        DueFilter::Any => {}
        DueFilter::Overdue => {
            clauses.push("(due_on IS NOT NULL AND due_on < ?)".into());
            out.push(Box::new(today.to_string()));
        }
        DueFilter::Today => {
            clauses.push("due_on = ?".into());
            out.push(Box::new(today.to_string()));
        }
        DueFilter::Week => {
            // Ueberfaelliges ausdruecklich mit drin: was letzte Woche faellig
            // war, gehoert in dieser Woche erst recht auf den Schirm.
            clauses.push("(due_on IS NOT NULL AND due_on <= ?)".into());
            out.push(Box::new(
                (Utc::now().date_naive() + chrono::Duration::days(7)).to_string(),
            ));
        }
        DueFilter::NoDate => clauses.push("due_on IS NULL".into()),
    }

    clauses
}

/// Liste mit Filter und frei waehlbarer Sortierung.
///
/// Die Ansicht liefert weiterhin die Grundbedingung; der Filter haengt sich mit
/// `AND` daran, die Sortierung ersetzt die der Ansicht.
///
/// Die Parameter sind anonyme `?` statt nummerierter `?1`: bei einer `IN`-Liste
/// veraenderlicher Laenge steht ihre Zahl nicht mehr fest. Der Preis ist, dass
/// "Heute" und "Demnaechst" das Datum zweimal binden statt einmal - deshalb
/// gibt `view_sql` mit zurueck, wie oft es gebraucht wird.
pub fn query_tasks(conn: &Connection, query: &ListQuery) -> Result<Vec<Task>> {
    let today = Utc::now().date_naive().to_string();
    let view = view_sql(&query.view);

    // Der Filter ersetzt die Statusvorgabe der Ansicht, statt sich danebenzu-
    // haengen. `Inherit` laesst ihr den Vortritt, `All` streicht sie ganz.
    let status = match query.filter.status {
        StatusFilter::Inherit => view.status,
        StatusFilter::Open => Some("open"),
        StatusFilter::Done => Some("done"),
        StatusFilter::All => None,
    };

    let mut where_clause = match status {
        Some(s) => format!("status = '{s}' AND {}", view.rest),
        None => view.rest.to_string(),
    };

    let mut params: Vec<Box<dyn ToSql>> = Vec::new();
    for _ in 0..view.date_params {
        params.push(Box::new(today.clone()));
    }
    if let TaskView::Project(id) = &query.view {
        params.push(Box::new(id.clone()));
    }

    for clause in filter_sql(&query.filter, &today, &mut params) {
        where_clause.push_str(" AND ");
        where_clause.push_str(&clause);
    }

    if !query.include_subtasks {
        where_clause.push_str(" AND parent_id IS NULL");
    }

    let order = query.sort.unwrap_or_default().order_by(view.order);

    let sql = format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE deleted_at IS NULL AND {where_clause}
         ORDER BY {order}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params.iter()), row_to_task)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Unteraufgaben einer Aufgabe, in ihrer Reihenfolge.
pub fn list_subtasks(conn: &Connection, parent_id: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE deleted_at IS NULL AND parent_id = ?1
         ORDER BY sort_order ASC"
    ))?;
    let rows = stmt.query_map([parent_id], row_to_task)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update_task(conn: &Connection, id: &str, fields: TaskFields) -> Result<Task> {
    let title = fields.title.trim();
    if title.is_empty() {
        return Err(CoreError::Invalid("Titel darf nicht leer sein".into()));
    }
    crate::calendar::validate_schedule(fields.scheduled_on.or(fields.due_on), fields.ends_on, fields.starts_at, fields.ends_at)?;

    let changed = conn.execute(
        "UPDATE tasks SET title = ?2, description = ?3, notes = ?4, project_id = ?5,
                          scheduled_on = ?6, due_on = ?7, priority = ?8,
                          estimate_minutes = ?9, updated_at = ?10,
                          starts_at = ?11, ends_at = ?12, ends_on = ?13
         WHERE id = ?1 AND deleted_at IS NULL",
        params![
            id,
            title,
            fields.description,
            fields.notes,
            fields.project_id,
            fields.scheduled_on,
            fields.due_on,
            fields.priority,
            fields.estimate_minutes,
            now(),
            fields.starts_at,
            fields.ends_at,
            fields.ends_on,
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(id.to_string()));
    }
    get_task(conn, id)
}

pub fn set_status(conn: &Connection, id: &str, status: Status) -> Result<Task> {
    let completed_at = match status {
        Status::Done => Some(now()),
        Status::Open => None,
    };
    let changed = conn.execute(
        "UPDATE tasks SET status = ?2, completed_at = ?3, updated_at = ?4
         WHERE id = ?1 AND deleted_at IS NULL",
        params![id, status.as_str(), completed_at, now()],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(id.to_string()));
    }
    get_task(conn, id)
}

/// Weiches Löschen: die Zeile bleibt, damit spätere Synchronisation das Löschen
/// überhaupt weitergeben kann.
pub fn delete_task(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    let changed = conn.execute(
        "UPDATE tasks SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
        params![id, ts],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(id.to_string()));
    }
    // `ON DELETE CASCADE` greift beim weichen Loeschen nicht - sonst blieben
    // die Unteraufgaben als elternlose Zeilen zurueck.
    conn.execute(
        "UPDATE tasks SET deleted_at = ?2, updated_at = ?2
         WHERE parent_id = ?1 AND deleted_at IS NULL",
        params![id, ts],
    )?;
    Ok(())
}

/// Setzt die Aufgabe zwischen zwei Nachbarn. `None` heißt jeweils Listenrand.
///
/// Der Mittelwert der Nachbarn kostet ein einziges UPDATE - die Alternative
/// wäre, bei jedem Verschieben die ganze Liste neu durchzunummerieren.
pub fn reorder_task(
    conn: &Connection,
    id: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> Result<()> {
    let order_of = |other: Option<&str>| -> Result<Option<f64>> {
        match other {
            None => Ok(None),
            Some(other_id) => Ok(Some(get_task(conn, other_id)?.sort_order)),
        }
    };

    let prev = order_of(before)?;
    let next = order_of(after)?;
    let target = match (prev, next) {
        (Some(a), Some(b)) => (a + b) / 2.0,
        (Some(a), None) => a + 1.0,
        (None, Some(b)) => b - 1.0,
        (None, None) => 0.0,
    };

    conn.execute(
        "UPDATE tasks SET sort_order = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, target, now()],
    )?;
    Ok(())
}

/// Volltextsuche ueber Titel, Kurzbeschreibung und Notizen.
///
/// Bewusst `LIKE` und kein FTS5-Index: die Datenbank ist persoenlich, ein paar
/// tausend Zeilen, die ein Tabellendurchlauf in unter einer Millisekunde
/// abarbeitet. Ein Index waere ein zweiter Schreibpfad, den jede Aenderung
/// mitpflegen muesste - Aufwand ohne spuerbaren Gewinn.
///
/// Offenes steht vor Erledigtem und Titeltreffer vor Fundstellen im Text:
/// wer sucht, will meist weiterarbeiten, nicht nachschlagen.
///
/// SQLite vergleicht bei `LIKE` nur ASCII ohne Ruecksicht auf Gross- und
/// Kleinschreibung - "OKTOBER" findet "oktober", "AeRGER" aber nicht "aerger".
/// Fuer Umlaute braeuchte es die ICU-Erweiterung; das ist es hier nicht wert.
pub fn search_tasks(conn: &Connection, query: &str, limit: i64) -> Result<Vec<Task>> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }

    // Ohne Maskierung wuerde "50%" jede Aufgabe finden und "a_b" fast jede.
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{escaped}%");

    let sql = format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE deleted_at IS NULL
           AND (title LIKE ?1 ESCAPE '\\'
                OR description LIKE ?1 ESCAPE '\\'
                OR notes LIKE ?1 ESCAPE '\\')
         ORDER BY status = 'done' ASC,
                  title LIKE ?1 ESCAPE '\\' DESC,
                  sort_order ASC
         LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![pattern, limit], row_to_task)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Platz direkt unter `after`: die Mitte zum naechsten Nachbarn, sonst dahinter.
fn order_below(conn: &Connection, after: f64) -> Result<f64> {
    let next: Option<f64> = conn.query_row(
        "SELECT MIN(sort_order) FROM tasks WHERE deleted_at IS NULL AND sort_order > ?1",
        [after],
        |r| r.get(0),
    )?;
    Ok(match next {
        Some(next) => (after + next) / 2.0,
        None => after + 1.0,
    })
}

/// Uebernimmt alle Felder in eine neue Aufgabe. Geht ueber `create_task`, damit
/// Validierung und Zeitstempel an genau einer Stelle stehen; `sort_order` wird
/// danach gesetzt, weil neue Aufgaben sonst grundsaetzlich oben landen.
fn copy_of(
    conn: &Connection,
    src: &Task,
    title: &str,
    parent_id: Option<String>,
    sort_order: f64,
) -> Result<Task> {
    let copy = create_task(
        conn,
        NewTask {
            title: title.to_string(),
            description: src.description.clone(),
            notes: src.notes.clone(),
            project_id: src.project_id.clone(),
            parent_id,
            scheduled_on: src.scheduled_on,
            due_on: src.due_on,
            starts_at: src.starts_at,
            ends_at: src.ends_at,
            ends_on: src.ends_on,
            priority: Some(src.priority),
            estimate_minutes: src.estimate_minutes,
        },
    )?;
    conn.execute(
        "UPDATE tasks SET sort_order = ?2 WHERE id = ?1",
        params![copy.id, sort_order],
    )?;
    get_task(conn, &copy.id)
}

/// Legt eine Kopie direkt unter dem Original an - mit allen Unteraufgaben.
///
/// Die Kopie ist immer offen, auch wenn das Original schon abgehakt ist:
/// dupliziert wird, was noch einmal zu tun ist. Aus demselben Grund tragen die
/// Unteraufgaben ihren Titel unveraendert - der Zusatz gehoert an die Aufgabe,
/// die man in der Liste wiederfinden muss, nicht an jede Zeile darunter.
///
/// Als Transaktion, weil eine halb kopierte Aufgabe schlimmer waere als gar
/// keine: sonst haengt bei einem Fehler eine Huelle ohne Unteraufgaben in der Liste.
pub fn duplicate_task(conn: &Connection, id: &str) -> Result<Task> {
    let tx = conn.unchecked_transaction()?;

    let src = get_task(&tx, id)?;
    let copy = copy_of(
        &tx,
        &src,
        &format!("{} (Kopie)", src.title),
        src.parent_id.clone(),
        order_below(&tx, src.sort_order)?,
    )?;

    for sub in list_subtasks(&tx, id)? {
        // Gleiche Reihenfolge wie im Original - die Werte kollidieren nicht,
        // weil Unteraufgaben immer nach ihrer Hauptaufgabe gefiltert werden.
        copy_of(&tx, &sub, &sub.title, Some(copy.id.clone()), sub.sort_order)?;
    }

    tx.commit()?;
    Ok(copy)
}

/// Setzt die Prioritaet aller Aufgaben zurueck, die auf `priority` zeigen.
///
/// Wird gebraucht, wenn eine Prioritaetsstufe geloescht wird - sonst zeigen
/// Aufgaben auf eine Stufe, die es nicht mehr gibt.
pub fn clear_priority(conn: &Connection, priority: i64) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE tasks SET priority = 0, updated_at = ?2
         WHERE priority = ?1 AND deleted_at IS NULL",
        params![priority, now()],
    )?)
}

// ---- Projekte -------------------------------------------------------------

pub fn create_project(conn: &Connection, name: &str, color: Option<&str>) -> Result<Project> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::Invalid("Name darf nicht leer sein".into()));
    }
    let max: Option<f64> = conn.query_row(
        "SELECT MAX(sort_order) FROM projects WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    let id = new_id();
    let ts = now();
    conn.execute(
        "INSERT INTO projects (id, name, color, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, name, color, max.unwrap_or(0.0) + 1.0, ts],
    )?;
    Ok(Project {
        id,
        name: name.to_string(),
        description: None,
        color: color.map(str::to_string),
        icon: None,
        due_on: None,
        sort_order: max.unwrap_or(0.0) + 1.0,
        show_subtasks: false,
        archived: false,
    })
}

/// `archived` steuert, ob die archivierten oder die aktiven Projekte kommen.
pub fn list_projects(conn: &Connection, archived: bool) -> Result<Vec<Project>> {
    let filter = if archived {
        "archived_at IS NOT NULL"
    } else {
        "archived_at IS NULL"
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT id, name, description, color, icon, due_on, sort_order,
                show_subtasks, archived_at
         FROM projects
         WHERE deleted_at IS NULL AND {filter}
         ORDER BY sort_order ASC"
    ))?;
    let rows = stmt.query_map([], |r| {
        Ok(Project {
            id: r.get(0)?,
            name: r.get(1)?,
            description: r.get(2)?,
            color: r.get(3)?,
            icon: r.get(4)?,
            due_on: r.get(5)?,
            sort_order: r.get(6)?,
            show_subtasks: r.get::<_, i64>(7)? != 0,
            archived: r.get::<_, Option<String>>(8)?.is_some(),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Offene Aufgaben je Projekt. Unteraufgaben zaehlen nicht mit - sonst waere
/// die Zahl neben dem Projektnamen groesser als die sichtbare Liste.
pub fn open_task_counts(conn: &Connection) -> Result<HashMap<String, i64>> {
    let mut stmt = conn.prepare(
        "SELECT project_id, COUNT(*) FROM tasks
         WHERE deleted_at IS NULL AND status = 'open'
           AND parent_id IS NULL AND project_id IS NOT NULL
         GROUP BY project_id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<HashMap<_, _>>>()?)
}

/// Archivieren statt loeschen: Projekt und Aufgaben bleiben erhalten.
pub fn set_project_archived(conn: &Connection, id: &str, archived: bool) -> Result<()> {
    let ts = now();
    conn.execute(
        "UPDATE projects SET archived_at = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, archived.then(|| ts.clone()), ts],
    )?;
    Ok(())
}

/// Aufgaben des Projekts bleiben erhalten und rutschen in die Inbox
/// (`ON DELETE SET NULL` greift beim weichen Löschen nicht).
/// Aendert Name, Farbe und ob Unteraufgaben in der Liste erscheinen.
pub fn update_project(conn: &Connection, id: &str, fields: ProjectFields) -> Result<()> {
    let name = fields.name.trim();
    if name.is_empty() {
        return Err(CoreError::Invalid("Name darf nicht leer sein".into()));
    }
    conn.execute(
        "UPDATE projects SET name = ?2, description = ?3, color = ?4, icon = ?5,
                             due_on = ?6, show_subtasks = ?7, updated_at = ?8
         WHERE id = ?1 AND deleted_at IS NULL",
        params![
            id,
            name,
            fields.description,
            fields.color,
            fields.icon,
            fields.due_on,
            fields.show_subtasks as i64,
            now(),
        ],
    )?;
    Ok(())
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    conn.execute(
        "UPDATE tasks SET project_id = NULL, updated_at = ?2 WHERE project_id = ?1",
        params![id, ts],
    )?;
    conn.execute(
        "UPDATE projects SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, ts],
    )?;
    Ok(())
}
