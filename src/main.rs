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

use hal::{
    CleaningHal, ConfigHal, ControlHal, DispenseHal, ErrorInfo,
    GlassSensorState, JobItem, JobStatus, LevelState, RobotConfig, RobotState,
    SensorHal, StatusHal, StorageHal,
};
#[cfg(not(test))]
use server::{ApiServer, RobotHal};

// ============================================================================
// Stub HAL implementations
//
// TODO: Replace each stub with a real hardware driver implementing the
// corresponding trait from src/hal/mod.rs.
// ============================================================================

struct StubControlHal;

impl ControlHal for StubControlHal {
    fn power(&mut self, _on: bool) -> Result<(), ErrorInfo> {
        todo!()
    }
    fn power_save(&mut self, _enabled: bool) -> Result<(), ErrorInfo> {
        todo!()
    }
    fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
    fn reload_config(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubStatusHal;

impl StatusHal for StubStatusHal {
    fn state(&self) -> RobotState {
        RobotState::Off
    }
    fn active_errors(&self) -> Vec<ErrorInfo> {
        vec![]
    }
}

struct StubConfigHal;

impl ConfigHal for StubConfigHal {
    fn get_active_config(&self) -> RobotConfig {
        todo!()
    }
    fn update_active_config(
        &mut self,
        _cfg: RobotConfig,
    ) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubStorageHal;

impl StorageHal for StubStorageHal {
    fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo> {
        todo!()
    }
    fn store_storage_config(
        &mut self,
        _cfg: RobotConfig,
        _overwrite: bool,
    ) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubSensorHal;

impl SensorHal for StubSensorHal {
    fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        todo!()
    }
    fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        todo!()
    }
}

struct StubDispenseHal;

impl DispenseHal for StubDispenseHal {
    fn create_job(
        &mut self,
        _client_job_id: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<String, ErrorInfo> {
        todo!()
    }
    fn list_jobs(&self) -> Vec<JobStatus> {
        vec![]
    }
    fn job_status(&self, _job_id: &str) -> Result<JobStatus, ErrorInfo> {
        todo!()
    }
    fn cancel_job(&mut self, _job_id: &str) -> Result<(), ErrorInfo> {
        todo!()
    }
}

struct StubCleaningHal;

impl CleaningHal for StubCleaningHal {
    fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
    fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        todo!()
    }
}

// ============================================================================
// Entry point
//
// TODO (ESP32 bring-up): Replace this stub with the BSP-provided async entry
// point.  The typical pattern with esp-hal + esp-hal-embassy is:
//
//   #[esp_hal::main]
//   async fn main(_spawner: embassy_executor::Spawner) {
//       let peripherals = esp_hal::init(esp_hal::Config::default());
//       esp_hal_embassy::init(/* timer */);
//       // initialise embassy-net stack with esp-wifi ...
//       let net_stack = /* ... */;
//       ApiServer { hal: RobotHal { /* real drivers */ } }
//           .run(net_stack)
//           .await;
//   }
//
// embassy-executor's #[embassy_executor::main] macro is only available for
// cortex-m / riscv32 / avr arch features; std/spin targets use the raw
// Executor API.  Actual ESP32 deployment will use esp-hal's entry macro.
// ============================================================================

#[cfg(not(test))]
fn main() {
    // Placeholder — real MCU entry point is async (see TODO above).
    let mut control = StubControlHal;
    let mut status = StubStatusHal;
    let mut config = StubConfigHal;
    let mut storage = StubStorageHal;
    let mut sensors = StubSensorHal;
    let mut dispense = StubDispenseHal;
    let mut cleaning = StubCleaningHal;

    let _server = ApiServer {
        hal: RobotHal {
            control: &mut control,
            status: &mut status,
            config: &mut config,
            storage: &mut storage,
            sensors: &mut sensors,
            dispense: &mut dispense,
            cleaning: &mut cleaning,
        },
    };
    // Real call: _server.run(net_stack).await — inside the BSP async executor.
}
