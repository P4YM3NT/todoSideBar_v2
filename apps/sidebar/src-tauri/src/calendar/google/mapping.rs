//! Google Calendar event JSON ⇄ provider-independent `EventData`.
//! All-day ends are exclusive in Google and inclusive in `EventData`.
use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use todo_core::calendar::{CalendarEvent, EventData};

/// Private extended properties are invisible to other users of a shared
/// calendar and queryable with `privateExtendedProperty`.
pub const MARKER_KEY: &str = "todoSideBar";
pub const TASK_KEY: &str = "todoSideBarTaskId";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEvent {
    pub id: String,
    #[serde(rename = "iCalUID")]
    pub ical_uid: Option<String>,
    pub etag: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub updated: Option<String>,
    #[serde(default)]
    pub recurrence: Vec<String>,
    pub recurring_event_id: Option<String>,
    pub start: Option<GoogleTime>,
    pub end: Option<GoogleTime>,
    pub extended_properties: Option<ExtendedProperties>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleTime {
    pub date: Option<NaiveDate>,
    pub date_time: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExtendedProperties {
    #[serde(default)]
    pub private: HashMap<String, String>,
}

impl GoogleEvent {
    pub fn task_id(&self) -> Option<&str> {
        self.extended_properties
            .as_ref()?
            .private
            .get(TASK_KEY)
            .map(String::as_str)
    }

    pub fn cancelled(&self) -> bool {
        self.status.as_deref() == Some("cancelled")
    }

    pub fn modified_at(&self) -> f64 {
        self.updated
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.timestamp_millis() as f64 / 1000.0)
            .unwrap_or(f64::NAN)
    }

    pub fn to_calendar_event(&self) -> Result<CalendarEvent, String> {
        let title = self.summary.clone().unwrap_or_default();
        let label = if title.is_empty() { "Aufgabe" } else { &title };
        if !self.recurrence.is_empty() || self.recurring_event_id.is_some() {
            return Err(format!(
                "„{label}“: Wiederholungen werden noch nicht unterstützt. Wiederholung im Kalender entfernen. Diese Aufgabe wird bis dahin übersprungen."
            ));
        }
        let invalid =
            || format!("„{label}“: Ende muss nach Beginn liegen. Diese Aufgabe wird übersprungen.");
        let (start, end) = self
            .start
            .as_ref()
            .zip(self.end.as_ref())
            .ok_or_else(invalid)?;
        let data = match (start, end) {
            (
                GoogleTime {
                    date_time: Some(start),
                    ..
                },
                GoogleTime {
                    date_time: Some(end),
                    ..
                },
            ) => {
                if end <= start {
                    return Err(invalid());
                }
                let date = start.with_timezone(&Local).date_naive();
                let last = end.with_timezone(&Local).date_naive();
                EventData {
                    title,
                    notes: self.description.clone().filter(|s| !s.is_empty()),
                    date,
                    starts_at: Some(start.with_timezone(&Utc)),
                    ends_at: Some(end.with_timezone(&Utc)),
                    ends_on: (last != date).then_some(last),
                }
            }
            (
                GoogleTime {
                    date: Some(date), ..
                },
                GoogleTime {
                    date: Some(end), ..
                },
            ) => {
                if end <= date {
                    return Err(invalid());
                }
                let last = *end - Duration::days(1);
                EventData {
                    title,
                    notes: self.description.clone().filter(|s| !s.is_empty()),
                    date: *date,
                    starts_at: None,
                    ends_at: None,
                    ends_on: (last != *date).then_some(last),
                }
            }
            _ => return Err(invalid()),
        };
        Ok(CalendarEvent {
            event_id: self.id.clone(),
            external_id: self.ical_uid.clone(),
            data,
            modified_at: self.modified_at(),
        })
    }
}

/// Request body for insert and patch. Explicit nulls clear the other time
/// representation on patch, e.g. when a timed task becomes all-day.
pub fn event_body(task_id: &str, data: &EventData) -> Value {
    let (start, end) = match (data.starts_at, data.ends_at) {
        (Some(start), Some(end)) => (
            json!({"dateTime": start.to_rfc3339(), "date": null}),
            json!({"dateTime": end.to_rfc3339(), "date": null}),
        ),
        _ => {
            let last = data.ends_on.unwrap_or(data.date);
            (
                json!({"date": data.date, "dateTime": null}),
                json!({"date": last + Duration::days(1), "dateTime": null}),
            )
        }
    };
    json!({
        "summary": data.title,
        "description": data.notes,
        "start": start,
        "end": end,
        "extendedProperties": {"private": {MARKER_KEY: "1", TASK_KEY: task_id}},
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn parse(value: Value) -> GoogleEvent {
        serde_json::from_value(value).unwrap()
    }

    fn day(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn all_day_end_is_exclusive() {
        let event = parse(json!({
            "id": "e1", "iCalUID": "u1@google.com", "summary": "Steuer",
            "updated": "2026-10-20T08:00:00.500Z",
            "start": {"date": "2026-10-20"}, "end": {"date": "2026-10-21"},
            "extendedProperties": {"private": {TASK_KEY: "t1"}}
        }));
        let mapped = event.to_calendar_event().unwrap();
        assert_eq!(mapped.data.date, day("2026-10-20"));
        assert_eq!(mapped.data.ends_on, None);
        assert_eq!(mapped.external_id.as_deref(), Some("u1@google.com"));
        assert_eq!(event.task_id(), Some("t1"));
        assert!((mapped.modified_at - 1_792_483_200.5).abs() < 1e-6);

        let multi = parse(json!({"id": "e", "summary": "Urlaub",
            "start": {"date": "2026-10-20"}, "end": {"date": "2026-10-23"}}));
        assert_eq!(
            multi.to_calendar_event().unwrap().data.ends_on,
            Some(day("2026-10-22"))
        );
    }

    #[test]
    fn body_round_trips_all_day_and_timed() {
        let all_day = EventData {
            title: "Urlaub".into(),
            notes: Some("Koffer".into()),
            date: day("2026-10-20"),
            starts_at: None,
            ends_at: None,
            ends_on: Some(day("2026-10-22")),
        };
        let body = event_body("t1", &all_day);
        assert_eq!(body["end"]["date"], "2026-10-23");
        assert_eq!(body["extendedProperties"]["private"][TASK_KEY], "t1");
        let mut echoed = body.clone();
        echoed["id"] = json!("e");
        assert_eq!(parse(echoed).to_calendar_event().unwrap().data, all_day);

        let start = Local.with_ymd_and_hms(2026, 10, 20, 9, 30, 0).unwrap();
        let timed = EventData {
            title: "Call".into(),
            notes: None,
            date: day("2026-10-20"),
            starts_at: Some(start.with_timezone(&Utc)),
            ends_at: Some((start + Duration::hours(1)).with_timezone(&Utc)),
            ends_on: None,
        };
        let mut echoed = event_body("t2", &timed);
        assert_eq!(echoed["start"]["date"], Value::Null);
        echoed["id"] = json!("e");
        assert_eq!(parse(echoed).to_calendar_event().unwrap().data, timed);
    }

    #[test]
    fn rejects_recurring_and_inverted_events() {
        let recurring = parse(json!({"id": "e", "summary": "Jour fixe",
            "recurrence": ["RRULE:FREQ=WEEKLY"],
            "start": {"date": "2026-10-20"}, "end": {"date": "2026-10-21"}}));
        assert!(
            recurring
                .to_calendar_event()
                .unwrap_err()
                .contains("Wiederholungen")
        );
        let inverted = parse(json!({"id": "e", "summary": "X",
            "start": {"dateTime": "2026-10-20T10:00:00Z"},
            "end": {"dateTime": "2026-10-20T09:00:00Z"}}));
        assert!(inverted.to_calendar_event().unwrap_err().contains("Ende"));
    }
}
