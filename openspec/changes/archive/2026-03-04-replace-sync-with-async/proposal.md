## Why

All seven HAL traits define synchronous methods, but real hardware operations
(flash reads/writes, sensor polling, motor actuation) are inherently blocking
and must not stall the embassy executor. Converting the HAL trait interface to
`async fn` lets hardware drivers `await` their I/O without blocking other tasks,
and aligns the trait contract with embassy's cooperative async model.

## What Changes

- **BREAKING**: All methods on the seven HAL traits (`ControlHal`, `StatusHal`,
  `ConfigHal`, `StorageHal`, `SensorHal`, `DispenseHal`, `CleaningHal`) become
  `async fn`.
- The stub implementations in `src/main.rs` are updated to `async fn`.
- The ESP32 stub implementations in `src/esp32/` are updated to `async fn`.
- The server handler modules in `src/server/handlers/` gain `.await` at each
  HAL call site.
- `fn main()` in `src/main.rs` is converted to an async entry point driven by
  `embassy_executor::Executor` (spin variant), matching the existing
  `arch-spin` feature in Cargo.toml, with a note that ESP32 bring-up will swap
  this for `#[esp_hal::main]`.
- Semver bump: `0.1.0 → 0.2.0` (breaking public trait change).

## Capabilities

### New Capabilities

- `async-hal-traits`: HAL trait interface converted to async; all seven traits
  expose `async fn` methods compatible with embassy and no-std async runtimes.

### Modified Capabilities

- `async-http-server`: Server handlers now `.await` HAL trait calls; the
  `RobotHal` struct and `ApiServer::run` signature are unchanged but handler
  bodies change.
- `esp32-hal-impl`: ESP32 stub trait impls updated to satisfy the new async
  trait signatures.

## Impact

- **Breaking API**: Any crate that implements a HAL trait must add `async` to
  all method signatures. This is the public vendor contract.
- **`src/hal/mod.rs`**: All trait method signatures change.
- **`src/main.rs`**: Stub impls and entry point change.
- **`src/esp32/`**: All eight HAL impl files change.
- **`src/server/handlers/`**: All six handler modules gain `.await`.
- **`Cargo.toml`**: No new dependencies required; `embassy-executor` (already
  present with `arch-spin`) provides the spin executor for the async entry point.
- **`API.yaml`** and **`openspec/config.yaml`**: No changes required.
