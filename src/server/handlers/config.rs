// src/server/handlers/config.rs
//
// Handlers for GET/PATCH /v1/config and GET/POST /v1/storage/config

use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::{ConfigHal, PasswordHasher, RobotConfig, StorageHal};
use crate::server::http::{self, HttpRequest};

/// GET /v1/config — return active (RAM) configuration.
///
/// `admin_password` is always redacted (returned as `""`) so the stored hash
/// is not exposed to non-admin clients.
pub async fn handle_config_get<Cfg: ConfigHal, W: Write + Unpin>(config: &Cfg, socket: &mut W) {
    let mut cfg = config.get_active_config().await;
    cfg.admin_password = alloc::string::String::new();
    http::write_json(socket, 200, &serde_json::json!(cfg))
        .await
        .ok();
}

/// PATCH /v1/config — update active (RAM) configuration.
///
/// If `admin_password` is non-empty in the request body it is treated as a new
/// plaintext password: it is hashed by `hasher` before being stored.
/// If `admin_password` is empty the current hash is preserved unchanged.
pub async fn handle_config_patch<Cfg: ConfigHal, H: PasswordHasher, W: Write + Unpin>(
    config: &mut Cfg,
    hasher: &H,
    request: &HttpRequest,
    socket: &mut W,
) {
    let mut cfg: RobotConfig = match http::parse_body(request) {
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

    if cfg.admin_password.is_empty() {
        // Preserve the current stored hash — client sent "" meaning "keep current password".
        cfg.admin_password = config.get_active_config().await.admin_password;
    } else {
        // Hash the new plaintext password before storing.
        match hasher.hash(&cfg.admin_password) {
            Ok(hash) => {
                cfg.admin_password = hash;
            }
            Err(e) => {
                http::write_hal_error(socket, &e).await.ok();
                return;
            }
        }
    }

    match config.update_active_config(cfg).await {
        Ok(()) => {
            http::write_json(socket, 200, &serde_json::json!({"status": "updated"}))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/storage/config — read persistent configuration.
pub async fn handle_storage_read<Stor: StorageHal, W: Write + Unpin>(
    storage: &Stor,
    socket: &mut W,
) {
    match storage.load_storage_config().await {
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
pub async fn handle_storage_write<Stor: StorageHal, W: Write + Unpin>(
    storage: &mut Stor,
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

    match storage
        .store_storage_config(body.data, body.overwrite)
        .await
    {
        Ok(()) => {
            http::write_json(socket, 200, &serde_json::json!({"success": true}))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
