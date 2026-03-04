// src/server/handlers/cleaning.rs
//
// Handlers for POST /v1/cleaning/start and POST /v1/cleaning/stop

use embedded_io_async::Write;

use crate::server::http;
use crate::server::RobotHal;

/// POST /v1/cleaning/start — begin a cleaning program.
pub async fn handle_start<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    socket: &mut W,
)
{
    match hal.cleaning.start_cleaning()
    {
        Ok(()) =>
        {
            http::write_accepted(socket).await.ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/cleaning/stop — stop a running cleaning program.
pub async fn handle_stop<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    socket: &mut W,
)
{
    match hal.cleaning.stop_cleaning()
    {
        Ok(()) =>
        {
            http::write_accepted(socket).await.ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
