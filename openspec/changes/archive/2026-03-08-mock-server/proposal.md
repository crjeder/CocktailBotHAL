## Why

Client developers and integration testers need a fully functional server that responds to all API endpoints with realistic, stateful behavior — without requiring real robot hardware. The existing `dev` example only exercises the HTTP layer with stub HAL implementations that return empty/hardcoded responses; it cannot simulate real robot workflows (dispensing, cleaning, glass detection, error recovery).

## What Changes

- Add a new `mock-server` binary example (`examples/mock-server/`) that runs a complete, stateful HTTP server on the host (no ESP32 required)
- Implement a `MockStateEngine` that drives `RobotState` through realistic transitions (Idle → Prepared → Working → DrinkReady, Cleaning, Error, etc.) with configurable timing
- Implement all HAL traits with stateful in-memory behavior (not mere stubs): dispensing jobs progress over time, sensor readings cycle, glass detection responds to API calls, config persists across requests
- Expose a control surface (CLI flags or a secondary HTTP endpoint) to inject faults, set glass state, override sensor readings, and trigger error conditions — enabling client-side error-handling tests
- Serve the same OpenAPI 3.1.0 contract as the real robot

## Capabilities

### New Capabilities
- `mock-server`: A host-runnable binary example that provides a fully stateful, scriptable mock of the robot REST API for client integration testing

### Modified Capabilities
<!-- No existing spec-level requirements change. The mock-server is a new example that consumes the existing HAL traits without modifying them. -->

## Impact

- New files: `examples/mock-server/` (main.rs + state engine + per-trait impls)
- No changes to `src/hal/mod.rs`, `src/server/`, or `API.yaml`
- No new library dependencies beyond those already in Cargo.toml (uses `std` via existing `dev` example pattern); may add `tokio` or `smol` if spin executor is insufficient for host-side timer ticks
- Existing `cargo test` and `cargo check` unaffected
- New Cargo target: `[[example]] name = "mock-server"`
