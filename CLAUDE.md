## Project Overview

**CocktailBotHAL** is a Rust-based Hardware Abstraction Layer (HAL) for remotely
controlling autonomous cocktail mixing robots via a REST API. It defines a
trait-based interface that hardware vendors implement, and exposes a standardized
OpenAPI 3.1.0 HTTP API to clients. Target platform: ESP32. See `openspec/config.yaml`
for full tech stack, architecture, and code conventions.

- **Language:** Rust (Edition 2021)
- **License:** GNU GPL v3
- **Crate name:** `cocktail_bot_hal` v0.1.0
- Follow [semantic versioning](https://semver.org/)

---

## Session Start
1. Read `claude-progress.txt`
2. Read `git log --oneline -10`
3. Work on exactly ONE task

## Session End
1. `git commit` with descriptive message
2. Update `claude-progress.txt`
3. Document relevant changes in `CLAUDE.md` and `openspec/config.yaml`
4. Update `TODO.md` to reflect the changes
5. Document changes in CHANGELOG.md in the format of @https://keepachangelog.com/en/1.1.0/

---

## Repository Structure

```
CocktailBotHAL/
├── src/
│   ├── lib.rs           # Library crate root (pub mod hal, pub mod server)
│   ├── hal/mod.rs       # Core HAL trait definitions and data types
│   └── server/mod.rs    # Async HTTP server (embassy-net)
├── examples/
│   ├── dev/             # Host development example (spin executor + stubs)
│   │   ├── main.rs      # Entry point; all stub HAL impls + executor setup
│   │   └── storage.rs   # RamStorageHal (dev-only, RAM-backed StorageHal)
│   └── esp32/           # ESP32 reference implementation
│       ├── main.rs      # Entry point; wires sub-structs into ApiServer
│       ├── control.rs   # Esp32Control (ControlHal stub)
│       ├── status.rs    # Esp32Status (StatusHal stub)
│       ├── config.rs    # Esp32Config (ConfigHal stub)
│       ├── storage.rs   # Esp32Storage (StorageHal stub, NOT_IMPLEMENTED)
│       ├── sensors.rs   # Esp32Sensors (SensorHal stub)
│       ├── dispense.rs  # Esp32Dispense (DispenseHal stub)
│       ├── cleaning.rs  # Esp32Cleaning (CleaningHal stub)
│       └── hasher.rs    # Esp32PasswordHasher (PBKDF2-HMAC-SHA256)
├── openspec/
│   ├── config.yaml      # OpenSpec config — project context for AI spec generation
│   └── specs/           # Living specs
├── testdata/            # Sample cocktail recipes (manual API testing)
├── API.yaml             # OpenAPI 3.1.0 specification
└── Cargo.toml           # Rust project manifest
```

---

## Build & Run

```bash
cargo check
cargo build                          # Debug build (library only)
cargo run --example dev              # Run development server (port 8000)
cargo build --example esp32 --features esp32   # Build ESP32 example
cargo test                           # Run all tests (library only)
```

No Makefile or Docker. Standard Cargo only.

---

## Important Notes for AI Assistants

- **Do not break the HAL trait interface** (`src/hal/mod.rs`). It is the
  public contract for hardware implementors.
- **`src/lib.rs` is the crate root** — the library exports `hal` and `server`
  only. No concrete HAL implementations live in `src/`.
- **Examples are not part of the library** — `examples/dev/` and `examples/esp32/`
  are Cargo examples, compiled separately. Changes there do not affect `cargo test`.
- **ESP32 bring-up**: replace `examples/esp32/main.rs` spin executor with
  `#[esp_hal::main]` and wire to esp-hal-embassy + esp-wifi.
- **Cargo.lock is gitignored** — do not add it.
- **No `.env` files** — all config is hardcoded or loaded via HAL traits at runtime.
- **ESP32 example code** must use only `core` and `alloc` — no `std` imports.
- Before adding any dependency, check commented-out entries in `Cargo.toml` first.
- Run `cargo fmt` before every commit.
- Keep **API.yaml** up-to date
