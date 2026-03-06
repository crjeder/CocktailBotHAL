// src/server/handlers/sensors.rs
//
// Handlers for GET /v1/sensors/glass and GET /v1/sensors/levels

use embedded_io_async::Write;

use crate::hal::SensorHal;
use crate::server::http;

/// GET /v1/sensors/glass — glass presence and type detection.
pub async fn handle_glass<Sens: SensorHal, W: Write + Unpin>(sensors: &Sens, socket: &mut W) {
    match sensors.glass_state().await {
        Ok(state) => {
            http::write_json(socket, 200, &serde_json::json!(state))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/sensors/levels — liquid level readings for all channels.
pub async fn handle_levels<Sens: SensorHal, W: Write + Unpin>(sensors: &Sens, socket: &mut W) {
    match sensors.level_state().await {
        Ok(levels) => {
            http::write_json(socket, 200, &serde_json::json!(levels))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockSensorHal, MockWrite};
    use futures::executor::block_on;

    #[test]
    fn sensors_glass_returns_200_when_absent() {
        block_on(async {
            let hal = MockSensorHal::new();
            let mut buf = MockWrite::new();
            handle_glass(&hal, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"));
            assert!(resp.contains("\"present\":false"));
        });
    }

    #[test]
    fn sensors_glass_returns_200_when_present() {
        block_on(async {
            let hal = MockSensorHal::new().with_glass(true);
            let mut buf = MockWrite::new();
            handle_glass(&hal, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"));
            assert!(resp.contains("\"present\":true"));
        });
    }

    #[test]
    fn sensors_glass_hal_error_returns_500() {
        block_on(async {
            let hal = MockSensorHal {
                fail: Some(test_error()),
                ..MockSensorHal::new()
            };
            let mut buf = MockWrite::new();
            handle_glass(&hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }

    #[test]
    fn sensors_levels_returns_200() {
        block_on(async {
            let hal = MockSensorHal::new();
            let mut buf = MockWrite::new();
            handle_levels(&hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 200"));
        });
    }
}
