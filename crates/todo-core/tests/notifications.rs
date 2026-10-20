use chrono::{NaiveDate, NaiveTime};
use todo_core::{
    db,
    models::{NewTask, Status},
    notifications::{self, Channel},
    repo,
};

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 9, day).unwrap()
}
fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}
fn task(title: &str, due_on: Option<NaiveDate>) -> NewTask {
    NewTask {
        title: title.into(),
        due_on,
        ..Default::default()
    }
}

#[test]
fn due_today_waits_for_local_time_and_overdue_is_caught_up() {
    let c = db::open_in_memory().unwrap();
    repo::create_task(&c, task("heute", Some(date(28)))).unwrap();
    repo::create_task(&c, task("überfällig", Some(date(27)))).unwrap();
    repo::create_task(&c, task("morgen", Some(date(29)))).unwrap();
    repo::create_task(&c, task("ohne Termin", None)).unwrap();
    repo::create_task(
        &c,
        NewTask {
            title: "nur geplant".into(),
            scheduled_on: Some(date(28)),
            ..Default::default()
        },
    )
    .unwrap();

    let before =
        notifications::pending(&c, Channel::Popup, date(28), time(8, 59), time(9, 0)).unwrap();
    assert_eq!(
        before.iter().map(|r| r.title.as_str()).collect::<Vec<_>>(),
        ["überfällig"]
    );
    let at = notifications::pending(&c, Channel::Popup, date(28), time(9, 0), time(9, 0)).unwrap();
    assert_eq!(
        at.iter().map(|r| r.title.as_str()).collect::<Vec<_>>(),
        ["überfällig", "heute"]
    );
}

#[test]
fn deliveries_are_independent_per_channel_and_new_due_date() {
    let c = db::open_in_memory().unwrap();
    let task = repo::create_task(&c, task("Termin", Some(date(28)))).unwrap();
    let popup =
        notifications::pending(&c, Channel::Popup, date(28), time(10, 0), time(9, 0)).unwrap();
    notifications::mark_delivered(&c, Channel::Popup, &popup).unwrap();
    notifications::mark_delivered(&c, Channel::Popup, &popup).unwrap();
    assert!(
        notifications::pending(&c, Channel::Popup, date(28), time(10, 0), time(9, 0))
            .unwrap()
            .is_empty()
    );
    let email =
        notifications::pending(&c, Channel::Email, date(28), time(10, 0), time(9, 0)).unwrap();
    assert_eq!(email.len(), 1);
    // Fehlgeschlagenen Versand nicht markieren: er bleibt für einen Versuch offen.
    assert_eq!(
        notifications::pending(&c, Channel::Email, date(28), time(11, 0), time(9, 0)).unwrap(),
        email
    );
    notifications::mark_delivered(&c, Channel::Email, &email).unwrap();
    assert!(
        notifications::pending(&c, Channel::Email, date(29), time(10, 0), time(9, 0))
            .unwrap()
            .is_empty()
    );
    c.execute(
        "UPDATE tasks SET due_on = ?1 WHERE id = ?2",
        todo_core::rusqlite::params![date(29), task.id],
    )
    .unwrap();
    assert_eq!(
        notifications::pending(&c, Channel::Popup, date(29), time(9, 0), time(9, 0))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        notifications::pending(&c, Channel::Email, date(29), time(9, 0), time(9, 0))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn completed_deleted_and_archived_tasks_are_excluded_including_subtasks() {
    let c = db::open_in_memory().unwrap();
    let done = repo::create_task(&c, task("erledigt", Some(date(28)))).unwrap();
    repo::set_status(&c, &done.id, Status::Done).unwrap();
    repo::create_task(
        &c,
        NewTask {
            parent_id: Some(done.id),
            ..task("Kind erledigter Aufgabe", Some(date(28)))
        },
    )
    .unwrap();
    let deleted = repo::create_task(&c, task("gelöscht", Some(date(28)))).unwrap();
    repo::delete_task(&c, &deleted.id).unwrap();
    let project = repo::create_project(&c, "Archiv", None).unwrap();
    repo::set_project_archived(&c, &project.id, true).unwrap();
    repo::create_task(
        &c,
        NewTask {
            project_id: Some(project.id),
            ..task("archiviert", Some(date(28)))
        },
    )
    .unwrap();
    let parent = repo::create_task(&c, task("offene Hauptaufgabe", None)).unwrap();
    repo::create_task(
        &c,
        NewTask {
            parent_id: Some(parent.id),
            ..task("offene Unteraufgabe", Some(date(28)))
        },
    )
    .unwrap();

    let pending =
        notifications::pending(&c, Channel::Email, date(28), time(9, 0), time(9, 0)).unwrap();
    assert_eq!(
        pending.iter().map(|r| r.title.as_str()).collect::<Vec<_>>(),
        ["offene Unteraufgabe"]
    );
}

#[test]
fn delivery_history_survives_reopening_database() {
    let path =
        std::env::temp_dir().join(format!("todo-notifications-test-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let c = db::open(&path).unwrap();
        repo::create_task(&c, task("Termin", Some(date(28)))).unwrap();
        let batch =
            notifications::pending(&c, Channel::Email, date(28), time(9, 0), time(9, 0)).unwrap();
        notifications::mark_delivered(&c, Channel::Email, &batch).unwrap();
    }
    {
        let c = db::open(&path).unwrap();
        assert!(
            notifications::pending(&c, Channel::Email, date(29), time(9, 0), time(9, 0))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            notifications::pending(&c, Channel::Popup, date(29), time(9, 0), time(9, 0))
                .unwrap()
                .len(),
            1
        );
    }
    let _ = std::fs::remove_file(path);
}
