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

---

## Repository Structure

```
CocktailBotHAL/
├── src/
│   ├── main.rs          # Placeholder entry point (sync stub; see TODO for async BSP pattern)
│   ├── hal/mod.rs       # Core HAL trait definitions and data types
│   └── server/mod.rs    # Async HTTP server (embassy-net)
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
cargo build          # Debug build
cargo run            # Starts on port 8000
```

No Makefile or Docker. Standard Cargo only.
`generic_cocktail` must be present at `../generic-cocktail` or the build will fail.

---

## Important Notes for AI Assistants

- **Do not break the HAL trait interface** (`src/hal/mod.rs`). It is the
  public contract for hardware implementors.
- **`main.rs` is the only binary entry point** — `api/mod.rs` no longer exists.
  For ESP32 bring-up, replace `fn main()` with `#[esp_hal::main]` async entry point
  (see TODO comment in `src/main.rs`).
- **Cargo.lock is gitignored** — do not add it.
- **No `.env` files** — all config is hardcoded or loaded via HAL traits at runtime.
- **ESP32 code** (`src/esp32/`) must use only `core` and `alloc` — no `std` imports.
- Before adding any dependency, check commented-out entries in `Cargo.toml` first.
- Run `cargo fmt` before every commit.
- Keep **API.yaml** up-to date
