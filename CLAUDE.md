# CLAUDE.md — AI Assistant Guide for CocktailBotHAL

## Project Overview

**CocktailBotHAL** is a Rust-based Hardware Abstraction Layer (HAL) for remotely
controlling autonomous cocktail mixing robots via a REST API. It defines a
trait-based interface that hardware vendors implement, and exposes a standardized
OpenAPI 3.1.0 HTTP API to clients.

- **Language:** Rust (Edition 2021)
- **License:** GNU GPL v3
- **Author:** crjeder <crjeder@gmail.com>
- **Crate name:** `cocktail_bot_hal` v0.1.0

---
## Session Start
1. read `claude-progress.txt`
2. read `git log --oneline -10`
3. work on exactly ONE task

## Session End  
1. git commit with descriptive message
2. updadate `claude-progress.txt`

## Repository Structure

```
CocktailBotHAL/
├── src/
│   ├── main.rs              # Entry point + HAL stub implementations
│   ├── hal/
│   │   ├── mod.rs           # Core HAL trait definitions and data types
│   │   └── tests.rs         # Unit tests with mock HAL implementations
│   └── server/
│       ├── mod.rs           # Async HTTP server + route dispatch (embassy-net)
│       ├── http.rs          # Minimal HTTP/1.1 parser and JSON response writer
│       ├── sse.rs           # Server-Sent Events server (port 9000)
│       └── handlers/
│           ├── status.rs    # GET /v1/status
│           ├── control.rs   # POST /v1/control/*
│           ├── config.rs    # GET/PATCH /v1/config, GET/POST /v1/storage/config
│           ├── sensors.rs   # GET /v1/sensors/*
│           ├── dispense.rs  # POST/GET /v1/dispense/jobs[/{job_id}]
│           └── cleaning.rs  # POST /v1/cleaning/*
├── testdata/
│   ├── margarita.json       # Sample Margarita recipe
│   ├── vesper.json          # Sample Vesper recipe
│   └── vesper2.json         # Vesper recipe (alternate format)
├── API.yaml                 # OpenAPI 3.1.0 specification
├── Cargo.toml               # Rust project manifest
├── rustfmt.toml             # Formatting rules
├── claude-progress.txt      # AI session progress tracking
├── TODO.md                  # Open implementation work
├── README.md                # Project overview and feature list
└── LICENSE                  # GPL v3
```

---

## Technology Stack

| Category         | Library / Tool                                      |
|------------------|-----------------------------------------------------|
| Async networking | embassy-net, embedded-io-async (not yet in Cargo.toml) |
| Serialization    | serde 1.0 + derive, serde_json 1.0, serde_derive    |
| Timing           | `core::time::Duration` (standard library)           |
| Formatting       | rustfmt (config in `rustfmt.toml`)                  |

> **Note:** Legacy Rocket 0.4 code and the `generic_cocktail` local path
> dependency have been removed. The embassy crates are used in source but not
> yet added to `Cargo.toml` — see `TODO.md` for blockers.

---

## Build & Run

```bash
# Debug build (will fail until embassy crates are added — see TODO.md)
cargo build

# Run the test suite (57 tests, works without embassy dependencies)
cargo test

# Format code before committing
cargo fmt
```

> **Important:** `cargo build` and `cargo run` currently fail because the
> embassy crates are not yet in `Cargo.toml`. However, `cargo test` works
> because the `server` module is gated with `#[cfg(not(test))]`.

There is no Makefile, Docker setup, or additional build scripts. Standard Cargo
commands are sufficient.

---

## Code Architecture

### HAL Traits (`src/hal/mod.rs`)

The heart of the project. All hardware implementations must satisfy these traits:

