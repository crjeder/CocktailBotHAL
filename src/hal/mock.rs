// src/hal/mock.rs
//
// Test-only mock implementations for all 7 HAL traits plus a mock
// embedded_io_async::Write socket.
//
// Gate: #[cfg(test)] — never compiled into production binaries.
//
// Usage:
//   use crate::hal::mock::*;
//   let mut ctrl = MockControlHal::new();
//   ctrl.fail_next = Some(make_error("E001", "msg"));
//   block_on(async { ctrl.power(true).await.unwrap_err(); });
//   ctrl.done();  // asserts fail_next was consumed

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::hal::{
    AdminConfig, BackupPayload, CleaningHal, ConfigHal, ControlHal, DispenseHal, DispenseItem,
    ErrorInfo, GlassSensorState, GlassType, JobCreated, JobState, JobStatus, LevelReporting,
    LevelState, LiquidCalibration, LiquidConfig, PasswordHasher, RobotConfig, RobotState,
    SensorHal, StatusHal, StorageHal,
};

// ============================================================================
// MockWrite — a no-fail in-memory socket for handler tests
// ============================================================================

/// An in-memory socket that implements `embedded_io_async::Write + Unpin`.
///
/// Handlers write their HTTP response here; tests inspect the buffer.
pub struct MockWrite(pub Vec<u8>);

impl MockWrite {
    pub fn new() -> Self {
        MockWrite(Vec::new())
    }

    /// Return the buffer contents as a UTF-8 string (panics if non-UTF-8).
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap()
    }
}

impl embedded_io_async::ErrorType for MockWrite {
    type Error = core::convert::Infallible;
}

impl embedded_io_async::Write for MockWrite {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
}

// ============================================================================
// MockControlHal
// ============================================================================

pub struct MockControlHal {
    pub powered_on: bool,
    pub power_save_on: bool,
    pub errors_reset_count: u32,
    pub fail_next: Option<ErrorInfo>,
}

impl MockControlHal {
    pub fn new() -> Self {
        Self {
            powered_on: false,
            power_save_on: false,
            errors_reset_count: 0,
            fail_next: None,
        }
    }

    /// Assert all injected errors were consumed.
    pub fn done(&self) {
        assert!(
            self.fail_next.is_none(),
            "MockControlHal: unconsumed fail_next"
        );
    }
}

impl ControlHal for MockControlHal {
    async fn power(&mut self, on: bool) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.powered_on = on;
        Ok(())
    }

    async fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.power_save_on = enabled;
        Ok(())
    }

    async fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.errors_reset_count += 1;
        Ok(())
    }
}

// ============================================================================
// MockStatusHal
// ============================================================================

pub struct MockStatusHal {
    pub state: RobotState,
    pub errors: Vec<ErrorInfo>,
}

impl MockStatusHal {
    pub fn new() -> Self {
        Self {
            state: RobotState::Off,
            errors: Vec::new(),
        }
    }

    pub fn with_state(mut self, state: RobotState) -> Self {
        self.state = state;
        self
    }

    pub fn with_errors(mut self, errors: Vec<ErrorInfo>) -> Self {
        self.errors = errors;
        self
    }

    pub fn done(&self) {}
}

impl StatusHal for MockStatusHal {
    async fn state(&self) -> RobotState {
        self.state.clone()
    }

    async fn active_errors(&self) -> Vec<ErrorInfo> {
        self.errors.clone()
    }
}

// ============================================================================
// MockConfigHal
// ============================================================================

pub struct MockConfigHal {
    pub config: RobotConfig,
    pub fail_next: Option<ErrorInfo>,
}

impl MockConfigHal {
    pub fn new(config: RobotConfig) -> Self {
        Self {
            config,
            fail_next: None,
        }
    }

    pub fn done(&self) {
        assert!(
            self.fail_next.is_none(),
            "MockConfigHal: unconsumed fail_next"
        );
    }
}

impl ConfigHal for MockConfigHal {
    async fn get_active_config(&self) -> RobotConfig {
        self.config.clone()
    }

    async fn update_active_config(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.config.liquids = cfg.liquids;
        self.config.glass_types = cfg.glass_types;
        self.config.token = cfg.token;
        self.config.admin_password = cfg.admin_password;
        Ok(())
    }
}

// ============================================================================
// MockStorageHal
// ============================================================================

pub struct MockStorageHal {
    pub payload: Option<BackupPayload>,
    pub restore_fail: Option<ErrorInfo>,
}

impl MockStorageHal {
    pub fn new() -> Self {
        Self {
            payload: None,
            restore_fail: None,
        }
    }

    pub fn done(&self) {
        assert!(
            self.restore_fail.is_none(),
            "MockStorageHal: unconsumed restore_fail"
        );
    }
}

impl StorageHal for MockStorageHal {
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo> {
        self.payload.clone().ok_or_else(|| ErrorInfo {
            code: "NO_CONFIG".to_string(),
            message: "No stored configuration".to_string(),
            hint: None,
            recoverable: false,
        })
    }

    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        if let Some(e) = self.restore_fail.take() {
            return Err(e);
        }
        self.payload = Some(BackupPayload {
            data: cfg,
            checksum: "00000000".to_string(),
            backed_up_at: "2026-01-01T00:00:00Z".to_string(),
        });
        Ok(())
    }
}

// ============================================================================
// MockSensorHal
// ============================================================================

