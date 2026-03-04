## MODIFIED Requirements

### Requirement: ControlHal stub implementation
`Esp32ControlHal` SHALL implement `ControlHal`. All methods SHALL be declared
`async fn` and SHALL return `Ok(())` as a stub, with a `// TODO: wire to hardware`
comment in the body.

#### Scenario: power stub returns Ok (async)
- **WHEN** `power(true)` or `power(false)` is awaited on an `Esp32ControlHal`
- **THEN** the method returns `Ok(())`

#### Scenario: power_save stub returns Ok (async)
- **WHEN** `power_save(true)` or `power_save(false)` is awaited
- **THEN** the method returns `Ok(())`

#### Scenario: reset_errors stub returns Ok (async)
- **WHEN** `reset_errors()` is awaited
- **THEN** the method returns `Ok(())`

#### Scenario: reload_config stub returns Ok (async)
- **WHEN** `reload_config()` is awaited
- **THEN** the method returns `Ok(())`

### Requirement: StatusHal stub implementation
`Esp32StatusHal` SHALL implement `StatusHal`. `state()` and `active_errors()`
SHALL be declared `async fn`. `state()` SHALL return `RobotState::Idle` and
`active_errors()` SHALL return an empty `Vec`.

#### Scenario: state returns Idle (async)
- **WHEN** `state()` is awaited on a freshly constructed `Esp32StatusHal`
- **THEN** the method returns `RobotState::Idle`

#### Scenario: active_errors returns empty (async)
- **WHEN** `active_errors()` is awaited
- **THEN** the method returns an empty Vec

### Requirement: ConfigHal stub implementation
`Esp32ConfigHal` SHALL implement `ConfigHal`. Both methods SHALL be declared
`async fn`. `get_active_config()` SHALL return a default `RobotConfig` with an
empty liquids list and `token` set to an empty string.
`update_active_config()` SHALL return `Ok(())`.

#### Scenario: get_active_config returns default config (async)
- **WHEN** `get_active_config()` is awaited
- **THEN** a `RobotConfig` is returned with `liquids` as an empty Vec and `token` as an empty string

#### Scenario: update_active_config returns Ok (async)
- **WHEN** `update_active_config(cfg)` is awaited with any config
- **THEN** the method returns `Ok(())`

### Requirement: StorageHal stub implementation
`Esp32StorageHal` SHALL implement `StorageHal`. Both methods SHALL be declared
`async fn` and SHALL return `Err` with an `ErrorInfo` indicating the feature is
not yet implemented.

#### Scenario: load_storage_config returns error (async)
- **WHEN** `load_storage_config()` is awaited
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

#### Scenario: store_storage_config returns error (async)
- **WHEN** `store_storage_config(cfg, overwrite)` is awaited
- **THEN** the method returns `Err(ErrorInfo { code: "NOT_IMPLEMENTED", ... })`

### Requirement: SensorHal stub implementation
`Esp32SensorHal` SHALL implement `SensorHal`. Both methods SHALL be declared
`async fn`. `glass_state()` SHALL return a `GlassSensorState` with
`present: false`. `level_state()` SHALL return an empty `Vec`.

#### Scenario: glass_state reports no glass (async)
- **WHEN** `glass_state()` is awaited
- **THEN** the method returns `Ok(GlassSensorState { present: false, ... })`

#### Scenario: level_state returns empty (async)
- **WHEN** `level_state()` is awaited
- **THEN** the method returns `Ok(vec![])`

### Requirement: DispenseHal stub implementation
`Esp32DispenseHal` SHALL implement `DispenseHal`. All four methods SHALL be
declared `async fn`. `create_job()` SHALL return `Ok` with a generated stub job
ID. `list_jobs()` and `job_status()` SHALL return empty/default values.
`cancel_job()` SHALL return `Ok(())`.

#### Scenario: create_job returns a job id (async)
- **WHEN** `create_job(...)` is awaited with any valid parameters
- **THEN** the method returns `Ok(job_id)` where `job_id` is a non-empty String

#### Scenario: list_jobs returns empty (async)
- **WHEN** `list_jobs()` is awaited
- **THEN** the method returns an empty Vec

#### Scenario: cancel_job returns Ok (async)
- **WHEN** `cancel_job(job_id)` is awaited
- **THEN** the method returns `Ok(())`

### Requirement: CleaningHal stub implementation
`Esp32CleaningHal` SHALL implement `CleaningHal`. Both `start_cleaning()` and
`stop_cleaning()` SHALL be declared `async fn` and SHALL return `Ok(())`.

#### Scenario: start_cleaning returns Ok (async)
- **WHEN** `start_cleaning()` is awaited
- **THEN** the method returns `Ok(())`

#### Scenario: stop_cleaning returns Ok (async)
- **WHEN** `stop_cleaning()` is awaited
- **THEN** the method returns `Ok(())`
