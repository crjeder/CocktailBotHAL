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
│   ├── main.rs          # Legacy REST API entry point (Rocket 0.4)
│   ├── api/mod.rs       # Newer alternative API implementation (Rocket 0.4)
│   ├── hal/mod.rs       # Core HAL trait definitions and data types
│   └── server/mod.rs    # Async HTTP server (embassy-net)
├── testdata/
│   ├── margarita.json   # Sample Margarita recipe
│   ├── vesper.json      # Sample Vesper recipe
│   └── vesper2.json     # Vesper recipe (alternate format)
├── API.yaml             # OpenAPI 3.1.0 specification
├── Cargo.toml           # Rust project manifest
├── rustfmt.toml         # Formatting rules
├── README.md            # Project overview and feature list
└── LICENSE              # GPL v3
```

---

## Technology Stack

| Category         | Library / Tool                                      |
|------------------|-----------------------------------------------------|
| Web framework    | Rocket 0.4, rocket_contrib 0.4.11                   |
| Async networking | embassy-net, embedded-io-async                      |
| Serialization    | serde 1.0 + derive, serde_json 1.0, serde_derive    |
| Domain logic     | generic_cocktail (local path dep: `../generic-cocktail`) |
| Timing           | embassy_time                                        |
| Formatting       | rustfmt (config in `rustfmt.toml`)                  |

Local path dependency `generic_cocktail` lives at `../generic-cocktail` relative
to this repository. Both must be present for a successful build.

---

## Build & Run

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run the server (starts on port 8000)
cargo run
```

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

Handler sub-modules are declared but not yet fully implemented — this is the
main area of active development.

### Legacy API (`src/main.rs` and `src/api/mod.rs`)

Two overlapping Rocket 0.4 implementations exist. `main.rs` is the original
with hardcoded dispenser state. `api/mod.rs` is a cleaner refactor. Both
expose similar routes:

```
GET  /REST/get/liquids
GET  /REST/get/glasses
POST /REST/post/cocktail
```

These are being superseded by the `v1` async API defined in `API.yaml`.

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

## Test Data

Sample cocktail recipes in `testdata/` are JSON files used for manual API
testing. No automated test suite is currently configured. There are no
`#[cfg(test)]` modules and the `test-case` dependency is commented out in
`Cargo.toml`.

To run manual tests, start the server and POST/GET against the endpoints using
the JSON files as request bodies where applicable.

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
- Error types use custom enums (e.g., `BarBotError` in `main.rs`).
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

1. **Implement handler sub-modules** in `src/server/` (`status.rs`,
   `control.rs`, `config.rs`, `sensors.rs`, `dispense.rs`, `cleaning.rs`).
2. **Reconcile the two API implementations** — `src/main.rs` vs `src/api/mod.rs`.
   The long-term target is the async `v1` API.
3. **Add automated tests** — re-enable the `test-case` dependency and add
   `#[cfg(test)]` modules, especially for HAL trait behavior.
4. **Implement `StorageHal`** — persistent config read/write is defined but
   not yet implemented.
5. **Fix the typo** in `API.yaml` line 82 (`integerlö` → `integer`).

---

## Important Notes for AI Assistants

- **Do not break the HAL trait interface** (`src/hal/mod.rs`). It is the
  public contract for hardware implementors.
- **Both `main.rs` and `api/mod.rs` define `fn main()`** — only one can be the
  binary entry point. The project currently compiles with `main.rs` as the
  default. When implementing the new async server, coordinate which entry point
  to use.
- **The `generic_cocktail` crate is a local path dependency** at
  `../generic-cocktail`. If that directory is missing, the build will fail.
- **Cargo.lock is gitignored** — do not add it.
- **No environment variables or `.env` files** are used; all configuration is
  currently hardcoded or loaded via the HAL traits at runtime.
- **Port 8000** is the default for the Rocket server. The async server port
  is determined by the embassy-net configuration (not yet finalized).
