// src/hal/tests.rs
//
// Unit tests for HAL trait implementations using mock objects.

use super::*;
use core::time::Duration;

// ============================================================================
// Test helpers
// ============================================================================

fn test_config() -> RobotConfig {
    RobotConfig {
        version: String::from("1.0.0"),
        liquids: vec![LiquidConfig {
            id: String::from("vodka"),
            name: String::from("Vodka"),
            position: 0,
            calibration: LiquidCalibration {
                ml_per_sec: 10.0,
                prime_ms: 500,
                viscosity_factor: 1.0,
            },
        }],
        part_ml: 30.0,
        max_total_parts: 10,
        max_channels_per_job: 4,
        capabilities: Capabilities {
            level_reporting: LevelReporting::Binary,
            glass_typing: false,
            simultaneous_channels: 2,
        },
    }
}

fn test_error() -> ErrorInfo {
    ErrorInfo {
        code: String::from("E001"),
        message: String::from("Test error"),
        hint: Some(String::from("Fix it")),
        recoverable: true,
    }
}

// ============================================================================
// Mock: ControlHal
// ============================================================================

struct MockControlHal {
    powered_on: bool,
    power_save_on: bool,
    errors_reset_count: u32,
    config_reloaded_count: u32,
    fail_next: Option<ErrorInfo>,
}

impl MockControlHal {
    fn new() -> Self {
        Self {
            powered_on: false,
            power_save_on: false,
            errors_reset_count: 0,
            config_reloaded_count: 0,
            fail_next: None,
        }
    }
}

impl ControlHal for MockControlHal {
    fn power(&mut self, on: bool) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.powered_on = on;
        Ok(())
    }

    fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.power_save_on = enabled;
        Ok(())
    }

    fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.errors_reset_count += 1;
        Ok(())
    }

    fn reload_config(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.config_reloaded_count += 1;
        Ok(())
    }
}

// ============================================================================
// Mock: StatusHal
// ============================================================================

struct MockStatusHal {
    state: RobotState,
    errors: Vec<ErrorInfo>,
}

impl MockStatusHal {
    fn new() -> Self {
        Self { state: RobotState::Off, errors: vec![] }
    }

    fn with_state(mut self, state: RobotState) -> Self {
        self.state = state;
        self
    }

    fn with_errors(mut self, errors: Vec<ErrorInfo>) -> Self {
        self.errors = errors;
        self
    }
}

impl StatusHal for MockStatusHal {
    fn state(&self) -> RobotState {
        self.state.clone()
    }

    fn active_errors(&self) -> Vec<ErrorInfo> {
        self.errors.clone()
    }
}

// ============================================================================
// Mock: ConfigHal
// ============================================================================

struct MockConfigHal {
    config: RobotConfig,
    fail_next: Option<ErrorInfo>,
}

impl MockConfigHal {
    fn new() -> Self {
        Self { config: test_config(), fail_next: None }
    }
}

impl ConfigHal for MockConfigHal {
    fn get_active_config(&self) -> RobotConfig {
        self.config.clone()
    }

    fn update_active_config(
        &mut self,
        cfg: RobotConfig,
    ) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.config = cfg;
        Ok(())
    }
}

// ============================================================================
// Mock: StorageHal
// ============================================================================

struct MockStorageHal {
    stored: Option<RobotConfig>,
    fail_next: Option<ErrorInfo>,
}

impl MockStorageHal {
    fn new() -> Self {
        Self { stored: None, fail_next: None }
    }
}

impl StorageHal for MockStorageHal {
    fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo> {
        self.stored.clone().ok_or_else(|| ErrorInfo {
            code: String::from("NO_CONFIG"),
            message: String::from("No stored configuration"),
            hint: None,
            recoverable: false,
        })
    }

    fn store_storage_config(
        &mut self,
        cfg: RobotConfig,
        overwrite: bool,
    ) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        if self.stored.is_some() && !overwrite {
            return Err(ErrorInfo {
                code: String::from("EXISTS"),
                message: String::from("Config exists; set overwrite=true"),
                hint: Some(String::from("Use overwrite flag")),
                recoverable: true,
            });
        }
        self.stored = Some(cfg);
        Ok(())
    }
}

