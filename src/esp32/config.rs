// src/esp32/config.rs
//
// ESP32 stub implementation of ConfigHal.

use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::{Capabilities, ConfigHal, ErrorInfo, LevelReporting, RobotConfig};

/// Stub implementation of [`ConfigHal`] for ESP32.
pub struct Esp32Config;

impl Esp32Config {
    pub fn new() -> Self {
        Esp32Config
    }
}

fn default_config() -> RobotConfig {
    RobotConfig {
        version: String::from("0.0.0-stub"),
        liquids: Vec::new(),
        part_ml: 30.0,
        max_total_parts: 10,
        max_channels_per_job: 1,
        capabilities: Capabilities {
            level_reporting: LevelReporting::Binary,
            glass_typing: false,
            simultaneous_channels: 1,
        },
        token: String::new(),
    }
}

impl ConfigHal for Esp32Config {
    async fn get_active_config(&self) -> RobotConfig {
        // TODO: wire to hardware — return config from in-RAM config store
        default_config()
    }

    async fn update_active_config(&mut self, _cfg: RobotConfig) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — apply config to in-RAM config store
        Ok(())
    }
}
