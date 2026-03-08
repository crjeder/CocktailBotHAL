// examples/mock-server/config.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{AdminConfig, ConfigHal, ErrorInfo, RobotConfig};

use crate::state::MockState;

/// [`ConfigHal`] backed by shared mock state.
///
/// Updates replace the admin-owned fields while preserving the fixed
/// `Capabilities` block.
pub struct MockConfig {
    pub state: Arc<Mutex<MockState>>,
}

impl ConfigHal for MockConfig {
    async fn get_active_config(&self) -> RobotConfig {
        self.state.lock().unwrap().config.clone()
    }

    async fn update_active_config(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        let caps = s.config.capabilities.clone();
        s.config = RobotConfig {
            liquids: cfg.liquids,
            glass_types: cfg.glass_types,
            capabilities: caps,
            token: cfg.token,
            admin_password: cfg.admin_password,
            glass_wait_timeout_secs: cfg.glass_wait_timeout_secs,
            drink_ready_timeout_secs: cfg.drink_ready_timeout_secs,
        };
        Ok(())
    }
}
