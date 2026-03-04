# TODO — CocktailBotHAL

Open implementation work. The handler stubs have been replaced with real
implementations, `http_smol` has been replaced with `src/server/http.rs`,
all API routes are wired, and `list_jobs()` has been added to `DispenseHal`.

---

## Blockers (project will not compile without these)

### Missing Cargo.toml dependencies

The following crates are used in source files but not listed in `Cargo.toml`.
Add correct versions for your target MCU platform:

- `embassy-net` (used in `src/server/mod.rs`)
- `embedded-io-async` (used in `src/server/mod.rs`, `src/server/http.rs`,
  and all handler modules)
- `embassy_time` (used in `src/hal/mod.rs` and `src/server/handlers/dispense.rs`)

Embassy crates are released together — pick a consistent snapshot compatible
with your target (STM32, ESP32, RP2040, etc.).

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

## Authentication

`API.yaml` declares Bearer token auth as a global security requirement.
The server does not currently parse `Authorization` headers or validate
tokens. Add token validation to `src/server/mod.rs` before dispatching to
handlers.

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
