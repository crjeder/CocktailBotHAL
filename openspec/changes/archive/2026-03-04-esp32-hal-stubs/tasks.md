## 1. Project Configuration

- [x] 1.1 Add `[features] esp32 = []` section to `Cargo.toml`
- [x] 1.2 Gate `mod esp32;` in `src/main.rs` (or `src/lib.rs`) behind `#[cfg(feature = "esp32")]`

## 2. Module Scaffold

- [x] 2.1 Create `src/esp32/mod.rs` with `pub struct Esp32Hal` and `Esp32Hal::new()` constructor
- [x] 2.2 Declare sub-modules in `src/esp32/mod.rs`: `control`, `status`, `config`, `storage`, `sensors`, `dispense`, `cleaning`
- [x] 2.3 Create `src/esp32/control.rs` with `pub struct Esp32Control`
- [x] 2.4 Create `src/esp32/status.rs` with `pub struct Esp32Status`
- [x] 2.5 Create `src/esp32/config.rs` with `pub struct Esp32Config`
- [x] 2.6 Create `src/esp32/storage.rs` with `pub struct Esp32Storage`
- [x] 2.7 Create `src/esp32/sensors.rs` with `pub struct Esp32Sensors`
- [x] 2.8 Create `src/esp32/dispense.rs` with `pub struct Esp32Dispense`
- [x] 2.9 Create `src/esp32/cleaning.rs` with `pub struct Esp32Cleaning`

## 3. Trait Implementations

- [x] 3.1 Implement `ControlHal` for `Esp32Control` — all methods return `Ok(())` with `// TODO: wire to hardware`
- [x] 3.2 Implement `StatusHal` for `Esp32Status` — `state()` returns `RobotState::Idle`, `active_errors()` returns `vec![]`
- [x] 3.3 Implement `ConfigHal` for `Esp32Config` — `get_active_config()` returns a default `RobotConfig`, `update_active_config()` returns `Ok(())`
- [x] 3.4 Implement `StorageHal` for `Esp32Storage` — both methods return `Err(ErrorInfo { code: "NOT_IMPLEMENTED".into(), ... })`
- [x] 3.5 Implement `SensorHal` for `Esp32Sensors` — `glass_state()` returns `Ok(GlassSensorState { present: false, glass_type: None, confidence: 0.0 })`, `level_state()` returns `Ok(vec![])`
- [x] 3.6 Implement `DispenseHal` for `Esp32Dispense` — `create_job()` returns `Ok("stub-job-0".into())`, `list_jobs()` returns `vec![]`, `job_status()` returns `Err`, `cancel_job()` returns `Ok(())`
- [x] 3.7 Implement `CleaningHal` for `Esp32Cleaning` — both methods return `Ok(())`

## 4. Composite Struct Delegation

- [x] 4.1 Add sub-struct fields to `Esp32Hal` (`control: Esp32Control`, `status: Esp32Status`, etc.)
- [x] 4.2 Implement all 7 traits on `Esp32Hal` by delegating to the corresponding sub-struct field
- [x] 4.3 Verify `Esp32Hal::new()` constructs all sub-structs with default values

## 5. Build Verification

- [x] 5.1 Run `cargo build --features esp32` and confirm zero errors and zero warnings
- [x] 5.2 Run `cargo build` (no feature) and confirm existing tests still pass
- [x] 5.3 Run `cargo test` and confirm all 57 existing tests still pass
