## 1. Dependency and Version Prep

- [x] 1.1 Check `Cargo.toml` for a commented-out `static-cell` entry; add `static-cell = "0.3"` (or latest) as a dependency if absent
- [x] 1.2 Bump `version` in `Cargo.toml` from `0.1.0` to `0.2.0`

## 2. HAL Trait Interface (src/hal/mod.rs)

- [x] 2.1 Add `async fn` to all four `ControlHal` methods (`power`, `power_save`, `reset_errors`, `reload_config`)
- [x] 2.2 Add `async fn` to both `StatusHal` methods (`state`, `active_errors`)
- [x] 2.3 Add `async fn` to both `ConfigHal` methods (`get_active_config`, `update_active_config`)
- [x] 2.4 Add `async fn` to both `StorageHal` methods (`load_storage_config`, `store_storage_config`)
- [x] 2.5 Add `async fn` to both `SensorHal` methods (`glass_state`, `level_state`)
- [x] 2.6 Add `async fn` to all four `DispenseHal` methods (`create_job`, `list_jobs`, `job_status`, `cancel_job`)
- [x] 2.7 Add `async fn` to both `CleaningHal` methods (`start_cleaning`, `stop_cleaning`)

## 3. Server Module — Generic RobotHal (src/server/mod.rs)

- [x] 3.1 Replace the seven `&mut dyn Trait` fields in `RobotHal` with seven owned generic type parameters (`Ctrl: ControlHal`, `Stat: StatusHal`, `Cfg: ConfigHal`, `Stor: StorageHal`, `Sens: SensorHal`, `Disp: DispenseHal`, `Clean: CleaningHal`)
- [x] 3.2 Update `ApiServer` to be generic over the same seven type parameters (or wrap `RobotHal<…>` directly)
- [x] 3.3 Verify the `tokens_equal` helper and `handle_connection` function in `server/mod.rs` still compile; fix any lifetime or borrow errors introduced by the generics change

## 4. Handler Call Sites (src/server/handlers/)

- [x] 4.1 Add `.await` to `hal.status.state()` and `hal.status.active_errors()` in `handlers/status.rs`
- [x] 4.2 Add `.await` to `hal.control.power()`, `hal.control.power_save()`, `hal.control.reset_errors()`, `hal.control.reload_config()` in `handlers/control.rs`
- [x] 4.3 Add `.await` to `hal.config.get_active_config()`, `hal.config.update_active_config()`, `hal.storage.load_storage_config()`, `hal.storage.store_storage_config()` in `handlers/config.rs`
- [x] 4.4 Add `.await` to `hal.sensors.glass_state()` and `hal.sensors.level_state()` in `handlers/sensors.rs`
- [x] 4.5 Add `.await` to `hal.dispense.create_job()`, `hal.dispense.list_jobs()`, `hal.dispense.job_status()`, `hal.dispense.cancel_job()` in `handlers/dispense.rs`
- [x] 4.6 Add `.await` to `hal.cleaning.start_cleaning()` and `hal.cleaning.stop_cleaning()` in `handlers/cleaning.rs`

## 5. Stub Implementations (src/main.rs)

- [x] 5.1 Add `async fn` to all `StubControlHal` method bodies
- [x] 5.2 Add `async fn` to all `StubStatusHal` method bodies
- [x] 5.3 Add `async fn` to all `StubConfigHal` method bodies
- [x] 5.4 Add `async fn` to all `StubStorageHal` method bodies
- [x] 5.5 Add `async fn` to all `StubSensorHal` method bodies
- [x] 5.6 Add `async fn` to all `StubDispenseHal` method bodies
- [x] 5.7 Add `async fn` to all `StubCleaningHal` method bodies

## 6. Async Entry Point (src/main.rs)

- [x] 6.1 Add `use static_cell::StaticCell;` and `use embassy_executor::Executor;` imports
- [x] 6.2 Replace the `fn main()` body with the `StaticCell<Executor>` + `executor.run(|spawner| …)` pattern
- [x] 6.3 Define an `#[embassy_executor::task]` async task (e.g., `async fn async_main(spawner: Spawner)`) that constructs all stub HAL instances, builds `RobotHal`, builds `ApiServer`, and calls `server.run(net_stack).await`
- [x] 6.4 Update the type alias (or inline type) for `RobotHal` in `main.rs` to use the concrete stub types instead of `dyn Trait` references

## 7. ESP32 Stub Implementations (src/esp32/)

- [x] 7.1 Add `async fn` to all `ControlHal` methods in `src/esp32/control.rs`
- [x] 7.2 Add `async fn` to all `StatusHal` methods in `src/esp32/status.rs`
- [x] 7.3 Add `async fn` to all `ConfigHal` methods in `src/esp32/config.rs`
- [x] 7.4 Add `async fn` to all `StorageHal` methods in `src/esp32/storage.rs`
- [x] 7.5 Add `async fn` to all `SensorHal` methods in `src/esp32/sensors.rs`
- [x] 7.6 Add `async fn` to all `DispenseHal` methods in `src/esp32/dispense.rs`
- [x] 7.7 Add `async fn` to all `CleaningHal` methods in `src/esp32/cleaning.rs`

## 8. Verification

- [x] 8.1 Run `cargo check` and fix all compilation errors
- [x] 8.2 Run `cargo check --features esp32` and fix any ESP32-specific errors
- [x] 8.3 Run `cargo fmt` and verify formatting is clean
- [x] 8.4 Confirm no `dyn ControlHal` / `dyn StatusHal` / etc. references remain in `src/server/mod.rs`
- [x] 8.5 Confirm no bare HAL calls (missing `.await`) remain in `src/server/handlers/`
