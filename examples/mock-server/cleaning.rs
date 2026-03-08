// examples/mock-server/cleaning.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{CleaningHal, ErrorInfo, RobotState};

use crate::state::MockState;

/// [`CleaningHal`] backed by shared mock state.
///
/// Cleaning runs for 30 seconds by default (300 ticks of 100 ms).
const DEFAULT_CLEANING_TICKS: u32 = 300;

pub struct MockCleaning {
    pub state: Arc<Mutex<MockState>>,
}

impl CleaningHal for MockCleaning {
    async fn start_cleaning(&mut self) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        match &s.robot_state {
            RobotState::Idle => {}
            _ => {
                return Err(ErrorInfo {
                    code: String::from("WRONG_STATE"),
                    message: String::from("Robot must be Idle to start cleaning"),
                    hint: Some(String::from("Wait until state is idle")),
                    recoverable: true,
                });
            }
        }
        s.cleaning_remaining_ticks = Some(DEFAULT_CLEANING_TICKS);
        s.robot_state = RobotState::Cleaning;
        Ok(())
    }

    async fn stop_cleaning(&mut self) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        s.cleaning_remaining_ticks = None;
        if matches!(s.robot_state, RobotState::Cleaning) {
            s.robot_state = RobotState::Idle;
        }
        Ok(())
    }
}
