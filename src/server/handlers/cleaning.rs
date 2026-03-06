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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockCleaningHal, MockWrite};
    use futures::executor::block_on;

    #[test]
    fn cleaning_start_returns_202() {
        block_on(async {
            let mut hal = MockCleaningHal::new();
            let mut buf = MockWrite::new();
            handle_start(&mut hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 202"));
            assert!(hal.cleaning);
        });
    }

    #[test]
    fn cleaning_stop_returns_202() {
        block_on(async {
            let mut hal = MockCleaningHal::new();
            hal.cleaning = true;
            let mut buf = MockWrite::new();
            handle_stop(&mut hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 202"));
            assert!(!hal.cleaning);
        });
    }

    #[test]
    fn cleaning_start_hal_error_returns_500() {
        block_on(async {
            let mut hal = MockCleaningHal::new();
            hal.fail_next = Some(test_error());
            let mut buf = MockWrite::new();
            handle_start(&mut hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }

    #[test]
    fn cleaning_stop_hal_error_returns_500() {
        block_on(async {
            let mut hal = MockCleaningHal::new();
            hal.fail_next = Some(test_error());
            let mut buf = MockWrite::new();
            handle_stop(&mut hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }
}
