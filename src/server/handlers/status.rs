// src/server/handlers/status.rs
//
// Handler for GET /v1/status

use embedded_io_async::Write;

use crate::hal::StatusHal;
use crate::server::http;

/// GET /v1/status — return current robot state and active errors.
///
/// Response: `{ "state": "<RobotState>", "errors": [...] }`
pub async fn handle_status<Stat: StatusHal, W: Write + Unpin>(status: &Stat, socket: &mut W) {
    let state = status.state().await;
    let errors = status.active_errors().await;

    http::write_json(
        socket,
        200,
        &serde_json::json!({
            "state": state,
            "errors": errors,
        }),
    )
    .await
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockStatusHal, MockWrite};
    use crate::hal::RobotState;
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
    fn status_includes_errors() {
        block_on(async {
            let hal = MockStatusHal::new()
                .with_state(RobotState::Error)
                .with_errors(alloc::vec![test_error()]);
            let mut buf = MockWrite::new();
            handle_status(&hal, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"));
            assert!(resp.contains("E001"));
        });
    }
}
