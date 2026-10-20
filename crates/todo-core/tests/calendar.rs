use chrono::{Duration, NaiveDate, TimeZone, Utc};
use todo_core::{
    calendar::*,
    db,
    models::{NewTask, Status, Task, TaskFields},
    repo,
};

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}

fn task(conn: &todo_core::rusqlite::Connection) -> Task {
    let mut task = repo::create_task(
        conn,
        NewTask {
            title: "Write proposal".into(),
            notes: Some("Original notes".into()),
            description: Some("Keep description".into()),
            scheduled_on: Some(date("2026-10-25")),
            due_on: Some(date("2026-10-30")),
            priority: Some(3),
            estimate_minutes: Some(45),
            ..Default::default()
        },
    )
    .unwrap();
    task.updated_at = "2026-09-28T12:00:00Z".into();
    conn.execute(
        "UPDATE tasks SET updated_at = ?2 WHERE id = ?1",
        todo_core::rusqlite::params![task.id, task.updated_at],
    )
    .unwrap();
    task
}

fn link(task: &Task, calendar: &str) -> CalendarLink {
    CalendarLink {
        task_id: task.id.clone(),
        calendar_id: calendar.into(),
        event_id: Some(format!("event-{calendar}")),
        external_id: Some(format!("external-{calendar}")),
        snapshot: projection(task).unwrap(),
        missing_since: None,
        pending: None,
        schedule_version: 1,
    }
}

fn event(link: &CalendarLink, title: &str, modified_at: &str) -> CalendarEvent {
    let mut data = link.snapshot.data.clone();
    data.title = title.into();
    CalendarEvent {
        event_id: link.event_id.clone().unwrap(),
        external_id: link.external_id.clone(),
        data,
        modified_at: chrono::DateTime::parse_from_rfc3339(modified_at)
            .unwrap()
            .timestamp() as f64,
    }
}

#[test]
fn planned_date_wins_due_date_and_done_or_undated_tasks_have_no_event() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    assert_eq!(projection(&task).unwrap().data.date, date("2026-10-25"));
    task.scheduled_on = None;
    assert_eq!(projection(&task).unwrap().date_field, DateField::Due);
    task.status = Status::Done;
    assert!(projection(&task).is_none());
    task.status = Status::Open;
    task.due_on = None;
    assert!(projection(&task).is_none());
}

#[test]
fn remote_only_edit_wins_even_when_priority_changed_more_recently() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    let link = link(&task, "apple");
    let event = event(&link, "Remote title", "2026-09-28T11:00:00Z");
    task.priority = 1;
    let replicas = [(&link, &event)];
    assert!(remote_winner(&task, &replicas).is_some());
}

#[test]
fn latest_changed_replica_wins_across_apple_and_google() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let apple = link(&task, "apple");
    let google = link(&task, "google");
    let apple_event = event(&apple, "Apple", "2026-09-28T12:10:00Z");
    let google_event = event(&google, "Google", "2026-09-28T12:20:00Z");
    let replicas = [(&apple, &apple_event), (&google, &google_event)];
    assert_eq!(
        remote_winner(&task, &replicas).unwrap().1.data.title,
        "Google"
    );
}

#[test]
fn newest_local_edit_wins_conflict_and_task_wins_ties() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    let link = link(&task, "google");
    task.title = "Local change".into();
    for time in ["2026-09-28T11:59:00Z", "2026-09-28T12:00:00Z"] {
        let event = event(&link, "Remote change", time);
        assert!(remote_winner(&task, &[(&link, &event)]).is_none());
    }
    let event = event(&link, "Newer remote", "2026-09-28T12:01:00Z");
    assert!(remote_winner(&task, &[(&link, &event)]).is_some());
}

#[test]
fn unchanged_replica_and_empty_remote_title_never_replace_task() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let link = link(&task, "apple");
    for title in [task.title.as_str(), "   "] {
        let event = event(&link, title, "2026-09-28T13:00:00Z");
        assert!(remote_winner(&task, &[(&link, &event)]).is_none());
    }
}

#[test]
fn imported_calendar_edit_preserves_description_deadline_and_task_metadata() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let link = link(&task, "apple");
    let mut event = event(&link, "Changed title", "2026-09-28T13:00:00Z");
    event.data.notes = Some("Changed notes".into());
    event.data.date = date("2026-10-26");
    assert!(import_event(&conn, &task, DateField::Scheduled, &event.data).unwrap());
    let imported = repo::get_task(&conn, &task.id).unwrap();
    assert_eq!(imported.title, event.data.title);
    assert_eq!(imported.notes, event.data.notes);
    assert_eq!(imported.scheduled_on, Some(event.data.date));
    assert_eq!(imported.due_on, task.due_on);
    assert_eq!(imported.description, task.description);
    assert_eq!(imported.priority, task.priority);
    assert_eq!(imported.estimate_minutes, task.estimate_minutes);
    assert_eq!(imported.created_at, task.created_at);
}