// ============================================================================
// Mock: SensorHal
// ============================================================================

struct MockSensorHal {
    glass: GlassSensorState,
    levels: Vec<LevelState>,
    fail_next: Option<ErrorInfo>,
}

impl MockSensorHal {
    fn new() -> Self {
        Self {
            glass: GlassSensorState {
                present: false,
                glass_type: None,
                confidence: 0.0,
            },
            levels: vec![],
            fail_next: None,
        }
    }

    fn with_glass(mut self, present: bool) -> Self {
        self.glass.present = present;
        if present {
            self.glass.confidence = 0.95;
        }
        self
    }
}

impl SensorHal for MockSensorHal {
    fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        if let Some(ref e) = self.fail_next {
            return Err(e.clone());
        }
        Ok(self.glass.clone())
    }

    fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        if let Some(ref e) = self.fail_next {
            return Err(e.clone());
        }
        Ok(self.levels.clone())
    }
}

// ============================================================================
// Mock: DispenseHal
// ============================================================================

struct MockDispenseHal {
    jobs: Vec<JobStatus>,
    next_id: u32,
    fail_next: Option<ErrorInfo>,
}

impl MockDispenseHal {
    fn new() -> Self {
        Self {
            jobs: vec![],
            next_id: 1,
            fail_next: None,
        }
    }
}

impl DispenseHal for MockDispenseHal {
    fn create_job(
        &mut self,
        client_job_id: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<String, ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        let job_id = format!("job-{}", self.next_id);
        self.next_id += 1;
        self.jobs.push(JobStatus {
            job_id: job_id.clone(),
            client_job_id,
            state: JobState::Queued,
            progress_pct: 0,
        });
        Ok(job_id)
    }

    fn list_jobs(&self) -> Vec<JobStatus> {
        self.jobs.clone()
    }

    fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        self.jobs.iter().find(|j| j.job_id == job_id).cloned().ok_or_else(
            || ErrorInfo {
                code: String::from("NOT_FOUND"),
                message: format!("Job {} not found", job_id),
                hint: None,
                recoverable: false,
            },
        )
    }

    fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo> {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.job_id == job_id) {
            job.state = JobState::Cancelled;
            Ok(())
        } else {
            Err(ErrorInfo {
                code: String::from("NOT_FOUND"),
                message: format!("Job {} not found", job_id),
                hint: None,
                recoverable: false,
            })
        }
    }
}

// ============================================================================
// Mock: CleaningHal
// ============================================================================

struct MockCleaningHal {
    cleaning: bool,
    fail_next: Option<ErrorInfo>,
}

impl MockCleaningHal {
    fn new() -> Self {
        Self { cleaning: false, fail_next: None }
    }
}

impl CleaningHal for MockCleaningHal {
    fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        if self.cleaning {
            return Err(ErrorInfo {
                code: String::from("ALREADY_CLEANING"),
                message: String::from("Cleaning already in progress"),
                hint: None,
                recoverable: true,
            });
        }
        self.cleaning = true;
        Ok(())
    }

    fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        self.cleaning = false;
        Ok(())
    }
}

// ============================================================================
// ControlHal Tests
// ============================================================================

#[test]
fn control_power_on() {
    let mut hal = MockControlHal::new();
    assert!(!hal.powered_on);
    hal.power(true).unwrap();
    assert!(hal.powered_on);
}

#[test]
fn control_power_off() {
    let mut hal = MockControlHal::new();
    hal.power(true).unwrap();
    hal.power(false).unwrap();
    assert!(!hal.powered_on);
}

#[test]
fn control_power_save_toggle() {
    let mut hal = MockControlHal::new();
    hal.power_save(true).unwrap();
    assert!(hal.power_save_on);
    hal.power_save(false).unwrap();
    assert!(!hal.power_save_on);
}

#[test]
fn control_reset_errors() {
    let mut hal = MockControlHal::new();
    hal.reset_errors().unwrap();
    assert_eq!(hal.errors_reset_count, 1);
    hal.reset_errors().unwrap();
    assert_eq!(hal.errors_reset_count, 2);
}

