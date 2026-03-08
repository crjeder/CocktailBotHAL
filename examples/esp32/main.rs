// examples/esp32/main.rs
//
// ESP32 reference entry point for CocktailBotHAL.
//
// CROSS-COMPILATION REQUIRED
// This file uses #[esp_hal::main] and can only be built for an ESP32 target:
//
//   Xtensa (ESP32, ESP32-S2, ESP32-S3):
//     cargo build --example esp32 --features esp32 \
//       --target xtensa-esp32s3-none-elf
//
//   RISC-V (ESP32-C3, C6, H2):
//     cargo build --example esp32 --features esp32 \
//       --target riscv32imc-unknown-none-elf
//
// Install the toolchain first:
//   cargo install espup && espup install
//
// Note: the library currently depends on `serde_json` which links against `std`.
// For a bare-metal no_std ESP32 build, replace `serde_json` with
// `serde-json-core`, or enable serde_json's alloc feature in Cargo.toml:
//   serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
//
// All hardware-specific bindings are marked with todo!() and must be
// completed for actual hardware bring-up.
#![no_std]
#![no_main]

extern crate alloc;

mod cleaning;
mod config;
mod control;
mod dispense;
mod hasher;
mod sensors;
mod status;
mod storage;

use cleaning::Esp32Cleaning;
use cocktail_bot_hal::server::{ApiServer, RobotHal};
use config::Esp32Config;
use control::Esp32Control;
use dispense::Esp32Dispense;
use hasher::Esp32PasswordHasher;
use sensors::Esp32Sensors;
use status::Esp32Status;
use storage::Esp32Storage;

// Heap allocator — must appear at module level before any heap allocation.
// Adjust the size for the RAM available on your chip variant.
esp_alloc::heap_allocator!(72 * 1024);

/// Main async entry point — provided by esp-hal.
///
/// esp-hal initialises chip peripherals and passes a `Spawner` here.
/// ApiServer serves all REST routes and SSE (GET /v1/events) on port 80.
#[esp_hal::main]
async fn main(_spawner: embassy_executor::Spawner) {
    // 1. Initialise esp-hal (clocks, GPIO, watchdog, etc.).
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // 2. Initialise the embassy timer for async task scheduling.
    //    Uncomment the correct block for your chip variant.
    //
    //    Classic ESP32 / ESP32-S2 — timer group (TIMG):
    //      let timg0 =
    //          esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    //      esp_hal_embassy::init(timg0.timer0);
    //
    //    ESP32-S3, C3, C6, H2 — system timer (preferred):
    //      let systimer =
    //          esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    //      esp_hal_embassy::init(systimer.alarm0.into_periodic());
    //
    todo!("uncomment and complete the embassy timer init block for your chip");

    // 3. Initialise Wi-Fi and build the embassy-net Stack.
    //    See https://github.com/esp-rs/esp-wifi for full setup examples.
    //    Sketch:
    //      let init = esp_wifi::init(..., peripherals.RADIO_CLK, &clocks)?;
    //      let (wifi_iface, controller) =
    //          esp_wifi::wifi::new_with_mode(&init, peripherals.WIFI, WifiStaDevice)?;
    //      let config = embassy_net::Config::dhcpv4(Default::default());
    //      let net_stack = embassy_net::Stack::new(wifi_iface, config, ...);
    //
    todo!("initialise esp-wifi and create the embassy-net Stack");

    // 4. Build and run the REST API server.
    //    Replace todo!() in .run() with the embassy-net Stack from step 3.
    //    SSE is served as GET /v1/events on the same port (80).
    let mut server = ApiServer {
        hal: RobotHal {
            control: Esp32Control::new(),
            status: Esp32Status::new(),
            config: Esp32Config::new(),
            storage: Esp32Storage::new(),
            sensors: Esp32Sensors::new(),
            dispense: Esp32Dispense::new(),
            cleaning: Esp32Cleaning::new(),
            hasher: Esp32PasswordHasher,
        },
    };
    server
        .run(todo!("pass the embassy-net Stack from step 3"))
        .await;
}
