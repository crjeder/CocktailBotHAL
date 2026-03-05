// src/esp32/config.rs
//
// ESP32 stub implementation of ConfigHal.

use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::{Capabilities, ConfigHal, ErrorInfo, GlassType, LevelReporting, RobotConfig};

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
        glass_types: alloc::vec![
            GlassType {
                id: String::from("short"),
                volume_ml: 100.0
            },
            GlassType {
                id: String::from("medium"),
                volume_ml: 150.0
            },
            GlassType {
                id: String::from("long"),
                volume_ml: 200.0
            },
        ],
        max_total_parts: 10,
        capabilities: Capabilities {
            level_reporting: LevelReporting::Binary,
            glass_typing: false,
            simultaneous_channels: 1,
            max_queue_depth: 5,
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