#[test]
fn control_reload_config() {
    let mut hal = MockControlHal::new();
    hal.reload_config().unwrap();
    assert_eq!(hal.config_reloaded_count, 1);
}

#[test]
fn control_power_error_propagation() {
    let mut hal = MockControlHal::new();
    hal.fail_next = Some(test_error());
    let err = hal.power(true).unwrap_err();
    assert_eq!(err.code, "E001");
    assert!(!hal.powered_on);
}

#[test]
fn control_error_clears_after_one_call() {
    let mut hal = MockControlHal::new();
    hal.fail_next = Some(test_error());
    assert!(hal.power(true).is_err());
    assert!(hal.power(true).is_ok());
    assert!(hal.powered_on);
}

// ============================================================================
// StatusHal Tests
// ============================================================================

#[test]
fn status_default_is_off() {
    let hal = MockStatusHal::new();
    assert_eq!(hal.state(), RobotState::Off);
}

#[test]
fn status_returns_configured_state() {
    let hal = MockStatusHal::new().with_state(RobotState::Idle);
    assert_eq!(hal.state(), RobotState::Idle);
}

#[test]
fn status_no_errors_by_default() {
    let hal = MockStatusHal::new();
    assert!(hal.active_errors().is_empty());
}

#[test]
fn status_returns_configured_errors() {
    let hal = MockStatusHal::new().with_errors(vec![test_error()]);
    let errors = hal.active_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "E001");
}

#[test]
fn status_all_robot_states() {
    let states = vec![
        RobotState::Off,
        RobotState::Booting,
        RobotState::SelfTest,
        RobotState::Idle,
        RobotState::Prepared,
        RobotState::Working,
        RobotState::Cleaning,
        RobotState::DrinkReady,
        RobotState::Error,
    ];
    for state in states {
        let hal = MockStatusHal::new().with_state(state.clone());
        assert_eq!(hal.state(), state);
    }
}

// ============================================================================
// ConfigHal Tests
// ============================================================================

#[test]
fn config_get_returns_default() {
    let hal = MockConfigHal::new();
    let cfg = hal.get_active_config();
    assert_eq!(cfg.version, "1.0.0");
    assert_eq!(cfg.liquids.len(), 1);
}

#[test]
fn config_update_and_retrieve() {
    let mut hal = MockConfigHal::new();
    let mut cfg = test_config();
    cfg.version = String::from("2.0.0");
    cfg.part_ml = 45.0;
    hal.update_active_config(cfg).unwrap();

    let retrieved = hal.get_active_config();
    assert_eq!(retrieved.version, "2.0.0");
    assert_eq!(retrieved.part_ml, 45.0);
}

#[test]
fn config_update_replaces_liquids() {
    let mut hal = MockConfigHal::new();
    let mut cfg = test_config();
    cfg.liquids.push(LiquidConfig {
        id: String::from("rum"),
        name: String::from("Rum"),
        position: 1,
        calibration: LiquidCalibration {
            ml_per_sec: 8.0,
            prime_ms: 600,
            viscosity_factor: 1.1,
        },
    });
    hal.update_active_config(cfg).unwrap();

    let retrieved = hal.get_active_config();
    assert_eq!(retrieved.liquids.len(), 2);
    assert_eq!(retrieved.liquids[1].id, "rum");
}

#[test]
fn config_update_error() {
    let mut hal = MockConfigHal::new();
    hal.fail_next = Some(test_error());
    let result = hal.update_active_config(test_config());
    assert!(result.is_err());
}

// ============================================================================
// StorageHal Tests
// ============================================================================

#[test]
fn storage_load_empty_returns_error() {
    let hal = MockStorageHal::new();
    let result = hal.load_storage_config();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "NO_CONFIG");
}

#[test]
fn storage_store_and_load() {
    let mut hal = MockStorageHal::new();
    let cfg = test_config();
    hal.store_storage_config(cfg.clone(), false).unwrap();

    let loaded = hal.load_storage_config().unwrap();
    assert_eq!(loaded.version, cfg.version);
    assert_eq!(loaded.liquids.len(), cfg.liquids.len());
}

#[test]
fn storage_no_overwrite_fails_when_exists() {
    let mut hal = MockStorageHal::new();
    hal.store_storage_config(test_config(), false).unwrap();

    let result = hal.store_storage_config(test_config(), false);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "EXISTS");
}

