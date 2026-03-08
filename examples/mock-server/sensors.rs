// examples/mock-server/sensors.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{ErrorInfo, GlassSensorState, LevelState, SensorHal};

use crate::state::MockState;

/// [`SensorHal`] backed by shared mock state.
pub struct MockSensors {
    pub state: Arc<Mutex<MockState>>,
}

impl SensorHal for MockSensors {
    async fn glass_state(&self) -> Result<GlassSensorState, ErrorInfo> {
        Ok(self.state.lock().unwrap().glass.clone())
    }

    async fn level_state(&self) -> Result<Vec<LevelState>, ErrorInfo> {
        // Return plausible static levels for all configured liquids.
        let s = self.state.lock().unwrap();
        let levels = s
            .config
            .liquids
            .iter()
            .map(|l| LevelState::Binary {
                id: l.id.clone(),
                ok: true,
            })
            .collect();
        Ok(levels)
    }
}
