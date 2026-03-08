// examples/mock-server/storage.rs
//
// RAM-backed StorageHal for the mock server.
// Config is stored in memory only; changes are lost on restart.

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{crc32_hex, AdminConfig, BackupPayload, ErrorInfo, StorageHal};

use crate::state::MockState;

/// [`StorageHal`] backed by shared mock state.
///
/// Converts the active [`RobotConfig`] into an [`AdminConfig`] on backup and
/// applies [`AdminConfig`] on restore.  All storage is in RAM only.
pub struct MockStorage {
    pub state: Arc<Mutex<MockState>>,
}

impl StorageHal for MockStorage {
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo> {
        let s = self.state.lock().unwrap();
        let cfg = &s.config;
        let admin = AdminConfig {
            liquids: cfg.liquids.clone(),
            glass_types: cfg.glass_types.clone(),
            token: cfg.token.clone(),
            admin_password: cfg.admin_password.clone(),
            glass_wait_timeout_secs: cfg.glass_wait_timeout_secs,
            drink_ready_timeout_secs: cfg.drink_ready_timeout_secs,
        };
        let json = serde_json::to_string(&admin).unwrap_or_default();
        let checksum = crc32_hex(json.as_bytes());
        Ok(BackupPayload {
            data: admin,
            checksum,
            backed_up_at: String::from("1970-01-01T00:00:00Z"),
        })
    }

    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        let caps = s.config.capabilities.clone();
        s.config = cocktail_bot_hal::hal::RobotConfig {
            liquids: cfg.liquids,
            glass_types: cfg.glass_types,
            capabilities: caps,
            token: cfg.token,
            admin_password: cfg.admin_password,
            glass_wait_timeout_secs: cfg.glass_wait_timeout_secs,
            drink_ready_timeout_secs: cfg.drink_ready_timeout_secs,
        };
        Ok(())
    }
}
