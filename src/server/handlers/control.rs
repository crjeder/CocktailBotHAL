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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockControlHal, MockWrite};
    use crate::server::http::HttpRequest;
    use futures::executor::block_on;

    fn power_request(on: bool) -> HttpRequest {
        HttpRequest {
            method: "POST".to_string(),
            path: "/v1/control/power".to_string(),
            headers: alloc::vec![],
            body: alloc::format!("{{\"on\":{}}}", on).into_bytes(),
        }
    }

    #[test]
    fn control_power_on_returns_202() {
        block_on(async {
            let mut hal = MockControlHal::new();
            let mut buf = MockWrite::new();
            handle_power(&mut hal, &power_request(true), &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 202"));
            assert!(hal.powered_on);
        });
    }

    #[test]
    fn control_power_off_returns_202() {
        block_on(async {
            let mut hal = MockControlHal::new();
            let mut buf = MockWrite::new();
            handle_power(&mut hal, &power_request(false), &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 202"));
        });
    }

    #[test]
    fn control_power_hal_error_returns_500() {
        block_on(async {
            let mut hal = MockControlHal::new();
            hal.fail_next = Some(test_error());
            let mut buf = MockWrite::new();
            handle_power(&mut hal, &power_request(true), &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }

    #[test]
    fn control_reset_returns_202() {
        block_on(async {
            let mut hal = MockControlHal::new();
            let mut buf = MockWrite::new();
            handle_reset(&mut hal, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 202"));
        });
    }
}
