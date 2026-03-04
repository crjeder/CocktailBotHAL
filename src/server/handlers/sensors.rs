// src/server/handlers/sensors.rs
//
// Handlers for GET /v1/sensors/glass and GET /v1/sensors/levels

use embedded_io_async::Write;

use crate::server::http;
use crate::server::RobotHal;

/// GET /v1/sensors/glass — glass presence and type detection.
pub async fn handle_glass<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
)
{
    match hal.sensors.glass_state()
    {
        Ok(state) =>
        {
            http::write_json(
                socket,
                200,
                &serde_json::json!(state),
            )
            .await
            .ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/sensors/levels — liquid level readings for all channels.
pub async fn handle_levels<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
)
{
    match hal.sensors.level_state()
    {
        Ok(levels) =>
        {
            http::write_json(
                socket,
                200,
                &serde_json::json!(levels),
            )
            .await
            .ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
