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
- `embassy_time` (used in `src/server/sse.rs` and `src/server/handlers/dispense.rs`)

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
- **Legacy Rocket 0.4 code** — `src/api/mod.rs` removed; `main.rs` no longer
  uses Rocket.
- **HAL unit tests** — 57 tests in `src/hal/tests.rs` with mock
  implementations for all 7 traits, including error injection and
  serialization roundtrip tests.
- **`core::time::Duration`** — replaced `embassy_time::Duration` in
  `src/hal/mod.rs` for portability (allows tests without embassy).
- **Test compilation** — `mod server` gated with `#[cfg(not(test))]` and
  `extern crate alloc` added to `main.rs`.
- **SSE server** — `src/server/sse.rs` implemented with polling-based
  change detection for robot state and job updates.

---

## Server-Sent Events (SSE) — Partially Done

`src/server/sse.rs` implements an SSE server on port 9000 that polls HAL
traits every 500ms for state and job changes. It emits `state_change` and
`job_update` events with 30-second keepalive comments.

Remaining work:

- Wire SSE into the main API server (currently a separate `SseServer` struct)
- Add error events (currently only tracks state and job changes)
- Consider an async channel approach instead of polling for lower latency

---

## Authentication

`API.yaml` declares Bearer token auth as a global security requirement.
The server does not currently parse `Authorization` headers or validate
tokens. Add token validation to `src/server/mod.rs` before dispatching to
handlers.

---

## `StorageHal` Implementation

No concrete (non-stub) implementation of `StorageHal` exists. The test suite
includes `MockStorageHal` (RAM-backed with overwrite semantics) which can
serve as a reference. A production implementation should target EEPROM, flash
sector, SD card, etc.

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

## Testing — Partially Done

**Completed:** `src/hal/tests.rs` contains 57 unit tests with mock
implementations for all 7 HAL traits. Run with `cargo test`. The `server`
module is gated with `#[cfg(not(test))]` so tests compile without embassy
dependencies. The HAL module uses `core::time::Duration` instead of
`embassy_time::Duration` for test portability.

Remaining work:

1. **Handler integration tests** — test HTTP request/response cycles against
   mock HAL implementations. Requires either abstracting the socket layer or
   adding `embedded-io-async` mock support.
2. **Parameterized tests** — uncomment `test-case = "3.2"` in
   `[dev-dependencies]` if needed for data-driven test cases.
3. **SSE server tests** — verify event emission and keepalive behavior.
