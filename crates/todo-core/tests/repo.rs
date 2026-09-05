use chrono::{Duration, Utc};
use todo_core::models::{NewTask, ProjectFields, Status, TaskFields, TaskView};
use todo_core::{db, repo};

fn conn() -> rusqlite::Connection {
    db::open_in_memory().unwrap()
}

fn task(title: &str) -> NewTask {
    NewTask {
        title: title.into(),
        ..Default::default()
    }
}

#[test]
fn create_and_read_back() {
    let c = conn();
    let t = repo::create_task(&c, task("Einkaufen")).unwrap();
    assert_eq!(t.title, "Einkaufen");
    assert_eq!(t.status, Status::Open);
    assert_eq!(repo::get_task(&c, &t.id).unwrap(), t);
}

#[test]
fn title_is_trimmed_and_must_not_be_empty() {
    let c = conn();
    let t = repo::create_task(&c, task("  Milch  ")).unwrap();
    assert_eq!(t.title, "Milch");
    assert!(repo::create_task(&c, task("   ")).is_err());
}

#[test]
fn new_tasks_go_to_the_top() {
    let c = conn();
    let first = repo::create_task(&c, task("erste")).unwrap();
    let second = repo::create_task(&c, task("zweite")).unwrap();
    assert!(second.sort_order < first.sort_order);

    let list = repo::list_tasks(&c, &TaskView::Inbox, false).unwrap();
    let titles: Vec<_> = list.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(titles, vec!["zweite", "erste"]);
}

#[test]
fn reorder_places_task_between_neighbours() {
    let c = conn();
    let a = repo::create_task(&c, task("a")).unwrap();
    let b = repo::create_task(&c, task("b")).unwrap();
    let m = repo::create_task(&c, task("m")).unwrap();

    // Reihenfolge ist aktuell m, b, a - m soll zwischen b und a.
    repo::reorder_task(&c, &m.id, Some(&b.id), Some(&a.id)).unwrap();

    let titles: Vec<_> = repo::list_tasks(&c, &TaskView::Inbox, false)
        .unwrap()
        .iter()
        .map(|t| t.title.clone())
        .collect();
    assert_eq!(titles, vec!["b", "m", "a"]);
}

#[test]
fn completing_sets_timestamp_and_moves_to_done_view() {
    let c = conn();
    let t = repo::create_task(&c, task("fertig")).unwrap();

    let done = repo::set_status(&c, &t.id, Status::Done).unwrap();
    assert_eq!(done.status, Status::Done);
    assert!(done.completed_at.is_some());
    assert!(repo::list_tasks(&c, &TaskView::Inbox, false).unwrap().is_empty());
    assert_eq!(repo::list_tasks(&c, &TaskView::Done, false).unwrap().len(), 1);

    // Wieder öffnen muss den Zeitstempel entfernen, sonst bleibt die Aufgabe
    // in Auswertungen als "erledigt am" hängen.
    let open = repo::set_status(&c, &t.id, Status::Open).unwrap();
    assert!(open.completed_at.is_none());
}

#[test]
fn today_covers_scheduled_and_due_including_overdue() {
    let c = conn();
    let today = Utc::now().date_naive();

    repo::create_task(&c, NewTask { title: "heute geplant".into(), scheduled_on: Some(today), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "ueberfaellig".into(), due_on: Some(today - Duration::days(3)), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "spaeter".into(), scheduled_on: Some(today + Duration::days(5)), ..Default::default() }).unwrap();

    let mut heute: Vec<_> = repo::list_tasks(&c, &TaskView::Today, false).unwrap()
        .iter().map(|t| t.title.clone()).collect();
    heute.sort();
    assert_eq!(heute, vec!["heute geplant", "ueberfaellig"]);

    let spaeter: Vec<_> = repo::list_tasks(&c, &TaskView::Upcoming, false).unwrap()
        .iter().map(|t| t.title.clone()).collect();
    assert_eq!(spaeter, vec!["spaeter"]);
}

