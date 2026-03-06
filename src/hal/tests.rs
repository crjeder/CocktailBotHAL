// src/hal/tests.rs
//
// Unit tests for HAL trait implementations using the shared mock objects
// from src/hal/mock.rs.

use futures::executor::block_on;
use test_case::test_case;

use super::mock::*;
use super::*;

// ============================================================================
// ControlHal Tests
// ============================================================================

#[test]
fn control_power_on() {
    block_on(async {
        let mut hal = MockControlHal::new();
        assert!(!hal.powered_on);
        hal.power(true).await.unwrap();
        assert!(hal.powered_on);
    });
}

#[test]
fn control_power_off() {
    block_on(async {
        let mut hal = MockControlHal::new();
        hal.power(true).await.unwrap();
        hal.power(false).await.unwrap();
        assert!(!hal.powered_on);
    });
}

#[test]
fn control_power_save_toggle() {
    block_on(async {
        let mut hal = MockControlHal::new();
        hal.power_save(true).await.unwrap();
        assert!(hal.power_save_on);
        hal.power_save(false).await.unwrap();
        assert!(!hal.power_save_on);
    });
}

#[test]
fn control_reset_errors() {
    block_on(async {
        let mut hal = MockControlHal::new();
        hal.reset_errors().await.unwrap();
        assert_eq!(hal.errors_reset_count, 1);
        hal.reset_errors().await.unwrap();
        assert_eq!(hal.errors_reset_count, 2);
    });
}

#[test]
fn control_power_error_propagation() {
    block_on(async {
        let mut hal = MockControlHal::new();
        hal.fail_next = Some(test_error());
        let err = hal.power(true).await.unwrap_err();
        assert_eq!(err.code, "E001");
        assert!(!hal.powered_on);
    });
}

#[test]
fn control_error_clears_after_one_call() {
    block_on(async {
        let mut hal = MockControlHal::new();
        hal.fail_next = Some(test_error());
        assert!(hal.power(true).await.is_err());
        assert!(hal.power(true).await.is_ok());
        assert!(hal.powered_on);
    });
}

// ============================================================================
// StatusHal Tests
// ============================================================================

#[test]
fn status_default_is_off() {
    block_on(async {
        let hal = MockStatusHal::new();
        assert_eq!(hal.state().await, RobotState::Off);
    });
}

#[test]
fn status_returns_configured_state() {
    block_on(async {
        let hal = MockStatusHal::new().with_state(RobotState::Idle);
        assert_eq!(hal.state().await, RobotState::Idle);
    });
}

#[test]
fn status_no_errors_by_default() {
    block_on(async {
        let hal = MockStatusHal::new();
        assert!(hal.active_errors().await.is_empty());
    });
}

#[test]
fn status_returns_configured_errors() {
    block_on(async {
        let hal = MockStatusHal::new().with_errors(alloc::vec![test_error()]);
        let errors = hal.active_errors().await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E001");
    });
}

#[test_case(RobotState::Off ; "off")]
#[test_case(RobotState::SelfTest ; "self_test")]
#[test_case(RobotState::Idle ; "idle")]
#[test_case(RobotState::Prepared ; "prepared")]
#[test_case(RobotState::Working ; "working")]
#[test_case(RobotState::Cleaning ; "cleaning")]
#[test_case(RobotState::DrinkReady ; "drink_ready")]
#[test_case(RobotState::Error ; "error")]
#[test_case(RobotState::Provisioning ; "provisioning")]
fn status_state_roundtrip(state: RobotState) {
    block_on(async {
        let hal = MockStatusHal::new().with_state(state.clone());
        assert_eq!(hal.state().await, state);
    });
}

// ============================================================================
// ConfigHal Tests
// ============================================================================

#[test]
fn config_get_returns_stored() {
    block_on(async {
        let hal = MockConfigHal::new(test_robot_config());
        let cfg = hal.get_active_config().await;
        assert_eq!(cfg.liquids.len(), 1);
        assert_eq!(cfg.liquids[0].id, "vodka");
    });
}

#[test]
fn config_update_and_retrieve() {
    block_on(async {
        let mut hal = MockConfigHal::new(test_robot_config());
        let mut admin = test_admin_config();
        admin.max_total_parts = 20;
        hal.update_active_config(admin).await.unwrap();
        let cfg = hal.get_active_config().await;
        assert_eq!(cfg.max_total_parts, 20);
    });
}

