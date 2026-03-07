// src/lib.rs
//
// CocktailBotHAL — library crate root.
//
// Exports:
//   hal    — HAL trait definitions, data types, and test mocks
//   server — async HTTP server (ApiServer, SseServer, RobotHal)
//
// Hardware implementations and runnable entry points live in examples/:
//   examples/dev/   — host development example (spin executor + stubs)
//   examples/esp32/ — ESP32 reference implementation

#![allow(dead_code)]

extern crate alloc;

pub mod hal;
pub mod server;

// ---------------------------------------------------------------------------
// embassy-time linker stubs
//
// `mod server` pulls in embassy_time. When running `cargo test` there is no
// real hardware timer driver, so the linker cannot resolve the two symbols
// below. Provide no-op implementations so the test binary links cleanly.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[no_mangle]
fn _embassy_time_now() -> u64 {
    0
}

#[cfg(test)]
#[no_mangle]
fn _embassy_time_schedule_wake(_at: u64, _waker: &core::task::Waker) {}
