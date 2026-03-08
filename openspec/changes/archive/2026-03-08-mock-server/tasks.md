## 1. Scaffold and Cargo Setup

- [x] 1.1 Create `examples/mock-server/` directory with `main.rs` skeleton (CLI arg parsing for `--port`, `--glass-present`, `--glass-absent`, `--dispense-duration-secs`)
- [x] 1.2 Add `[[example]] name = "mock-server"` to `Cargo.toml`
- [x] 1.3 Verify `cargo check --example mock-server` compiles with no errors

## 2. Shared State Type

- [x] 2.1 Define `MockState` struct in `examples/mock-server/state.rs` with fields: `robot_state: RobotState`, `errors: Vec<RobotError>`, `glass: GlassSensorState`, `config: RobotConfig`, `active_job: Option<MockJob>`, `cleaning_remaining_secs: Option<u32>`
- [x] 2.2 Define `MockJob` struct with `job_id`, `name`, `items`, `progress_percent: u8`, `state: JobState`, `started_at_tick: u64`
- [x] 2.3 Wrap `MockState` in `Arc<Mutex<MockState>>` and pass clones to each HAL struct
- [x] 2.4 Implement `MockState::new(glass: GlassSensorState, dispense_duration_secs: u32) -> Self` with `RobotState::Booting` as initial state

## 3. State Machine Ticker Task

- [x] 3.1 Implement an async embassy task `ticker_task` that loops with `embassy_time::Timer::after(Duration::from_millis(100))` and calls `MockState::tick()`
- [x] 3.2 Implement `MockState::tick()` to advance boot sequence: `Booting` → `SelfTest` (after 1s) → `Idle` (after 2s)
- [x] 3.3 Implement `MockState::tick()` to advance active dispense job: increment `progress_percent` linearly, transition to `DrinkReady` at 100%
- [x] 3.4 Implement `MockState::tick()` to advance cleaning: decrement `cleaning_remaining_secs`, transition back to `Idle` at 0
- [x] 3.5 Verify time driver advances on host (check embassy-executor spin + embassy-time integration); document finding in design.md if a workaround is needed

## 4. HAL Trait Implementations

- [x] 4.1 Implement `MockStatus` in `examples/mock-server/status.rs`: `robot_state()` reads from `MockState`, `active_errors()` returns `MockState.errors`
- [x] 4.2 Implement `MockControl` in `examples/mock-server/control.rs`: `power_on/off` set state; `reset_errors` clears errors + transitions to `Idle`; `reload_config` is a no-op
- [x] 4.3 Implement `MockConfig` in `examples/mock-server/config.rs`: `get_config()` / `update_config()` read/write `MockState.config` in RAM
- [x] 4.4 Implement `MockStorage` in `examples/mock-server/storage.rs`: `load_config()` / `store_config()` operate on an in-memory copy (not persistent)
- [x] 4.5 Implement `MockSensors` in `examples/mock-server/sensors.rs`: `glass_sensor()` reads `MockState.glass`; `liquid_levels()` returns plausible static values
- [x] 4.6 Implement `MockDispense` in `examples/mock-server/dispense.rs`: `create_job()` validates state (must be `Idle`/`Prepared`), inserts `MockJob`, transitions state to `Working`; `job_status()` reads from state; `cancel_job()` marks job cancelled and transitions back to `Idle`
- [x] 4.7 Implement `MockCleaning` in `examples/mock-server/cleaning.rs`: `start_cleaning()` validates state is `Idle`, sets `cleaning_remaining_secs`, transitions to `Cleaning`; `stop_cleaning()` clears and returns to `Idle`
- [x] 4.8 Implement `MockPasswordHasher` in `examples/mock-server/hasher.rs` (can reuse the PBKDF2 impl from `examples/esp32/hasher.rs` or use a trivial equality check for test convenience)

## 5. Mock Control Endpoint

- [x] 5.1 Add a `POST /mock/control` route handler outside of `ApiServer` (injected at the TCP dispatch layer or via a wrapper that intercepts the path before passing to `ApiServer`)
- [x] 5.2 Handle `{ "glass": "present" | "absent" }` to update `MockState.glass`
- [x] 5.3 Handle `{ "inject_error": "<code>" }` to push an error into `MockState.errors` and set state to `Error`
- [x] 5.4 Return `{ "ok": true, "mock": true }` from all successful control requests

## 6. Wire Everything in main.rs

- [x] 6.1 Parse CLI args and construct `MockState` with initial values
- [x] 6.2 Instantiate all Mock* HAL structs sharing the same `Arc<Mutex<MockState>>`
- [x] 6.3 Spawn `ticker_task` on the embassy executor
- [x] 6.4 Bind TCP listener and start `ApiServer::run` (reuse pattern from `examples/dev/main.rs`)
- [x] 6.5 Intercept `/mock/` paths before delegating to `ApiServer`

## 7. Verification

- [x] 7.1 `cargo run --example mock-server` starts and prints listen address
- [x] 7.2 `GET /v1/status` returns valid `RobotState` JSON after boot sequence completes
- [x] 7.3 Full dispense flow: `POST /v1/dispense` → poll status → observe `Working` → `DrinkReady`
- [x] 7.4 Cleaning flow: `POST /v1/cleaning/start` → observe `Cleaning` → `Idle`
- [x] 7.5 Glass control: `POST /mock/control {"glass":"present"}` → `GET /v1/sensors/glass` returns `present`
- [x] 7.6 Error injection: `POST /mock/control {"inject_error":"E_OVERFLOW"}` → status shows `Error` → `POST /v1/control/reset-errors` → returns to `Idle`
- [ ] 7.7 SSE: connect to `GET /v1/events`, trigger a state change, observe event emitted _(deferred: `/v1/events` is served by SseServer, not ApiServer; requires separate wiring)_
- [x] 7.8 `cargo fmt` passes with no changes
- [x] 7.9 `cargo check` and `cargo check --features esp32` both clean