| Trait          | Responsibility                                          |
|----------------|---------------------------------------------------------|
| `ControlHal`   | Power on/off, power-save, reset errors, reload config   |
| `StatusHal`    | Query robot state (`RobotState`), active errors         |
| `ConfigHal`    | Get/update the active (RAM) `RobotConfig`               |
| `StorageHal`   | Load/store `RobotConfig` to non-volatile storage        |
| `SensorHal`    | Glass detection (`GlassSensorState`), liquid levels     |
| `DispenseHal`  | Create/query/cancel dispensing jobs                     |
| `CleaningHal`  | Start/stop cleaning programs                            |

Key types in this module:
- `RobotState` enum: `Off | Booting | SelfTest | Idle | Prepared | Working | Cleaning | DrinkReady | Error`
- `RobotConfig` / `LiquidConfig`: configuration structs with calibration data
- `JobItem` / `JobStatus`: dispensing job management
- `Capabilities`: what optional features the robot supports

### Server (`src/server/mod.rs`)

`RobotHal` composes all HAL trait objects into a single struct. `ApiServer`
accepts TCP connections and dispatches requests to handler sub-modules:
`status`, `control`, `config`, `sensors`, `dispense`, `cleaning`.

All handler sub-modules are implemented and call the corresponding HAL trait
methods. All `API.yaml` routes (except auth) are wired, including dynamic
path extraction for `/v1/dispense/jobs/{job_id}`.

The server module is gated with `#[cfg(not(test))]` so tests compile without
the embassy-net and embedded-io-async dependencies.

### SSE Server (`src/server/sse.rs`)

A Server-Sent Events server running on port 9000. Polls HAL traits every
500ms for state and job changes, emitting typed SSE events (`state_change`,
`job_update`) to connected clients. Includes 30-second keepalive comments.

### Entry Point (`src/main.rs`)

Contains `fn main()` placeholder and `Stub*Hal` implementations for all 7
HAL traits (all methods return `todo!()`). The stub implementations serve as
a template for real hardware drivers.

> **Note:** Legacy Rocket 0.4 code (`src/api/mod.rs`) has been removed.
> The project now targets only the async `v1` API.

---

## REST API

Full spec lives in `API.yaml` (OpenAPI 3.1.0). Base URL: `http://robot.local/v1`.
Authentication: Bearer token.

| Method | Path                        | Description                         |
|--------|-----------------------------|-------------------------------------|
| GET    | /v1/status                  | Robot state + active errors         |
| POST   | /v1/control/power           | Power on/off                        |
| POST   | /v1/control/power-save      | Enter power-save mode               |
| POST   | /v1/control/reset           | Clear errors, return to idle        |
| POST   | /v1/control/reload-config   | Reload config from storage          |
| GET    | /v1/config                  | Read active (RAM) config            |
| PATCH  | /v1/config                  | Update active config                |
| GET    | /v1/storage/config          | Read persistent config              |
| POST   | /v1/storage/config          | Write persistent config             |
| GET    | /v1/sensors/glass           | Glass presence and type             |
| GET    | /v1/sensors/levels          | Liquid levels                       |
| POST   | /v1/dispense/jobs           | Create dispensing job               |
| GET    | /v1/dispense/jobs           | List job queue and history          |
| GET    | /v1/dispense/jobs/{job_id}  | Job status with progress            |
| POST   | /v1/dispense/jobs/{job_id}  | Cancel a job                        |
| POST   | /v1/cleaning/start          | Start cleaning program              |
| POST   | /v1/cleaning/stop           | Stop cleaning                       |
| GET    | /v1/events                  | Server-Sent Events stream           |

> **Note:** There is a typo on line 82 of `API.yaml` (`integerlö` instead of
> `integer`). Fix this if regenerating client/server stubs.

---

## Testing

### Automated Tests (`src/hal/tests.rs`)

Run with `cargo test`. The test suite contains **57 unit tests** covering:

- **Mock HAL implementations** for all 7 traits (`MockControlHal`,
  `MockStatusHal`, `MockConfigHal`, `MockStorageHal`, `MockSensorHal`,
  `MockDispenseHal`, `MockCleaningHal`)
