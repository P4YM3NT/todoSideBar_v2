//! Filter und Sortierung. Die schlichte Listenabfrage bewacht `repo.rs`.

use chrono::{Duration, Utc};
use todo_core::models::{
    DueFilter, ListQuery, NewTask, SortDirection, SortField, SortSpec, Status, StatusFilter,
    TaskFilter, TaskView,
};
use todo_core::{db, repo};

fn conn() -> rusqlite::Connection {
    db::open_in_memory().unwrap()
}

fn titles(c: &rusqlite::Connection, query: ListQuery) -> Vec<String> {
    repo::query_tasks(c, &query)
        .unwrap()
        .iter()
        .map(|t| t.title.clone())
        .collect()
}

fn all(filter: TaskFilter) -> ListQuery {
    ListQuery {
        view: TaskView::All,
        filter,
        ..Default::default()
    }
}

// --- Sortierung ------------------------------------------------------------

#[test]
fn manual_keeps_the_order_the_view_prescribes() {
    let c = conn();
    repo::create_task(&c, NewTask { title: "erste".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "zweite".into(), ..Default::default() }).unwrap();

    // Ohne Sortierung und mit Manual muss dasselbe herauskommen wie bei
    // `list_tasks` - das ist der Sinn von Manual.
    let ohne = titles(&c, ListQuery { view: TaskView::Inbox, ..Default::default() });
    let manual = titles(&c, ListQuery {
        view: TaskView::Inbox,
        sort: Some(SortSpec { field: SortField::Manual, direction: SortDirection::Asc }),
        ..Default::default()
    });

    assert_eq!(ohne, vec!["zweite", "erste"]);
    assert_eq!(manual, ohne);
}

#[test]
fn dates_without_a_value_sort_last_in_both_directions() {
    let c = conn();
    let today = Utc::now().date_naive();

    repo::create_task(&c, NewTask { title: "ohne".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "spaet".into(), due_on: Some(today + Duration::days(5)), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "frueh".into(), due_on: Some(today), ..Default::default() }).unwrap();

    let by_due = |direction| {
        titles(&c, ListQuery {
            view: TaskView::All,
            sort: Some(SortSpec { field: SortField::DueOn, direction }),
            ..Default::default()
        })
    };

    assert_eq!(by_due(SortDirection::Asc), vec!["frueh", "spaet", "ohne"]);
    // Auch absteigend darf die Terminlose nicht nach oben rutschen.
    assert_eq!(by_due(SortDirection::Desc), vec!["spaet", "frueh", "ohne"]);
}

#[test]
fn priority_zero_means_none_and_lands_at_the_bottom() {
    let c = conn();
    repo::create_task(&c, NewTask { title: "keine".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "hoch".into(), priority: Some(3), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "mittel".into(), priority: Some(2), ..Default::default() }).unwrap();

    let sorted = titles(&c, ListQuery {
        view: TaskView::All,
        sort: Some(SortSpec { field: SortField::Priority, direction: SortDirection::Desc }),
        ..Default::default()
    });
    assert_eq!(sorted, vec!["hoch", "mittel", "keine"]);
}

#[test]
fn title_sorting_ignores_case() {
    let c = conn();
    for t in ["banane", "Apfel", "citrone"] {
        repo::create_task(&c, NewTask { title: t.into(), ..Default::default() }).unwrap();
    }

    let sorted = titles(&c, ListQuery {
        view: TaskView::All,
        sort: Some(SortSpec { field: SortField::Title, direction: SortDirection::Asc }),
        ..Default::default()
    });
    assert_eq!(sorted, vec!["Apfel", "banane", "citrone"]);
}

// --- Filter ----------------------------------------------------------------

#[test]
fn empty_filter_changes_nothing() {
    let c = conn();
    repo::create_task(&c, NewTask { title: "a".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "b".into(), ..Default::default() }).unwrap();

    assert_eq!(
        titles(&c, all(TaskFilter::default())),
        titles(&c, ListQuery { view: TaskView::All, ..Default::default() }),
    );
}

#[test]
fn project_filter_can_include_tasks_without_a_project() {
    let c = conn();
    let p = repo::create_project(&c, "Arbeit", None).unwrap();
    repo::create_task(&c, NewTask { title: "im projekt".into(), project_id: Some(p.id.clone()), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "in der inbox".into(), ..Default::default() }).unwrap();

    // Nur das Projekt.
    assert_eq!(
        titles(&c, all(TaskFilter { projects: vec![Some(p.id.clone())], ..Default::default() })),
        vec!["im projekt"],
    );

    // Nur "ohne Projekt" - `IN` allein traefe das nie, das ist der NULL-Zweig.
    assert_eq!(
        titles(&c, all(TaskFilter { projects: vec![None], ..Default::default() })),
        vec!["in der inbox"],
    );

    // Beides zusammen.
    let mut beides = titles(&c, all(TaskFilter { projects: vec![Some(p.id), None], ..Default::default() }));
    beides.sort();
    assert_eq!(beides, vec!["im projekt", "in der inbox"]);
}

