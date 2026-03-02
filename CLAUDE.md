# CLAUDE.md — CocktailBotHAL

Developer guide for AI assistants working on this repository.

---

## Project Overview

**CocktailBotHAL** is a Rust library defining a **Hardware Abstraction Layer (HAL)** for remotely controlled cocktail-mixing robots. The project exposes a set of composable Rust traits that hardware implementations must fulfil, together with a REST API server that bridges HTTP clients to those traits.

- **Language**: Rust (Edition 2021)
- **Version**: 0.1.0 (early development)
- **License**: GPL-3.0
- **External local dependency**: `../generic-cocktail` — must exist as a sibling directory

---

## Repository Structure

```
CocktailBotHAL/
├── API.yaml              # OpenAPI 3.1.0 specification (source of truth for routes)
├── Cargo.toml            # Rust manifest and dependencies
├── rustfmt.toml          # Formatter configuration (enforce before every commit)
├── README.md             # High-level project description
├── src/
│   ├── main.rs           # Rocket-based REST server (prototype / "quick and dirty")
│   ├── hal/
│   │   └── mod.rs        # HAL trait definitions and all shared data types
│   └── server/
│       └── mod.rs        # Embassy-based async server and route dispatcher
└── testdata/
    ├── margarita.json    # Sample cocktail recipe fixture
    ├── vesper.json       # Sample cocktail recipe fixture
    └── vesper2.json      # Alternative Vesper variant fixture
```

---

## Architecture

### Two Parallel Approaches

The project is evolving from a quick prototype toward a proper embedded-first design. Both exist simultaneously in the tree:

| Module | Framework | Purpose |
|---|---|---|
| `src/main.rs` | Rocket 0.4 (sync) | Prototype REST server; runs on a standard host |
| `src/server/mod.rs` | Embassy (async) | Production-targeted embedded-friendly server |

The **Embassy server** (`src/server/mod.rs`) is the long-term target. New work should prefer it.

### HAL Trait System (`src/hal/mod.rs`)

The HAL is split into seven focused traits. Any hardware implementation must implement the relevant subset:

| Trait | Responsibility |
|---|---|
| `ControlHal` | Power on/off, power-save mode, error reset, config reload |
| `StatusHal` | Query current robot state and active errors |
| `ConfigHal` | Read/write active (RAM) configuration |
| `StorageHal` | Persist and load configuration from non-volatile storage (flash) |
| `SensorHal` | Glass presence/type detection, liquid level readings |
| `DispenseHal` | Create, query, and cancel dispensing jobs |
| `CleaningHal` | Start and stop the cleaning cycle |

All trait methods return `Result<T, ErrorInfo>` (except the two `StatusHal` queries which never fail).

### Robot State Machine

```
Off → Booting → SelfTest → Idle ─┬→ Working → DrinkReady → Idle
                                  ├→ Cleaning → Idle
                                  └→ Error
                         Prepared ─┘ (glass detected, waiting for job)
```

States are defined in `RobotState` enum in `src/hal/mod.rs`.

### `RobotHal` Composition (`src/server/mod.rs`)

The async server aggregates all trait objects into a single `RobotHal<'a>` struct using mutable references:

```rust
pub struct RobotHal<'a> {
    pub control:  &'a mut dyn ControlHal,
    pub status:   &'a mut dyn StatusHal,
    pub config:   &'a mut dyn ConfigHal,
    pub storage:  &'a mut dyn StorageHal,
    pub sensors:  &'a mut dyn SensorHal,
    pub dispense: &'a mut dyn DispenseHal,
    pub cleaning: &'a mut dyn CleaningHal,
}
```

### Dispense Units

Liquids are always measured in **"parts"** (an abstract, robot-independent unit). One part equals `part_ml` millilitres as defined in `RobotConfig`. This keeps recipes portable across different hardware.

---

## Key Data Types

All types live in `src/hal/mod.rs`:

- **`RobotConfig`** — complete runtime config (liquid list, part size, capabilities)
- **`LiquidConfig`** — per-liquid ID, name, pump position, calibration data
- **`LiquidCalibration`** — `ml_per_sec`, `prime_ms`, `viscosity_factor`
- **`Capabilities`** — what the hardware supports (`level_reporting`, `glass_typing`, `simultaneous_channels`)
- **`JobItem`** — `{ liquid_id: String, parts: u32 }`
- **`JobStatus`** — tracks job lifecycle via `JobState` enum
- **`GlassSensorState`** — presence flag, optional type string, confidence float
- **`LevelState`** — enum with `Binary { id, ok }` and `Decimal { id, remaining_ml }` variants
- **`ErrorInfo`** — structured error: `code`, `message`, optional `hint`, `recoverable` flag

---

## API Specification

`API.yaml` is the **source of truth** for all HTTP endpoints. Always keep implementation and spec in sync.

- **Base URL**: `http://robot.local/v1`
- **Auth**: Bearer token
- **Format**: OpenAPI 3.1.0

Key endpoint groups:

| Prefix | Description |
|---|---|
| `GET /status` | System status and state |
| `POST /control/*` | Power, power-save, reset, reload-config |
| `GET/PATCH /config` | Active (RAM) configuration |
| `GET/POST /storage/config` | Persistent (flash) configuration |
| `GET /sensors/glass` | Glass detection |
| `GET /sensors/levels` | Liquid levels |
| `POST /dispense/jobs` | Create dispensing job |
| `GET /dispense/jobs/{job_id}` | Query job status |
| `POST /dispense/jobs/{job_id}` | Cancel job |
| `POST /cleaning/start|stop` | Cleaning lifecycle |
| `GET /events` | Server-Sent Events stream |

