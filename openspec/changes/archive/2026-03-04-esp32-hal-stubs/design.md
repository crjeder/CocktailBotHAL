## Context

The HAL trait interface (`src/hal/mod.rs`) defines 7 traits covering all robot
functionality. Mock implementations exist in `src/hal/tests.rs` for unit
testing. No production hardware driver exists yet.

The target platform is ESP32 running bare-metal Rust using the `no_std` +
`alloc` model. The existing codebase uses `core::time::Duration` and
`alloc::{String, Vec}` to stay portable, which aligns well with ESP32 targets.

Relevant commented-out dependencies in `Cargo.toml` signal the intended
direction: `embassy-net`, `embedded-io-async`, `embassy_time` — all part of the
Embassy async embedded Rust ecosystem.

## Goals / Non-Goals

**Goals:**
- Create a `src/esp32/` module with stub `struct` implementations for all 7 HAL traits
- Each stub compiles under `no_std` + `alloc` (ESP32 target)
- Stubs return hardcoded/default values with clear `// TODO: wire to hardware` comments
- Gate the module behind an `esp32` Cargo feature so host builds are unaffected
- Serve as the skeleton for incremental hardware integration

**Non-Goals:**
- Real GPIO/SPI/I2C peripheral access (future work)
- Embassy async task integration (future work)
- ESP-IDF or `esp-idf-svc` dependencies (not needed for stubs)
- Flash/NVS storage access for `StorageHal` (future work)

## Decisions

### Decision 1: Feature-gated module, not a separate crate

**Choice:** `src/esp32/` module behind `[features] esp32 = []` in `Cargo.toml`.

**Rationale:** Keeps the codebase cohesive without introducing a workspace.
The HAL traits are defined in this crate, so the implementation naturally lives
here too. A separate crate would require publishing or path dependencies.

**Alternative considered:** A sibling crate `cocktail_bot_hal_esp32`. Rejected
because it adds workspace complexity before any real hardware code exists.

### Decision 2: One file per logical trait group

**Choice:** Split into `control.rs`, `status.rs`, `config.rs`, `storage.rs`,
`sensors.rs`, `dispense.rs`, `cleaning.rs` under `src/esp32/`.

**Rationale:** Mirrors the server handler sub-module layout. Each file can be
owned by the developer wiring that peripheral. Avoids a single huge file.

**Alternative considered:** Single `src/esp32/mod.rs` with all impls. Rejected
as it would become unwieldy once real hardware code is added.

### Decision 3: Single composite struct `Esp32Hal` that implements all traits

**Choice:** One `struct Esp32Hal` in `src/esp32/mod.rs` that holds sub-structs
and delegates to them. Each sub-struct implements its trait.

**Rationale:** `RobotHal` in `src/server/mod.rs` already composes HAL traits
via trait objects. Providing a concrete `Esp32Hal` that bundles all sub-impls
makes it easy to construct a `RobotHal` for the ESP32 target.

### Decision 4: No new dependencies for stubs

**Choice:** Stubs use only `alloc`, `core`, and the crate's own HAL types.

**Rationale:** Adding embassy or esp-idf crates requires cross-compilation
toolchain setup. Stubs should compile in CI on the host (with `esp32` feature)
without a full embedded toolchain. Real peripherals can be added behind
additional feature flags or once the toolchain is established.

## Risks / Trade-offs

- [Risk] Stubs return fake data → callers can't distinguish stubs from real hardware at runtime.
  → Mitigation: Log a prominent `// STUB` warning message in each method body
  (as a comment now, wired to a logging backend later).

- [Risk] `no_std` compile errors on host when `esp32` feature is enabled.
  → Mitigation: The module uses only `core` and `alloc` which are available in
  both `std` and `no_std` environments. The `esp32` feature does not change
  `#![no_std]` status of the crate — that is a per-binary concern.

- [Trade-off] Feature flag `esp32` on a library crate is non-standard; normally
  the final binary crate controls `no_std`. This is acceptable for a HAL crate
  that may eventually target only embedded.

## Open Questions

- Should `StorageHal` stubs simulate NVS (Non-Volatile Storage) via a static
  `Mutex<Option<RobotConfig>>`? For now, return `Err` with a "not implemented"
  error.
- Which Embassy version and ESP32 board package to use once real peripherals
  are wired? This decision can be deferred to a follow-up change.
