## Context

Rocket has already been removed from `Cargo.toml` and `src/api/mod.rs` no longer exists. The async server in `src/server/` is functional and all 6 handler sub-modules are implemented. However, `src/main.rs` still uses a synchronous `fn main()` — it constructs `ApiServer` but never calls `.run()` inside an async executor. `embassy-executor` is also not yet a dependency. Project documentation (`CLAUDE.md`, `openspec/config.yaml`) still references Rocket as an active tech-stack item.

**Current state:**
- `src/main.rs`: synchronous `fn main()`, constructs `ApiServer` but cannot call `.run()`
- `Cargo.toml`: has `embassy-net`, `embassy-time`, `embedded-io-async` — no `embassy-executor`
- `src/server/`: fully wired route dispatch + Bearer auth + 6 handler sub-modules
- Docs: still list Rocket as part of the stack

## Goals / Non-Goals

**Goals:**
- Add `embassy-executor` to `Cargo.toml` with appropriate features for the development/host target
- Replace synchronous `fn main()` in `src/main.rs` with `#[embassy_executor::main]` so `ApiServer::run()` is actually called
- Remove all Rocket references from `CLAUDE.md` and `openspec/config.yaml`
- Fix the known typo in `API.yaml` line 82 (`integerlö` → `integer`)

**Non-Goals:**
- Implementing real hardware drivers (stubs in `src/main.rs` stay)
- Implementing `StorageHal` (separate open work item)
- Adding automated tests (separate open work item)
- Changing the HAL trait interface

## Decisions

### embassy-executor features

**Decision**: Use `arch-std` + `executor-thread` features for the current development/host build. The ESP32 target will use a different executor configuration when the `esp32` feature is active, controlled via a feature flag in `Cargo.toml`.

**Rationale**: The project compiles for a host std environment right now (no `no_std` yet for `main.rs` itself; only `src/esp32/` is `no_std`). Using `arch-std` lets the async entry point work in a standard environment for development and CI. When the ESP32 bring-up happens, the executor feature can be swapped to `arch-xtensa` or a board-specific HAL executor via a feature flag, without touching the server code.

**Alternative considered**: `executor-interrupt` for bare-metal ESP32 — deferred because ESP32-specific bring-up is a separate work item.

### Async entry point macro

**Decision**: Keep `fn main()` synchronous for the host placeholder. Document the expected BSP-provided entry point (`#[esp_hal::main]`) in a TODO comment block. Do not use `#[embassy_executor::main]` in this file.

**Rationale**: `#[embassy_executor::main]` is only available when an arch feature (`arch-cortex-m`, `arch-riscv32`, `arch-avr`) is active — it is gated out for generic arch features like `arch-spin` and `arch-std`. For ESP32 with embassy, the correct async entry point is provided by `esp-hal` + `esp-hal-embassy` BSP packages, not by `embassy-executor` directly. The host placeholder is scaffolding only; forcing a broken async entry point provides no value.

**Alternative considered**: `#[embassy_executor::main]` with `arch-spin` or `arch-std` — rejected because the macro is not available for these features in embassy-executor 0.9.x.

### Network stack in development builds

**Decision**: Keep the net stack initialization stubbed with a `todo!()` comment in `main.rs`. The type signature of `ApiServer::run` already takes `embassy_net::Stack<'_>`, so the async wiring can be completed without a real NIC driver.

**Rationale**: The goal of this change is to make the entry point structurally async-correct, not to provide a working network stack for the host. A `todo!()` is honest about what remains.

## Risks / Trade-offs

- [Risk] `embassy-executor` version must be compatible with `embassy-net 0.8.0` and `embassy-time 0.5.0` → **Mitigation**: Pin to the same release epoch (embassy 0.5.x / 0.8.x are contemporaneous); check `embassy-net`'s dependency tree with `cargo tree` to confirm.
- [Risk] `arch-std` requires `std` which is incompatible with ESP32 no-std → **Mitigation**: Gate the `arch-std` feature behind `#[cfg(not(feature = "esp32"))]` in Cargo.toml using a feature-conditional dependency.
- [Risk] Spurious dead-code warnings from stub HAL impls after removing `#![allow(dead_code)]` → **Mitigation**: Keep `#![allow(dead_code)]` at crate level; HAL traits are a public API not called from within the crate.

## Migration Plan

1. Add `embassy-executor` to `Cargo.toml` (std feature for default, bare-metal for esp32 feature)
2. Update `src/main.rs`: add `#[embassy_executor::main]`, convert `fn main()` to `async fn main(spawner)`
3. Fix `API.yaml` line 82 typo
4. Update `CLAUDE.md`: remove Rocket from tech stack, update entry point notes
5. Update `openspec/config.yaml`: remove Rocket from tech stack listing
6. `cargo check` to verify clean compile

No rollback required — all changes are file edits in a version-controlled repo.

## Open Questions

- Should the `async fn main` call `spawner.spawn(server_task(...))` or just `.await` directly on `ApiServer::run()`? Either works; `.await` is simpler for a single-task server.