#[test]
fn storage_overwrite_succeeds() {
    let mut hal = MockStorageHal::new();
    hal.store_storage_config(test_config(), false).unwrap();

    let mut cfg2 = test_config();
    cfg2.version = String::from("2.0.0");
    hal.store_storage_config(cfg2, true).unwrap();

    let loaded = hal.load_storage_config().unwrap();
    assert_eq!(loaded.version, "2.0.0");
}

#[test]
fn storage_store_error_propagation() {
    let mut hal = MockStorageHal::new();
    hal.fail_next = Some(test_error());
    let result = hal.store_storage_config(test_config(), false);
    assert!(result.is_err());
    assert!(hal.stored.is_none());
}

// ============================================================================
// SensorHal Tests
// ============================================================================

#[test]
fn sensor_glass_absent() {
    let hal = MockSensorHal::new();
    let state = hal.glass_state().unwrap();
    assert!(!state.present);
    assert!(state.glass_type.is_none());
}

#[test]
fn sensor_glass_present() {
    let hal = MockSensorHal::new().with_glass(true);
    let state = hal.glass_state().unwrap();
    assert!(state.present);
    assert!(state.confidence > 0.9);
}

#[test]
fn sensor_levels_empty() {
    let hal = MockSensorHal::new();
    let levels = hal.level_state().unwrap();
    assert!(levels.is_empty());
}

#[test]
fn sensor_levels_binary() {
    let mut hal = MockSensorHal::new();
    hal.levels =
        vec![LevelState::Binary { id: String::from("vodka"), ok: true }];
    let levels = hal.level_state().unwrap();
    assert_eq!(levels.len(), 1);
    match &levels[0] {
        LevelState::Binary { id, ok } => {
            assert_eq!(id, "vodka");
            assert!(ok);
        }
        _ => panic!("Expected Binary level state"),
    }
}

#[test]
fn sensor_levels_decimal() {
    let mut hal = MockSensorHal::new();
    hal.levels = vec![LevelState::Decimal {
        id: String::from("rum"),
        remaining_ml: 250.0,
    }];
    let levels = hal.level_state().unwrap();
    match &levels[0] {
        LevelState::Decimal { id, remaining_ml } => {
            assert_eq!(id, "rum");
            assert!((remaining_ml - 250.0).abs() < f32::EPSILON);
        }
        _ => panic!("Expected Decimal level state"),
    }
}

#[test]
fn sensor_error_propagation() {
    let hal = MockSensorHal {
        fail_next: Some(test_error()),
        ..MockSensorHal::new()
    };
    assert!(hal.glass_state().is_err());
    assert!(hal.level_state().is_err());
}

// ============================================================================
// DispenseHal Tests
// ============================================================================

