## ADDED Requirements

### Requirement: Async entry point
The binary SHALL have an `#[embassy_executor::main]` async entry point that constructs all HAL trait objects and calls `ApiServer::run()` inside the embassy async runtime.

#### Scenario: Server starts without panicking
- **WHEN** the binary is executed on a target with an initialised embassy runtime
- **THEN** `ApiServer::run()` is called and the server begins accepting TCP connections on port 80

#### Scenario: Synchronous fn main is absent
- **WHEN** the crate is compiled
- **THEN** there SHALL be no `fn main()` without the `#[embassy_executor::main]` macro

### Requirement: embassy-executor dependency
The crate SHALL declare `embassy-executor` as a dependency in `Cargo.toml` with features that enable the async entry-point macro and a host (std) executor for development builds.

#### Scenario: cargo check passes with embassy-executor
- **WHEN** `cargo check` is run without the `esp32` feature
- **THEN** the build completes without errors or unexpected warnings

#### Scenario: ESP32 feature is not broken
- **WHEN** `cargo check --features esp32` is run
- **THEN** the build completes without errors (executor features may differ per target)

### Requirement: API.yaml typo corrected
The `API.yaml` file SHALL NOT contain the string `integerlö` on any line; the correct spelling is `integer`.

#### Scenario: Typo absent from spec
- **WHEN** the file `API.yaml` is opened at the previously-known offending line
- **THEN** the field type reads `integer` without any extraneous characters

### Requirement: Rocket references removed from documentation
Project documentation (`CLAUDE.md`, `openspec/config.yaml`) SHALL NOT list Rocket or `rocket_contrib` as an active dependency or tech-stack component.

#### Scenario: CLAUDE.md is Rocket-free
- **WHEN** `CLAUDE.md` is searched for the string "Rocket"
- **THEN** no active tech-stack entry referencing Rocket is found

#### Scenario: openspec config is Rocket-free
- **WHEN** `openspec/config.yaml` is searched for the string "Rocket"
- **THEN** no tech-stack listing referencing Rocket is found
