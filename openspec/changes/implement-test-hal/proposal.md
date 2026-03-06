## Why

The ESP32 HAL implementations (`src/esp32/`) interact directly with hardware
peripherals (GPIO pins, SPI, I2C), making them untestable without physical
hardware. `embedded-hal-mock` (already in `Cargo.toml`, commented out) provides
transaction-based mock implementations of embedded-hal traits that allow
verifying hardware I/O behavior in unit tests.

## What Changes

- Enable `embedded-hal-mock` and `test-case` dependencies in `Cargo.toml` (dev-deps)
- Add a `src/test_hal/` module: full implementations of all 7 custom HAL traits
  (`ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`, `SensorHal`,
  `DispenseHal`, `CleaningHal`) backed by `embedded-hal-mock` primitives
- Expose `MockPin`, `MockSpi`, `MockI2c` transaction sequences so tests can
  assert exact hardware I/O patterns
- Add `#[cfg(test)]` integration tests in `src/test_hal/tests.rs` covering each
  HAL trait's observable behavior via the mocked hardware layer
- Update `Cargo.toml` to gate `embedded-hal-mock` under `[dev-dependencies]`
  (no production build impact)

## Capabilities

### New Capabilities

- `test-hal`: A test-only HAL implementation backed by `embedded-hal-mock`
  that simulates hardware I/O for unit and integration testing of the ESP32
  HAL layer without requiring physical hardware.

### Modified Capabilities

*(none — no existing spec requirements are changing)*

## Impact

- **`Cargo.toml`**: Uncomment `embedded-hal-mock = "0.7.2"` and `test-case = "3.2"`
  as `[dev-dependencies]`
- **`src/test_hal/`**: New module (dev/test only); no changes to production code paths
- **`src/hal/mod.rs`**: No changes — trait interface is stable
- **`src/esp32/`**: No changes — existing impls are what the test HAL will exercise
- **No API changes**, no version bump required (dev tooling only)