#[test]
fn dispense_create_job() {
    let mut hal = MockDispenseHal::new();
    let job_id = hal
        .create_job(
            String::from("client-1"),
            vec![JobItem {
                liquid_id: String::from("vodka"),
                parts: 2,
            }],
            true,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    assert_eq!(job_id, "job-1");
}

#[test]
fn dispense_job_ids_increment() {
    let mut hal = MockDispenseHal::new();
    let items = vec![JobItem {
        liquid_id: String::from("vodka"),
        parts: 1,
    }];
    let id1 = hal
        .create_job(
            String::from("c1"),
            items.clone(),
            false,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    let id2 = hal
        .create_job(
            String::from("c2"),
            items,
            false,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    assert_eq!(id1, "job-1");
    assert_eq!(id2, "job-2");
}

#[test]
fn dispense_list_jobs_empty() {
    let hal = MockDispenseHal::new();
    assert!(hal.list_jobs().is_empty());
}

#[test]
fn dispense_list_jobs_after_create() {
    let mut hal = MockDispenseHal::new();
    hal.create_job(
        String::from("c1"),
        vec![],
        false,
        false,
        Duration::from_secs(30),
    )
    .unwrap();
    assert_eq!(hal.list_jobs().len(), 1);
}

#[test]
fn dispense_job_status() {
    let mut hal = MockDispenseHal::new();
    let job_id = hal
        .create_job(
            String::from("c1"),
            vec![],
            false,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    let status = hal.job_status(&job_id).unwrap();
    assert_eq!(status.client_job_id, "c1");
    assert_eq!(status.progress_pct, 0);
}

#[test]
fn dispense_unknown_job_returns_error() {
    let hal = MockDispenseHal::new();
    let result = hal.job_status("nonexistent");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[test]
fn dispense_cancel_job() {
    let mut hal = MockDispenseHal::new();
    let job_id = hal
        .create_job(
            String::from("c1"),
            vec![],
            false,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    hal.cancel_job(&job_id).unwrap();

    let status = hal.job_status(&job_id).unwrap();
    let state_json = serde_json::to_string(&status.state).unwrap();
    assert_eq!(state_json, "\"cancelled\"");
}

#[test]
fn dispense_cancel_unknown_job() {
    let mut hal = MockDispenseHal::new();
    let result = hal.cancel_job("nonexistent");
    assert!(result.is_err());
}

#[test]
fn dispense_create_job_error() {
    let mut hal = MockDispenseHal::new();
    hal.fail_next = Some(test_error());
    let result = hal.create_job(
        String::from("c1"),
        vec![],
        false,
        false,
        Duration::from_secs(30),
    );
    assert!(result.is_err());
    assert!(hal.jobs.is_empty());
}

#[test]
fn dispense_multiple_jobs_listed() {
    let mut hal = MockDispenseHal::new();
    for i in 0..3 {
        hal.create_job(
            format!("client-{}", i),
            vec![],
            false,
            false,
            Duration::from_secs(30),
        )
        .unwrap();
    }
    let jobs = hal.list_jobs();
    assert_eq!(jobs.len(), 3);
    assert_eq!(jobs[0].client_job_id, "client-0");
    assert_eq!(jobs[2].client_job_id, "client-2");
}

// ============================================================================
// CleaningHal Tests
// ============================================================================

#[test]
fn cleaning_start() {
    let mut hal = MockCleaningHal::new();
    assert!(!hal.cleaning);
    hal.start_cleaning().unwrap();
    assert!(hal.cleaning);
}

#[test]
fn cleaning_stop() {
    let mut hal = MockCleaningHal::new();
    hal.start_cleaning().unwrap();
    hal.stop_cleaning().unwrap();
    assert!(!hal.cleaning);
}

#[test]
fn cleaning_double_start_fails() {
    let mut hal = MockCleaningHal::new();
    hal.start_cleaning().unwrap();
    let result = hal.start_cleaning();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "ALREADY_CLEANING");
}

#[test]
fn cleaning_stop_when_not_cleaning() {
    let mut hal = MockCleaningHal::new();
    hal.stop_cleaning().unwrap();
    assert!(!hal.cleaning);
}

#[test]
fn cleaning_error_propagation() {
    let mut hal = MockCleaningHal::new();
    hal.fail_next = Some(test_error());
    let result = hal.start_cleaning();
    assert!(result.is_err());
    assert!(!hal.cleaning);
}

#[test]
fn cleaning_restart_after_stop() {
    let mut hal = MockCleaningHal::new();
    hal.start_cleaning().unwrap();
    hal.stop_cleaning().unwrap();
    hal.start_cleaning().unwrap();
    assert!(hal.cleaning);
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn robot_state_serializes_to_snake_case() {
    let state = RobotState::DrinkReady;
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, "\"drink_ready\"");
}

#[test]
fn robot_state_deserializes_from_snake_case() {
    let state: RobotState = serde_json::from_str("\"self_test\"").unwrap();
    assert_eq!(state, RobotState::SelfTest);
}

#[test]
fn robot_state_all_variants_roundtrip() {
    let variants = vec![
        (RobotState::Off, "\"off\""),
        (RobotState::Booting, "\"booting\""),
        (RobotState::SelfTest, "\"self_test\""),
        (RobotState::Idle, "\"idle\""),
        (RobotState::Prepared, "\"prepared\""),
        (RobotState::Working, "\"working\""),
        (RobotState::Cleaning, "\"cleaning\""),
        (RobotState::DrinkReady, "\"drink_ready\""),
        (RobotState::Error, "\"error\""),
    ];
    for (state, expected_json) in variants {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, expected_json);
        let parsed: RobotState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, state);
    }
}

#[test]
fn robot_config_json_roundtrip() {
    let cfg = test_config();
    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: RobotConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, cfg.version);
    assert_eq!(parsed.liquids.len(), cfg.liquids.len());
    assert_eq!(parsed.part_ml, cfg.part_ml);
    assert_eq!(parsed.max_total_parts, cfg.max_total_parts);
    assert_eq!(parsed.max_channels_per_job, cfg.max_channels_per_job);
}

#[test]
fn job_state_serializes_correctly() {
    assert_eq!(serde_json::to_string(&JobState::Queued).unwrap(), "\"queued\"");
    assert_eq!(
        serde_json::to_string(&JobState::Running).unwrap(),
        "\"running\""
    );
    assert_eq!(serde_json::to_string(&JobState::Done).unwrap(), "\"done\"");
    assert_eq!(
        serde_json::to_string(&JobState::Cancelled).unwrap(),
        "\"cancelled\""
    );
    assert_eq!(
        serde_json::to_string(&JobState::Error(String::from("fail"))).unwrap(),
        "\"error\""
    );
}

#[test]
fn error_info_serializes() {
    let err = test_error();
    let json = serde_json::to_string(&err).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["code"], "E001");
    assert_eq!(value["message"], "Test error");
    assert_eq!(value["recoverable"], true);
    assert_eq!(value["hint"], "Fix it");
}

#[test]
fn error_info_none_hint_serializes_as_null() {
    let err = ErrorInfo {
        code: String::from("E002"),
        message: String::from("No hint"),
        hint: None,
        recoverable: false,
    };
    let json = serde_json::to_string(&err).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value["hint"].is_null());
    assert_eq!(value["recoverable"], false);
}

