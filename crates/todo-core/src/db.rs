use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

/// Eingebettete Migrationen, in Reihenfolge angewendet. Neue Dateien hier anhängen.
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../migrations/001_init.sql")),
    (
        "002_task_fields",
        include_str!("../migrations/002_task_fields.sql"),
    ),
    (
        "003_subtasks_as_tasks",
        include_str!("../migrations/003_subtasks_as_tasks.sql"),
    ),
    (
        "004_project_fields",
        include_str!("../migrations/004_project_fields.sql"),
    ),
];

/// Öffnet die Datenbank und bringt sie auf den aktuellen Schemastand.
///
/// WAL ist bewusst ab Tag 1 aktiv: die spätere Voll-App greift als zweiter
/// Prozess auf dieselbe Datei zu, und ohne WAL blockieren sich Leser und
/// Schreiber gegenseitig.
pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

/// In-Memory-Datenbank für Tests.
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
             name       TEXT PRIMARY KEY,
             applied_at TEXT NOT NULL
         )",
    )?;

    for (name, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?1)",
            [name],
            |row| row.get(0),
        )?;
        if applied {
            continue;
        }
        // Migration und ihr Protokolleintrag müssen gemeinsam gelten oder gar nicht.
        conn.execute_batch(&format!("BEGIN; {sql} COMMIT;"))?;
        conn.execute(
            "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
            rusqlite::params![name, chrono::Utc::now().to_rfc3339()],
        )?;
    }
    Ok(())
}
