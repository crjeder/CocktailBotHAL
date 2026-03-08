// examples/mock-server/status.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{ErrorInfo, RobotState, StatusHal};

use crate::state::MockState;

/// [`StatusHal`] backed by shared mock state.
pub struct MockStatus {
    pub state: Arc<Mutex<MockState>>,
}

impl StatusHal for MockStatus {
    async fn state(&self) -> RobotState {
        self.state.lock().unwrap().robot_state.clone()
    }

    async fn active_errors(&self) -> Vec<ErrorInfo> {
        // Errors are embedded in RobotState::Error — return a synthesised
        // ErrorInfo when that variant is active.
        let s = self.state.lock().unwrap();
        match &s.robot_state {
            RobotState::Error {
                code,
                message,
                recoverable,
                ..
            } => vec![ErrorInfo {
                code: code.clone(),
                message: message.clone(),
                hint: None,
                recoverable: *recoverable,
            }],
            _ => vec![],
        }
    }
}
