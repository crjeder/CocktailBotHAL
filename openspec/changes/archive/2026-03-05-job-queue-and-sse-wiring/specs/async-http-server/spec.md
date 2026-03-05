## MODIFIED Requirements

### Requirement: Async entry point
The binary SHALL have an async entry point that constructs all HAL
implementations and calls `ApiServer::run()` inside the embassy async runtime.
The entry point SHALL be implemented as a standard `fn main()` that creates an
`embassy_executor::Executor`, spawns the async main task, and calls
`Executor::run()`. The `#[embassy_executor::main]` macro SHALL NOT be used
because it is unavailable when the `arch-spin` executor feature is active.
The async main task SHALL also spawn a second `#[embassy_executor::task]` for
`SseServer` before entering the `ApiServer` run loop.

#### Scenario: Server starts without panicking
- **WHEN** the binary is executed on a target with a configured embassy spin executor
- **THEN** both `ApiServer::run()` (port 80) and `SseServer::run()` (port 9000) are called and both servers begin accepting TCP connections

#### Scenario: fn main uses Executor::run pattern
- **WHEN** `src/main.rs` is inspected
- **THEN** `fn main()` constructs an `embassy_executor::Executor`, calls `.run()` on it, and spawns the async task containing both the API server loop and the SSE server task

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

## ADDED Requirements

### Requirement: create_job handler returns JobCreated
The `handle_create_job` handler SHALL parse a `name` field (not `client_job_id`)
from the JSON request body and return a JSON response containing both `job_id`
and `queue_position` from the HAL's `JobCreated` return value.

#### Scenario: Successful creation response includes queue_position
- **WHEN** `POST /v1/dispense/jobs` succeeds
- **THEN** the response body is `{ "job_id": "...", "queue_position": N }` with HTTP 201

### Requirement: 503 on queue full
The `handle_create_job` handler SHALL return HTTP 503 with body
`{ "error": "queue full" }` when the HAL returns an `ErrorInfo` with
`code == "QUEUE_FULL"`.

#### Scenario: Queue full returns 503
- **WHEN** the HAL returns `Err(ErrorInfo { code: "QUEUE_FULL", .. })`
- **THEN** the handler responds with HTTP 503 and `{ "error": "queue full" }`
