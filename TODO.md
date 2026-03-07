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
- **API.yaml schema gaps** — `GlassType` added; `LiquidCalibration` collapsed to
  `factor: f32`; `Config` flattened and aligned; `JobCreateRequest` gets required
  `size` field; `RobotState::Booting` removed. All Rust types and API schema now match.
- **Test HAL (mock infrastructure)** — `src/hal/mock.rs` provides `Mock*Hal` structs
  for all 7 HAL traits plus `MockWrite` and `MockPasswordHasher`. `test-case = "3.2"`
  and `embedded-hal-mock = "0.7.2"` active as dev-dependencies. Handler integration
  tests added to all 6 handler modules. 106 tests pass (`cargo test`).
- **Job queue and SSE wiring** — `JobCreated { job_id, queue_position }` return type
  for `DispenseHal::create_job`; `max_queue_depth` in `Capabilities`; job_id generation
  in `handle_create_job`; HTTP 503 on `QUEUE_FULL`; `SseServer` spawned as embassy task
  in `main.rs` (polls state/jobs at 500 ms, emits typed events over TCP).
- **Admin password authentication** — `PasswordHasher` trait with `hash`/`verify`;
  `admin_password` field in `RobotConfig`; Basic Auth validated for all admin routes
  (`PATCH /config`, power/cleaning/reset/reload-config); constant-time compare;
  `Esp32PasswordHasher` using `pbkdf2` + `sha2` (ESP32 feature); `StubPasswordHasher`
  in `main.rs`; `basicAuth` security scheme in `API.yaml`.
- **Admin config / storage redesign** — `AdminConfig` struct splits operator fields from
  `RobotConfig`; `BackupPayload { data, checksum, backed_up_at }` for portable snapshots;
  `StorageHal::backup` / `restore` replace the old `load_storage_config`/`store_storage_config`;
  `GET /config/backup` and `POST /config/restore` routes replace `/storage/config`;
  `POST /control/reload-config` removed; `RobotState::Provisioning` added; 503 gate for
  non-admin routes when provisioning; `version` moved into `Capabilities`; crate bumped to
  `v0.5.0`.
- **RAM-backed StorageHal** — `src/storage/ram.rs` provides `RamStorageHal` (holds
  `Option<AdminConfig>` in RAM); `backup()` serializes + CRC32-checksums; `restore()`
  stores the payload; pre-seeded default config (`token: "dev"`, `short`/`long` glass
  types); replaces `StubStorageHal` in `main.rs`.
- **Dispense size → volume scaling** — `handle_create_job` now resolves `size` →
  `GlassType.volume` (abstract operator-defined unit), normalizes recipe ratios
  (`amount_i = (r_i / Σr) × glass.volume`), builds `Vec<DispenseItem>` with
  pre-computed `amount: f32`, and passes it to `DispenseHal::create_job`. Returns
  HTTP 422 for unknown size or empty items. `max_total_parts` removed (redundant);
  `GlassType.volume_ml` renamed to `GlassType.volume`. Crate bumped to `v0.6.0`.
- **SSE job completion events** — `SseServer` poll loop now emits a terminal
  `job_update` (last-known state) when a job disappears from `list_jobs()`.
  Keep-alive timer resets on terminal events. `API.yaml` `/events` description
  updated to document terminal event behavior.
- **Glass-aware state machine (v0.6.0)** — `RobotState` is a data-carrying
  tagged-union enum with `WaitingForGlass`, `Working`, `DrinkReady`, `Error`
  variants carrying job context and timeout countdowns. `GlassWaitReason` and
  `RecoveryAction` enums added. Glass detection is HAL-internal; server layer
  no longer pre-checks glass presence. `JobState::Running` replaced by `Active`.
  Cleaning handler gates on `Idle`/`Provisioning`, returns 409 otherwise.
  `AdminConfig` gains `glass_wait_timeout_secs` (60s) and `drink_ready_timeout_secs`
  (300s). `Capabilities` gains `has_cancel_button` and `has_power_button`.

---



