## Why

The HAL trait interface is well-defined and tested with mock implementations, but no real hardware driver exists yet. ESP32 is the target microcontroller platform, and stub implementations are needed as the bridge between the abstract HAL traits and actual ESP32 peripheral access — enabling incremental hardware integration and physical testing.

## What Changes

- Add a new `src/esp32/` module containing stub implementations of all 7 HAL traits
- Each stub compiles for `no_std` (ESP32 target) with `alloc` support
- Stubs return sensible default/placeholder values and are clearly marked for future hardware wiring
- A feature flag (`esp32`) gates the module so the project can still compile for `std` (development/test)
- Cargo.toml updated with the `esp32` feature and any ESP32-specific dependencies (e.g., `esp-idf-svc`, `esp-idf-hal`)

## Capabilities

### New Capabilities

- `esp32-hal-impl`: Concrete stub implementations of all HAL traits targeting the ESP32 platform, structured to be extended with real peripheral code

### Modified Capabilities

(none — HAL trait interface is unchanged)

## Impact

- New module `src/esp32/mod.rs` and sub-modules per trait group
- `Cargo.toml`: new `[features]` section for `esp32`, optional ESP32 crate dependencies
- `src/main.rs` / entry point: conditional compilation to select ESP32 impls when feature is active
- No breaking changes to existing HAL traits or mock test code
