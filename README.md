# CocktailBotHAL

A Rust Hardware Abstraction Layer (HAL) for remotely controlling autonomous
cocktail mixing robots via a REST API.

## Project Status

**Pre-1.0 — active development.** The HAL trait interface (
`cocktail_bot_hal::hal`) is the public contract for hardware implementors and
follows [semantic versioning](https://semver.org/). Breaking changes will
increment the major version and be documented in
[CHANGELOG.md](CHANGELOG.md).

## Description

CocktailBotHAL is a library crate that defines a trait-based interface for
cocktail robot hardware. Hardware vendors implement the HAL traits for their
specific microcontroller and mechanics; the library provides a complete async
HTTP server that exposes a standardised
[OpenAPI 3.1.0](API.yaml) REST API to client applications. The primary target
platform is the **ESP32** family (via
[esp-hal](https://github.com/esp-rs/esp-hal)), but the library compiles for
any platform that supports `alloc` and an async executor.

## Architecture

The crate exposes two public modules:

- **`cocktail_bot_hal::hal`** — trait definitions, data types, and utilities.
  Hardware vendors work exclusively with this module.
- **`cocktail_bot_hal::server`** — the async HTTP server (`ApiServer`) and
  SSE streaming server. This is provided by the library; vendors do not modify
  it.

`ApiServer` is generic over eight type parameters, one per HAL trait:

```
ApiServer<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean, Hasher>
```

Static generic dispatch is used instead of `dyn Trait` because Rust's `async
fn` in traits is not object-safe in the current edition. Each HTTP handler
receives only the specific HAL reference it needs (e.g., `handle_status`
receives only `&Stat: StatusHal`).

Inside `ApiServer`, `RobotHal` composes all eight concrete HAL implementations
into a single owned struct. The server dispatches incoming HTTP requests to
handler sub-modules in `server::handlers::{status, control, config, sensors,
dispense, cleaning}`.

Auth model: admin routes (power, config write, cleaning) require **HTTP Basic
Auth** with the admin password; all other routes require a **Bearer token**.
Both credentials are stored in `AdminConfig` and are configurable at runtime.

## HAL Traits

| Trait | Responsibility |
|---|---|
| `ControlHal` | Power on/off, power-save mode, reset error state |
| `StatusHal` | Query current `RobotState` and active errors |
| `ConfigHal` | Get/update active (RAM) configuration |
| `StorageHal` | Load/store `AdminConfig` to non-volatile storage (flash) |
| `SensorHal` | Glass detection, liquid level sensing |
| `DispenseHal` | Create, list, query, and cancel dispensing jobs |
| `CleaningHal` | Start/stop the cleaning program |
| `PasswordHasher` | Hash and verify admin passwords (synchronous) |

All trait methods are `async fn`. The `PasswordHasher` trait is synchronous
and separate because hashing is CPU-bound and must not hold async locks.

## Key Types

| Type | Role |
|---|---|
| `RobotState` | Tagged-union enum of every robot state (`Off`, `Idle`, `Working`, `DrinkReady`, `Error`, …). Carries job context and timeout countdowns. Serialised with `#[serde(tag = "state")]`. |
| `AdminConfig` | Admin-owned configuration: liquids, glass sizes, bearer token, hashed admin password, and timeout values. Persisted to flash via `StorageHal`. |
| `RobotConfig` | Merged API view returned by `GET /config`: `AdminConfig` fields + hardware-fixed `Capabilities`. |
| `LiquidConfig` | Per-liquid configuration: ID, name, dispenser position, and `LiquidCalibration`. |
| `LiquidCalibration` | Hardware-agnostic calibration multiplier (`factor: f32`) that the HAL applies to convert abstract volume into hardware commands. |
| `Capabilities` | Hardware-fixed properties: firmware version, level reporting mode, glass typing, simultaneous channels, max queue depth, button presence. |
| `DispenseItem` | Per-ingredient dispense instruction: `liquid_id` + pre-computed `amount` in the operator's volume unit. |
| `JobItem` | Client-facing ingredient specification by ratio (`liquid_id` + `parts`). The server converts this to `DispenseItem` before calling `DispenseHal`. |
| `JobStatus` | Job state snapshot: `job_id`, `name`, `JobState`, `progress_pct`. |
| `ApiServer` | The HTTP server. Construct with a `RobotHal` and call `.run(net_stack).await`. |

## Installation / Build & Run

```bash
# Type-check the library
cargo check

# Debug build (library only)
cargo build

# Run the development server on port 8000 (host machine, stub HALs)
cargo run --example dev

# Build the ESP32 reference example (requires cross-compilation toolchain)
cargo build --example esp32 --features esp32
```

No Makefile or Docker required. Standard Cargo only.

## Usage

The snippet below shows the minimum code needed to implement one HAL trait and
wire everything into `ApiServer`. See [`examples/dev/main.rs`](examples/dev/main.rs)
for a complete working example with all traits stubbed out.

```rust
use cocktail_bot_hal::hal::{
    ErrorInfo, RobotState, StatusHal,
    // ... other HAL traits and types
};
use cocktail_bot_hal::server::{ApiServer, RobotHal};

// 1. Implement the traits on your hardware structs.
struct MyStatus;

impl StatusHal for MyStatus {
    async fn state(&self) -> RobotState {
        RobotState::Idle
    }
    async fn active_errors(&self) -> Vec<ErrorInfo> {
        vec![]
    }
}

// 2. Wire everything into ApiServer.
let server = ApiServer {
    hal: RobotHal {
        status: MyStatus,
        // control, config, storage, sensors, dispense, cleaning, hasher …
    },
};

// 3. Run the server (requires an embassy-net Stack).
server.run(net_stack).await;
```

## Implementing the HAL

1. **Create a struct per trait.** Each HAL trait maps to one struct that owns
   the hardware peripheral(s) it needs.

2. **Implement all eight traits.** The required traits are listed in the
   [HAL Traits](#hal-traits) table above. Start with `StatusHal` and
   `ConfigHal` — those are called on every request.

3. **Handle `RobotState` transitions.** Your `StatusHal::state()` must return
   the correct `RobotState` variant at all times. The server uses this to gate
   endpoints and produce SSE events.

4. **Implement `PasswordHasher`.** For production, use PBKDF2-HMAC-SHA256 (see
   [`examples/esp32/hasher.rs`](examples/esp32/hasher.rs)). For development,
   a plaintext stub is fine.

5. **Construct `ApiServer`.** Pass all eight implementations to `RobotHal`,
   wrap it in `ApiServer`, and call `.run(net_stack).await` from your async
   entry point.

6. **ESP32 specifics.** Use `#[esp_hal::main]` as the async entry point, not
   `#[embassy_executor::main]`. See [`examples/esp32/main.rs`](examples/esp32/main.rs)
   for the full wiring pattern including the heap allocator. SSE is served by
   `ApiServer` on port 80 as `GET /v1/events` — no separate task is required.

All HAL trait methods are `async fn`. Your implementations may call `await` on
hardware drivers, but must not block the executor with synchronous waits.

## Example

to run it:
```
cargo run --example mock-server [-- --port 8080 --glass-present --dispense-duration-secs 5]
# Bearer token: changeme  |  Admin: Basic YWRtaW46Y2hhbmdlbWU=
```

## Support

Found a bug or have a question? Please
[open an issue](https://github.com/crjeder/CocktailBotHAL/issues) on GitHub.

## Contributing

Contributions are welcome. Before opening a pull request:

```bash
cargo test    # all tests must pass
cargo fmt     # code must be formatted with rustfmt
cargo check --features esp32   # ESP32 example must still compile
```

Please open an issue first for significant changes so the approach can be
discussed before implementation.

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE) or
[SPDX: GPL-3.0-only](https://spdx.org/licenses/GPL-3.0-only.html).
