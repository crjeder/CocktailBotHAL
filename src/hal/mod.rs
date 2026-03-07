// src/hal/mod.rs

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Reason the robot is waiting for a glass in [`RobotState::WaitingForGlass`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GlassWaitReason {
    /// No glass detected at the dispense position.
    NoGlass,
    /// A glass was detected but its capacity is smaller than the required
    /// dispense volume. Only produced when `Capabilities::glass_typing` is `true`.
    TooSmall {
        detected_volume: f32,
        required_volume: f32,
    },
}

/// Describes how a client (or physical user) can recover from
/// [`RobotState::Error`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    /// Glass was removed mid-pour; replace it to resume dispensing.
    /// Timeout applies (`glass_wait_timeout_secs`).
    PutGlassBack,
    /// Drink was not collected; remove the glass to return to `Idle`.
    /// No timeout — waits indefinitely.
    RemoveGlass,
    /// Firmware or HAL error; call `ControlHal::reset_errors()` to recover.
    CallResetErrors,
    /// Unrecoverable error; requires manual intervention.
    None,
}

/// Current state of the robot, as returned by [`StatusHal::state`].
///
/// Active variants carry job context and timeout countdowns so that a single
/// call to `state()` provides complete information without additional queries.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RobotState {
    Off,
    SelfTest,
    /// Robot has no stored configuration (first boot or factory reset).
    /// Only admin endpoints are active in this state.
    Provisioning,
    Idle,
    Cleaning,
    /// Robot is waiting for a glass to be placed before dispensing begins.
    WaitingForGlass {
        job_id: String,
        reason: GlassWaitReason,
        /// Seconds until the job is cancelled. `None` when timeout is disabled.
        timeout_remaining_secs: Option<u32>,
    },
    /// Robot is actively dispensing.
    Working {
        job_id: String,
        progress_pct: u8,
    },
    /// Dispensing is complete; waiting for the glass to be removed.
    DrinkReady {
        job_id: String,
        /// Seconds until a `DRINK_ABANDONED` error. `None` when timeout is disabled.
        timeout_remaining_secs: Option<u32>,
    },
    /// An error has occurred.
    Error {
        code: String,
        message: String,
        job_id: Option<String>,
        recoverable: bool,
        recovery: RecoveryAction,
        /// Seconds until job is cancelled (`PutGlassBack` only). `None` for other recoveries.
        timeout_remaining_secs: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub recoverable: bool,
}

/// Hardware-agnostic calibration for a single liquid.
///
/// The `factor` is a dimensionless multiplier the HAL implementation uses to
/// compensate for liquid-specific properties (density, viscosity, flow rate).
/// Its exact interpretation depends on the hardware: for weight-based
/// dispensing it is the density in g/ml; for time-based dispensing it is a
/// seconds-per-ml multiplier; etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidCalibration {
    pub factor: f32,
}

/// Maps a named cocktail size to a target glass volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlassType {
    /// Size name, e.g. `"short"`, `"medium"`, `"long"`.
    pub id: String,
    /// Target fill volume in the operator's chosen unit (ml, cl, oz, etc.).
    /// Must be consistent with all other `volume` values and all
    /// `LiquidCalibration.factor` values within a single robot.
    pub volume: f32,
}

/// Per-ingredient dispenser volume passed to [`DispenseHal::create_job`].
///
/// `amount` is pre-computed by the server layer from the recipe ratio and the
/// requested glass volume: `amount_i = (r_i / Σr) × glass.volume`.  The HAL
/// implementation applies `LiquidCalibration.factor` internally to convert
/// this abstract volume into hardware-specific commands (pump duration,
/// weight target, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispenseItem {
    /// Identifier of the liquid to dispense (matches `LiquidConfig.id`).
    pub liquid_id: String,
    /// Pre-computed volume in the operator's abstract volume unit.
    pub amount: f32,
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
    /// Firmware / hardware version string (e.g. `"0.5.0"`).
    pub version: String,
    pub level_reporting: LevelReporting,
    pub glass_typing: bool,
    pub simultaneous_channels: u8,
    /// Maximum number of jobs (running + queued) accepted simultaneously.
    pub max_queue_depth: u8,
    /// `true` if the hardware has a physical button that cancels the active job
    /// when pressed. Clients should hide the software cancel button when `true`.
    #[serde(default)]
    pub has_cancel_button: bool,
    /// `true` if the hardware has a physical button that powers the robot off
    /// when held. Separate capability from `has_cancel_button` even if on the
    /// same physical button (long vs. short press).
    #[serde(default)]
    pub has_power_button: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelReporting {
    Binary,
    Decimal,
}

fn default_glass_wait_timeout() -> u32 {
    60
}
fn default_drink_ready_timeout() -> u32 {
    300
}

