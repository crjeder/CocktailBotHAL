// src/esp32/storage.rs
//
// ESP32 stub implementation of StorageHal.
// NVS flash access is not yet implemented.

use alloc::string::String;

use crate::hal::{ErrorInfo, RobotConfig, StorageHal};

/// Stub implementation of [`StorageHal`] for ESP32.
pub struct Esp32Storage;

impl Esp32Storage {
    pub fn new() -> Self {
        Esp32Storage
    }
}

fn not_implemented() -> ErrorInfo {
    ErrorInfo {
        code: String::from("NOT_IMPLEMENTED"),
        message: String::from("StorageHal is not yet implemented for ESP32"),
        hint: Some(String::from(
            "Wire to NVS flash via esp-idf-svc nvs partition",
        )),
        recoverable: false,
    }
}

impl StorageHal for Esp32Storage {
    fn load_storage_config(&self) -> Result<RobotConfig, ErrorInfo> {
        // TODO: wire to hardware — read RobotConfig from NVS flash
        Err(not_implemented())
    }

    fn store_storage_config(
        &mut self,
        _cfg: RobotConfig,
        _overwrite: bool,
    ) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — write RobotConfig to NVS flash
        Err(not_implemented())
    }
}
