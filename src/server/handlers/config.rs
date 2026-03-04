// src/server/handlers/config.rs
//
// Handlers for GET/PATCH /v1/config and GET/POST /v1/storage/config

use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::RobotConfig;
use crate::server::http::{self, HttpRequest};
use crate::server::RobotHal;

/// GET /v1/config — return active (RAM) configuration.
pub async fn handle_config_get<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
) {
    let cfg = hal.config.get_active_config();
    http::write_json(socket, 200, &serde_json::json!(cfg)).await.ok();
}

/// PATCH /v1/config — update active (RAM) configuration.
pub async fn handle_config_patch<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    request: &HttpRequest,
    socket: &mut W,
) {
    let cfg: RobotConfig = match http::parse_body(request) {
        Ok(c) => c,
        Err(_) => {
            http::write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid request body"}),
            )
            .await
            .ok();
            return;
        }
    };

    match hal.config.update_active_config(cfg) {
        Ok(()) => {
            http::write_json(
                socket,
                200,
                &serde_json::json!({"status": "updated"}),
            )
            .await
            .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/storage/config — read persistent configuration.
pub async fn handle_storage_read<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
) {
    match hal.storage.load_storage_config() {
        Ok(cfg) => {
            http::write_json(socket, 200, &serde_json::json!({"data": cfg}))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/storage/config — write persistent configuration.
/// Body: `{ "data": <RobotConfig>, "overwrite": bool }`
pub async fn handle_storage_write<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct Body {
        data: RobotConfig,
        #[serde(default)]
        overwrite: bool,
    }

    let body: Body = match http::parse_body(request) {
        Ok(b) => b,
        Err(_) => {
            http::write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid request body"}),
            )
            .await
            .ok();
            return;
        }
    };

    match hal.storage.store_storage_config(body.data, body.overwrite) {
        Ok(()) => {
            http::write_json(
                socket,
                200,
                &serde_json::json!({"success": true}),
            )
            .await
            .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