#[test]
fn config_update_replaces_liquids() {
    block_on(async {
        let mut hal = MockConfigHal::new(test_robot_config());
        let mut admin = test_admin_config();
        admin.liquids.push(LiquidConfig {
            id: "rum".to_string(),
            name: "Rum".to_string(),
            position: 1,
            calibration: LiquidCalibration { factor: 1.1 },
        });
        hal.update_active_config(admin).await.unwrap();
        let cfg = hal.get_active_config().await;
        assert_eq!(cfg.liquids.len(), 2);
        assert_eq!(cfg.liquids[1].id, "rum");
    });
}

#[test]
fn config_update_error() {
    block_on(async {
        let mut hal = MockConfigHal::new(test_robot_config());
        hal.fail_next = Some(test_error());
        let result = hal.update_active_config(test_admin_config()).await;
        assert!(result.is_err());
        hal.done();
    });
}

// ============================================================================
// StorageHal Tests
// ============================================================================

#[test]
fn storage_backup_no_config_returns_error() {
    block_on(async {
        let hal = MockStorageHal::new();
        let err = hal.backup().await.unwrap_err();
        assert_eq!(err.code, "NO_CONFIG");
    });
}

#[test]
fn storage_restore_and_backup() {
    block_on(async {
        let mut hal = MockStorageHal::new();
        hal.restore(test_admin_config()).await.unwrap();
        let payload = hal.backup().await.unwrap();
        assert_eq!(payload.data.liquids[0].id, "vodka");
    });
}

#[test]
fn storage_restore_replaces_previous() {
    block_on(async {
        let mut hal = MockStorageHal::new();
        hal.restore(test_admin_config()).await.unwrap();
        let mut admin2 = test_admin_config();
        admin2.max_total_parts = 99;
        hal.restore(admin2).await.unwrap();
        let payload = hal.backup().await.unwrap();
        assert_eq!(payload.data.max_total_parts, 99);
    });
}

#[test]
fn storage_restore_error() {
    block_on(async {
        let mut hal = MockStorageHal::new();
        hal.restore_fail = Some(test_error());
        let result = hal.restore(test_admin_config()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "E001");
        hal.done();
    });
}

// ============================================================================
// SensorHal Tests
// ============================================================================

#[test]
fn sensor_glass_absent() {
    block_on(async {
        let hal = MockSensorHal::new();
        let state = hal.glass_state().await.unwrap();
        assert!(!state.present);
        assert!(state.glass_type.is_none());
    });
}

#[test]
fn sensor_glass_present() {
    block_on(async {
        let hal = MockSensorHal::new().with_glass(true);
        let state = hal.glass_state().await.unwrap();
        assert!(state.present);
        assert!(state.confidence > 0.9);
    });
}

#[test]
fn sensor_levels_empty() {
    block_on(async {
        let hal = MockSensorHal::new();
        assert!(hal.level_state().await.unwrap().is_empty());
    });
}

#[test]
fn sensor_levels_binary() {
    block_on(async {
        let mut hal = MockSensorHal::new();
        hal.levels = alloc::vec![LevelState::Binary {
            id: "vodka".to_string(),
            ok: true,
        }];
        let levels = hal.level_state().await.unwrap();
        assert_eq!(levels.len(), 1);
        match &levels[0] {
            LevelState::Binary { id, ok } => {
                assert_eq!(id, "vodka");
                assert!(ok);
            }
            _ => panic!("Expected Binary level state"),
        }
    });
}

#[test]
fn sensor_levels_decimal() {
    block_on(async {
        let mut hal = MockSensorHal::new();
        hal.levels = alloc::vec![LevelState::Decimal {
            id: "rum".to_string(),
            remaining_ml: 250.0,
        }];
        let levels = hal.level_state().await.unwrap();
        match &levels[0] {
            LevelState::Decimal { id, remaining_ml } => {
                assert_eq!(id, "rum");
                assert!((remaining_ml - 250.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected Decimal level state"),
        }
    });
}

#[test]
fn sensor_error_propagation() {
    block_on(async {
        let hal = MockSensorHal {
            fail: Some(test_error()),
            ..MockSensorHal::new()
        };
        assert!(hal.glass_state().await.is_err());
        assert!(hal.level_state().await.is_err());
    });
}

// ============================================================================
// DispenseHal Tests
// ============================================================================

#[test]
fn dispense_create_job() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        let created = hal
            .create_job(
                "job-1".to_string(),
                "My Drink".to_string(),
                alloc::vec![JobItem {
                    liquid_id: "vodka".to_string(),
                    parts: 2,
                }],
                false,
            )
            .await
            .unwrap();
        assert_eq!(created.job_id, "job-1");
        assert_eq!(created.queue_position, 1);
    });
}

