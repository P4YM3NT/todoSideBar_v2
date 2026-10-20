use chrono::{Duration, Local, NaiveDate, TimeZone, Utc};
use todo_core::{
    calendar::*,
    db,
    models::{NewTask, TaskFields},
    repo,
};

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}
fn instant(day: &str, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Local
        .from_local_datetime(&date(day).and_hms_opt(hour, minute, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc)
}
fn timed_task() -> NewTask {
    NewTask {
        title: "Night shift".into(),
        scheduled_on: Some(date("2026-09-28")),
        due_on: Some(date("2026-10-02")),
        starts_at: Some(instant("2026-09-28", 23, 30)),
        ends_at: Some(instant("2026-09-29", 1, 0)),
        ends_on: Some(date("2026-09-29")),
        ..Default::default()
    }
}

#[test]
fn timed_task_and_duplicate_keep_absolute_interval_and_end_day() {
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(&c, timed_task()).unwrap();
    let copy = repo::duplicate_task(&c, &task.id).unwrap();
    for saved in [task, copy] {
        let data = projection(&saved).unwrap().data;
        assert_eq!(data.starts_at, timed_task().starts_at);
        assert_eq!(data.ends_at, timed_task().ends_at);
        assert_eq!(data.ends_on, Some(date("2026-09-29")));
        assert_eq!(
            data.ends_at.unwrap() - data.starts_at.unwrap(),
            Duration::minutes(90)
        );
    }
}

#[test]
fn incomplete_backward_and_mismatched_intervals_are_rejected_without_writing() {
    let c = db::open_in_memory().unwrap();
    for invalid in [
        NewTask {
            ends_at: None,
            ..timed_task()
        },
        NewTask {
            ends_at: timed_task().starts_at,
            ends_on: None,
            ..timed_task()
        },
        NewTask {
            ends_on: None,
            ..timed_task()
        },
        NewTask {
            scheduled_on: None,
            due_on: None,
            ..timed_task()
        },
        NewTask {
            starts_at: Some(instant("2026-09-27", 23, 30)),
            ..timed_task()
        },
    ] {
        assert!(repo::create_task(&c, invalid).is_err());
    }
    assert_eq!(
        c.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn update_can_set_times_and_switch_back_to_all_day_without_losing_date() {
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(
        &c,
        NewTask {
            title: "Day".into(),
            scheduled_on: Some(date("2026-09-28")),
            ..Default::default()
        },
    )
    .unwrap();
    let fields = || TaskFields {
        title: task.title.clone(),
        scheduled_on: task.scheduled_on,
        starts_at: Some(instant("2026-09-28", 9, 0)),
        ends_at: Some(instant("2026-09-28", 10, 0)),
        ..Default::default()
    };
    let updated = repo::update_task(&c, &task.id, fields()).unwrap();
    assert!(updated.starts_at.is_some());
    let cleared = repo::update_task(
        &c,
        &task.id,
        TaskFields {
            starts_at: None,
            ends_at: None,
            ..fields()
        },
    )
    .unwrap();
    assert_eq!(cleared.scheduled_on, task.scheduled_on);
    assert!(projection(&cleared).unwrap().data.starts_at.is_none());
}

#[test]
fn time_only_remote_edit_wins_and_import_preserves_deadline() {
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(&c, timed_task()).unwrap();
    let link = CalendarLink {
        task_id: task.id.clone(),
        calendar_id: "google".into(),
        event_id: Some("event".into()),
        external_id: None,
        snapshot: projection(&task).unwrap(),
        missing_since: None,
        pending: None,
        schedule_version: 1,
    };
    let mut data = link.snapshot.data.clone();
    data.starts_at = Some(instant("2026-09-28", 22, 0));
    data.ends_at = Some(instant("2026-09-29", 2, 0));
    let event = CalendarEvent {
        event_id: "event".into(),
        external_id: None,
        data: data.clone(),
        modified_at: 1.0,
    };
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
    assert!(import_event(&c, &task, DateField::Scheduled, &data).unwrap());
    let saved = repo::get_task(&c, &task.id).unwrap();
    assert_eq!(saved.starts_at, data.starts_at);
    assert_eq!(saved.ends_at, data.ends_at);
    assert_eq!(saved.ends_on, data.ends_on);
    assert_eq!(saved.due_on, task.due_on);
    // The calendar can also turn a timed event back into a multi-day all-day event.
    data.starts_at = None;
    data.ends_at = None;
    data.ends_on = Some(date("2026-10-01"));
    assert!(import_event(&c, &saved, DateField::Scheduled, &data).unwrap());
    assert_eq!(
        projection(&repo::get_task(&c, &task.id).unwrap())
            .unwrap()
            .data,
        data
    );
}

#[test]
fn persisted_pre_time_snapshot_is_backward_compatible() {
    let data: EventData =
        serde_json::from_str(r#"{"title":"Existing","notes":null,"date":"2026-09-28"}"#).unwrap();
    assert_eq!(data.starts_at, None);
    assert_eq!(data.ends_at, None);
    assert_eq!(data.ends_on, None);
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(
        &c,
        NewTask {
            title: data.title.clone(),
            scheduled_on: Some(data.date),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(projection(&task).unwrap().data, data);
}

#[test]
fn interrupted_legacy_extra_day_is_repaired_but_real_calendar_changes_win() {
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(
        &c,
        NewTask {
            title: "Legacy".into(),
            scheduled_on: Some(date("2026-09-28")),
            ..Default::default()
        },
    )
    .unwrap();
    let projection = projection(&task).unwrap();
    let mut link = CalendarLink {
        task_id: task.id.clone(),
        calendar_id: "apple".into(),
        event_id: None,
        external_id: None,
        snapshot: projection.clone(),
        pending: Some(projection.clone()),
        missing_since: None,
        schedule_version: 0,
    };
    let mut event = CalendarEvent {
        event_id: "recovered".into(),
        external_id: None,
        data: EventData {
            ends_on: Some(date("2026-09-29")),
            ..projection.data
        },
        modified_at: 1.0,
    };
    assert!(remote_winner(&task, &[(&link, &event)]).is_none());
    // A different end day or edited title is a user's remote change.
    event.data.ends_on = Some(date("2026-09-30"));
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
    event.data.ends_on = Some(date("2026-09-29"));
    event.data.title = "Changed by user".into();
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
    event.data.title = task.title.clone();
    link.schedule_version = 1;
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
    link.schedule_version = 0;
    link.event_id = Some(event.event_id.clone());
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
}

#[test]
fn migration_preserves_existing_tasks_and_calendar_journals() {
    let path = std::env::temp_dir().join(format!("todo-times-upgrade-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let c = db::open(&path).unwrap();
        // Reconstruct the immediately preceding schema, retaining real rows.
        let task = repo::create_task(
            &c,
            NewTask {
                title: "Existing task".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let old_journal = serde_json::json!({
            "taskId": task.id, "calendarId": "apple", "eventId": "existing-event", "externalId": null,
            "snapshot": {"dateField": "scheduled", "data": {"title": task.title, "notes": null, "date": "2026-09-28"}},
            "missingSince": null,
        });
        c.execute(
            "INSERT INTO calendar_links (task_id, calendar_id, value) VALUES (?1, 'apple', ?2)",
            todo_core::rusqlite::params![task.id, old_journal.to_string()],
        )
        .unwrap();
        c.execute_batch("ALTER TABLE tasks DROP COLUMN starts_at; ALTER TABLE tasks DROP COLUMN ends_at; ALTER TABLE tasks DROP COLUMN ends_on; DELETE FROM _migrations WHERE name = '007_task_times';").unwrap();
        c.execute(
            "INSERT INTO settings (key, value) VALUES ('test.taskId', ?1)",
            [format!("\"{}\"", task.id)],
        )
        .unwrap();
    }
    let c = db::open(&path).unwrap();
    let id: String = todo_core::settings::get_or(&c, "test.taskId", String::new());
    let saved = repo::get_task(&c, &id).unwrap();
    assert_eq!(saved.title, "Existing task");
    assert_eq!(saved.starts_at, None);
    let restored = links(&c).unwrap().remove(0);
    assert_eq!(restored.event_id.as_deref(), Some("existing-event"));
    assert_eq!(restored.snapshot.data.starts_at, None);
    drop(c);
    let _ = std::fs::remove_file(path);
}
