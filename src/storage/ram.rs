// src/storage/ram.rs
//
// RAM-backed StorageHal implementation for development and testing.
//
// This implementation stores AdminConfig in heap memory only. Config is lost on
// reboot. It is NOT suitable for production — use an NVS or flash-backed
// implementation on real hardware.

use alloc::string::String;
use alloc::vec;

use crate::hal::{crc32_hex, AdminConfig, BackupPayload, ErrorInfo, GlassType, StorageHal};

/// RAM-backed implementation of [`StorageHal`].
///
/// # WARNING
/// Config stored here is lost on reboot. For development and testing only.
/// Do NOT use in production. Replace with a flash-backed implementation
/// (e.g. ESP32 NVS) for any deployment that must survive power cycles.
pub struct RamStorageHal {
    stored: Option<AdminConfig>,
}

impl RamStorageHal {
    /// Create a pre-seeded instance with development defaults.
    ///
    /// The store is initialised with an `AdminConfig` containing:
    /// - `token`: `"dev"` — change before deploying
    /// - `liquids`: empty (add via `PATCH /config`)
    /// - `glass_types`: `short` (100) and `long` (200) in abstract volume units
    /// - `admin_password`: `""` (server falls back to compile-time default)
    ///
    /// Because the store is non-empty, the robot boots into `Idle` state
    /// without requiring a provisioning step.
    pub fn new() -> Self {
        let defaults = AdminConfig {
            token: String::from("dev"),
            liquids: vec![],
            glass_types: vec![
                GlassType {
                    id: String::from("short"),
                    volume: 100.0,
                },
                GlassType {
                    id: String::from("long"),
                    volume: 200.0,
                },
            ],
            admin_password: String::new(),
            glass_wait_timeout_secs: 60,
            drink_ready_timeout_secs: 300,
        };
        RamStorageHal {
            stored: Some(defaults),
        }
    }

    /// Create an empty instance (no stored config).
    ///
    /// `backup()` will return `Err(NO_CONFIG)` until `restore()` is called.
    /// Use this in tests that exercise the `Provisioning` boot path.
    pub fn empty() -> Self {
        RamStorageHal { stored: None }
    }
}

impl StorageHal for RamStorageHal {
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo> {
        let admin_cfg = self.stored.clone().ok_or_else(|| ErrorInfo {
            code: String::from("NO_CONFIG"),
            message: String::from("No configuration stored"),
            hint: Some(String::from("POST /v1/config/restore to provision")),
            recoverable: true,
        })?;
        let json = serde_json::to_string(&admin_cfg).unwrap_or_default();
        let checksum = crc32_hex(json.as_bytes());
        Ok(BackupPayload {
            data: admin_cfg,
            checksum,
            backed_up_at: String::from("1970-01-01T00:00:00Z"),
        })
    }

    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        self.stored = Some(cfg);
        Ok(())
    }
}