#[test]
fn priority_filter_matches_zero_as_no_priority() {
    let c = conn();
    repo::create_task(&c, NewTask { title: "ohne".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "mit".into(), priority: Some(2), ..Default::default() }).unwrap();

    assert_eq!(titles(&c, all(TaskFilter { priorities: vec![0], ..Default::default() })), vec!["ohne"]);
    assert_eq!(titles(&c, all(TaskFilter { priorities: vec![2], ..Default::default() })), vec!["mit"]);
}

#[test]
fn status_filter_overrides_the_view_but_inherit_leaves_it_alone() {
    let c = conn();
    let t = repo::create_task(&c, NewTask { title: "erledigt".into(), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "offen".into(), ..Default::default() }).unwrap();
    repo::set_status(&c, &t.id, Status::Done).unwrap();

    let inbox = |status| -> Vec<String> {
        titles(&c, ListQuery {
            view: TaskView::Inbox,
            filter: TaskFilter { status, ..Default::default() },
            ..Default::default()
        })
    };

    // Die Inbox zeigt von sich aus nur Offenes.
    assert_eq!(inbox(StatusFilter::Inherit), vec!["offen"]);
    assert_eq!(inbox(StatusFilter::Open), vec!["offen"]);

    // Der Filter *ersetzt* die Vorgabe der Ansicht, statt sie zu ergaenzen -
    // sonst liesse sich Erledigtes in der Inbox nie hervorholen.
    assert_eq!(inbox(StatusFilter::Done), vec!["erledigt"]);

    let mut beides = inbox(StatusFilter::All);
    beides.sort();
    assert_eq!(beides, vec!["erledigt", "offen"]);

    // Und in "Alle", wo die Ansicht sich nicht festlegt, ebenso.
    let done_only = titles(&c, all(TaskFilter { status: StatusFilter::Done, ..Default::default() }));
    assert_eq!(done_only, vec!["erledigt"]);
}

#[test]
fn due_filter_covers_the_windows() {
    let c = conn();
    let today = Utc::now().date_naive();

    repo::create_task(&c, NewTask { title: "ueberfaellig".into(), due_on: Some(today - Duration::days(2)), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "heute".into(), due_on: Some(today), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "naechste woche".into(), due_on: Some(today + Duration::days(20)), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "ohne datum".into(), ..Default::default() }).unwrap();

    let by = |due| {
        let mut t = titles(&c, all(TaskFilter { due, ..Default::default() }));
        t.sort();
        t
    };

    assert_eq!(by(DueFilter::Overdue), vec!["ueberfaellig"]);
    assert_eq!(by(DueFilter::Today), vec!["heute"]);
    // Die Woche nimmt Ueberfaelliges ausdruecklich mit.
    assert_eq!(by(DueFilter::Week), vec!["heute", "ueberfaellig"]);
    assert_eq!(by(DueFilter::NoDate), vec!["ohne datum"]);
    assert_eq!(by(DueFilter::Any).len(), 4);
}

#[test]
fn filters_combine_with_and() {
    let c = conn();
    let today = Utc::now().date_naive();
    let p = repo::create_project(&c, "Arbeit", None).unwrap();

    repo::create_task(&c, NewTask { title: "treffer".into(), project_id: Some(p.id.clone()), priority: Some(3), due_on: Some(today), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "falsche prio".into(), project_id: Some(p.id.clone()), priority: Some(1), due_on: Some(today), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "falsches projekt".into(), priority: Some(3), due_on: Some(today), ..Default::default() }).unwrap();

    let hits = titles(&c, all(TaskFilter {
        projects: vec![Some(p.id)],
        priorities: vec![3],
        due: DueFilter::Today,
        ..Default::default()
    }));
    assert_eq!(hits, vec!["treffer"]);
}

#[test]
fn filter_and_sort_still_respect_the_view() {
    let c = conn();
    let today = Utc::now().date_naive();

    repo::create_task(&c, NewTask { title: "heute a".into(), due_on: Some(today), priority: Some(1), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "heute b".into(), due_on: Some(today), priority: Some(3), ..Default::default() }).unwrap();
    repo::create_task(&c, NewTask { title: "spaeter".into(), scheduled_on: Some(today + Duration::days(9)), ..Default::default() }).unwrap();

    let heute = titles(&c, ListQuery {
        view: TaskView::Today,
        sort: Some(SortSpec { field: SortField::Priority, direction: SortDirection::Desc }),
        ..Default::default()
    });
    // "spaeter" gehoert nicht in die Ansicht, egal wie sortiert wird.
    assert_eq!(heute, vec!["heute b", "heute a"]);
}
