// src/server/handlers/control.rs
//
// Handlers for POST /v1/control/*

use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::ControlHal;
use crate::server::http::{self, HttpRequest};

/// POST /v1/control/power — power the robot on or off.
/// Body: `{ "on": bool }`
pub async fn handle_power<Ctrl: ControlHal, W: Write + Unpin>(
    control: &mut Ctrl,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct Body {
        on: bool,
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

    match control.power(body.on).await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/control/power-save — enter or exit power-save mode.
/// Body: `{ "enabled": bool }`
pub async fn handle_power_save<Ctrl: ControlHal, W: Write + Unpin>(
    control: &mut Ctrl,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct Body {
        enabled: bool,
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

    match control.power_save(body.enabled).await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/control/reset — clear errors and return to idle.
pub async fn handle_reset<Ctrl: ControlHal, W: Write + Unpin>(control: &mut Ctrl, socket: &mut W) {
    match control.reset_errors().await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/control/reload-config — reload config from persistent
/// storage.
pub async fn handle_reload_config<Ctrl: ControlHal, W: Write + Unpin>(
    control: &mut Ctrl,
    socket: &mut W,
) {
    match control.reload_config().await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
