// src/hal/mod.rs

use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidCalibration {
    pub ml_per_sec: f32,
    pub prime_ms: u32,
    pub viscosity_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidConfig {
    pub id: String,
    pub name: String,
    pub position: u8,
    pub calibration: LiquidCalibration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub level_reporting: LevelReporting,
    pub glass_typing: bool,
    pub simultaneous_channels: u8,
    /// Maximum number of jobs (running + queued) accepted simultaneously.
    pub max_queue_depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelReporting {
    Binary,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotConfig {
    pub version: String,
    pub liquids: Vec<LiquidConfig>,
    pub part_ml: f32,
    pub max_total_parts: u16,
    pub max_channels_per_job: u8,
    pub capabilities: Capabilities,
    /// Bearer token required to authenticate API requests.
    /// An empty string causes the server to fall back to the compile-time default.
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlassSensorState {
    pub present: bool,
    pub glass_type: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum LevelState {
    Binary { id: String, ok: bool },
    Decimal { id: String, remaining_ml: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Serialize for JobState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JobState::Queued => serializer.serialize_str("queued"),
            JobState::Running => serializer.serialize_str("running"),
            JobState::Done => serializer.serialize_str("done"),
            JobState::Cancelled => serializer.serialize_str("cancelled"),
            JobState::Error(_) => serializer.serialize_str("error"),
        }
    }
}

/// Returned by `DispenseHal::create_job` on success.
#[derive(Debug, Clone, Serialize)]
pub struct JobCreated {
    /// Server-generated unique job identifier.
    pub job_id: String,
    /// 1-based position in the queue (1 = next to be dispensed).
    pub queue_position: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobStatus {
    pub job_id: String,
    /// Human-readable label supplied by the client at creation time.
    pub name: String,
    pub state: JobState,
    pub progress_pct: u8,
}

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// Power / Reset / Reload Config
pub trait ControlHal {
    async fn power(&mut self, on: bool) -> Result<(), ErrorInfo>;
    async fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo>;
    async fn reset_errors(&mut self) -> Result<(), ErrorInfo>;
    async fn reload_config(&mut self) -> Result<(), ErrorInfo>;
}

/// Status information
pub trait StatusHal {
    async fn state(&self) -> RobotState;
    async fn active_errors(&self) -> Vec<ErrorInfo>;
}

/// Active config (RAM)
pub trait ConfigHal {
    async fn get_active_config(&self) -> RobotConfig;
    async fn update_active_config(&mut self, cfg: RobotConfig) -> Result<(), ErrorInfo>;
}

/// Persistent config (Flash)
pub trait StorageHal {
    async fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo>;
    async fn store_storage_config(
        &mut self,
        cfg: RobotConfig,
        overwrite: bool,
    ) -> Result<(), ErrorInfo>;
}

/// Sensor access
pub trait SensorHal {
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo>;
    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo>;
}

/// Create, run, cancel jobs
pub trait DispenseHal {
    async fn create_job(
        &mut self,
        job_id: String,
        name: String,
        items: Vec<JobItem>,
        require_glass: bool,
        parallel: bool,
        timeout: Duration,
    ) -> Result<JobCreated, ErrorInfo>;

    async fn list_jobs(&self) -> Vec<JobStatus>;
    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo>;
    async fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo>;
}

/// Cleaning control
pub trait CleaningHal {
    async fn start_cleaning(&mut self) -> Result<(), ErrorInfo>;
    async fn stop_cleaning(&mut self) -> Result<(), ErrorInfo>;
}

#[cfg(test)]
mod tests;
