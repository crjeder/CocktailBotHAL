// src/server/handlers/control.rs
//
// Handlers for POST /v1/control/*
//
// TODO: Implement each handler. Wire handle_power_save, handle_reset, and
// handle_reload_config into the route dispatch in src/server/mod.rs
// (currently only handle_power is dispatched).

use embedded_io_async::Write;

use crate::server::RobotHal;

/// POST /v1/control/power — power the robot on or off.
/// Body: { "on": bool }
/// TODO: Parse body, call hal.control.power(on), return 202 or error JSON.
pub async fn handle_power<R, W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _request: R,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/control/power")
}

/// POST /v1/control/power-save — enter or exit power-save mode.
/// Body: { "enabled": bool }
/// TODO: Parse body, call hal.control.power_save(enabled), return 202.
pub async fn handle_power_save<R, W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _request: R,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/control/power-save")
}

/// POST /v1/control/reset — clear errors and return to idle.
/// TODO: Call hal.control.reset_errors(), return 202 or error JSON.
pub async fn handle_reset<W: Write + Unpin>(_hal: &mut RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement POST /v1/control/reset")
}

/// POST /v1/control/reload-config — reload config from persistent storage.
/// TODO: Call hal.control.reload_config(), return 202 or error JSON.
pub async fn handle_reload_config<W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/control/reload-config")
}