#[test]
fn update_clears_fields_that_are_none() {
    let c = conn();
    let t = repo::create_task(&c, NewTask {
        title: "mit Notiz".into(),
        notes: Some("etwas".into()),
        estimate_minutes: Some(30),
        ..Default::default()
    }).unwrap();
    assert_eq!(t.notes.as_deref(), Some("etwas"));

    let updated = repo::update_task(&c, &t.id, TaskFields {
        title: "ohne Notiz".into(),
        description: None,
        notes: None,
        project_id: None,
        scheduled_on: None,
        due_on: None,
        priority: 0,
        estimate_minutes: None,
    }).unwrap();

    assert_eq!(updated.title, "ohne Notiz");
    assert_eq!(updated.notes, None, "Ueberschreib-Semantik: None leert das Feld");
    assert_eq!(updated.estimate_minutes, None);
}

#[test]
fn soft_deleted_tasks_disappear_from_reads() {
    let c = conn();
    let t = repo::create_task(&c, task("weg")).unwrap();
    repo::delete_task(&c, &t.id).unwrap();

    assert!(repo::get_task(&c, &t.id).is_err());
    assert!(repo::list_tasks(&c, &TaskView::All, false).unwrap().is_empty());
    // Die Zeile muss aber noch existieren, sonst kann Sync das Loeschen nicht melden.
    let rows: i64 = c.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0)).unwrap();
    assert_eq!(rows, 1);
}

#[test]
fn deleting_a_project_keeps_its_tasks_and_moves_them_to_inbox() {
    let c = conn();
    let p = repo::create_project(&c, "Arbeit", Some("#f00")).unwrap();
    let t = repo::create_task(&c, NewTask {
        title: "im Projekt".into(),
        project_id: Some(p.id.clone()),
        ..Default::default()
    }).unwrap();

    assert_eq!(repo::list_tasks(&c, &TaskView::Project(p.id.clone()), false).unwrap().len(), 1);
    assert!(repo::list_tasks(&c, &TaskView::Inbox, false).unwrap().is_empty());

    repo::delete_project(&c, &p.id).unwrap();

    assert!(repo::list_projects(&c, false).unwrap().is_empty());
    let inbox = repo::list_tasks(&c, &TaskView::Inbox, false).unwrap();
    assert_eq!(inbox.len(), 1, "Aufgabe darf nicht mitgeloescht werden");
    assert_eq!(inbox[0].id, t.id);
}

#[test]
fn clearing_a_priority_only_touches_matching_tasks() {
    let c = conn();
    let a = repo::create_task(&c, NewTask { title: "hoch".into(), priority: Some(3), ..Default::default() }).unwrap();
    let b = repo::create_task(&c, NewTask { title: "mittel".into(), priority: Some(2), ..Default::default() }).unwrap();

    let touched = repo::clear_priority(&c, 3).unwrap();
    assert_eq!(touched, 1);

    assert_eq!(repo::get_task(&c, &a.id).unwrap().priority, 0);
    assert_eq!(repo::get_task(&c, &b.id).unwrap().priority, 2, "andere Stufen bleiben");
}

#[test]
fn subtasks_stay_out_of_lists_unless_asked_for() {
    let c = conn();
    let parent = repo::create_task(&c, task("Umzug")).unwrap();
    repo::create_task(&c, NewTask {
        title: "Kartons kaufen".into(),
        parent_id: Some(parent.id.clone()),
        ..Default::default()
    }).unwrap();

    // Ohne Flag stuende die Unteraufgabe doppelt in der Liste.
    let ohne = repo::list_tasks(&c, &TaskView::Inbox, false).unwrap();
    assert_eq!(ohne.len(), 1);
    assert_eq!(ohne[0].id, parent.id);

    let mit = repo::list_tasks(&c, &TaskView::Inbox, true).unwrap();
    assert_eq!(mit.len(), 2);

    let kinder = repo::list_subtasks(&c, &parent.id).unwrap();
    assert_eq!(kinder.len(), 1);
    assert_eq!(kinder[0].title, "Kartons kaufen");
}

