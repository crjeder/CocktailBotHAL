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
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        // TODO: wire to hardware — read capacitive / IR glass sensor
        // Optimistic default: no sensor wired → assume glass is present.
        // confidence: 0.0 signals this is not a real hardware reading.
        Ok(GlassSensorState {
            present: true,
            glass_type: None,
            confidence: 0.0,
        })
    }

    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        // TODO: wire to hardware — read level sensors for each liquid channel
        Ok(Vec::new())
    }
}
