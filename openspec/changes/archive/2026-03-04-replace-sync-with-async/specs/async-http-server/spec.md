## MODIFIED Requirements

### Requirement: Async entry point
The binary SHALL have an async entry point that constructs all HAL
implementations and calls `ApiServer::run()` inside the embassy async runtime.
The entry point SHALL be implemented as a standard `fn main()` that creates an
`embassy_executor::Executor`, spawns the async main task, and calls
`Executor::run()`. The `#[embassy_executor::main]` macro SHALL NOT be used
because it is unavailable when the `arch-spin` executor feature is active.

#### Scenario: Server starts without panicking
- **WHEN** the binary is executed on a target with a configured embassy spin executor
- **THEN** `ApiServer::run()` is called and the server begins accepting TCP connections on port 80

#### Scenario: fn main uses Executor::run pattern
- **WHEN** `src/main.rs` is inspected
- **THEN** `fn main()` constructs an `embassy_executor::Executor`, calls `.run()` on it, and spawns the async task containing the server loop

#### Scenario: ESP32 build remains valid
- **WHEN** `cargo check --features esp32` is run
- **THEN** the build completes without errors (the `fn main()` entry point is replaced by `#[esp_hal::main]` at ESP32 bring-up time)

### Requirement: embassy-executor dependency
The crate SHALL declare `embassy-executor` as a dependency in `Cargo.toml` with
the `arch-spin` feature for host (development) builds. The `static-cell` crate
SHALL also be declared as a dependency to allow a `static Executor` to be
initialized once.

#### Scenario: cargo check passes with embassy-executor
- **WHEN** `cargo check` is run without the `esp32` feature
- **THEN** the build completes without errors or unexpected warnings

#### Scenario: ESP32 feature is not broken
- **WHEN** `cargo check --features esp32` is run
- **THEN** the build completes without errors (executor features may differ per target)
