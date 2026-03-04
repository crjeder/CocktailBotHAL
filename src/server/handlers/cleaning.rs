// src/server/handlers/cleaning.rs
//
// Handlers for POST /v1/cleaning/start and POST /v1/cleaning/stop
//
// TODO: Wire handle_stop into the route dispatch in src/server/mod.rs
// (currently only handle_start is dispatched).

use embedded_io_async::Write;

use crate::server::RobotHal;

/// POST /v1/cleaning/start — begin a cleaning program.
/// TODO: Call hal.cleaning.start_cleaning(), return 202 or error JSON.
pub async fn handle_start<W: Write + Unpin>(_hal: &mut RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement POST /v1/cleaning/start")
}

/// POST /v1/cleaning/stop — stop a running cleaning program.
/// TODO: Call hal.cleaning.stop_cleaning(), return 202 or error JSON.
pub async fn handle_stop<W: Write + Unpin>(_hal: &mut RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement POST /v1/cleaning/stop")
}
