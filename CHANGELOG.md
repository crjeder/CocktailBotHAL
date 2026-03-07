# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — esp32-async-executor
- `examples/esp32/main.rs`: replaced `fn main()` + `StaticCell<Executor>` spin-executor
  pattern with `#[esp_hal::main] async fn main(spawner: Spawner)` — the correct async
  entry point for ESP32 hardware using esp-hal
- `examples/esp32/main.rs`: removed `async_main` task and `EXECUTOR` static; executor
  is now managed by the `#[esp_hal::main]` macro
- `examples/esp32/main.rs`: added `esp_alloc::heap_allocator!(72 * 1024)` at module
  level; added `esp_hal::init()` and `esp_hal_embassy::init()` call stubs
- `examples/esp32/main.rs`: `ApiServer::run()` is now called (via `todo!()` placeholder)
  instead of being left in a comment
- `Cargo.toml`: added `esp-hal 0.23`, `esp-hal-embassy 0.6`, `esp-wifi 0.12`,
  `esp-alloc 0.5` as optional dependencies activated by the `esp32` feature
- `Cargo.toml`: moved `embassy-executor` to target-conditional sections to avoid
  `arch-spin` / ESP32 arch-feature conflict:
  - host / test builds: `arch-spin + executor-thread` (for the dev spin executor)
  - xtensa / riscv32 builds: no arch feature (executor provided by esp-hal-embassy)

### Build note
`examples/esp32` now requires cross-compilation:
```
cargo build --example esp32 --features esp32 --target xtensa-esp32s3-none-elf
```
(requires `espup install`; host-only builds with `--features esp32` are not supported)

## [0.6.0] - 2026-03-07

### Changed — extract-library-crate
- Crate converted from binary to library; `src/lib.rs` is now the crate root
- `src/esp32/` moved to `examples/esp32/` — a Cargo `[[example]]` compiled only with `--features esp32`; all `use crate::hal::` imports updated to `use cocktail_bot_hal::hal::`
- `src/storage/ram.rs` (`RamStorageHal`) moved to `examples/dev/storage.rs`; no longer exported from the library
- All stub HAL implementations and executor setup moved from `src/main.rs` to `examples/dev/main.rs`
- `src/esp32/mod.rs` (composite `Esp32Hal` delegation struct) removed; sub-structs wired directly in `examples/esp32/main.rs`
- `#![allow(dead_code)]` and embassy-time linker stubs (`_embassy_time_now`, `_embassy_time_schedule_wake`) moved to `src/lib.rs`
- `Cargo.toml`: added `[lib]`, `[[example]] name = "dev"`, `[[example]] name = "esp32" required-features = ["esp32"]`
- Development invocation changed from `cargo run` to `cargo run --example dev`

### Added — implement-test-hal (noted in [0.5.0], moved here for accuracy)
- `#[no_mangle]` no-op stubs `_embassy_time_now` and `_embassy_time_schedule_wake` now live in `src/lib.rs` under `#[cfg(test)]`

### Breaking Changes — glass-presence-check
- `RobotState` is now a data-carrying tagged-union enum serialised with `#[serde(tag="state")]`; the flat string-enum is removed
- `JobState::Running` renamed to `JobState::Active`
- `RobotState` no longer derives `Deserialize` or `Eq` (fields contain `f32`)
- `RobotState::Prepared` variant removed; terminal waiting state is now `DrinkReady`
- `DispenseHal::create_job` signature changed: `require_glass` and `timeout` parameters dropped; glass handling is state-machine-driven inside the HAL

### Added — glass-presence-check
- `GlassWaitReason` enum: `NoGlass`, `TooSmall { detected_volume, required_volume }`
- `RecoveryAction` enum: `PutGlassBack`, `RemoveGlass`, `CallResetErrors`, `None`
- `RobotState` data-carrying variants: `WaitingForGlass { job_id, reason, timeout_remaining_secs }`, `Working { job_id, progress_pct }`, `DrinkReady { job_id, timeout_remaining_secs }`, `Error { code, message, job_id, recoverable, recovery, timeout_remaining_secs }`
- `AdminConfig::glass_wait_timeout_secs` (default: 60) and `drink_ready_timeout_secs` (default: 300)
- `Capabilities::has_cancel_button` and `has_power_button` fields
- `glass-aware-state-machine` living spec documenting all valid state transitions, glass type validation rules, cleaning terminal state, and glass polling interval
- Cleaning handler returns `409 Conflict` if robot is not `Idle` or `Provisioning`
- SSE `emit_state_event` serialises the full tagged `RobotState` payload directly

