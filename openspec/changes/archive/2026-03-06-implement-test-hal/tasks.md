## 1. Dependencies

- [x] 1.1 In `Cargo.toml`, uncomment `embedded-hal-mock = "0.7.2"` and move it to `[dev-dependencies]` (remove from `[dependencies]` if present)
- [x] 1.2 In `Cargo.toml`, uncomment `test-case = "3.2"` and ensure it is under `[dev-dependencies]`
- [x] 1.3 Run `cargo check` to confirm the dependency additions compile cleanly

## 2. Mock HAL module

- [x] 2.1 Create `src/hal/mock.rs` gated with `#[cfg(test)]`; add `#[cfg(test)] mod mock;` to `src/hal/mod.rs`
- [x] 2.2 Implement `MockControlHal` with transaction queue for `power(on: bool)`, `power_save(enabled: bool)`, `reset_errors()`
- [x] 2.3 Implement `MockStatusHal` with transaction queue for `state()` and `active_errors()`
- [x] 2.4 Implement `MockConfigHal` with transaction queue for `get_active_config()` and `update_active_config(cfg)`
- [x] 2.5 Implement `MockStorageHal` with transaction queue for `backup()` and `restore(cfg)`
- [x] 2.6 Implement `MockSensorHal` with transaction queue for `glass_state()` and `level_state()`
- [x] 2.7 Implement `MockDispenseHal` with transaction queue for `create_job(...)`, `list_jobs()`, `job_status(id)`, `cancel_job(id)`
- [x] 2.8 Implement `MockCleaningHal` with transaction queue for `start_cleaning()` and `stop_cleaning()`
- [x] 2.9 Add `done()` method to each mock that panics if unconsumed transactions remain
- [x] 2.10 Run `cargo test` — all existing 57 tests in `src/hal/tests.rs` must still pass

## 3. Refactor existing HAL unit tests to use MockHal

- [x] 3.1 Replace hand-rolled mock structs in `src/hal/tests.rs` with the new `Mock*Hal` types from `src/hal/mock.rs`
- [x] 3.2 Parameterise repetitive serialisation tests (e.g., `RobotState` variants) using `#[test_case]`
- [x] 3.3 Run `cargo test` — all tests must pass; count should be equal or greater

## 4. Server handler integration tests

- [x] 4.1 Add `#[cfg(test)]` block to `src/server/handlers/status.rs` — test that `handle_status_get` returns 200 with `"idle"` when mock returns `RobotState::Idle`
- [x] 4.2 Add `#[cfg(test)]` block to `src/server/handlers/control.rs` — test power-on and power-off paths (success and HAL error → 500)
- [x] 4.3 Add `#[cfg(test)]` block to `src/server/handlers/config.rs` — test GET config returns current config and PATCH config calls `update_active_config`
- [x] 4.4 Add `#[cfg(test)]` block to `src/server/handlers/sensors.rs` — test glass-present and no-glass states
- [x] 4.5 Add `#[cfg(test)]` block to `src/server/handlers/dispense.rs` — test successful job creation and job listing
- [x] 4.6 Add `#[cfg(test)]` block to `src/server/handlers/cleaning.rs` — test start and stop cleaning (success + error)
- [x] 4.7 Run `cargo test` — all handler tests must pass

## 5. Verify production builds are clean

- [x] 5.1 Run `cargo build` and confirm zero errors or mock-related warnings
- [x] 5.2 Run `cargo check --features esp32` and confirm zero errors
- [x] 5.3 Run `cargo fmt` and commit