#[test]
fn deleting_a_parent_also_removes_its_subtasks() {
    let c = conn();
    let parent = repo::create_task(&c, task("Eltern")).unwrap();
    repo::create_task(&c, NewTask {
        title: "Kind".into(),
        parent_id: Some(parent.id.clone()),
        ..Default::default()
    }).unwrap();

    repo::delete_task(&c, &parent.id).unwrap();

    // Weiches Loeschen loest kein ON DELETE CASCADE aus - ohne die Nachfassung
    // blieben elternlose Zeilen uebrig.
    assert!(repo::list_subtasks(&c, &parent.id).unwrap().is_empty());
    assert!(repo::list_tasks(&c, &TaskView::All, true).unwrap().is_empty());
}

#[test]
fn project_can_opt_into_showing_subtasks() {
    let c = conn();
    let p = repo::create_project(&c, "Arbeit", None).unwrap();
    assert!(!p.show_subtasks, "standardmaessig aus");

    repo::update_project(&c, &p.id, ProjectFields {
        name: "Arbeit".into(),
        description: None,
        color: Some("#0f0".into()),
        icon: None,
        due_on: None,
        show_subtasks: true,
    }).unwrap();
    let loaded = &repo::list_projects(&c, false).unwrap()[0];
    assert!(loaded.show_subtasks);
    assert_eq!(loaded.color.as_deref(), Some("#0f0"));
}

#[test]
fn archiving_hides_a_project_but_keeps_its_tasks() {
    let c = conn();
    let p = repo::create_project(&c, "Umzug", None).unwrap();
    repo::create_task(&c, NewTask {
        title: "Kartons".into(),
        project_id: Some(p.id.clone()),
        ..Default::default()
    }).unwrap();

    repo::set_project_archived(&c, &p.id, true).unwrap();

    assert!(repo::list_projects(&c, false).unwrap().is_empty(), "aus der Liste raus");
    assert_eq!(repo::list_projects(&c, true).unwrap().len(), 1, "im Archiv drin");
    // Anders als beim Loeschen behalten die Aufgaben ihr Projekt.
    let tasks = repo::list_tasks(&c, &TaskView::Project(p.id.clone()), false).unwrap();
    assert_eq!(tasks.len(), 1);

    repo::set_project_archived(&c, &p.id, false).unwrap();
    assert_eq!(repo::list_projects(&c, false).unwrap().len(), 1, "wieder hervorholbar");
}

#[test]
fn open_counts_ignore_done_and_subtasks() {
    let c = conn();
    let p = repo::create_project(&c, "Arbeit", None).unwrap();
    let base = || NewTask { project_id: Some(p.id.clone()), ..Default::default() };

    repo::create_task(&c, NewTask { title: "offen".into(), ..base() }).unwrap();
    let done = repo::create_task(&c, NewTask { title: "erledigt".into(), ..base() }).unwrap();
    let parent = repo::create_task(&c, NewTask { title: "eltern".into(), ..base() }).unwrap();
    repo::create_task(&c, NewTask {
        title: "kind".into(),
        parent_id: Some(parent.id),
        ..base()
    }).unwrap();
    repo::set_status(&c, &done.id, Status::Done).unwrap();

    let counts = repo::open_task_counts(&c).unwrap();
    // "offen" und "eltern" - nicht das Erledigte, nicht die Unteraufgabe.
    assert_eq!(counts.get(&p.id), Some(&2));
}