### Changed — glass-presence-check
- `API.yaml`: `RobotState` modelled as `oneOf` tagged-union with `GlassWaitReason` and `RecoveryAction` sub-schemas; `JobStatus.state` value `running` replaced with `active`; timeout fields added to `AdminConfig` and `Config` schemas
- Status handler serialises `RobotState` via `serde_json::to_value` and merges `errors` key at top level
- ESP32 sensor stub `glass_state()` returns `present: true, confidence: 0.0` (optimistic; no real sensor in stub mode)

## [0.5.0] - 2026-03-06

### Breaking Changes — redesign-admin-config-storage
- `RobotConfig` split into `AdminConfig` (admin-owned, persisted) and `Capabilities` (hardware-fixed, read-only); `version` moves to `Capabilities`
- `StorageHal::load_storage_config` / `store_storage_config` replaced by `backup` / `restore` methods operating on `AdminConfig`
- `ControlHal::reload_config` removed; `restore` activates immediately, making a separate reload redundant
- `GET /storage/config` + `POST /storage/config` replaced by `GET /config/backup` and `POST /config/restore`

### Breaking Changes — implement-size-volume-scaling
- `GlassType.volume_ml` renamed to `GlassType.volume` (unit is operator-defined, not necessarily ml)
- `AdminConfig.max_total_parts` and `RobotConfig.max_total_parts` removed; the glass volume is the implicit cap
- `DispenseHal::create_job` `items` parameter type changes from `Vec<JobItem>` to `Vec<DispenseItem>` carrying pre-computed per-ingredient volumes

### Breaking Changes — add-admin-password-auth
- `ApiServer` / `RobotHal` gain an eighth generic parameter `Hasher: PasswordHasher`

### Added — redesign-admin-config-storage
- `RobotState::Provisioning` — entered on first boot when no config exists; only admin endpoints active until a successful restore
- `GET /config/backup` and `POST /config/restore` endpoints; backup payload includes checksum and timestamp
- Both `PATCH /config` and `POST /config/restore` auto-persist to flash and share a pre-flight: flush job queue, then wait for any active job to finish

### Added — ram-storage-hal
- `RamStorageHal`: concrete RAM-backed `StorageHal` for development and testing; pre-seeded with a default `AdminConfig` so the robot boots into `Idle` without real flash hardware (now lives in `examples/dev/storage.rs`)

### Added — add-admin-password-auth
- `PasswordHasher` trait with `hash` and `verify` async methods
- HTTP Basic Auth for all admin routes (`/config`, `/control`, `/cleaning`); Bearer token retained for non-admin routes
- `ADMIN_ROUTES` constant; `DEFAULT_ADMIN_PASSWORD` fallback
- `Esp32PasswordHasher` (`examples/esp32/hasher.rs`): PBKDF2-HMAC-SHA256 via `pbkdf2` + `sha2`; format `pbkdf2$<iters>$<salt_hex>$<dk_hex>`
- `StubPasswordHasher` in `examples/dev/main.rs`: constant-time XOR verify using `stub$<pw>` format for development
- `handle_config_get` redacts `admin_password` (always returns `""` to clients)
- `handle_config_patch` hashes non-empty `admin_password`; preserves existing hash when field is empty
- `API.yaml`: `basicAuth` security scheme; `admin_password` write-only field in Config schema; all nine admin routes annotated with `security: [{basicAuth: []}]`

### Added — implement-size-volume-scaling
- `DispenseItem { liquid_id: String, amount: f32 }` type in `hal/mod.rs`; server handler pre-computes per-ingredient volumes before calling HAL via `amount = (r_i / Σr) × glass.volume`
- Returns `HTTP 422` if the requested `size` is not found in `glass_types`

### Added — implement-test-hal
- Test mocks (`src/hal/mock.rs`): `MockControlHal`, `MockStatusHal`, `MockConfigHal`, `MockStorageHal`, `MockSensorHal`, `MockDispenseHal`, `MockCleaningHal`, `MockPasswordHasher`, `MockWrite`; each with stateful inspection fields, error injection (`fail_next`, `restore_fail`, `fail`), and `done()` assertions
- Handler integration tests in all six `src/server/handlers/*.rs` files
- `#[no_mangle]` no-op stubs `_embassy_time_now` and `_embassy_time_schedule_wake` under `#[cfg(test)]` to satisfy embassy-time linker requirements in test builds (now in `src/lib.rs`)
- `futures = "0.3"` dev-dependency (for `block_on` in async tests); `test-case = "3.2"` for parameterised enum serialisation tests

### Fixed — sse-job-completion
- SSE poll loop now emits a terminal `job_update` event for every job that disappears from `list_jobs()`; clients no longer get stuck showing a stale `Working 80%` state after a job completes

