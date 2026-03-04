// src/esp32/cleaning.rs
//
// ESP32 stub implementation of CleaningHal.

use crate::hal::{CleaningHal, ErrorInfo};

/// Stub implementation of [`CleaningHal`] for ESP32.
pub struct Esp32Cleaning;

impl Esp32Cleaning {
    pub fn new() -> Self {
        Esp32Cleaning
    }
}

impl CleaningHal for Esp32Cleaning {
    fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — start cleaning pump sequence
        Ok(())
    }

    fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — stop cleaning pump and flush lines
        Ok(())
    }
}