#[test]
fn search_covers_title_description_and_notes() {
    let c = conn();
    repo::create_task(&c, task("Zahnarzt anrufen")).unwrap();
    repo::create_task(&c, NewTask {
        title: "Bericht".into(),
        description: Some("beim Zahnarzt abgeben".into()),
        ..Default::default()
    }).unwrap();
    repo::create_task(&c, NewTask {
        title: "Notiz".into(),
        notes: Some("Termin Zahnarzt verschieben".into()),
        ..Default::default()
    }).unwrap();
    repo::create_task(&c, task("Einkaufen")).unwrap();

    let hits = repo::search_tasks(&c, "zahnarzt", 20).unwrap();
    assert_eq!(hits.len(), 3, "Gross- und Kleinschreibung darf nicht zaehlen");
    assert_eq!(hits[0].title, "Zahnarzt anrufen", "Titeltreffer stehen vorne");

    assert!(repo::search_tasks(&c, "   ", 20).unwrap().is_empty());
}

#[test]
fn search_puts_open_before_done_and_respects_the_limit() {
    let c = conn();
    let erledigt = repo::create_task(&c, task("Bericht abgeben")).unwrap();
    repo::set_status(&c, &erledigt.id, Status::Done).unwrap();
    repo::create_task(&c, task("Bericht schreiben")).unwrap();

    let hits = repo::search_tasks(&c, "Bericht", 20).unwrap();
    assert_eq!(hits[0].title, "Bericht schreiben");
    assert_eq!(hits[1].title, "Bericht abgeben");
    assert_eq!(repo::search_tasks(&c, "Bericht", 1).unwrap().len(), 1);
}

#[test]
fn search_treats_wildcards_as_plain_text() {
    let c = conn();
    repo::create_task(&c, task("Rabatt 50% sichern")).unwrap();
    repo::create_task(&c, task("Nichts damit zu tun")).unwrap();

    // Ohne Maskierung wuerde "%" als Platzhalter jede Aufgabe finden.
    let hits = repo::search_tasks(&c, "50%", 20).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Rabatt 50% sichern");
}

#[test]
fn duplicate_copies_fields_and_subtasks_below_the_original() {
    let c = conn();
    let p = repo::create_project(&c, "Arbeit", None).unwrap();
    let original = repo::create_task(&c, NewTask {
        title: "Release vorbereiten".into(),
        notes: Some("Checkliste".into()),
        project_id: Some(p.id.clone()),
        priority: Some(2),
        ..Default::default()
    }).unwrap();
    repo::create_task(&c, NewTask {
        title: "Changelog".into(),
        parent_id: Some(original.id.clone()),
        ..Default::default()
    }).unwrap();

    let copy = repo::duplicate_task(&c, &original.id).unwrap();

    assert_eq!(copy.title, "Release vorbereiten (Kopie)");
    assert_eq!(copy.notes.as_deref(), Some("Checkliste"));
    assert_eq!(copy.project_id, Some(p.id.clone()));
    assert_eq!(copy.priority, 2);
    assert_ne!(copy.id, original.id);

    // Unteraufgaben kommen mit, aber ohne Zusatz im Titel.
    let subs = repo::list_subtasks(&c, &copy.id).unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].title, "Changelog");
    assert_eq!(repo::list_subtasks(&c, &original.id).unwrap().len(), 1, "Original bleibt");

    let titles: Vec<_> = repo::list_tasks(&c, &TaskView::Project(p.id), false)
        .unwrap().iter().map(|t| t.title.clone()).collect();
    assert_eq!(titles, vec!["Release vorbereiten", "Release vorbereiten (Kopie)"]);
}

#[test]
fn duplicate_of_a_done_task_starts_open() {
    let c = conn();
    let t = repo::create_task(&c, task("Woechentlich putzen")).unwrap();
    repo::set_status(&c, &t.id, Status::Done).unwrap();

    let copy = repo::duplicate_task(&c, &t.id).unwrap();
    assert_eq!(copy.status, Status::Open);
    assert!(copy.completed_at.is_none());
}