#[test]
fn glass_sensor_state_serializes() {
    let state = GlassSensorState {
        present: true,
        glass_type: Some(String::from("highball")),
        confidence: 0.95,
    };
    let json = serde_json::to_string(&state).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["present"], true);
    assert_eq!(value["glass_type"], "highball");
}

#[test]
fn level_state_binary_serializes_with_tag() {
    let level = LevelState::Binary { id: String::from("vodka"), ok: true };
    let json = serde_json::to_string(&level).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["mode"], "binary");
    assert_eq!(value["id"], "vodka");
    assert_eq!(value["ok"], true);
}

#[test]
fn level_state_decimal_serializes_with_tag() {
    let level = LevelState::Decimal {
        id: String::from("rum"),
        remaining_ml: 500.0,
    };
    let json = serde_json::to_string(&level).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["mode"], "decimal");
    assert_eq!(value["remaining_ml"], 500.0);
}

#[test]
fn job_item_json_roundtrip() {
    let item = JobItem { liquid_id: String::from("gin"), parts: 3 };
    let json = serde_json::to_string(&item).unwrap();
    let parsed: JobItem = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.liquid_id, "gin");
    assert_eq!(parsed.parts, 3);
}

#[test]
fn liquid_calibration_json_roundtrip() {
    let cal = LiquidCalibration {
        ml_per_sec: 12.5,
        prime_ms: 750,
        viscosity_factor: 1.2,
    };
    let json = serde_json::to_string(&cal).unwrap();
    let parsed: LiquidCalibration = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.ml_per_sec, 12.5);
    assert_eq!(parsed.prime_ms, 750);
}

#[test]
fn capabilities_json_roundtrip() {
    let caps = Capabilities {
        level_reporting: LevelReporting::Decimal,
        glass_typing: true,
        simultaneous_channels: 4,
    };
    let json = serde_json::to_string(&caps).unwrap();
    let parsed: Capabilities = serde_json::from_str(&json).unwrap();
    assert!(parsed.glass_typing);
    assert_eq!(parsed.simultaneous_channels, 4);
}

#[test]
fn level_reporting_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&LevelReporting::Binary).unwrap(),
        "\"binary\""
    );
    assert_eq!(
        serde_json::to_string(&LevelReporting::Decimal).unwrap(),
        "\"decimal\""
    );
}
