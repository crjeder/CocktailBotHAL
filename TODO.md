# TODO — CocktailBotHAL



---

## Blockers (project will not compile on target without these)

### Entry point: ESP32 async executor

`src/main.rs` currently runs an embassy spin executor (development mode)
with stub HAL implementations. For ESP32 deployment, replace the entry
point with the BSP-provided async entry point (see the detailed TODO
comment in `src/main.rs`). Use `esp-hal` + `esp-hal-embassy` — not
`#[embassy_executor::main]`, which is only available for
cortex-m/riscv32/avr targets, not arch-spin/arch-std.

Add `esp-hal`, `esp-hal-embassy`, `esp-wifi` to `Cargo.toml` and
add a `#[panic_handler]` and global allocator for the ESP32 target.

---

## Completed

The following items have been implemented and are no longer open:

- **`http_smol` replacement** — `src/server/http.rs` provides
  `read_http_request()`, `write_json()`, `write_accepted()`,
  `write_hal_error()`, and `parse_body()` using `embedded-io-async`.
- **All handler implementations** — every handler in `src/server/handlers/`
  now calls the corresponding HAL trait methods and writes proper HTTP
  responses.
- **Route dispatch** — all `API.yaml` routes (except SSE and auth) are wired
  in `src/server/mod.rs`, including dynamic path extraction for
  `/v1/dispense/jobs/{job_id}`.
- **`DispenseHal::list_jobs()`** — added to the trait in `src/hal/mod.rs`
  and stubbed in `main.rs`.
- **Serde derives** — all HAL types now derive `Serialize` (and
  `Deserialize` where needed) for JSON serialization.
- **`generic_cocktail` dependency** — removed from `Cargo.toml` (no longer
  used after legacy Rocket code removal).
- **Bearer token authentication** — `Authorization: Bearer <token>` validated
  in `handle_connection` before dispatch; constant-time comparison; token
  configurable via `RobotConfig::token`; falls back to compile-time default.
- **Embassy dependencies** — `embassy-net 0.8.0`, `embassy-time 0.5.0`,
  `embedded-io-async 0.7.0` added to `Cargo.toml`.
---

## Server-Sent Events (SSE)

`GET /v1/events` is specified in `API.yaml`. No trait method or handler
exists for it. Design the event model before implementing:

- What events are emitted? (state changes, job completion, errors, etc.)
- How does the HAL signal events? (callback registration, polling, async
  channel)
- SSE over raw embassy TCP sockets requires `text/event-stream` content type
  and chunked transfer encoding.

---

## `StorageHal` Implementation

No concrete implementation of `StorageHal` exists anywhere. Provide at
minimum a RAM-backed stub that satisfies the trait for testing purposes,
then replace with a real implementation (EEPROM, flash sector, SD card,
etc.) for production.

---

## API.yaml Schema Gaps

The following fields present in Rust types are missing from the `API.yaml`
schemas:

- `Config.part_ml` (`f32`) — present in `RobotConfig` but absent from
  schema
- `Config.max_channels_per_job` (`u8`) — present in `RobotConfig` but
  absent
- `JobCreateRequest.require_glass` and `timeout` — parameters on
  `DispenseHal::create_job` but not in the request schema

---

## Testing

No automated tests exist. To add them:

1. Uncomment `test-case = "3.2"` in `[dev-dependencies]` in `Cargo.toml`.
2. Uncomment `embedded-hal-mock = "0.7.2"` to create mock HAL
   implementations for unit testing.
3. Add `#[cfg(test)]` modules to each handler file testing against mock HAL
   implementations.
4. Add integration tests that send HTTP requests to a test server.
