// src/esp32/status.rs
//
// ESP32 stub implementation of StatusHal.

use alloc::vec::Vec;

use crate::hal::{ErrorInfo, RobotState, StatusHal};

/// Stub implementation of [`StatusHal`] for ESP32.
pub struct Esp32Status;

impl Esp32Status {
    pub fn new() -> Self {
        Esp32Status
    }
}

impl StatusHal for Esp32Status {
    async fn state(&self) -> RobotState {
        // TODO: wire to hardware — read actual state from shared state machine
        RobotState::Idle
    }

    async fn active_errors(&self) -> Vec<ErrorInfo> {
        // TODO: wire to hardware — read from error ring buffer
        Vec::new()
    }
}
