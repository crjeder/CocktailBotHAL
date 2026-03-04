// src/server/handlers/sensors.rs
//
// Handlers for GET /v1/sensors/glass and GET /v1/sensors/levels
//
// TODO: Wire both handlers into the route dispatch in src/server/mod.rs
// (currently neither sensors route is dispatched).

use embedded_io_async::Write;

use crate::server::RobotHal;

/// GET /v1/sensors/glass — glass presence and type detection.
/// TODO: Call hal.sensors.glass_state(), serialize GlassSensorState as JSON.
/// Response schema: { "present": bool, "glass_type": string|null,
///                    "confidence": float }
pub async fn handle_glass<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/sensors/glass")
}

/// GET /v1/sensors/levels — liquid level readings for all channels.
/// TODO: Call hal.sensors.level_state(), serialize Vec<LevelState> as JSON.
/// Each entry is either Binary { id, ok } or Decimal { id, remaining_ml }.
pub async fn handle_levels<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/sensors/levels")
}