#[test]
fn due_only_calendar_edit_changes_deadline_without_creating_planned_date() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    conn.execute(
        "UPDATE tasks SET scheduled_on = NULL WHERE id = ?1",
        [&task.id],
    )
    .unwrap();
    task.scheduled_on = None;
    let link = link(&task, "google");
    let mut event = event(&link, "Deadline", "2026-09-28T13:00:00Z");
    event.data.date = date("2026-11-02");
    import_event(&conn, &task, link.snapshot.date_field, &event.data).unwrap();
    let imported = repo::get_task(&conn, &task.id).unwrap();
    assert_eq!(imported.scheduled_on, None);
    assert_eq!(imported.due_on, Some(event.data.date));
}

#[test]
fn concurrent_app_edit_prevents_stale_calendar_import() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let link = link(&task, "apple");
    let event = event(&link, "Remote", "2026-09-28T13:00:00Z");
    repo::update_task(
        &conn,
        &task.id,
        TaskFields {
            title: "Edited during sync".into(),
            description: task.description.clone(),
            notes: task.notes.clone(),
            project_id: task.project_id.clone(),
            scheduled_on: task.scheduled_on,
            due_on: task.due_on,
            priority: task.priority,
            estimate_minutes: task.estimate_minutes,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!import_event(&conn, &task, DateField::Scheduled, &event.data).unwrap());
    assert_eq!(
        repo::get_task(&conn, &task.id).unwrap().title,
        "Edited during sync"
    );
}

#[test]
fn missing_events_require_two_reads_one_minute_apart_and_keep_task() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let mut link = link(&task, "apple");
    let now = Utc.with_ymd_and_hms(2026, 9, 28, 12, 0, 0).unwrap();
    assert!(!confirm_missing(&mut link, now));
    assert!(!confirm_missing(&mut link, now + Duration::seconds(59)));
    assert!(confirm_missing(&mut link, now + Duration::seconds(60)));
    assert_eq!(link.event_id, None);
    assert_eq!(link.external_id, None);
    save_link(&conn, &link).unwrap();
    let restored = links(&conn).unwrap().remove(0);
    assert_eq!(restored.event_id, None);
    assert_eq!(restored.snapshot, projection(&task).unwrap());
    assert!(current_task(&conn, &task.id).unwrap().is_some());
}

#[test]
fn links_survive_deleted_tasks_for_outgoing_calendar_cleanup() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    save_link(&conn, &link(&task, "apple")).unwrap();
    save_link(&conn, &link(&task, "google")).unwrap();
    repo::delete_task(&conn, &task.id).unwrap();
    assert!(current_task(&conn, &task.id).unwrap().is_none());
    assert_eq!(links(&conn).unwrap().len(), 2);
    remove_link(&conn, &task.id, "apple").unwrap();
    assert_eq!(links(&conn).unwrap().len(), 1);
}

#[test]
fn updating_link_after_save_replaces_it_instead_of_creating_duplicate() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let mut link = link(&task, "apple");
    save_link(&conn, &link).unwrap();
    link.event_id = Some("reassigned-id".into());
    save_link(&conn, &link).unwrap();
    let links = links(&conn).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].event_id, link.event_id);
}

#[test]
fn completing_task_prevents_calendar_edits_from_reopening_it() {
    let conn = db::open_in_memory().unwrap();
    let task = task(&conn);
    let link = link(&task, "apple");
    let event = event(&link, "Calendar edit", "2026-09-28T13:00:00Z");
    let done = repo::set_status(&conn, &task.id, Status::Done).unwrap();
    assert!(remote_winner(&done, &[(&link, &event)]).is_none());
    assert!(!import_event(&conn, &task, DateField::Scheduled, &event.data).unwrap());
}

#[test]
fn interrupted_provider_write_does_not_override_a_newer_local_edit() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    let mut link = link(&task, "apple");
    task.title = "Our outgoing write".into();
    link.pending = projection(&task);
    let event = event(&link, "Our outgoing write", "2026-09-28T13:00:00Z");
    // A user edit happened while our calendar write was in flight. Although
    // EventKit stamped our write later, it must not be imported as a remote edit.
    task.title = "Edited during outgoing write".into();
    assert!(remote_winner(&task, &[(&link, &event)]).is_none());
    save_link(&conn, &link).unwrap();
    assert_eq!(links(&conn).unwrap()[0].pending, link.pending);
}

#[test]
fn write_journal_survives_task_deletion_and_remembers_uncommitted_event_date() {
    let conn = db::open_in_memory().unwrap();
    let mut task = task(&conn);
    let mut link = link(&task, "google");
    task.scheduled_on = Some(date("2026-12-03"));
    link.pending = projection(&task);
    save_link(&conn, &link).unwrap();
    repo::delete_task(&conn, &task.id).unwrap();
    let persisted = links(&conn).unwrap().remove(0);
    assert!(current_task(&conn, &task.id).unwrap().is_none());
    assert_eq!(persisted.pending.unwrap().data.date, date("2026-12-03"));
}
