// examples/esp32/main.rs
//
// ESP32 reference entry point for CocktailBotHAL.
//
// This example demonstrates how to wire all HAL trait implementations into
// ApiServer for an ESP32 target. All sub-modules are stub implementations
// with TODO comments marking hardware wiring points.
//
// TODO (ESP32 bring-up): Replace the spin executor below with:
//
//   #[esp_hal::main]
//   async fn main(spawner: embassy_executor::Spawner) {
//       let peripherals = esp_hal::init(esp_hal::Config::default());
//       esp_hal_embassy::init(/* timer */);
//       // initialise embassy-net stack with esp-wifi ...
//       let net_stack = /* ... */;
//       spawner.spawn(sse_task(/* ... */)).unwrap();
//       ApiServer { hal: RobotHal { /* real drivers */ } }
//           .run(net_stack)
//           .await;
//   }
//
// Required Cargo.toml additions for real ESP32 bringup:
//   esp-hal, esp-hal-embassy, esp-wifi, esp-alloc

extern crate alloc;

mod cleaning;
mod config;
mod control;
mod dispense;
mod hasher;
mod sensors;
mod status;
mod storage;

use alloc::vec::Vec;

use cleaning::Esp32Cleaning;
use cocktail_bot_hal::hal::{ErrorInfo, RobotState};
use cocktail_bot_hal::server::{ApiServer, RobotHal};
use config::Esp32Config;
use control::Esp32Control;
use dispense::Esp32Dispense;
use embassy_executor::{Executor, Spawner};
use hasher::Esp32PasswordHasher;
use sensors::Esp32Sensors;
use static_cell::StaticCell;
use status::Esp32Status;
use storage::Esp32Storage;

// Static instances for the SSE read path.
static SSE_STATUS: StaticCell<Esp32Status> = StaticCell::new();
static SSE_DISPENSE: StaticCell<Esp32Dispense> = StaticCell::new();
static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// SSE server task.
///
/// TODO: wire `net_stack` from esp-hal-embassy and call:
///   cocktail_bot_hal::server::sse::SseServer { status, dispense }.run(net_stack).await;
#[embassy_executor::task]
async fn sse_task(status: &'static Esp32Status, dispense: &'static Esp32Dispense) {
    let _ = (status, dispense);
}

#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    let sse_status = SSE_STATUS.init(Esp32Status::new());
    let sse_dispense = SSE_DISPENSE.init(Esp32Dispense::new());
    spawner.spawn(sse_task(sse_status, sse_dispense)).unwrap();

    let _server = ApiServer {
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
    // TODO: _server.run(net_stack).await — wire to embassy-net stack.
}

fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}
