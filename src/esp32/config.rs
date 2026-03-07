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
        version: String::from("0.6.0"),
        level_reporting: LevelReporting::Binary,
        glass_typing: false,
        simultaneous_channels: 1,
        max_queue_depth: 5,
        has_cancel_button: false,
        has_power_button: false,
    }
}

fn default_admin_config() -> AdminConfig {
    AdminConfig {
        liquids: Vec::new(),
        glass_types: alloc::vec![
            GlassType {
                id: String::from("short"),
                volume: 100.0,
            },
            GlassType {
                id: String::from("medium"),
                volume: 150.0,
            },
            GlassType {
                id: String::from("long"),
                volume: 200.0,
            },
        ],
        token: String::new(),
        admin_password: String::new(),
        glass_wait_timeout_secs: 60,
        drink_ready_timeout_secs: 300,
    }
}

impl ConfigHal for Esp32Config {
    async fn get_active_config(&self) -> RobotConfig {
        // TODO: wire to hardware — return config from in-RAM config store
        let admin = default_admin_config();
        RobotConfig {
            liquids: admin.liquids,
            glass_types: admin.glass_types,
            capabilities: stub_capabilities(),
            token: admin.token,
            admin_password: admin.admin_password,
            glass_wait_timeout_secs: admin.glass_wait_timeout_secs,
            drink_ready_timeout_secs: admin.drink_ready_timeout_secs,
        }
    }

    async fn update_active_config(&mut self, _cfg: AdminConfig) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — apply config to in-RAM config store
        Ok(())
    }
}