pub struct MockSensorHal {
    pub glass: GlassSensorState,
    pub levels: Vec<LevelState>,
    /// Persistent error returned by both glass_state and level_state.
    /// Uses peek semantics (&self) — stays set until cleared manually.
    pub fail: Option<ErrorInfo>,
}

impl MockSensorHal {
    pub fn new() -> Self {
        Self {
            glass: GlassSensorState {
                present: false,
                glass_type: None,
                confidence: 0.0,
            },
            levels: Vec::new(),
            fail: None,
        }
    }

    pub fn with_glass(mut self, present: bool) -> Self {
        self.glass.present = present;
        if present {
            self.glass.confidence = 0.95;
        }
        self
    }

    pub fn done(&self) {
        assert!(self.fail.is_none(), "MockSensorHal: unconsumed fail");
    }
}

impl SensorHal for MockSensorHal {
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        if let Some(ref e) = self.fail {
            return Err(e.clone());
        }
        Ok(self.glass.clone())
    }

    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        if let Some(ref e) = self.fail {
            return Err(e.clone());
        }
        Ok(self.levels.clone())
    }
}

// ============================================================================
// MockDispenseHal
// ============================================================================

pub struct MockDispenseHal {
    pub jobs: Vec<JobStatus>,
    pub next_id: u32,
    pub fail_next: Option<ErrorInfo>,
}

impl MockDispenseHal {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
            fail_next: None,
        }
    }

    pub fn done(&self) {
        assert!(
            self.fail_next.is_none(),
            "MockDispenseHal: unconsumed fail_next"
        );
    }
}

impl DispenseHal for MockDispenseHal {
    async fn create_job(
        &mut self,
        job_id: String,
        name: String,
        _items: Vec<DispenseItem>,
        _parallel: bool,
    ) -> Result<JobCreated, ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        let queue_position = self.jobs.len() as u8 + 1;
        self.jobs.push(JobStatus {
            job_id: job_id.clone(),
            name,
            state: JobState::Queued,
            progress_pct: 0,
        });
        Ok(JobCreated {
            job_id,
            queue_position,
        })
    }

    async fn list_jobs(&self) -> Vec<JobStatus> {
        self.jobs.clone()
    }

    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        self.jobs
            .iter()
            .find(|j| j.job_id == job_id)
            .cloned()
            .ok_or_else(|| ErrorInfo {
                code: "NOT_FOUND".to_string(),
                message: alloc::format!("Job {} not found", job_id),
                hint: None,
                recoverable: false,
            })
    }

    async fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo> {
        match self.jobs.iter_mut().find(|j| j.job_id == job_id) {
            Some(job) => {
                job.state = JobState::Cancelled;
                Ok(())
            }
            None => Err(ErrorInfo {
                code: "NOT_FOUND".to_string(),
                message: alloc::format!("Job {} not found", job_id),
                hint: None,
                recoverable: false,
            }),
        }
    }
}

// ============================================================================
// MockCleaningHal
// ============================================================================

pub struct MockCleaningHal {
    pub cleaning: bool,
    pub fail_next: Option<ErrorInfo>,
}

impl MockCleaningHal {
    pub fn new() -> Self {
        Self {
            cleaning: false,
            fail_next: None,
        }
    }

    pub fn done(&self) {
        assert!(
            self.fail_next.is_none(),
            "MockCleaningHal: unconsumed fail_next"
        );
    }
}

impl CleaningHal for MockCleaningHal {
    async fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        if self.cleaning {
            return Err(ErrorInfo {
                code: "ALREADY_CLEANING".to_string(),
                message: "Cleaning already in progress".to_string(),
                hint: None,
                recoverable: true,
            });
        }
        self.cleaning = true;
        Ok(())
    }

    async fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.cleaning = false;
        Ok(())
    }
}

// ============================================================================
// MockPasswordHasher
// ============================================================================

pub struct MockPasswordHasher;

impl PasswordHasher for MockPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        Ok(alloc::format!("mock${}", password))
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        stored_hash == alloc::format!("mock${}", password)
    }
}

// ============================================================================
// Test data helpers
// ============================================================================

pub fn test_error() -> ErrorInfo {
    ErrorInfo {
        code: "E001".to_string(),
        message: "Test error".to_string(),
        hint: Some("Fix it".to_string()),
        recoverable: true,
    }
}

pub fn test_robot_config() -> RobotConfig {
    use crate::hal::Capabilities;
    RobotConfig {
        liquids: alloc::vec![LiquidConfig {
            id: "vodka".to_string(),
            name: "Vodka".to_string(),
            position: 0,
            calibration: LiquidCalibration { factor: 1.0 },
        }],
        glass_types: alloc::vec![GlassType {
            id: "medium".to_string(),
            volume: 200.0,
        }],
        capabilities: Capabilities {
            version: "0.6.0".to_string(),
            level_reporting: LevelReporting::Binary,
            glass_typing: false,
            simultaneous_channels: 2,
            max_queue_depth: 4,
        },
        token: String::new(),
        admin_password: String::new(),
    }
}

pub fn test_admin_config() -> AdminConfig {
    AdminConfig {
        liquids: alloc::vec![LiquidConfig {
            id: "vodka".to_string(),
            name: "Vodka".to_string(),
            position: 0,
            calibration: LiquidCalibration { factor: 1.0 },
        }],
        glass_types: alloc::vec![GlassType {
            id: "medium".to_string(),
            volume: 200.0,
        }],
        token: String::new(),
        admin_password: String::new(),
    }
}
