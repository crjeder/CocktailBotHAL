// src/esp32/mod.rs
//
// ESP32 HAL implementation stubs.
//
// This module is compiled only when the `esp32` Cargo feature is enabled.
// Each sub-module contains a stub struct implementing one group of HAL traits.
// All stubs return sensible default/placeholder values. Look for
// `// TODO: wire to hardware` comments to find the points where real
// peripheral code should be added.

mod cleaning;
mod config;
mod control;
mod dispense;
mod hasher;
mod sensors;
mod status;
mod storage;

use alloc::string::String;
use alloc::vec::Vec;

use cleaning::Esp32Cleaning;
use config::Esp32Config;
use control::Esp32Control;
use dispense::Esp32Dispense;
pub use hasher::Esp32PasswordHasher;
use sensors::Esp32Sensors;
use status::Esp32Status;
use storage::Esp32Storage;

use crate::hal::{
    AdminConfig, BackupPayload, CleaningHal, ConfigHal, ControlHal, DispenseHal, ErrorInfo,
    GlassSensorState, JobCreated, JobItem, JobStatus, LevelState, PasswordHasher, RobotConfig,
    RobotState, SensorHal, StatusHal, StorageHal,
};

/// Composite HAL implementation for ESP32.
///
/// Bundles all sub-struct implementations and delegates each trait's
/// methods to the appropriate sub-struct. Construct with
/// [`Esp32Hal::new()`].
pub struct Esp32Hal {
    control: Esp32Control,
    status: Esp32Status,
    config: Esp32Config,
    storage: Esp32Storage,
    sensors: Esp32Sensors,
    dispense: Esp32Dispense,
    cleaning: Esp32Cleaning,
    pub hasher: Esp32PasswordHasher,
}

impl Esp32Hal {
    /// Create a new `Esp32Hal` with all sub-structs initialised to their
    /// default stub state.
    pub fn new() -> Self {
        Esp32Hal {
            control: Esp32Control::new(),
            status: Esp32Status::new(),
            config: Esp32Config::new(),
            storage: Esp32Storage::new(),
            sensors: Esp32Sensors::new(),
            dispense: Esp32Dispense::new(),
            cleaning: Esp32Cleaning::new(),
            hasher: Esp32PasswordHasher,
        }
    }
}

impl PasswordHasher for Esp32Hal {
    fn hash(&self, password: &str) -> Result<alloc::string::String, ErrorInfo> {
        self.hasher.hash(password)
    }

    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        self.hasher.verify(password, stored_hash)
    }
}

// ---------------------------------------------------------------------------
// Trait delegation
// ---------------------------------------------------------------------------

impl ControlHal for Esp32Hal {
    async fn power(&mut self, on: bool) -> Result<(), ErrorInfo> {
        self.control.power(on).await
    }

    async fn power_save(&mut self, enabled: bool) -> Result<(), ErrorInfo> {
        self.control.power_save(enabled).await
    }

    async fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        self.control.reset_errors().await
    }
}

impl StatusHal for Esp32Hal {
    async fn state(&self) -> RobotState {
        self.status.state().await
    }

    async fn active_errors(&self) -> Vec<ErrorInfo> {
        self.status.active_errors().await
    }
}

impl ConfigHal for Esp32Hal {
    async fn get_active_config(&self) -> RobotConfig {
        self.config.get_active_config().await
    }

    async fn update_active_config(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        self.config.update_active_config(cfg).await
    }
}

impl StorageHal for Esp32Hal {
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo> {
        self.storage.backup().await
    }

    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        self.storage.restore(cfg).await
    }
}

impl SensorHal for Esp32Hal {
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        self.sensors.glass_state().await
    }

    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        self.sensors.level_state().await
    }
}

impl DispenseHal for Esp32Hal {
    async fn create_job(
        &mut self,
        job_id: String,
        name: String,
        items: Vec<JobItem>,
        parallel: bool,
    ) -> Result<JobCreated, ErrorInfo> {
        self.dispense
            .create_job(job_id, name, items, parallel)
            .await
    }

    async fn list_jobs(&self) -> Vec<JobStatus> {
        self.dispense.list_jobs().await
    }

    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        self.dispense.job_status(job_id).await
    }

    async fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo> {
        self.dispense.cancel_job(job_id).await
    }
}

impl CleaningHal for Esp32Hal {
    async fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        self.cleaning.start_cleaning().await
    }

    async fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        self.cleaning.stop_cleaning().await
    }
}
