// examples/dev/main.rs
//
// Development entry point for CocktailBotHAL.
//
// Runs the embassy spin executor on the host with stub HAL implementations.
// Demonstrates how to wire ApiServer with all HAL traits.
//
// For ESP32 bring-up, see examples/esp32/main.rs.

extern crate alloc;

mod storage;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use cocktail_bot_hal::hal::{
    AdminConfig, Capabilities, CleaningHal, ConfigHal, ControlHal, DispenseHal, DispenseItem,
    ErrorInfo, GlassSensorState, GlassType, JobCreated, JobStatus, LevelReporting, LevelState,
    LiquidCalibration, LiquidConfig, PasswordHasher, RobotConfig, RobotState, SensorHal, StatusHal,
};
use cocktail_bot_hal::server::{ApiServer, RobotHal};
use embassy_executor::{Executor, Spawner};
use static_cell::StaticCell;
use storage::RamStorageHal;

// ============================================================================
// Stub HAL implementations
//
// Replace each stub with a real hardware driver implementing the corresponding
// trait from cocktail_bot_hal::hal.
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
}

struct StubStatusHal;

impl StatusHal for StubStatusHal {
    async fn state(&self) -> RobotState {
        RobotState::Idle
    }

    async fn active_errors(&self) -> Vec<ErrorInfo> {
        vec![]
    }
}

struct StubConfigHal;

impl ConfigHal for StubConfigHal {
    async fn get_active_config(&self) -> RobotConfig {
        RobotConfig {
            liquids: vec![LiquidConfig {
                id: String::from("water"),
                name: String::from("Water"),
                position: 0,
                calibration: LiquidCalibration { factor: 1.0 },
            }],
            glass_types: vec![
                GlassType {
                    id: String::from("short"),
                    volume: 100.0,
                },
                GlassType {
                    id: String::from("medium"),
                    volume: 150.0,
                },
                GlassType {
                    id: String::from("long"),
                    volume: 200.0,
                },
            ],
            capabilities: Capabilities {
                version: String::from("0.6.0"),
                level_reporting: LevelReporting::Binary,
                glass_typing: false,
                simultaneous_channels: 1,
                max_queue_depth: 5,
                has_cancel_button: false,
                has_power_button: false,
            },
            token: String::new(),
            admin_password: String::new(),
            glass_wait_timeout_secs: 60,
            drink_ready_timeout_secs: 300,
        }
    }

    async fn update_active_config(&mut self, _cfg: AdminConfig) -> Result<(), ErrorInfo> {
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
        _items: Vec<DispenseItem>,
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

    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        Err(ErrorInfo {
            code: String::from("NOT_FOUND"),
            message: format!("Job '{}' not found (stub)", job_id),
            hint: None,
            recoverable: true,
        })
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
/// a constant-time comparison. Do NOT use in production.
struct StubPasswordHasher;

impl PasswordHasher for StubPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        Ok(format!("stub${}", password))
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        let expected = format!("stub${}", password);
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
// TODO (ESP32 bring-up): see examples/esp32/main.rs for the BSP entry point
// using #[esp_hal::main].
// ============================================================================

/// Static storage for the StatusHal instance used by the SSE server.
static SSE_STATUS: StaticCell<StubStatusHal> = StaticCell::new();

/// Static storage for the DispenseHal instance used by the SSE server (read
/// path only — job listing for change detection).
static SSE_DISPENSE: StaticCell<StubDispenseHal> = StaticCell::new();

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// SSE server task — streams robot state and job updates to the display client.
///
/// In a real bring-up, wire `net_stack` from esp-hal-embassy and call:
///   cocktail_bot_hal::server::sse::SseServer { status, dispense }.run(net_stack).await;
#[embassy_executor::task]
async fn sse_task(status: &'static StubStatusHal, dispense: &'static StubDispenseHal) {
    // Real call (requires embassy-net stack):
    //   cocktail_bot_hal::server::sse::SseServer { status, dispense }.run(net_stack).await;
    let _ = (status, dispense);
}

/// Async stub entry task — constructs all HAL stubs and the API server.
#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    let sse_status = SSE_STATUS.init(StubStatusHal);
    let sse_dispense = SSE_DISPENSE.init(StubDispenseHal);
    spawner.spawn(sse_task(sse_status, sse_dispense)).unwrap();

    let _server = ApiServer {
        hal: RobotHal {
            control: StubControlHal,
            status: StubStatusHal,
            config: StubConfigHal,
            storage: RamStorageHal::new(),
            sensors: StubSensorHal,
            dispense: StubDispenseHal,
            cleaning: StubCleaningHal,
            hasher: StubPasswordHasher,
        },
    };
    // Real call: _server.run(net_stack).await — wire to embassy-net stack.
}

fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}
