// src/server/handlers/status.rs
//
// Handler for GET /v1/status

use embedded_io_async::Write;

use crate::server::http;
use crate::server::RobotHal;

/// GET /v1/status — return current robot state and active errors.
///
/// Response: `{ "state": "<RobotState>", "errors": [...] }`
pub async fn handle_status<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
) {
    let state = hal.status.state();
    let errors = hal.status.active_errors();

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
