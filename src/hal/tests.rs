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

#[test]
fn status_state_roundtrip_unit_variants() {
    block_on(async {
        for state in [
            RobotState::Off,
            RobotState::SelfTest,
            RobotState::Idle,
            RobotState::Cleaning,
            RobotState::Provisioning,
        ] {
            let hal = MockStatusHal::new().with_state(state.clone());
            assert_eq!(hal.state().await, state);
        }
    });
}

#[test]
fn status_state_roundtrip_working() {
    block_on(async {
        let state = RobotState::Working {
            job_id: "j1".to_string(),
            progress_pct: 42,
        };
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
        admin.token = "new-token".to_string();
        hal.update_active_config(admin).await.unwrap();
        let cfg = hal.get_active_config().await;
        assert_eq!(cfg.token, "new-token");
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
        admin2.token = "updated-token".to_string();
        hal.restore(admin2).await.unwrap();
        let payload = hal.backup().await.unwrap();
        assert_eq!(payload.data.token, "updated-token");
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
                alloc::vec![DispenseItem {
                    liquid_id: "vodka".to_string(),
                    amount: 60.0,
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
        let items = alloc::vec![DispenseItem {
            liquid_id: "vodka".to_string(),
            amount: 100.0,
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

/// Unit variants serialize as `{"state":"<name>"}` (tagged union).
#[test_case(RobotState::Off, "off" ; "off")]
#[test_case(RobotState::SelfTest, "self_test" ; "self_test")]
#[test_case(RobotState::Idle, "idle" ; "idle")]
#[test_case(RobotState::Cleaning, "cleaning" ; "cleaning")]
#[test_case(RobotState::Provisioning, "provisioning" ; "provisioning")]
fn robot_state_unit_variant_serializes(state: RobotState, tag: &str) {
    let val: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(val["state"], tag);
}

#[test]
fn robot_state_working_serializes() {
    let state = RobotState::Working {
        job_id: "j1".to_string(),
        progress_pct: 75,
    };
    let val: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(val["state"], "working");
    assert_eq!(val["job_id"], "j1");
    assert_eq!(val["progress_pct"], 75);
}

#[test]
fn robot_state_waiting_for_glass_serializes() {
    let state = RobotState::WaitingForGlass {
        job_id: "j2".to_string(),
        reason: GlassWaitReason::NoGlass,
        timeout_remaining_secs: Some(55),
    };
    let val: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(val["state"], "waiting_for_glass");
    assert_eq!(val["job_id"], "j2");
    assert_eq!(val["reason"]["type"], "no_glass");
    assert_eq!(val["timeout_remaining_secs"], 55);
}

#[test]
fn robot_state_drink_ready_serializes() {
    let state = RobotState::DrinkReady {
        job_id: "j3".to_string(),
        timeout_remaining_secs: None,
    };
    let val: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(val["state"], "drink_ready");
    assert_eq!(val["job_id"], "j3");
    assert!(val["timeout_remaining_secs"].is_null());
}

#[test]
fn robot_state_error_serializes() {
    let state = RobotState::Error {
        code: "GLASS_REMOVED".to_string(),
        message: "Glass removed mid-pour".to_string(),
        job_id: Some("j4".to_string()),
        recoverable: true,
        recovery: RecoveryAction::PutGlassBack,
        timeout_remaining_secs: Some(30),
    };
    let val: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert_eq!(val["state"], "error");
    assert_eq!(val["code"], "GLASS_REMOVED");
    assert_eq!(val["recovery"], "put_glass_back");
    assert_eq!(val["recoverable"], true);
    assert_eq!(val["timeout_remaining_secs"], 30);
}

#[test_case(JobState::Queued, "\"queued\"" ; "queued")]
#[test_case(JobState::Active, "\"active\"" ; "active")]
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

// ============================================================================
// GlassWaitReason Serialization Tests (Task 4.2)
// ============================================================================

#[test]
fn glass_wait_reason_no_glass_serializes() {
    let val: serde_json::Value = serde_json::to_value(&GlassWaitReason::NoGlass).unwrap();
    assert_eq!(val["type"], "no_glass");
}

#[test]
fn glass_wait_reason_too_small_serializes() {
    let reason = GlassWaitReason::TooSmall {
        detected_volume: 150.0,
        required_volume: 200.0,
    };
    let val: serde_json::Value = serde_json::to_value(&reason).unwrap();
    assert_eq!(val["type"], "too_small");
    assert!((val["detected_volume"].as_f64().unwrap() - 150.0).abs() < 0.01);
    assert!((val["required_volume"].as_f64().unwrap() - 200.0).abs() < 0.01);
}

// ============================================================================
// RecoveryAction Serialization Tests (Task 4.2)
// ============================================================================

#[test_case(RecoveryAction::PutGlassBack, "\"put_glass_back\"" ; "put_glass_back")]
#[test_case(RecoveryAction::RemoveGlass, "\"remove_glass\"" ; "remove_glass")]
#[test_case(RecoveryAction::CallResetErrors, "\"call_reset_errors\"" ; "call_reset_errors")]
#[test_case(RecoveryAction::None, "\"none\"" ; "none")]
fn recovery_action_serializes(action: RecoveryAction, expected: &str) {
    assert_eq!(serde_json::to_string(&action).unwrap(), expected);
}

// ============================================================================
// AdminConfig Timeout Default Tests (Task 4.3)
// ============================================================================

#[test]
fn admin_config_timeout_defaults_on_missing_fields() {
    // Minimal JSON without timeout fields — defaults should kick in.
    let json = r#"{
        "token": "",
        "liquids": [],
        "glass_types": [],
        "admin_password": ""
    }"#;
    let cfg: AdminConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.glass_wait_timeout_secs, 60);
    assert_eq!(cfg.drink_ready_timeout_secs, 300);
}

#[test]
fn admin_config_timeout_zero_means_indefinite() {
    let json = r#"{
        "token": "",
        "liquids": [],
        "glass_types": [],
        "admin_password": "",
        "glass_wait_timeout_secs": 0,
        "drink_ready_timeout_secs": 0
    }"#;
    let cfg: AdminConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.glass_wait_timeout_secs, 0);
    assert_eq!(cfg.drink_ready_timeout_secs, 0);
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
    assert_eq!(parsed.glass_types.len(), cfg.glass_types.len());
    assert_eq!(parsed.capabilities.version, "0.6.0");
}

#[test]
fn crc32_hex_consistent() {
    let a = crc32_hex(b"hello");
    let b = crc32_hex(b"hello");
    assert_eq!(a, b);
    assert_ne!(a, crc32_hex(b"world"));
    assert_eq!(a.len(), 8);
}
