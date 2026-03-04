// src/main.rs
//
// Placeholder entry point for CocktailBotHAL.
//
// TODO: Replace this file with a hardware-specific entry point targeting
// your MCU (STM32, ESP32, RP2040, etc.). See TODO.md for the full list of
// open work, including the required Cargo.toml dependencies.

mod hal;
mod server;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use embassy_time::Duration;

use hal::{
    CleaningHal,
    ConfigHal,
    ControlHal,
    DispenseHal,
    ErrorInfo,
    GlassSensorState,
    JobItem,
    JobStatus,
    LevelState,
    RobotConfig,
    RobotState,
    SensorHal,
    StatusHal,
    StorageHal,
};
use server::{ApiServer, RobotHal};

// ============================================================================
// Stub HAL implementations
//
// TODO: Replace each stub with a real hardware driver implementing the
// corresponding trait from src/hal/mod.rs.
// ============================================================================

struct StubControlHal;

impl ControlHal for StubControlHal
{
    fn power(&mut self, _on: bool) -> Result<(), ErrorInfo> { todo!() }
    fn power_save(&mut self, _enabled: bool) -> Result<(), ErrorInfo> { todo!() }
    fn reset_errors(&mut self) -> Result<(), ErrorInfo> { todo!() }
    fn reload_config(&mut self) -> Result<(), ErrorInfo> { todo!() }
}

struct StubStatusHal;

impl StatusHal for StubStatusHal
{
    fn state(&self) -> RobotState { RobotState::Off }
    fn active_errors(&self) -> Vec<ErrorInfo> { vec![] }
}

struct StubConfigHal;

impl ConfigHal for StubConfigHal
{
    fn get_active_config(&self) -> RobotConfig { todo!() }
    fn update_active_config(
        &mut self,
        _cfg: RobotConfig,
    ) -> Result<(), ErrorInfo>
    {
        todo!()
    }
}

struct StubStorageHal;

impl StorageHal for StubStorageHal
{
    fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo> { todo!() }
    fn store_storage_config(
        &mut self,
        _cfg: RobotConfig,
        _overwrite: bool,
    ) -> Result<(), ErrorInfo>
    {
        todo!()
    }
}

struct StubSensorHal;

impl SensorHal for StubSensorHal
{
    fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> { todo!() }
    fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> { todo!() }
}

struct StubDispenseHal;

impl DispenseHal for StubDispenseHal
{
    fn create_job(
        &mut self,
        _client_job_id: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<String, ErrorInfo>
    {
        todo!()
    }
    fn list_jobs(&self) -> Vec<JobStatus> { vec![] }
    fn job_status(&self, _job_id: &str) -> Result<JobStatus, ErrorInfo>
    {
        todo!()
    }
    fn cancel_job(&mut self, _job_id: &str) -> Result<(), ErrorInfo> { todo!() }
}

struct StubCleaningHal;

impl CleaningHal for StubCleaningHal
{
    fn start_cleaning(&mut self) -> Result<(), ErrorInfo> { todo!() }
    fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> { todo!() }
}

// ============================================================================
// Entry point
//
// TODO: Replace with an embassy executor entry point for your target MCU.
//   #[embassy_executor::main]
//   async fn main(spawner: embassy_executor::Spawner) { ... }
// ============================================================================

fn main()
{
    // TODO: Initialise the embassy async runtime, network stack, and real
    // HAL drivers, then pass them to ApiServer::run().
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
    // TODO: call _server.run(net_stack).await inside an async executor
}
