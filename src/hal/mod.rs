// src/hal/mod.rs

use alloc::string::String;
use alloc::vec::Vec;
use embassy_time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobotState {
    Off,
    Booting,
    SelfTest,
    Idle,
    Prepared,
    Working,
    Cleaning,
    DrinkReady,
    Error,
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub struct LiquidCalibration {
    pub ml_per_sec: f32,
    pub prime_ms: u32,
    pub viscosity_factor: f32,
}

#[derive(Debug, Clone)]
pub struct LiquidConfig {
    pub id: String,
    pub name: String,
    pub position: u8,
    pub calibration: LiquidCalibration,
}

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub level_reporting: LevelReporting,
    pub glass_typing: bool,
    pub simultaneous_channels: u8,
}

#[derive(Debug, Clone)]
pub enum LevelReporting {
    Binary,
    Decimal,
}

#[derive(Debug, Clone)]
pub struct RobotConfig {
    pub version: String,
    pub liquids: Vec<LiquidConfig>,
    pub part_ml: f32,
    pub max_total_parts: u16,
    pub max_channels_per_job: u8,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone)]
pub struct GlassSensorState {
    pub present: bool,
    pub glass_type: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum LevelState {
    Binary { id: String, ok: bool },
    Decimal { id: String, remaining_ml: f32 },
}

#[derive(Debug, Clone)]
pub struct JobItem {
    pub liquid_id: String,
    pub parts: u32,
}

#[derive(Debug, Clone)]
pub enum JobState {
    Queued,
    Running,
    Done,
    Cancelled,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct JobStatus {
    pub job_id: String,
    pub client_job_id: String,
    pub state: JobState,
    pub progress_pct: u8,
}

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// Power / Reset / Reload Config
pub trait ControlHal {
    fn power(&mut self, on: bool) -> Result<(), ErrorInfo>;
    fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo>;
    fn reset_errors(&mut self) -> Result<(), ErrorInfo>;
    fn reload_config(&mut self) -> Result<(), ErrorInfo>;
}

/// Status information
pub trait StatusHal {
    fn state(&self) -> RobotState;
    fn active_errors(&self) -> Vec<ErrorInfo>;
}

/// Active config (RAM)
pub trait ConfigHal {
    fn get_active_config(&self) -> RobotConfig;
    fn update_active_config(&mut self, cfg: RobotConfig) -> Result<(), ErrorInfo>;
}

/// Persistent config (Flash)
pub trait StorageHal {
    fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo>;
    fn store_storage_config(&mut self, cfg: RobotConfig, overwrite: bool)
        -> Result<(), ErrorInfo>;
}

/// Sensor access
pub trait SensorHal {
    fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo>;
    fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo>;
}

/// Create, run, cancel jobs
pub trait DispenseHal {
    fn create_job(
        &mut self,
        client_job_id: String,
        items: Vec<JobItem>,
        require_glass: bool,
        parallel: bool,
        timeout: Duration,
    ) -> Result<String, ErrorInfo>; // returns job_id

    fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo>;
    fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo>;
}

/// Cleaning control
pub trait CleaningHal {
    fn start_cleaning(&mut self) -> Result<(), ErrorInfo>;
    fn stop_cleaning(&mut self) -> Result<(), ErrorInfo>;
}
