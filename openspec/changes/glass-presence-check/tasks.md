## 1. HAL Type Definitions

- [x] 1.1 Add `GlassWaitReason` enum to `src/hal/mod.rs` with variants `NoGlass` and `TooSmall { detected_volume: f32, required_volume: f32 }`; derive `Debug`, `Clone`, `Serialize`
- [x] 1.2 Add `RecoveryAction` enum to `src/hal/mod.rs` with variants `PutGlassBack`, `RemoveGlass`, `CallResetErrors`, `None`; derive `Debug`, `Clone`, `Serialize`
- [x] 1.3 Replace flat `RobotState` enum with data-carrying variant form: `Off`, `SelfTest`, `Provisioning`, `Idle`, `Cleaning`, `WaitingForGlass { job_id, reason, timeout_remaining_secs }`, `Working { job_id, progress_pct }`, `DrinkReady { job_id, timeout_remaining_secs }`, `Error { code, message, job_id, recoverable, recovery, timeout_remaining_secs }`; remove `Prepared`
- [x] 1.4 Replace `JobState::Running` with `JobState::Active`
- [x] 1.5 Add `glass_wait_timeout_secs: u32` and `drink_ready_timeout_secs: u32` to `AdminConfig` with `#[serde(default)]` and sensible defaults (60 and 300)
- [x] 1.6 Add `has_cancel_button: bool` and `has_power_button: bool` to `Capabilities`

## 2. Fix Compile Errors

- [x] 2.1 Update `src/hal/mock.rs`: `MockStatusHal::state()` returns new `RobotState` variants; update `MockDispenseHal` for `JobState::Active`
- [x] 2.2 Update `src/main.rs` stubs: `StubStatusHal::state()` and `StubDispenseHal` match new types; add new `Capabilities` fields; add timeout fields to `AdminConfig` default
- [x] 2.3 Update `src/esp32/mod.rs` and `src/esp32/sensors.rs`: stub `glass_state()` returns `present: true, confidence: 0.0`; fix any `RobotState` pattern matches
- [x] 2.4 Update `src/esp32/config.rs`: add new `Capabilities` fields with stub values (`has_cancel_button: false`, `has_power_button: false`)
- [x] 2.5 Update `src/storage/ram.rs`: add timeout defaults to `AdminConfig` initialisation
- [x] 2.6 Run `cargo check` and fix any remaining compile errors

## 3. Server Layer Updates

- [x] 3.1 Update `src/server/handlers/status.rs`: serialise new `RobotState` tagged union using `#[serde(tag = "state", rename_all = "snake_case")]`
- [x] 3.2 Update `src/server/handlers/cleaning.rs`: gate `start_cleaning` on admin auth; return `409 Conflict` if state is not `Idle` or `Provisioning`; cancel all queued jobs on entry
- [x] 3.3 Update `src/server/sse.rs`: emit full `RobotState` payload in `state_change` events; add `glass_size_warning` event type

## 4. Tests

- [x] 4.1 Update existing `RobotState` serialisation tests in `src/hal/tests.rs` for new variant shapes
- [x] 4.2 Add tests for `GlassWaitReason` and `RecoveryAction` serialisation
- [x] 4.3 Add tests for `AdminConfig` timeout field defaults (missing fields deserialise to 60/300)
- [x] 4.4 Update `MockStatusHal` test helpers to cover `WaitingForGlass`, `Working`, `DrinkReady`, and `Error` variants
- [x] 4.5 Update status handler tests for new JSON shape (tagged union with payloads)
- [x] 4.6 Add cleaning handler test: `409` when not in `Idle`/`Provisioning`

## 5. API Spec and Version

- [x] 5.1 Update `API.yaml`: change `RobotState` schema to a `oneOf` tagged union matching all variants and their fields
- [x] 5.2 Update `API.yaml`: add `glass_wait_timeout_secs` and `drink_ready_timeout_secs` to `AdminConfig` schema
- [x] 5.3 Update `API.yaml`: add `has_cancel_button` and `has_power_button` to `Capabilities` schema
- [x] 5.4 Bump crate version in `Cargo.toml` from `0.5.0` to `0.6.0`

## 6. Finalise

- [x] 6.1 Run `cargo fmt`
- [x] 6.2 Run `cargo test` — all tests pass
- [x] 6.3 Run `cargo check --features esp32` — no warnings
- [x] 6.4 Commit with message `feat(hal): glass-aware state machine, v0.6.0 breaking`
