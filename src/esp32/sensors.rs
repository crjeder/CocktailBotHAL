// src/esp32/sensors.rs
//
// ESP32 stub implementation of SensorHal.

use alloc::vec::Vec;

use crate::hal::{ErrorInfo, GlassSensorState, LevelState, SensorHal};

/// Stub implementation of [`SensorHal`] for ESP32.
pub struct Esp32Sensors;

impl Esp32Sensors {
    pub fn new() -> Self {
        Esp32Sensors
    }
}

impl SensorHal for Esp32Sensors {
    fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        // TODO: wire to hardware — read capacitive / IR glass sensor
        Ok(GlassSensorState {
            present: false,
            glass_type: None,
            confidence: 0.0,
        })
    }

    fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        // TODO: wire to hardware — read level sensors for each liquid channel
        Ok(Vec::new())
    }
}
