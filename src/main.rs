// src/main.rs
//
// Placeholder entry point for CocktailBotHAL.
//
// TODO: Replace this file with a hardware-specific entry point targeting
// your MCU (STM32, ESP32, RP2040, etc.). See TODO.md for the full list of
// open work, including the required Cargo.toml dependencies.

// HAL traits and types are a public API for hardware vendors; they will not
// all be referenced from within the crate itself.
#![allow(dead_code)]

extern crate alloc;

#[cfg(feature = "esp32")]
mod esp32;
mod hal;
#[cfg(not(test))]
mod server;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

#[cfg(not(test))]
use embassy_executor::Executor;
use hal::{
    CleaningHal, ConfigHal, ControlHal, DispenseHal, ErrorInfo, GlassSensorState, JobItem,
    JobStatus, LevelState, RobotConfig, RobotState, SensorHal, StatusHal, StorageHal,
};
#[cfg(not(test))]
use server::{ApiServer, RobotHal};
#[cfg(not(test))]
use static_cell::StaticCell;

// ============================================================================
// Stub HAL implementations
//
// TODO: Replace each stub with a real hardware driver implementing the
// corresponding trait from src/hal/mod.rs.
// ============================================================================

struct StubControlHal;

impl ControlHal for StubControlHal {
    async fn power(&mut self, _on: bool) -> Result<(), ErrorInfo> {
        todo!()
    }
    async fn power_save(&mut self, _enabled: bool) -> Result<(), ErrorInfo> {
        todo!()
    }
    async fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
    async fn reload_config(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubStatusHal;

impl StatusHal for StubStatusHal {
    async fn state(&self) -> RobotState {
        RobotState::Off
    }
    async fn active_errors(&self) -> Vec<ErrorInfo> {
        vec![]
    }
}

struct StubConfigHal;

impl ConfigHal for StubConfigHal {
    async fn get_active_config(&self) -> RobotConfig {
        todo!()
    }
    async fn update_active_config(&mut self, _cfg: RobotConfig) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubStorageHal;

impl StorageHal for StubStorageHal {
    async fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo> {
        todo!()
    }
    async fn store_storage_config(
        &mut self,
        _cfg: RobotConfig,
        _overwrite: bool,
    ) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubSensorHal;

impl SensorHal for StubSensorHal {
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        todo!()
    }
    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        todo!()
    }
}

struct StubDispenseHal;

impl DispenseHal for StubDispenseHal {
    async fn create_job(
        &mut self,
        _client_job_id: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<String, ErrorInfo> {
        todo!()
    }
    async fn list_jobs(&self) -> Vec<JobStatus> {
        vec![]
    }
    async fn job_status(&self, _job_id: &str) -> Result<JobStatus, ErrorInfo> {
        todo!()
    }
    async fn cancel_job(&mut self, _job_id: &str) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubCleaningHal;

impl CleaningHal for StubCleaningHal {
    async fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
    async fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
}

// ============================================================================
// Entry point
//
// Runs the embassy spin executor for host/development builds.
// Constructs all stub HAL instances and creates ApiServer.
//
// TODO (ESP32 bring-up): Replace this entire section with the BSP-provided
// async entry point using esp-hal:
//
//   #[esp_hal::main]
//   async fn main(spawner: embassy_executor::Spawner) {
//       let peripherals = esp_hal::init(esp_hal::Config::default());
//       esp_hal_embassy::init(/* timer */);
//       // initialise embassy-net stack with esp-wifi ...
//       let net_stack = /* ... */;
//       ApiServer { hal: RobotHal { /* real drivers */ } }
//           .run(net_stack)
//           .await;
//   }
//
// NOTE: #[embassy_executor::main] is unavailable with arch-spin.
// The spin executor is initialised manually below.
// ============================================================================

#[cfg(not(test))]
static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// Async stub entry task — constructs all HAL stubs and the API server.
///
/// In a real bring-up this function is replaced by the BSP entry point and
/// wired to actual hardware drivers and a live embassy-net stack.
#[cfg(not(test))]
#[embassy_executor::task]
async fn async_main() {
    let control = StubControlHal;
    let status = StubStatusHal;
    let config = StubConfigHal;
    let storage = StubStorageHal;
    let sensors = StubSensorHal;
    let dispense = StubDispenseHal;
    let cleaning = StubCleaningHal;

    let _server = ApiServer {
        hal: RobotHal {
            control,
            status,
            config,
            storage,
            sensors,
            dispense,
            cleaning,
        },
    };
    // Real call: _server.run(net_stack).await — wire to embassy-net stack.
}

#[cfg(not(test))]
fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(async_main()).unwrap();
    });
}