## [0.4.0] - 2026-03-05

### Breaking Changes — fix-api-schema-gaps
- `RobotConfig.part_ml` removed; cocktail volume is resolved at job-creation time from `glass_types` config
- `RobotConfig.max_channels_per_job` removed; `Capabilities.simultaneous_channels` already covers this constraint
- `LiquidCalibration` replaced with a single `factor: f32` hardware-agnostic multiplier
- `limits` nesting removed from `Config` schema; `max_total_parts` is a top-level field
- `State::booting` removed from the state enum; the HTTP server does not respond during boot so this state is unreachable by any API client
- `JobCreateRequest`: `require_glass` and `timeout` fields removed (server policy, not per-job client parameters)

### Added — fix-api-schema-gaps
- `glass_types: Vec<GlassType>` added to `RobotConfig`; each entry maps a size name (`short`, `medium`, `long`) to a volume
- `size` field (required, enum `short | medium | long`) added to `JobCreateRequest`; server resolves volume from `glass_types`
- `cocktail-sizing` capability: glass-size-driven volume scaling; `part_ml = glass_volume / total_parts` computed per job
- `Capabilities` exposed via `GET /capabilities` endpoint

## [0.3.0] - 2026-03-05

### Breaking Changes — job-queue-and-sse-wiring
- `client_job_id` field renamed to `name` on `JobStatus`, `JobCreated`, and in SSE events
- `DispenseHal::create_job` return type changed from `Result<String, ErrorInfo>` to `Result<JobCreated, ErrorInfo>`; new `JobCreated { job_id, queue_position }` struct

### Added — job-queue-and-sse-wiring
- Bounded job queue; `Capabilities.max_queue_depth` declares capacity; `create_job` returns `HTTP 503` when queue is full
- Deterministic `job_id` generation: `<name>-<DD><MM*3><time_1_10s_hex>` — unique within 24 h, no random source required
- SSE server on port 9000 (`src/server/sse.rs`) for real-time robot state events; spawned as a dedicated embassy task
- `state_change` and `job_update` SSE events with typed payloads
- `API.yaml`: `Capabilities`, `JobCreateRequest`, `JobCreateResponse`, `JobStatus` schemas updated

## [0.2.0] - 2026-03-04

### Breaking Changes — replace-sync-with-async / replace-legacy-web-framework
- All methods on the seven HAL traits become `async fn`; any HAL implementor must update all method signatures
- Rocket 0.4 removed; `src/api/mod.rs` deleted; `src/server/mod.rs` (embassy-net) is now the sole binary entry point

### Added — replace-sync-with-async
- `async-hal-traits` capability: HAL trait interface fully async, compatible with embassy and no-std async runtimes
- `embassy-net`, `embassy-time`, `embedded-io-async` dependencies; embassy spin executor in `main.rs` via `StaticCell<Executor>`

### Added — replace-legacy-web-framework
- `async-http-server` capability: all `API.yaml` v1 routes wired to handler sub-modules with Bearer token auth under embassy-net

### Removed — replace-legacy-web-framework
- `rocket` and `rocket_contrib` dependencies
- `src/api/mod.rs` (second Rocket entry point)

## [0.1.0] - 2026-03-04

### Added
- Core HAL trait definitions (`src/hal/mod.rs`): `ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`, `SensorHal`, `DispenseHal`, `CleaningHal` — the public vendor contract
- Key data types: `RobotState`, `RobotConfig`, `Capabilities`, `GlassState`, `LevelReading`, `LevelReporting`, `IngredientItem`, `JobCreated`
- ESP32 stub implementations (`src/esp32/`): all seven HAL traits as `no_std` + `alloc` stubs, gated behind the `esp32` Cargo feature
- Bearer token authentication: `Authorization: Bearer <token>` validated on every request; `token` field in `RobotConfig`; `HTTP 401` returned before any HAL call on failure
- Handler stubs for all OpenAPI routes: `/status`, `/capabilities`, `/config`, `/control`, `/dispense`, `/cleaning`, `/sensors`
- `API.yaml`: initial OpenAPI 3.1.0 specification
- `openspec/` directory with living specs and AI-assisted change workflow
- `testdata/` sample cocktail recipes for manual API testing
- `TODO.md` tracking open work items

[Unreleased]: https://github.com/crjeder/CocktailBotHAL/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/crjeder/CocktailBotHAL/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/crjeder/CocktailBotHAL/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/crjeder/CocktailBotHAL/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/crjeder/CocktailBotHAL/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/crjeder/CocktailBotHAL/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/crjeder/CocktailBotHAL/releases/tag/v0.1.0
