use todo_core::db;

#[test]
fn migrations_create_expected_schema() {
    let conn = db::open_in_memory().expect("open");
    let mut names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    names.sort();
    assert_eq!(
        names,
        vec![
            "_migrations",
            "attachments",
            "calendar_links",
            "notification_deliveries",
            "projects",
            "settings",
            "tags",
            "task_links",
            "task_tags",
            "tasks",
        ]
    );
}

#[test]
fn migrations_are_idempotent() {
    // Zweimal migrieren darf nicht scheitern - das ist der Fall bei jedem App-Start.
    let path = std::env::temp_dir().join(format!("todo-core-test-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    db::open(&path).expect("first open");
    db::open(&path).expect("second open");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn status_check_constraint_is_enforced() {
    let conn = db::open_in_memory().unwrap();
    let err = conn.execute(
        "INSERT INTO tasks (id, title, status, sort_order, created_at, updated_at)
         VALUES ('x', 't', 'bogus', 1.0, '', '')",
        [],
    );
    assert!(err.is_err(), "ungültiger Status muss abgelehnt werden");
}

#[test]
fn migration_002_adds_columns_to_existing_data() {
    // Der wichtige Fall: eine Datenbank, die bereits mit 001 lief und Daten
    // enthaelt, muss 002 ohne Verlust ueberstehen.
    let conn = db::open_in_memory().unwrap();
    conn.execute(
        "INSERT INTO tasks (id, title, status, sort_order, created_at, updated_at)
         VALUES ('t1', 'bestehend', 'open', 1.0, '2026-01-01', '2026-01-01')",
        [],
    )
    .unwrap();

    let (title, desc, est): (String, Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT title, description, estimate_minutes FROM tasks WHERE id = 't1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();

    assert_eq!(title, "bestehend");
    assert_eq!(desc, None, "neue Spalte muss fuer alte Zeilen leer sein");
    assert_eq!(est, None);
}
