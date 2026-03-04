// src/server/handlers/config.rs
//
// Handlers for GET/PATCH /v1/config and GET/POST /v1/storage/config
//
// TODO: Wire handle_config_get into the route dispatch in src/server/mod.rs
// (currently only handle_config_patch, handle_storage_read, and
// handle_storage_write are dispatched).

use embedded_io_async::Write;

use crate::server::RobotHal;

/// GET /v1/config — return active (RAM) configuration.
/// TODO: Call hal.config.get_active_config(), serialize RobotConfig as JSON.
pub async fn handle_config_get<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/config")
}

/// PATCH /v1/config — update active (RAM) configuration.
/// TODO: Deserialize RobotConfig from body, call
/// hal.config.update_active_config(cfg), return 200 or error JSON.
pub async fn handle_config_patch<R, W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _request: R,
    _socket: &mut W,
)
{
    todo!("Implement PATCH /v1/config")
}

/// GET /v1/storage/config — read persistent configuration.
/// TODO: Call hal.storage.load_storage_config(), serialize result as JSON.
pub async fn handle_storage_read<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/storage/config")
}

/// POST /v1/storage/config — write persistent configuration.
/// TODO: Deserialize RobotConfig and overwrite flag from body, call
/// hal.storage.store_storage_config(cfg, overwrite), return 200 or error.
pub async fn handle_storage_write<R, W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _request: R,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/storage/config")
}
