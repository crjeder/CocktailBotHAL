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

#[cfg(not(test))]
use embassy_executor::{Executor, Spawner};
use hal::{
    Capabilities, CleaningHal, ConfigHal, ControlHal, DispenseHal, ErrorInfo, GlassSensorState,
    GlassType, JobCreated, JobItem, JobStatus, LevelReporting, LevelState, LiquidCalibration,
    LiquidConfig, PasswordHasher, RobotConfig, RobotState, SensorHal, StatusHal, StorageHal,
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
        RobotConfig {
            version: String::from("0.4.0"),
            liquids: vec![LiquidConfig {
                id: String::from("water"),
                name: String::from("Water"),
                position: 0,
                calibration: LiquidCalibration { factor: 1.0 },
            }],
            glass_types: vec![
                GlassType {
                    id: String::from("short"),
                    volume_ml: 100.0,
                },
                GlassType {
                    id: String::from("medium"),
                    volume_ml: 150.0,
                },
                GlassType {
                    id: String::from("long"),
                    volume_ml: 200.0,
                },
            ],
            max_total_parts: 10,
            capabilities: Capabilities {
                level_reporting: LevelReporting::Binary,
                glass_typing: false,
                simultaneous_channels: 1,
                max_queue_depth: 5,
            },
            token: String::new(),
            admin_password: String::new(),
        }
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
        job_id: String,
        _name: String,
        _items: Vec<JobItem>,
        _parallel: bool,
    ) -> Result<JobCreated, ErrorInfo> {
        Ok(JobCreated {
            job_id,
            queue_position: 1,
        })
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

/// Development-only password hasher.
///
/// Stores passwords in the format `stub$<plaintext>` and verifies them with
/// a constant-time comparison.  Do NOT use in production.
struct StubPasswordHasher;

impl PasswordHasher for StubPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        Ok(alloc::format!("stub${}", password))
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        let expected = alloc::format!("stub${}", password);
        let a = expected.as_bytes();
        let b = stored_hash.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        let mut diff: u8 = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }
}

// ============================================================================
// Entry point
//
// Runs the embassy spin executor for host/development builds.
// Constructs all stub HAL instances and creates ApiServer + SseServer.
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
//       spawner
//           .spawn(sse_task(
//               SSE_STATUS.init(/* real StatusHal */),
//               SSE_DISPENSE.init(/* real DispenseHal */),
//               net_stack,
//           ))
//           .unwrap();
//       ApiServer { hal: RobotHal { /* real drivers */ } }
//           .run(net_stack)
//           .await;
//   }
//
// NOTE: #[embassy_executor::main] is unavailable with arch-spin.
// The spin executor is initialised manually below.
// ============================================================================

/// Static storage for the StatusHal instance used by the SSE server.
///
/// Kept separate from the ApiServer's HAL so SseServer can hold a `'static`
/// reference without requiring a `Mutex` for its read-only access pattern.
#[cfg(not(test))]
static SSE_STATUS: StaticCell<StubStatusHal> = StaticCell::new();

/// Static storage for the DispenseHal instance used by the SSE server (read
/// path only — job listing for change detection).
#[cfg(not(test))]
static SSE_DISPENSE: StaticCell<StubDispenseHal> = StaticCell::new();

#[cfg(not(test))]
static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// SSE server task — streams robot state and job updates to the display client
/// on port 9000 (single connection at a time).
///
/// In a real bring-up, wire `net_stack` from esp-hal-embassy and call:
///   server::sse::SseServer { status, dispense }.run(net_stack).await;
#[cfg(not(test))]
#[embassy_executor::task]
async fn sse_task(status: &'static StubStatusHal, dispense: &'static StubDispenseHal) {
    // Real call (requires embassy-net stack):
    //   server::sse::SseServer { status, dispense }.run(net_stack).await;
    let _ = (status, dispense);
}

/// Async stub entry task — constructs all HAL stubs and the API server.
///
/// In a real bring-up this function is replaced by the BSP entry point and
/// wired to actual hardware drivers and a live embassy-net stack.
#[cfg(not(test))]
#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    // Initialise static HAL instances for the SSE read path.
    let sse_status = SSE_STATUS.init(StubStatusHal);
    let sse_dispense = SSE_DISPENSE.init(StubDispenseHal);
    spawner.spawn(sse_task(sse_status, sse_dispense)).unwrap();

    let _server = ApiServer {
        hal: RobotHal {
            control: StubControlHal,
            status: StubStatusHal,
            config: StubConfigHal,
            storage: StubStorageHal,
            sensors: StubSensorHal,
            dispense: StubDispenseHal,
            cleaning: StubCleaningHal,
            hasher: StubPasswordHasher,
        },
    };
    // Real call: _server.run(net_stack).await — wire to embassy-net stack.
}

#[cfg(not(test))]
fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}
