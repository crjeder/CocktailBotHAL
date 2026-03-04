## ADDED Requirements

### Requirement: HAL traits expose async methods
All seven HAL traits (`ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`,
`SensorHal`, `DispenseHal`, `CleaningHal`) in `src/hal/mod.rs` SHALL declare
every method as `async fn`. No method SHALL be synchronous.

#### Scenario: ControlHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** all four methods of `ControlHal` (`power`, `power_save`, `reset_errors`, `reload_config`) are declared `async fn`

#### Scenario: StatusHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** both methods of `StatusHal` (`state`, `active_errors`) are declared `async fn`

#### Scenario: ConfigHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** both methods of `ConfigHal` (`get_active_config`, `update_active_config`) are declared `async fn`

#### Scenario: StorageHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** both methods of `StorageHal` (`load_storage_config`, `store_storage_config`) are declared `async fn`

#### Scenario: SensorHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** both methods of `SensorHal` (`glass_state`, `level_state`) are declared `async fn`

#### Scenario: DispenseHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** all four methods of `DispenseHal` (`create_job`, `list_jobs`, `job_status`, `cancel_job`) are declared `async fn`

#### Scenario: CleaningHal methods are async
- **WHEN** a developer inspects `src/hal/mod.rs`
- **THEN** both methods of `CleaningHal` (`start_cleaning`, `stop_cleaning`) are declared `async fn`

### Requirement: HAL traits compile without async-trait crate
The async HAL traits SHALL rely solely on native Rust `async fn` in traits
(Rust 1.75+). The `async-trait` crate SHALL NOT be added as a dependency.

#### Scenario: async-trait absent from Cargo.toml
- **WHEN** `Cargo.toml` is inspected
- **THEN** `async-trait` does not appear as a dependency

#### Scenario: cargo check passes with async trait methods
- **WHEN** `cargo check` is run
- **THEN** the build completes without errors related to async trait compilation

### Requirement: RobotHal uses generic type parameters instead of dyn Trait
Because native async trait methods are not object-safe, `RobotHal` in
`src/server/mod.rs` SHALL hold each HAL implementation as a generic type
parameter rather than `&mut dyn Trait`. All seven HAL fields SHALL be owned
values (not references) bounded by their respective trait.

#### Scenario: RobotHal struct has seven generic parameters
- **WHEN** a developer inspects `src/server/mod.rs`
- **THEN** `RobotHal` is generic over seven type parameters, each bounded by one HAL trait

#### Scenario: No dyn Trait in RobotHal field types
- **WHEN** `src/server/mod.rs` is searched for `dyn ControlHal`, `dyn StatusHal`, etc.
- **THEN** no such `dyn` references exist in the `RobotHal` struct definition

### Requirement: Crate version bumped to 0.2.0
Because the HAL trait interface is a public contract and changing method
signatures is a breaking change, the crate version in `Cargo.toml` SHALL be
updated from `0.1.0` to `0.2.0`.

#### Scenario: Version is 0.2.0
- **WHEN** `Cargo.toml` is inspected
- **THEN** the `[package]` section shows `version = "0.2.0"`

### Requirement: Handler call sites await HAL methods
Every call to a HAL trait method inside `src/server/handlers/` SHALL be
followed by `.await`.

#### Scenario: No bare HAL calls in handlers
- **WHEN** the handler source files are inspected
- **THEN** every expression of the form `hal.<subsystem>.<method>(...)` is followed by `.await`
