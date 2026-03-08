// examples/mock-server/control.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{ControlHal, ErrorInfo, RobotState};

use crate::state::MockState;

/// [`ControlHal`] backed by shared mock state.
pub struct MockControl {
    pub state: Arc<Mutex<MockState>>,
}

impl ControlHal for MockControl {
    async fn power(&mut self, on: bool) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        if on {
            s.robot_state = RobotState::SelfTest;
            s.boot_ticks = 0;
        } else {
            s.robot_state = RobotState::Off;
        }
        Ok(())
    }

    async fn power_save(&mut self, _enabled: bool) -> Result<(), ErrorInfo> {
        // No-op: mock server has no hardware power state.
        Ok(())
    }

    async fn reset_errors(&mut self) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        if matches!(s.robot_state, RobotState::Error { .. }) {
            s.robot_state = RobotState::Idle;
        }
        Ok(())
    }
}
