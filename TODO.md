# TODO — CocktailBotHAL

Open implementation work after the code cleanup. The cleanup removed all legacy
Rocket 0.4 code and established the project structure around the async
embassy-net server and HAL trait interface.

---

## Blockers (project will not compile without these)

### Missing Cargo.toml dependencies

The following crates are used in source files but not listed in `Cargo.toml`.
Add correct versions for your target MCU platform:

- `embassy-net` (used in `src/server/mod.rs`)
- `embedded-io-async` (used in `src/server/mod.rs` and handler stubs)
- `embassy_time` (used in `src/hal/mod.rs`)

Embassy crates are released together — pick a consistent snapshot compatible
with your target (STM32, ESP32, RP2040, etc.).

### `http_smol` crate

`src/server/mod.rs` calls `http_smol::read_http_request` and
`http_smol::write_json`. This crate is not in `Cargo.toml` and does not
appear to be a published crate. Choose one approach:

- **Option A**: Write `src/server/http.rs` implementing bare HTTP/1.1 request
  parsing and JSON response writing directly with `embedded-io-async`.
  Remove the `http_smol` references from `src/server/mod.rs`.
- **Option B**: Find or publish a suitable `no_std` HTTP parsing crate
  and add it as a dependency.

### Entry point: `no_std` + async executor

`src/main.rs` currently contains a synchronous `fn main()` placeholder.
For an embedded target this must be replaced with an MCU-specific async
entry point, e.g.:

```rust
#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) { ... }
```

Add `embassy_executor` to `Cargo.toml` and add a `#[panic_handler]` and
global allocator appropriate for your target.

---

## Handler Implementations

All six modules under `src/server/handlers/` contain only stubs.
Implement each to match the contract in `API.yaml`.

### `status.rs` — GET /v1/status

Call `hal.status.state()` and `hal.status.active_errors()`, serialize as:
`{ "state": "<RobotState>", "errors": [...] }`, return HTTP 200.

### `control.rs` — POST /v1/control/*

- `handle_power`: parse `{ "on": bool }`, call `hal.control.power(on)`,
  return 202 on success or error JSON on failure.
- `handle_power_save`: parse `{ "enabled": bool }`, call
  `hal.control.power_save(enabled)`.
- `handle_reset`: call `hal.control.reset_errors()`.
- `handle_reload_config`: call `hal.control.reload_config()`.
- Wire `handle_power_save`, `handle_reset`, and `handle_reload_config` into
  the route dispatch table in `src/server/mod.rs` (only `handle_power` is
  currently dispatched).

### `config.rs` — GET/PATCH /v1/config, GET/POST /v1/storage/config

- `handle_config_get`: call `hal.config.get_active_config()`, serialize
  `RobotConfig` as JSON. Wire into dispatch (currently missing).
- `handle_config_patch`: deserialize `RobotConfig` from body, call
  `hal.config.update_active_config(cfg)`.
- `handle_storage_read`: call `hal.storage.load_storage_config()`, serialize.
- `handle_storage_write`: deserialize config + overwrite flag, call
  `hal.storage.store_storage_config(cfg, overwrite)`.

### `sensors.rs` — GET /v1/sensors/*

- `handle_glass`: call `hal.sensors.glass_state()`, serialize
  `GlassSensorState` as JSON.
- `handle_levels`: call `hal.sensors.level_state()`, serialize
  `Vec<LevelState>` as JSON.
- Wire both into `src/server/mod.rs` dispatch (currently neither is present).

### `dispense.rs` — POST/GET /v1/dispense/jobs[/{job_id}]

- `handle_create_job`: parse `JobCreateRequest` from body, call
  `hal.dispense.create_job(...)`, return 202 with `{ "job_id": "..." }`.
- `handle_list_jobs`: see design decision below.
- `handle_job_status`: extract `job_id` from path, call
  `hal.dispense.job_status(job_id)`.
- `handle_cancel_job`: extract `job_id` from path, call
  `hal.dispense.cancel_job(job_id)`.

### `cleaning.rs` — POST /v1/cleaning/start|stop

