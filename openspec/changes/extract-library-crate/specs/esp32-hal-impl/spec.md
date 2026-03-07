## MODIFIED Requirements

### Requirement: ControlHal stub implementation
`Esp32Control` SHALL implement `ControlHal`. All methods SHALL be declared
`async fn` and SHALL return `Ok(())` as a stub, with a `// TODO: wire to hardware`
comment in the body. The implementation SHALL live in `examples/esp32/control.rs`
and SHALL import HAL traits via `cocktail_bot_hal::hal::`.

#### Scenario: power stub returns Ok (async)
- **WHEN** `power(true)` or `power(false)` is awaited on an `Esp32Control`
- **THEN** the method returns `Ok(())`

#### Scenario: power_save stub returns Ok (async)
- **WHEN** `power_save(true)` or `power_save(false)` is awaited
- **THEN** the method returns `Ok(())`

#### Scenario: reset_errors stub returns Ok (async)
- **WHEN** `reset_errors()` is awaited
- **THEN** the method returns `Ok(())`

### Requirement: StatusHal stub implementation
`Esp32Status` SHALL implement `StatusHal`. `state()` and `active_errors()`
SHALL be declared `async fn`. `state()` SHALL return `RobotState::Idle` and
`active_errors()` SHALL return an empty `Vec`. The implementation SHALL live in
`examples/esp32/status.rs`.

#### Scenario: state returns Idle (async)
- **WHEN** `state()` is awaited on a freshly constructed `Esp32Status`
- **THEN** the method returns `RobotState::Idle`

#### Scenario: active_errors returns empty (async)
- **WHEN** `active_errors()` is awaited
- **THEN** the method returns an empty Vec

### Requirement: ConfigHal stub implementation
`Esp32Config` SHALL implement `ConfigHal`. Both methods SHALL be declared
`async fn`. `get_active_config()` SHALL return a default `RobotConfig` with an
empty liquids list and `token` set to an empty string.
`update_active_config()` SHALL return `Ok(())`. The implementation SHALL live in
`examples/esp32/config.rs`.

#### Scenario: get_active_config returns default config (async)
- **WHEN** `get_active_config()` is awaited
- **THEN** a `RobotConfig` is returned with `liquids` as an empty Vec and `token` as an empty string

#### Scenario: update_active_config returns Ok (async)
- **WHEN** `update_active_config(cfg)` is awaited with any config
- **THEN** the method returns `Ok(())`

### Requirement: StorageHal stub implementation
`Esp32Storage` SHALL implement `StorageHal`. Both methods SHALL be declared
`async fn` and SHALL return `Err` with an `ErrorInfo` indicating the feature is
not yet implemented. The implementation SHALL live in `examples/esp32/storage.rs`.

#### Scenario: backup returns NOT_IMPLEMENTED error (async)
- **WHEN** `backup()` is awaited
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

#### Scenario: restore returns NOT_IMPLEMENTED error (async)
- **WHEN** `restore(cfg)` is awaited
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

### Requirement: SensorHal stub implementation
`Esp32Sensors` SHALL implement `SensorHal`. Both methods SHALL be declared
`async fn`. `glass_state()` SHALL return a `GlassSensorState` with
`present: true` (optimistic default for no-sensor hardware).
`level_state()` SHALL return an empty `Vec`. The implementation SHALL live in
`examples/esp32/sensors.rs`.

#### Scenario: glass_state reports glass present (async)
- **WHEN** `glass_state()` is awaited
- **THEN** the method returns `Ok(GlassSensorState { present: true, ... })`

#### Scenario: level_state returns empty (async)
- **WHEN** `level_state()` is awaited
- **THEN** the method returns `Ok(vec![])`

### Requirement: DispenseHal stub implementation
`Esp32Dispense` SHALL implement `DispenseHal`. All four methods SHALL be
declared `async fn`. `create_job()` SHALL return `Ok` with a generated stub job
ID. `list_jobs()` and `job_status()` SHALL return empty/default values.
`cancel_job()` SHALL return `Ok(())`. The implementation SHALL live in
`examples/esp32/dispense.rs`.

#### Scenario: create_job returns a job id (async)
- **WHEN** `create_job(...)` is awaited with any valid parameters
- **THEN** the method returns `Ok(JobCreated { job_id, queue_position: 1 })`

#### Scenario: list_jobs returns empty (async)
- **WHEN** `list_jobs()` is awaited
- **THEN** the method returns an empty Vec

#### Scenario: cancel_job returns Ok (async)
- **WHEN** `cancel_job(job_id)` is awaited
- **THEN** the method returns `Ok(())`

### Requirement: CleaningHal stub implementation
`Esp32Cleaning` SHALL implement `CleaningHal`. Both `start_cleaning()` and
`stop_cleaning()` SHALL be declared `async fn` and SHALL return `Ok(())`.
The implementation SHALL live in `examples/esp32/cleaning.rs`.

#### Scenario: start_cleaning returns Ok (async)
- **WHEN** `start_cleaning()` is awaited
- **THEN** the method returns `Ok(())`

#### Scenario: stop_cleaning returns Ok (async)
- **WHEN** `stop_cleaning()` is awaited
- **THEN** the method returns `Ok(())`

### Requirement: esp32 example compiled as Cargo example
The ESP32 HAL implementation SHALL be a Cargo `[[example]]` named `esp32` at
`examples/esp32/main.rs` with `required-features = ["esp32"]`. It SHALL NOT be
compiled as part of the library crate. It SHALL compile with
`cargo build --example esp32 --features esp32`.

#### Scenario: esp32 example requires feature flag
- **WHEN** `cargo build --example esp32` is run without `--features esp32`
- **THEN** Cargo refuses to build the example with a missing required-feature error

#### Scenario: esp32 example builds with feature
- **WHEN** `cargo build --example esp32 --features esp32` is run
- **THEN** the build succeeds
