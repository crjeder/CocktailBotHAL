// src/esp32/config.rs
//
// ESP32 stub implementation of ConfigHal.

use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::{
    AdminConfig, Capabilities, ConfigHal, ErrorInfo, GlassType, LevelReporting, RobotConfig,
};

/// Stub implementation of [`ConfigHal`] for ESP32.
pub struct Esp32Config;

impl Esp32Config {
    pub fn new() -> Self {
        Esp32Config
    }
}

fn stub_capabilities() -> Capabilities {
    Capabilities {
        version: String::from("0.5.0"),
        level_reporting: LevelReporting::Binary,
        glass_typing: false,
        simultaneous_channels: 1,
        max_queue_depth: 5,
    }
}

fn default_admin_config() -> AdminConfig {
    AdminConfig {
        liquids: Vec::new(),
        glass_types: alloc::vec![
            GlassType {
                id: String::from("short"),
                volume_ml: 100.0,
            },
            GlassType {
                id: String::from("medium"),
                volume_ml: 150.0,
            },
            GlassType {
                id: String::from("long"),
                volume_ml: 200.0,
            },
        ],
        max_total_parts: 10,
        token: String::new(),
        admin_password: String::new(),
    }
}

impl ConfigHal for Esp32Config {
    async fn get_active_config(&self) -> RobotConfig {
        // TODO: wire to hardware — return config from in-RAM config store
        let admin = default_admin_config();
        RobotConfig {
            liquids: admin.liquids,
            glass_types: admin.glass_types,
            max_total_parts: admin.max_total_parts,
            capabilities: stub_capabilities(),
            token: admin.token,
            admin_password: admin.admin_password,
        }
    }

    async fn update_active_config(&mut self, _cfg: AdminConfig) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — apply config to in-RAM config store
        Ok(())
    }
}