#[test]
fn dispense_job_ids_increment() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        let items = alloc::vec![JobItem {
            liquid_id: "vodka".to_string(),
            parts: 1,
        }];
        let c1 = hal
            .create_job(
                "id-1".to_string(),
                "Drink A".to_string(),
                items.clone(),
                false,
            )
            .await
            .unwrap();
        let c2 = hal
            .create_job("id-2".to_string(), "Drink B".to_string(), items, false)
            .await
            .unwrap();
        assert_eq!(c1.queue_position, 1);
        assert_eq!(c2.queue_position, 2);
    });
}

#[test]
fn dispense_list_jobs_empty() {
    block_on(async {
        let hal = MockDispenseHal::new();
        assert!(hal.list_jobs().await.is_empty());
    });
}

#[test]
fn dispense_list_jobs_after_create() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        hal.create_job(
            "id-1".to_string(),
            "Drink".to_string(),
            alloc::vec![],
            false,
        )
        .await
        .unwrap();
        assert_eq!(hal.list_jobs().await.len(), 1);
    });
}

#[test]
fn dispense_job_status() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        hal.create_job(
            "id-1".to_string(),
            "MyDrink".to_string(),
            alloc::vec![],
            false,
        )
        .await
        .unwrap();
        let status = hal.job_status("id-1").await.unwrap();
        assert_eq!(status.name, "MyDrink");
        assert_eq!(status.progress_pct, 0);
    });
}

#[test]
fn dispense_unknown_job_returns_error() {
    block_on(async {
        let hal = MockDispenseHal::new();
        let err = hal.job_status("nonexistent").await.unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    });
}

#[test]
fn dispense_cancel_job() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        hal.create_job(
            "id-1".to_string(),
            "Drink".to_string(),
            alloc::vec![],
            false,
        )
        .await
        .unwrap();
        hal.cancel_job("id-1").await.unwrap();
        let status = hal.job_status("id-1").await.unwrap();
        let state_json = serde_json::to_string(&status.state).unwrap();
        assert_eq!(state_json, "\"cancelled\"");
    });
}

#[test]
fn dispense_cancel_unknown_job() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        assert!(hal.cancel_job("nonexistent").await.is_err());
    });
}

#[test]
fn dispense_create_job_error() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        hal.fail_next = Some(test_error());
        let result = hal
            .create_job(
                "id-1".to_string(),
                "Drink".to_string(),
                alloc::vec![],
                false,
            )
            .await;
        assert!(result.is_err());
        assert!(hal.jobs.is_empty());
        hal.done();
    });
}

#[test]
fn dispense_multiple_jobs_listed() {
    block_on(async {
        let mut hal = MockDispenseHal::new();
        for i in 0..3 {
            hal.create_job(
                alloc::format!("id-{}", i),
                alloc::format!("Drink {}", i),
                alloc::vec![],
                false,
            )
            .await
            .unwrap();
        }
        let jobs = hal.list_jobs().await;
        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].name, "Drink 0");
        assert_eq!(jobs[2].name, "Drink 2");
    });
}

// ============================================================================
// CleaningHal Tests
// ============================================================================

#[test]
fn cleaning_start() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        assert!(!hal.cleaning);
        hal.start_cleaning().await.unwrap();
        assert!(hal.cleaning);
    });
}

#[test]
fn cleaning_stop() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        hal.start_cleaning().await.unwrap();
        hal.stop_cleaning().await.unwrap();
        assert!(!hal.cleaning);
    });
}

#[test]
fn cleaning_double_start_fails() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        hal.start_cleaning().await.unwrap();
        let err = hal.start_cleaning().await.unwrap_err();
        assert_eq!(err.code, "ALREADY_CLEANING");
    });
}

#[test]
fn cleaning_stop_when_not_cleaning() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        hal.stop_cleaning().await.unwrap();
        assert!(!hal.cleaning);
    });
}

#[test]
fn cleaning_error_propagation() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        hal.fail_next = Some(test_error());
        assert!(hal.start_cleaning().await.is_err());
        assert!(!hal.cleaning);
        hal.done();
    });
}

