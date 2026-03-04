## Why

The project contains two dead Rocket 0.4 entry points (`src/main.rs` and `src/api/mod.rs`) that conflict with the async-first `src/server/` implementation built on embassy-net. Rocket 0.4 depends on `std` and synchronous I/O, making it incompatible with the ESP32 no-std target and the embassy async runtime already adopted for actual hardware support. Removing Rocket eliminates the dual-`fn main()` conflict, shrinks the dependency tree, and allows the async server to become the sole authoritative entry point.

## What Changes

- **BREAKING** Remove `src/main.rs` (Rocket entry point — currently the binary default)
- **BREAKING** Remove `src/api/mod.rs` (second Rocket entry point)
- Remove `rocket` and `rocket_contrib` from `Cargo.toml`
- Promote `src/server/mod.rs` (embassy-net async server) as the sole binary entry point via `#[embassy_executor::main]`
- Wire up the remaining unimplemented handler sub-modules in `src/server/` (status, control, config, sensors, dispense, cleaning) so the API surface matches `API.yaml`
- Fix known typo in `API.yaml` line 82 (`integerlö` → `integer`)

## Capabilities

### New Capabilities

- `async-http-server`: A fully wired async HTTP server (embassy-net) that dispatches all `API.yaml` v1 routes to their handler sub-modules with Bearer token auth, replacing the Rocket stubs.

### Modified Capabilities

- `bearer-auth`: Auth enforcement moves from Rocket middleware to the existing `handle_connection` check in `src/server/mod.rs` — no requirement change, implementation is already present; no delta spec needed.

## Impact

- **Deleted files**: `src/main.rs`, `src/api/mod.rs`
- **Cargo.toml**: Remove `rocket`, `rocket_contrib` dependencies
- **src/server/mod.rs**: Add `#[embassy_executor::main]` entry point, complete route dispatch
- **src/server/handlers/**: Implement all handler sub-modules (status, control, config, sensors, dispense, cleaning)
- **API.yaml**: Fix typo on line 82
- No changes to `src/hal/mod.rs` (HAL trait interface stays intact)
- No changes to `src/esp32/` (no-std stubs unaffected)
