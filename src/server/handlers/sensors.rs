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
