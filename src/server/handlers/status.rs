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
