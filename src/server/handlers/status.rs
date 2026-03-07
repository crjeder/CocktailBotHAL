// src/server/handlers/status.rs
//
// Handler for GET /v1/status

use embedded_io_async::Write;

use crate::hal::StatusHal;
use crate::server::http;

/// GET /v1/status — return current robot state and active errors.
///
/// Response: the `RobotState` tagged-union object with an added `"errors"` key.
/// Example: `{ "state": "working", "job_id": "...", "progress_pct": 50, "errors": [] }`
pub async fn handle_status<Stat: StatusHal, W: Write + Unpin>(status: &Stat, socket: &mut W) {
    let state = status.state().await;
    let errors = status.active_errors().await;

    // Serialize the data-carrying RobotState to its tagged JSON object, then
    // merge in the errors array at the top level.
    let mut val = serde_json::to_value(&state).unwrap_or_default();
    if let serde_json::Value::Object(ref mut map) = val {
        map.insert(
            "errors".to_string(),
            serde_json::to_value(&errors).unwrap_or_default(),
        );
    }
    http::write_json(socket, 200, &val).await.ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockStatusHal, MockWrite};
    use crate::hal::{RecoveryAction, RobotState};
    use futures::executor::block_on;

    #[test]
    fn status_returns_200_with_idle_state() {
        block_on(async {
            let hal = MockStatusHal::new().with_state(RobotState::Idle);
            let mut buf = MockWrite::new();
            handle_status(&hal, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"), "expected 200, got: {}", resp);
            assert!(resp.contains("\"idle\""), "expected idle in body: {}", resp);
        });
    }

    #[test]
    fn status_idle_body_has_state_tag_and_errors_key() {
        block_on(async {
            let hal = MockStatusHal::new().with_state(RobotState::Idle);
            let mut buf = MockWrite::new();
            handle_status(&hal, &mut buf).await;
            // Extract JSON body (after the blank line separating headers)
            let resp = buf.as_str();
            let body = resp.split("\r\n\r\n").nth(1).unwrap();
            let val: serde_json::Value = serde_json::from_str(body).unwrap();
            assert_eq!(val["state"], "idle");
            assert!(val["errors"].is_array(), "errors key must be present");
            assert!(val["errors"].as_array().unwrap().is_empty());
        });
    }

    #[test]
    fn status_working_body_includes_job_fields() {
        block_on(async {
            let hal = MockStatusHal::new().with_state(RobotState::Working {
                job_id: "abc".to_string(),
                progress_pct: 50,
            });
            let mut buf = MockWrite::new();
            handle_status(&hal, &mut buf).await;
            let resp = buf.as_str();
            let body = resp.split("\r\n\r\n").nth(1).unwrap();
            let val: serde_json::Value = serde_json::from_str(body).unwrap();
            assert_eq!(val["state"], "working");
            assert_eq!(val["job_id"], "abc");
            assert_eq!(val["progress_pct"], 50);
            assert!(val["errors"].is_array());
        });
    }

    #[test]
    fn status_includes_errors() {
        block_on(async {
            let hal = MockStatusHal::new()
                .with_state(RobotState::Error {
                    code: "E001".to_string(),
                    message: "Test error".to_string(),
                    job_id: None,
                    recoverable: true,
                    recovery: RecoveryAction::CallResetErrors,
                    timeout_remaining_secs: None,
                })
                .with_errors(alloc::vec![test_error()]);
            let mut buf = MockWrite::new();
            handle_status(&hal, &mut buf).await;
            let resp = buf.as_str();
            let body = resp.split("\r\n\r\n").nth(1).unwrap();
            let val: serde_json::Value = serde_json::from_str(body).unwrap();
            assert!(resp.contains("HTTP/1.1 200"));
            assert_eq!(val["state"], "error");
            assert_eq!(val["errors"][0]["code"], "E001");
        });
    }
}