- **Trait behavior tests:** power on/off, state transitions, config CRUD,
  storage with overwrite semantics, sensor readings, job lifecycle
  (create/list/status/cancel), cleaning start/stop
- **Error injection:** each mock supports a `fail_next` field to test error
  propagation through HAL trait methods
- **Serialization roundtrip tests:** JSON serialization/deserialization for
  all HAL data types (`RobotState`, `RobotConfig`, `JobState`, `ErrorInfo`,
  `GlassSensorState`, `LevelState`, `JobItem`, `LiquidCalibration`,
  `Capabilities`, `LevelReporting`)

### Test Data

Sample cocktail recipes in `testdata/` are JSON files for manual API testing.

---

## Code Conventions

### Formatting (`rustfmt.toml`)

| Rule                    | Value           |
|-------------------------|-----------------|
| `brace_style`           | `AlwaysNextLine`|
| `control_brace_style`   | `AlwaysNextLine`|
| `fn_single_line`        | `true`          |
| `indent_style`          | `Block`         |
| `max_width`             | `80`            |
| `struct_lit_width`      | `40`            |

Always run `cargo fmt` before committing.

### General Rust Conventions

- Use `///` doc comments on all public items (traits, structs, methods).
- Prefer trait objects (`Box<dyn Trait>`) for composing HAL implementations —
  see `RobotHal` in `src/server/mod.rs`.
- Error types use the `ErrorInfo` struct (code, message, hint, recoverable).
- JSON serialization via `#[derive(Serialize, Deserialize)]` from serde.
- Avoid adding dependencies without first checking what is already commented out
  in `Cargo.toml` — several libraries were intentionally deferred.

---

## Development Workflow

### Branching

- Main branch: `master`
- Feature branches follow the pattern: `claude/<description>-<id>`

### Commit Style

Use clear, descriptive English commit messages in imperative mood.
Examples from the log: `"Add HAL module with robot control traits and structs"`,
`"Refactor API.yaml by removing states and properties"`.

### Key Areas for Contribution

1. **Add embassy dependencies to `Cargo.toml`** — the server module uses
   `embassy-net`, `embedded-io-async`, and `embassy_time` but these are not
   yet in `Cargo.toml`. Pick versions matching your target MCU.
2. **Replace `Stub*Hal` with real hardware drivers** — the stubs in `main.rs`
   all use `todo!()`. Implement for your target platform.
3. **Add handler-level integration tests** — test HTTP request/response
   cycles against mock HAL implementations.
4. **Implement `StorageHal`** — persistent config read/write is defined but
   not yet implemented for any real backend.
5. **Add Bearer token authentication** — `API.yaml` declares it but the
   server does not validate tokens.
6. **Fix the typo** in `API.yaml` line 82 (`integerlö` → `integer`).

---

## Important Notes for AI Assistants

- **Do not break the HAL trait interface** (`src/hal/mod.rs`). It is the
  public contract for hardware implementors.
- **The HAL module uses `core::time::Duration`** (not `embassy_time::Duration`)
  for portability. This allows tests to run without embassy dependencies.
- **`mod server` is gated with `#[cfg(not(test))]`** — this is intentional
  so `cargo test` works without embassy-net and embedded-io-async. Do not
  remove this gate unless you add those dependencies to `Cargo.toml`.
- **`extern crate alloc;`** is declared in `main.rs` to make `alloc::string`
  and `alloc::vec` available throughout the crate (needed for eventual
  `no_std` support).
- **Cargo.lock is gitignored** — do not add it.
- **No environment variables or `.env` files** are used; all configuration is
  currently hardcoded or loaded via the HAL traits at runtime.
- **The async server** listens on port 80 (API) and port 9000 (SSE). These
  ports are configured in `src/server/mod.rs` and `src/server/sse.rs`.
