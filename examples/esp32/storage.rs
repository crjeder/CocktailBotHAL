// examples/esp32/storage.rs
//
// ESP32 stub implementation of StorageHal.
// NVS flash access is not yet implemented.

use alloc::string::String;

use cocktail_bot_hal::hal::{AdminConfig, BackupPayload, ErrorInfo, StorageHal};

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
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo> {
        // TODO: wire to hardware — read AdminConfig from NVS flash, compute CRC32
        Err(not_implemented())
    }

    async fn restore(&mut self, _cfg: AdminConfig) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — write AdminConfig to NVS flash
        Err(not_implemented())
    }
}
