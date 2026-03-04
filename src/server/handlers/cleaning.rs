// src/server/handlers/cleaning.rs
//
// Handlers for POST /v1/cleaning/start and POST /v1/cleaning/stop

use embedded_io_async::Write;

use crate::hal::CleaningHal;
use crate::server::http;

/// POST /v1/cleaning/start — begin a cleaning program.
pub async fn handle_start<Clean: CleaningHal, W: Write + Unpin>(
    cleaning: &mut Clean,
    socket: &mut W,
) {
    match cleaning.start_cleaning().await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/cleaning/stop — stop a running cleaning program.
pub async fn handle_stop<Clean: CleaningHal, W: Write + Unpin>(
    cleaning: &mut Clean,
    socket: &mut W,
) {
    match cleaning.stop_cleaning().await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