The async `src/server/mod.rs` router matches on `(method, path)` pairs. Unmatched routes return a JSON 404.

---

## Dependencies

### Runtime
| Crate | Version | Purpose |
|---|---|---|
| `rocket` | 0.4 | Prototype REST server (sync) |
| `rocket_contrib` | 0.4.11 | JSON support for Rocket |
| `serde` | 1.0 | Serialisation framework |
| `serde_json` | 1.0 | JSON (de)serialisation |
| `serde_derive` | 1.0 | Derive macros for Rocket |
| `generic_cocktail` | 0.1 (local path) | Cocktail recipe type definitions |
| `embassy_time` | (implicit) | `Duration` used in `DispenseHal::create_job` |
| `embassy_net` | (implicit) | `TcpSocket` used in the async server |
| `embedded_io_async` | (implicit) | Async `Write` trait for socket I/O |

### Local path dependency
`generic_cocktail` is resolved from `../generic-cocktail`. This sibling repo **must be present** for the project to compile.

### Commented-out / considered crates
Several crates are commented out in `Cargo.toml` — do not uncomment them unless implementing the corresponding feature. Candidates include `tokio`, `reqwest`, `regex`, `slint` (UI).

---

## Code Style

Enforced by `rustfmt.toml`. Run `cargo fmt` before every commit.

| Setting | Value |
|---|---|
| Max line width | 80 characters |
| Brace style (control flow) | `AlwaysNextLine` |
| Indent style | Block |
| `fn_single_line` | Enabled (trivial one-liners may stay on one line) |
| `use_small_heuristics` | Max |
| `struct_lit_width` | 40 |

**Additional conventions observed in the codebase:**
- Struct definitions use `AlwaysNextLine` braces — opening `{` on its own line.
- `impl` blocks place the opening `{` on a new line.
- Use `pub` / `pub(crate)` visibility explicitly; avoid implicit private-by-default for public API types.
- Derive macros go on a separate `#[derive(...)]` line above the type.
- Error types implement both `fmt::Debug` and `fmt::Display`.
- All HAL trait methods return `Result<T, ErrorInfo>` — do not use `unwrap()` in library code.

---

## Building

```bash
# Build (requires ../generic-cocktail sibling directory)
cargo build

# Build release
cargo build --release

# Check without producing an artifact (faster)
cargo check

# Format (always run before committing)
cargo fmt

# Lint
cargo clippy
```

> **Note:** The project uses `#![feature(decl_macro)]` in `main.rs` for Rocket 0.4 compatibility, which requires a nightly Rust toolchain for that file. The Embassy-based server in `src/server/mod.rs` targets a `no_std` / embedded environment.

---

## Testing

There are currently **no automated tests** (`[dev-dependencies]` section is empty apart from commented-out entries). The `testdata/` directory contains JSON fixtures for manual testing and future unit tests.

When adding tests:
- Place integration tests in `tests/` (standard Rust convention).
- Use the JSON fixtures in `testdata/` as input payloads.
- The commented-out `test-case` crate in `Cargo.toml` is available if parameterised tests are needed.

---

## Incomplete / Planned Areas

The following are explicitly planned but not yet implemented (tracked in README.md):

- [ ] `src/server/handlers/` submodules — `status`, `control`, `config`, `sensors`, `dispense`, `cleaning` — all referenced in `server/mod.rs` but not yet created
- [ ] Scales functionality (tare and calibration) — depends on `hx711_spi` driver
- [ ] Servo control — depends on `pwm-pca9685-rs` driver
- [ ] Pump control
- [ ] Traffic-light UI (Green: drink ready, Yellow: ready, Red: busy)
- [ ] Server-Sent Events (`GET /events`) endpoint
- [ ] Several `server/mod.rs` routes marked with `// TODO: weitere Endpunkte 1:1 nach OpenAPI`

When implementing handlers, follow the pattern in `server/mod.rs`: accept `&self.hal` or `&mut self.hal`, deserialise the request body with `serde_json`, call the appropriate HAL trait method, and respond via `http_smol::write_json`.

---

## Git Conventions

- Branch for AI-assisted work: `ccr-60e8cb3a-ZGjJ4`
- Commit messages are mostly English imperative sentences; some German messages exist in the history — prefer English for new commits.
- Example good messages: `Add HAL module with robot control traits and structs`, `Refactor API.yaml by removing states and properties`
- The `Cargo.lock` is gitignored — do not commit it.

---

## Sensitive / Gotchas

1. **`alloc` vs `std`**: `src/hal/mod.rs` uses `alloc::string::String` and `alloc::vec::Vec` (not `std::`) because it must compile in `no_std` embedded environments. Do not introduce `std` imports into the `hal` module.
2. **Thread safety in Rocket prototype**: `Cocktailbot` state is wrapped in `Mutex<Cocktailbot>` and injected via `rocket::State`. Always lock before mutating.
3. **Hardcoded inventory**: `main.rs` hard-codes 15 liquids and 2 glass types. This is prototype data; the production path is `RobotConfig` loaded from `StorageHal`.
4. **Local path dep**: If `../generic-cocktail` is missing, the build will fail with an unresolvable dependency error — this is not a network issue.
5. **Rocket nightly feature**: `#![feature(decl_macro)]` at the top of `main.rs` pins that file to nightly. The Embassy-based target does not have this constraint.
