### Requirement: dev example compiles as a Cargo example
A Cargo `[[example]]` named `dev` SHALL exist in `examples/dev/main.rs`. It SHALL
compile with `cargo build --example dev` on any host platform without additional
features. It SHALL NOT require the `esp32` feature.

#### Scenario: dev example builds on host
- **WHEN** `cargo build --example dev` is run without any feature flags
- **THEN** the build succeeds and produces an executable

### Requirement: dev example wires all HAL traits
The dev example SHALL provide stub implementations for all seven HAL traits
(`ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`, `SensorHal`, `DispenseHal`,
`CleaningHal`) and SHALL wire them into `ApiServer` via `RobotHal`.

#### Scenario: all HAL traits implemented
- **WHEN** the dev example is compiled
- **THEN** there are no missing-trait-impl compile errors

### Requirement: dev example uses RamStorageHal
The dev example SHALL use `RamStorageHal` (defined within `examples/dev/`) as the
`StorageHal` implementation, pre-seeded with a default development config.

#### Scenario: dev example boots with default config
- **WHEN** `cargo run --example dev` is executed
- **THEN** the server starts and `GET /v1/config` returns a non-empty config
  without requiring a provisioning step

### Requirement: dev example uses StubPasswordHasher
The dev example SHALL provide `StubPasswordHasher` implementing `PasswordHasher`
using a `stub$<plaintext>` scheme for development-only use. It SHALL NOT be
exported from the library.

#### Scenario: StubPasswordHasher verifies matching password
- **WHEN** `hash("secret")` is called and the result is passed to `verify("secret", hash)`
- **THEN** `verify` returns `true`

#### Scenario: StubPasswordHasher rejects wrong password
- **WHEN** `verify("wrong", hash_of_secret)` is called
- **THEN** `verify` returns `false`

### Requirement: dev example runs with embassy spin executor
The dev example SHALL initialise and run the embassy spin executor
(`embassy_executor::Executor` with `arch-spin`), spawning `async_main` and
`sse_task` as embassy tasks.

#### Scenario: executor starts without panicking
- **WHEN** `cargo run --example dev` is executed
- **THEN** the process starts and the executor loop runs without panicking at startup
