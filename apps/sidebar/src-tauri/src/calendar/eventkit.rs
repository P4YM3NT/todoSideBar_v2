//! macOS EventKit provider. iCloud and other accounts configured in macOS;
//! no credentials enter this app. All calls must hold `CalendarState::gate`.
use super::provider::{CalendarInfo, CalendarProvider, EventRef, Remote, Write};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use todo_core::calendar::CalendarEvent;

pub struct EventKit;

pub fn supported() -> bool {
    cfg!(target_os = "macos")
}

pub fn permission() -> Result<String, String> {
    native(json!({"operation": "status"}))
}

pub fn authorize() -> Result<String, String> {
    native(json!({"operation": "authorize"}))
}

impl CalendarProvider for EventKit {
    fn calendars(&self) -> Result<Vec<CalendarInfo>, String> {
        native(json!({"operation": "calendars"}))
    }

    fn read(&self, calendar_ids: &[String], refs: &[EventRef]) -> Result<Vec<Remote>, String> {
        native(json!({"operation": "read", "calendarIds": calendar_ids, "refs": refs}))
    }

    fn save(&self, write: &Write) -> Result<CalendarEvent, String> {
        native(request("save", write))
    }

    fn delete(&self, write: &Write) -> Result<(), String> {
        native::<Value>(request("delete", write)).map(drop)
    }
}

fn request(operation: &str, write: &Write) -> Value {
    let mut request = json!(write);
    request["operation"] = json!(operation);
    request
}

#[cfg(target_os = "macos")]
fn native<T: DeserializeOwned>(request: Value) -> Result<T, String> {
    use std::ffi::{CStr, CString, c_char};
    unsafe extern "C" {
        fn todosidebar_calendar_call(input: *const c_char) -> *mut c_char;
        fn todosidebar_calendar_free(value: *mut c_char);
    }
    let input = CString::new(request.to_string()).map_err(|e| e.to_string())?;
    // All callers hold CalendarState::gate. The pointer is owned by the bridge
    // and freed exactly once, even when the response cannot be parsed.
    let response = unsafe {
        let pointer = todosidebar_calendar_call(input.as_ptr());
        if pointer.is_null() {
            return Err("Kalenderantwort fehlt.".into());
        }
        let value = CStr::from_ptr(pointer).to_string_lossy().into_owned();
        todosidebar_calendar_free(pointer);
        value
    };
    let response: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;
    if let Some(error) = response.get("error").and_then(Value::as_str) {
        return Err(error.into());
    }
    serde_json::from_value(
        response
            .get("ok")
            .cloned()
            .ok_or("Kalenderantwort fehlt.")?,
    )
    .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
fn native<T: DeserializeOwned>(_: Value) -> Result<T, String> {
    Err("Apple-Kalender über Systemkonten ist nur unter macOS verfügbar.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn write_keeps_bridge_request_shape() {
        let write = Write {
            reference: EventRef {
                task_id: "t".into(),
                calendar_id: "c".into(),
                event_id: Some("e".into()),
                external_id: None,
                search_dates: BTreeSet::from(["2026-10-20".parse().unwrap()]),
            },
            expected_modified_at: Some(1.5),
            data: None,
        };
        let value = request("delete", &write);
        assert_eq!(value["operation"], "delete");
        assert_eq!(value["taskId"], "t");
        assert_eq!(value["calendarId"], "c");
        assert_eq!(value["eventId"], "e");
        assert_eq!(value["externalId"], Value::Null);
        assert_eq!(value["searchDates"], json!(["2026-10-20"]));
        assert_eq!(value["expectedModifiedAt"], 1.5);
    }
}
