## Why

The ESP32 example currently uses `embassy-executor`'s spin executor (`arch-spin`),
which is a host-compatible shim unsuitable for real ESP32 hardware. Actual ESP32
bring-up requires `esp-hal` + `esp-hal-embassy` to integrate with the hardware
timers and interrupt system that embassy-executor needs to schedule tasks correctly.

## What Changes

- Replace `fn main()` + `StaticCell<Executor>` spin-executor pattern in
  `examples/esp32/main.rs` with `#[esp_hal::main] async fn main(spawner: Spawner)`
- Add `esp-hal`, `esp-hal-embassy`, `esp-wifi`, and `esp-alloc` to `Cargo.toml`
  under the `esp32` feature gate
- Wire `embassy-net` stack through `esp-wifi` inside the new entry point (stub
  with `todo!()` placeholders for Wi-Fi credentials and peripheral bindings)
- Remove the `arch-spin` feature from `embassy-executor` in the `esp32` feature set
  (it conflicts with `esp-hal-embassy`'s own embassy integration)
- Update the `sse_task` and `async_main` tasks to receive the network stack from
  the new entry point

## Capabilities

### New Capabilities

*(none — no new API surface or HAL traits)*

### Modified Capabilities

- `esp32-hal-impl`: The ESP32 example entry point changes from a spin-executor
  stub to the real `#[esp_hal::main]` async entry point with proper hardware
  timer integration and an embassy-net stack wired through esp-wifi.

## Impact

- **Files changed**: `examples/esp32/main.rs`, `Cargo.toml`
- **No library changes** (`src/` is untouched)
- **No HAL trait changes** — existing `Esp32*` structs are reused as-is
- **New dependencies** (esp32-feature-gated): `esp-hal`, `esp-hal-embassy`,
  `esp-wifi`, `esp-alloc`
- **Removes dependency** on `arch-spin` feature of `embassy-executor` for ESP32
  target (still used for the `dev` example on host)
- The `dev` example (`examples/dev/main.rs`) is unaffected
