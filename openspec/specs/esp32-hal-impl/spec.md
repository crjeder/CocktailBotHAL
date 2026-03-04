## Requirements

### Requirement: ESP32 HAL module exists under feature flag
The crate SHALL expose a `src/esp32/` module that is compiled only when the
`esp32` Cargo feature is enabled. The module SHALL implement all 7 HAL traits
from `src/hal/mod.rs`.

#### Scenario: Module absent without feature
- **WHEN** the crate is compiled without the `esp32` feature
- **THEN** no code from `src/esp32/` is compiled or linked

#### Scenario: Module present with feature
- **WHEN** the crate is compiled with `--features esp32`
- **THEN** the module compiles successfully without errors or warnings

### Requirement: Esp32Hal composite struct
The module SHALL expose a public `struct Esp32Hal` that holds instances of all
sub-impl structs and can be constructed via `Esp32Hal::new()`.

#### Scenario: Construction succeeds
- **WHEN** `Esp32Hal::new()` is called
- **THEN** it returns a valid `Esp32Hal` instance with all sub-structs initialised

### Requirement: ControlHal stub implementation
`Esp32Hal` SHALL implement `ControlHal`. All methods SHALL return `Ok(())` as a
stub, with a `// TODO: wire to hardware` comment in the body.

#### Scenario: power stub returns Ok
- **WHEN** `power(true)` or `power(false)` is called on an `Esp32Hal`
- **THEN** the method returns `Ok(())`

#### Scenario: power_save stub returns Ok
- **WHEN** `power_save(true)` or `power_save(false)` is called
- **THEN** the method returns `Ok(())`

#### Scenario: reset_errors stub returns Ok
- **WHEN** `reset_errors()` is called
- **THEN** the method returns `Ok(())`

#### Scenario: reload_config stub returns Ok
- **WHEN** `reload_config()` is called
- **THEN** the method returns `Ok(())`

### Requirement: StatusHal stub implementation
`Esp32Hal` SHALL implement `StatusHal`. `state()` SHALL return `RobotState::Idle`
and `active_errors()` SHALL return an empty `Vec`.

#### Scenario: state returns Idle
- **WHEN** `state()` is called on a freshly constructed `Esp32Hal`
- **THEN** the method returns `RobotState::Idle`

#### Scenario: active_errors returns empty
- **WHEN** `active_errors()` is called
- **THEN** the method returns an empty Vec

### Requirement: ConfigHal stub implementation
`Esp32Hal` SHALL implement `ConfigHal`. `get_active_config()` SHALL return a
default `RobotConfig` with an empty liquids list. `update_active_config()` SHALL
return `Ok(())`.

#### Scenario: get_active_config returns default config
- **WHEN** `get_active_config()` is called
- **THEN** a `RobotConfig` is returned with `liquids` as an empty Vec

#### Scenario: update_active_config returns Ok
- **WHEN** `update_active_config(cfg)` is called with any config
- **THEN** the method returns `Ok(())`

### Requirement: StorageHal stub implementation
`Esp32Hal` SHALL implement `StorageHal`. Both methods SHALL return `Err` with an
`ErrorInfo` indicating the feature is not yet implemented.

#### Scenario: load_storage_config returns error
- **WHEN** `load_storage_config()` is called
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

#### Scenario: store_storage_config returns error
- **WHEN** `store_storage_config(cfg, overwrite)` is called
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

### Requirement: SensorHal stub implementation
`Esp32Hal` SHALL implement `SensorHal`. `glass_state()` SHALL return a
`GlassSensorState` with `present: false`. `level_state()` SHALL return an empty
`Vec`.

#### Scenario: glass_state reports no glass
- **WHEN** `glass_state()` is called
- **THEN** the method returns `Ok(GlassSensorState { present: false, ... })`

#### Scenario: level_state returns empty
- **WHEN** `level_state()` is called
- **THEN** the method returns `Ok(vec![])`

### Requirement: DispenseHal stub implementation
`Esp32Hal` SHALL implement `DispenseHal`. `create_job()` SHALL return `Ok` with
a generated stub job ID. `list_jobs()` and `job_status()` SHALL return empty /
default values. `cancel_job()` SHALL return `Ok(())`.

#### Scenario: create_job returns a job id
- **WHEN** `create_job(...)` is called with any valid parameters
- **THEN** the method returns `Ok(job_id)` where `job_id` is a non-empty String

#### Scenario: list_jobs returns empty
- **WHEN** `list_jobs()` is called
- **THEN** the method returns an empty Vec

#### Scenario: cancel_job returns Ok
- **WHEN** `cancel_job(job_id)` is called
- **THEN** the method returns `Ok(())`

### Requirement: CleaningHal stub implementation
`Esp32Hal` SHALL implement `CleaningHal`. Both `start_cleaning()` and
`stop_cleaning()` SHALL return `Ok(())`.

#### Scenario: start_cleaning returns Ok
- **WHEN** `start_cleaning()` is called
- **THEN** the method returns `Ok(())`

#### Scenario: stop_cleaning returns Ok
- **WHEN** `stop_cleaning()` is called
- **THEN** the method returns `Ok(())`

### Requirement: Module compiles for no_std + alloc target
All code in `src/esp32/` SHALL use only `core` and `alloc`; no `std` imports
are permitted.

#### Scenario: no std imports
- **WHEN** the source files in `src/esp32/` are reviewed
- **THEN** there are zero `use std::` statements

### Requirement: TODO comments mark unimplemented hardware wiring
Every stub method body that requires future hardware wiring SHALL contain a
`// TODO: wire to hardware` comment.

#### Scenario: TODO present in stub bodies
- **WHEN** a developer reads any stub method that wraps a hardware peripheral call
- **THEN** they find a `// TODO: wire to hardware` comment indicating what needs implementation
