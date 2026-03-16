// examples/mock-server/state.rs
//
// Shared in-memory state for the mock server.
// All HAL trait implementations hold an Arc<Mutex<MockState>> and read/write
// through it. A background ticker thread advances the state machine every
// 100 ms.

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{
    Capabilities, DispenseItem, GlassSensorState, GlassType, JobState, JobStatus, LevelReporting,
    LiquidCalibration, LiquidConfig, RobotConfig, RobotState,
};

/// Cloneable variant of [`JobState`] for internal tracking.
///
/// [`JobState`] has a custom serialiser but no `Clone`. We use this internally
/// and convert to [`JobState`] at the API boundary.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum MockJobState {
    Active,
    Done,
    Cancelled,
    Error(String),
}

impl MockJobState {
    pub fn to_job_state(&self) -> JobState {
        match self {
            MockJobState::Active => JobState::Active,
            MockJobState::Done => JobState::Done,
            MockJobState::Cancelled => JobState::Cancelled,
            MockJobState::Error(e) => JobState::Error(e.clone()),
        }
    }
}

/// A single dispensing job tracked by the mock state machine.
#[derive(Clone)]
pub struct MockJob {
    pub job_id: String,
    pub name: String,
    pub _items: Vec<DispenseItem>,
    pub state: MockJobState,
    pub progress_pct: u8,
    /// Total 100 ms ticks for a full dispense cycle.
    pub total_ticks: u32,
    /// Elapsed ticks since the job became active.
    pub elapsed_ticks: u32,
}

impl MockJob {
    pub fn to_job_status(&self) -> JobStatus {
        JobStatus {
            job_id: self.job_id.clone(),
            name: self.name.clone(),
            state: self.state.to_job_state(),
            progress_pct: self.progress_pct,
        }
    }
}

/// All mutable runtime state shared between HAL stubs and the ticker thread.
pub struct MockState {
    pub robot_state: RobotState,
    pub glass: GlassSensorState,
    pub config: RobotConfig,
    /// Job history (active + completed + cancelled).
    pub jobs: Vec<MockJob>,
    /// Remaining 100 ms ticks for the current cleaning cycle.
    pub cleaning_remaining_ticks: Option<u32>,
    /// Ticks spent in the current boot phase (`Off` or `SelfTest`).
    pub boot_ticks: u32,
    /// Total ticks for one dispense cycle (configurable via CLI).
    pub dispense_ticks: u32,
}

impl MockState {
    /// Create initial state.
    ///
    /// `glass_present` controls the initial glass sensor reading.
    /// `dispense_duration_secs` is how long a full dispense job takes.
    pub fn new(
        glass_present: bool,
        dispense_duration_secs: u32,
        initial_liquids: Option<Vec<cocktail_bot_hal::hal::LiquidConfig>>,
    ) -> Self {
        let mut config = default_config();
        if let Some(liquids) = initial_liquids {
            config.liquids = liquids;
        }
        MockState {
            robot_state: RobotState::Off,
            glass: GlassSensorState {
                present: glass_present,
                glass_type: None,
                confidence: 1.0,
            },
            config,
            jobs: Vec::new(),
            cleaning_remaining_ticks: None,
            boot_ticks: 0,
            dispense_ticks: dispense_duration_secs * 10,
        }
    }

    /// Advance the state machine by one 100 ms tick.
    pub fn tick(&mut self) {
        match self.robot_state.clone() {
            RobotState::Off => {
                self.boot_ticks += 1;
                if self.boot_ticks >= 10 {
                    self.robot_state = RobotState::SelfTest;
                    self.boot_ticks = 0;
                }
            }
            RobotState::SelfTest => {
                self.boot_ticks += 1;
                if self.boot_ticks >= 20 {
                    self.robot_state = RobotState::Idle;
                    self.boot_ticks = 0;
                }
            }
            RobotState::Working { .. } => {
                if let Some(job) = self
                    .jobs
                    .iter_mut()
                    .find(|j| matches!(j.state, MockJobState::Active))
                {
                    job.elapsed_ticks += 1;
                    let total = job.total_ticks.max(1);
                    let pct = ((job.elapsed_ticks * 100) / total).min(100) as u8;
                    job.progress_pct = pct;
                    if pct >= 100 {
                        let job_id = job.job_id.clone();
                        job.state = MockJobState::Done;
                        self.robot_state = RobotState::DrinkReady {
                            job_id,
                            timeout_remaining_secs: None,
                        };
                    } else {
                        let job_id = job.job_id.clone();
                        self.robot_state = RobotState::Working {
                            job_id,
                            progress_pct: pct,
                        };
                    }
                }
            }
            RobotState::Cleaning => match &mut self.cleaning_remaining_ticks {
                Some(0) | None => {
                    self.cleaning_remaining_ticks = None;
                    self.robot_state = RobotState::Idle;
                }
                Some(n) => {
                    *n -= 1;
                }
            },
            _ => {}
        }
    }

    /// Return a reference-counted handle to this state, wrapped in a Mutex.
    pub fn into_shared(self) -> Arc<Mutex<MockState>> {
        Arc::new(Mutex::new(self))
    }
}

fn default_config() -> RobotConfig {
    RobotConfig {
        liquids: vec![
            LiquidConfig {
                id: String::from("vodka"),
                name: String::from("Vodka"),
                position: 0,
                calibration: LiquidCalibration { factor: 1.0 },
            },
            LiquidConfig {
                id: String::from("juice"),
                name: String::from("Orange Juice"),
                position: 1,
                calibration: LiquidCalibration { factor: 1.0 },
            },
            LiquidConfig {
                id: String::from("water"),
                name: String::from("Sparkling Water"),
                position: 2,
                calibration: LiquidCalibration { factor: 1.0 },
            },
        ],
        glass_types: vec![
            GlassType {
                id: String::from("short"),
                volume: 100.0,
            },
            GlassType {
                id: String::from("medium"),
                volume: 150.0,
            },
            GlassType {
                id: String::from("long"),
                volume: 200.0,
            },
        ],
        capabilities: Capabilities {
            version: String::from("0.6.0-mock"),
            level_reporting: LevelReporting::Binary,
            glass_typing: false,
            simultaneous_channels: 1,
            max_queue_depth: 5,
            has_cancel_button: false,
            has_power_button: false,
        },
        // Empty token → server falls back to compile-time default "changeme".
        token: String::new(),
        // Empty hash → server falls back to compile-time default "changeme".
        admin_password: String::new(),
        glass_wait_timeout_secs: 60,
        drink_ready_timeout_secs: 300,
    }
}

/// Inject an error into state, transitioning to `RobotState::Error`.
pub fn inject_error(state: &Arc<Mutex<MockState>>, code: &str) {
    let mut s = state.lock().unwrap();
    s.robot_state = RobotState::Error {
        code: code.to_string(),
        message: format!("Injected error: {}", code),
        job_id: None,
        recoverable: true,
        recovery: cocktail_bot_hal::hal::RecoveryAction::CallResetErrors,
        timeout_remaining_secs: None,
    };
}
