// src/esp32/control.rs
//
// ESP32 stub implementation of ControlHal.
// All methods are placeholders — see TODO comments for hardware wiring points.

use crate::hal::{ControlHal, ErrorInfo};

/// Stub implementation of [`ControlHal`] for ESP32.
pub struct Esp32Control;

impl Esp32Control {
    pub fn new() -> Self {
        Esp32Control
    }
}

impl ControlHal for Esp32Control {
    fn power(&mut self, _on: bool) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — toggle main power relay / deep-sleep
        Ok(())
    }

    fn power_save(&mut self, _enabled: bool) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — configure ESP32 light-sleep or reduce clock
        Ok(())
    }

    fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — clear fault latches, reset error state
        Ok(())
    }

    fn reload_config(&mut self) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — reload config from NVS flash
        Ok(())
    }
}