- `handle_start`: call `hal.cleaning.start_cleaning()`, return 202.
- `handle_stop`: call `hal.cleaning.stop_cleaning()`, return 202. Wire into
  `src/server/mod.rs` dispatch (currently missing).

---

## Route Dispatch Gaps in `src/server/mod.rs`

The following `API.yaml` routes are absent from the match block:

| Method | Path                          | Handler                          |
|--------|-------------------------------|----------------------------------|
| POST   | /v1/control/power-save        | `control::handle_power_save`     |
| POST   | /v1/control/reset             | `control::handle_reset`          |
| POST   | /v1/control/reload-config     | `control::handle_reload_config`  |
| GET    | /v1/config                    | `config::handle_config_get`      |
| GET    | /v1/sensors/glass             | `sensors::handle_glass`          |
| GET    | /v1/sensors/levels            | `sensors::handle_levels`         |
| GET    | /v1/dispense/jobs/{job_id}    | `dispense::handle_job_status`    |
| POST   | /v1/dispense/jobs/{job_id}    | `dispense::handle_cancel_job`    |
| POST   | /v1/cleaning/stop             | `cleaning::handle_stop`          |
| GET    | /v1/events                    | SSE handler (see below)          |

---

## Path Parameter Routing

The current `match (method, path)` in `src/server/mod.rs` cannot match
`/v1/dispense/jobs/{job_id}` where `job_id` is a dynamic segment.
Implement one approach before wiring those routes:

- Simple prefix check: `path.starts_with("/v1/dispense/jobs/")` then extract
  the tail as `job_id`.
- A segment-based router if more complex parameters arise later.

---

## `DispenseHal` — Missing `list_jobs` Method

`GET /v1/dispense/jobs` is specified in `API.yaml` but there is no
`list_jobs()` method on the `DispenseHal` trait. Decide and implement one
approach before writing the handler:

- Add `fn list_jobs(&self) -> Vec<JobStatus>` to `DispenseHal` in
  `src/hal/mod.rs`, or
- Maintain an internal job registry inside `ApiServer` that is updated by the
  create/cancel handlers.

---

## Server-Sent Events (SSE)

`GET /v1/events` is specified in `API.yaml`. No trait method or handler
exists for it. Design the event model before implementing:

- What events are emitted? (state changes, job completion, errors, etc.)
- How does the HAL signal events? (callback registration, polling, async channel)
- SSE over raw embassy TCP sockets requires `text/event-stream` content type
  and chunked transfer encoding.

---

## `StorageHal` Implementation

No concrete implementation of `StorageHal` exists anywhere. Provide at minimum
a RAM-backed stub that satisfies the trait for testing purposes, then replace
with a real implementation (EEPROM, flash sector, SD card, etc.) for production.

---

## Authentication

`API.yaml` declares Bearer token auth as a global security requirement.
The server does not currently parse `Authorization` headers or validate tokens.
Add token validation to `src/server/mod.rs` before dispatching to handlers.

---

## API.yaml Schema Gaps

The following fields present in Rust types are missing from the `API.yaml`
schemas:

- `Config.part_ml` (`f32`) — present in `RobotConfig` but absent from schema
- `Config.max_channels_per_job` (`u8`) — present in `RobotConfig` but absent
- `JobCreateRequest.require_glass` and `timeout` — parameters on
  `DispenseHal::create_job` but not in the request schema

---

## Testing

No automated tests exist. To add them:

1. Uncomment `test-case = "3.2"` in `[dev-dependencies]` in `Cargo.toml`.
2. Uncomment `embedded-hal-mock = "0.7.2"` to create mock HAL implementations
   for unit testing.
3. Add `#[cfg(test)]` modules to each handler file testing against mock HAL
   implementations.
4. Add integration tests that send HTTP requests to a test server.

---

## `generic_cocktail` Dependency

The local path dependency `generic_cocktail` at `../generic-cocktail` was only
used in the deleted legacy Rocket code. Evaluate whether it is still needed
in the new async server layer. If not, remove it from `Cargo.toml` to
eliminate the external path dependency requirement.