/// Admin-owned configuration: persisted to flash, mutated via API.
///
/// Does not include hardware-fixed properties (`Capabilities`).  The server
/// merges `AdminConfig` with `Capabilities` when serving `GET /config`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub liquids: Vec<LiquidConfig>,
    /// Available glass sizes. The server resolves the target volume for a
    /// dispensing job by matching the requested size against this list.
    pub glass_types: Vec<GlassType>,
    /// Bearer token required to authenticate API requests.
    /// An empty string causes the server to fall back to the compile-time default.
    #[serde(default)]
    pub token: String,
    /// Hashed admin password protecting admin-only endpoints (HTTP Basic Auth).
    /// Stored as a hash string produced by the active `PasswordHasher`; never
    /// stored as plaintext.  An empty string causes the server to fall back to
    /// the compile-time default admin password.
    #[serde(default)]
    pub admin_password: String,
    /// Seconds to wait for a glass to be placed (or replaced after mid-pour
    /// removal) before cancelling the job.  `0` means wait indefinitely.
    #[serde(default = "default_glass_wait_timeout")]
    pub glass_wait_timeout_secs: u32,
    /// Seconds to wait for the drink to be collected from `DrinkReady` before
    /// transitioning to `Error { DRINK_ABANDONED }`.  `0` means wait indefinitely.
    #[serde(default = "default_drink_ready_timeout")]
    pub drink_ready_timeout_secs: u32,
}

/// Merged API-facing configuration returned by `GET /config`.
///
/// Combines admin-owned fields (`AdminConfig`) with hardware-fixed properties
/// (`Capabilities`).  Constructed by the HAL implementation; never stored
/// directly in flash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotConfig {
    pub liquids: Vec<LiquidConfig>,
    /// Available glass sizes.
    pub glass_types: Vec<GlassType>,
    /// Hardware-fixed properties for this firmware build.
    pub capabilities: Capabilities,
    /// Bearer token required to authenticate API requests.
    #[serde(default)]
    pub token: String,
    /// Hashed admin password (write-only; always `""` in GET responses).
    #[serde(default)]
    pub admin_password: String,
    /// Seconds to wait for glass placement before cancelling. `0` = indefinite.
    #[serde(default = "default_glass_wait_timeout")]
    pub glass_wait_timeout_secs: u32,
    /// Seconds to wait for drink collection before `DRINK_ABANDONED`. `0` = indefinite.
    #[serde(default = "default_drink_ready_timeout")]
    pub drink_ready_timeout_secs: u32,
}

/// Payload returned by `GET /config/backup` and accepted by
/// `POST /config/restore`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    /// The admin-owned configuration data.
    pub data: AdminConfig,
    /// CRC32/ISO-HDLC hex checksum of the JSON-serialised `data` field.
    pub checksum: String,
    /// ISO 8601 UTC timestamp of when the backup was created.
    pub backed_up_at: String,
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
    /// Job is currently active (dispensing or waiting for glass).
    /// Detail lives in `RobotState`; query `StatusHal::state()` for context.
    Active,
    Done,
    Cancelled,
    Error(String),
}

impl Serialize for JobState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JobState::Queued => serializer.serialize_str("queued"),
            JobState::Active => serializer.serialize_str("active"),
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

/// Power / Reset control
pub trait ControlHal {
    async fn power(&mut self, on: bool) -> Result<(), ErrorInfo>;
    async fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo>;
    async fn reset_errors(&mut self) -> Result<(), ErrorInfo>;
}

/// Status information
pub trait StatusHal {
    async fn state(&self) -> RobotState;
    async fn active_errors(&self) -> Vec<ErrorInfo>;
}

/// Active config (RAM)
pub trait ConfigHal {
    /// Return the merged view: admin-owned fields + hardware capabilities.
    async fn get_active_config(&self) -> RobotConfig;
    /// Apply admin-owned configuration to the in-RAM store.
    async fn update_active_config(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>;
}

/// Persistent config (Flash)
pub trait StorageHal {
    /// Read admin config from flash and return it with a CRC32 checksum and
    /// timestamp.  Returns `Err` when flash is empty or corrupt.
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo>;
    /// Write admin config to flash, overwriting any existing data.
    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>;
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
        items: Vec<DispenseItem>,
        parallel: bool,
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

/// Hashes and verifies admin passwords.
///
/// Implementations are platform-specific: a development stub uses a simple
/// constant-time string comparison, while the ESP32 implementation uses
/// PBKDF2-HMAC-SHA256.  The trait is synchronous — callers must not hold
/// async locks across a `hash` or `verify` call.
pub trait PasswordHasher {
    /// Hash `password` and return a storable hash string.
    fn hash(&self, password: &str) -> Result<String, ErrorInfo>;

    /// Return `true` iff `password` matches `stored_hash`.
    fn verify(&self, password: &str, stored_hash: &str) -> bool;
}

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

// ============================================================================
// Utilities
// ============================================================================

/// Compute a CRC32/ISO-HDLC checksum over `data` and return it as an 8-char
/// lowercase hex string (e.g. `"a1b2c3d4"`).
///
/// Used by [`StorageHal`] implementations to produce and verify the
/// [`BackupPayload::checksum`] field.
pub fn crc32_hex(data: &[u8]) -> String {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    alloc::format!("{:08x}", !crc)
}
