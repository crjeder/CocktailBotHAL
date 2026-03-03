// src/server/handlers/status.rs
//
// Handler for GET /v1/status
//
// TODO: Call hal.status.state() and hal.status.active_errors(), serialize
// both as JSON, and write an HTTP 200 response to the socket.
// Response schema: { "state": "<RobotState>", "errors": [...] }

use embedded_io_async::Write;

use crate::server::RobotHal;

/// GET /v1/status — return current robot state and active errors.
pub async fn handle_status<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/status")
}