#[test]
fn cleaning_restart_after_stop() {
    block_on(async {
        let mut hal = MockCleaningHal::new();
        hal.start_cleaning().await.unwrap();
        hal.stop_cleaning().await.unwrap();
        hal.start_cleaning().await.unwrap();
        assert!(hal.cleaning);
    });
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test_case(RobotState::Off, "\"off\"" ; "off")]
#[test_case(RobotState::SelfTest, "\"self_test\"" ; "self_test")]
#[test_case(RobotState::Idle, "\"idle\"" ; "idle")]
#[test_case(RobotState::Prepared, "\"prepared\"" ; "prepared")]
#[test_case(RobotState::Working, "\"working\"" ; "working")]
#[test_case(RobotState::Cleaning, "\"cleaning\"" ; "cleaning")]
#[test_case(RobotState::DrinkReady, "\"drink_ready\"" ; "drink_ready")]
#[test_case(RobotState::Error, "\"error\"" ; "error")]
#[test_case(RobotState::Provisioning, "\"provisioning\"" ; "provisioning")]
fn robot_state_serializes(state: RobotState, expected: &str) {
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, expected);
    let parsed: RobotState = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, state);
}

#[test_case(JobState::Queued, "\"queued\"" ; "queued")]
#[test_case(JobState::Running, "\"running\"" ; "running")]
#[test_case(JobState::Done, "\"done\"" ; "done")]
#[test_case(JobState::Cancelled, "\"cancelled\"" ; "cancelled")]
#[test_case(JobState::Error("fail".to_string()), "\"error\"" ; "error")]
fn job_state_serializes(state: JobState, expected: &str) {
    assert_eq!(serde_json::to_string(&state).unwrap(), expected);
}

#[test_case(LevelReporting::Binary, "\"binary\"" ; "binary")]
#[test_case(LevelReporting::Decimal, "\"decimal\"" ; "decimal")]
fn level_reporting_serializes(lr: LevelReporting, expected: &str) {
    assert_eq!(serde_json::to_string(&lr).unwrap(), expected);
}

#[test]
fn error_info_serializes() {
    let err = test_error();
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&err).unwrap()).unwrap();
    assert_eq!(value["code"], "E001");
    assert_eq!(value["message"], "Test error");
    assert_eq!(value["recoverable"], true);
    assert_eq!(value["hint"], "Fix it");
}

#[test]
fn error_info_none_hint_is_null() {
    let err = ErrorInfo {
        code: "E002".to_string(),
        message: "No hint".to_string(),
        hint: None,
        recoverable: false,
    };
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&err).unwrap()).unwrap();
    assert!(value["hint"].is_null());
    assert_eq!(value["recoverable"], false);
}

#[test]
fn glass_sensor_state_serializes() {
    let state = GlassSensorState {
        present: true,
        glass_type: Some("highball".to_string()),
        confidence: 0.95,
    };
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(value["present"], true);
    assert_eq!(value["glass_type"], "highball");
}

#[test]
fn level_state_binary_serializes_with_tag() {
    let level = LevelState::Binary {
        id: "vodka".to_string(),
        ok: true,
    };
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&level).unwrap()).unwrap();
    assert_eq!(value["mode"], "binary");
    assert_eq!(value["id"], "vodka");
    assert_eq!(value["ok"], true);
}

#[test]
fn level_state_decimal_serializes_with_tag() {
    let level = LevelState::Decimal {
        id: "rum".to_string(),
        remaining_ml: 500.0,
    };
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&level).unwrap()).unwrap();
    assert_eq!(value["mode"], "decimal");
    assert_eq!(value["remaining_ml"], 500.0);
}

#[test]
fn job_item_json_roundtrip() {
    let item = JobItem {
        liquid_id: "gin".to_string(),
        parts: 3,
    };
    let parsed: JobItem = serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
    assert_eq!(parsed.liquid_id, "gin");
    assert_eq!(parsed.parts, 3);
}

#[test]
fn liquid_calibration_json_roundtrip() {
    let cal = LiquidCalibration { factor: 1.25 };
    let parsed: LiquidCalibration =
        serde_json::from_str(&serde_json::to_string(&cal).unwrap()).unwrap();
    assert!((parsed.factor - 1.25).abs() < f32::EPSILON);
}

#[test]
fn robot_config_json_roundtrip() {
    let cfg = test_robot_config();
    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: RobotConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.liquids.len(), cfg.liquids.len());
    assert_eq!(parsed.max_total_parts, cfg.max_total_parts);
    assert_eq!(parsed.capabilities.version, "0.5.0");
}

#[test]
fn crc32_hex_consistent() {
    let a = crc32_hex(b"hello");
    let b = crc32_hex(b"hello");
    assert_eq!(a, b);
    assert_ne!(a, crc32_hex(b"world"));
    assert_eq!(a.len(), 8);
}
