// examples/mock-server/dispense.rs

use std::sync::{Arc, Mutex};

use cocktail_bot_hal::hal::{
    DispenseHal, DispenseItem, ErrorInfo, JobCreated, JobStatus, RobotState,
};

use crate::state::{MockJob, MockJobState, MockState};

/// [`DispenseHal`] backed by shared mock state.
pub struct MockDispense {
    pub state: Arc<Mutex<MockState>>,
}

impl DispenseHal for MockDispense {
    async fn create_job(
        &mut self,
        job_id: String,
        name: String,
        items: Vec<DispenseItem>,
        _parallel: bool,
    ) -> Result<JobCreated, ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        match &s.robot_state {
            RobotState::Idle => {}
            _ => {
                return Err(ErrorInfo {
                    code: String::from("WRONG_STATE"),
                    message: String::from("Robot must be Idle to start a job"),
                    hint: Some(String::from("Wait until state is idle")),
                    recoverable: true,
                });
            }
        }
        let total_ticks = s.dispense_ticks;
        let job = MockJob {
            job_id: job_id.clone(),
            name,
            _items: items,
            state: MockJobState::Active,
            progress_pct: 0,
            total_ticks,
            elapsed_ticks: 0,
        };
        s.robot_state = RobotState::Working {
            job_id: job_id.clone(),
            progress_pct: 0,
        };
        s.jobs.push(job);
        Ok(JobCreated {
            job_id,
            queue_position: 1,
        })
    }

    async fn list_jobs(&self) -> Vec<JobStatus> {
        self.state
            .lock()
            .unwrap()
            .jobs
            .iter()
            .map(|j| j.to_job_status())
            .collect()
    }

    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        let s = self.state.lock().unwrap();
        s.jobs
            .iter()
            .find(|j| j.job_id == job_id)
            .map(|j| j.to_job_status())
            .ok_or_else(|| ErrorInfo {
                code: String::from("NOT_FOUND"),
                message: format!("Job '{}' not found", job_id),
                hint: None,
                recoverable: false,
            })
    }

    async fn cancel_job(&mut self, job_id: &str) -> Result<(), ErrorInfo> {
        let mut s = self.state.lock().unwrap();
        let job = s.jobs.iter_mut().find(|j| j.job_id == job_id);
        match job {
            Some(j) if matches!(j.state, MockJobState::Active) => {
                j.state = MockJobState::Cancelled;
                // Return to Idle if the cancelled job was the active one.
                if matches!(&s.robot_state, RobotState::Working { job_id: jid, .. } if jid == job_id)
                {
                    s.robot_state = RobotState::Idle;
                }
                Ok(())
            }
            Some(_) => Err(ErrorInfo {
                code: String::from("NOT_ACTIVE"),
                message: format!("Job '{}' is not active", job_id),
                hint: Some(String::from("Only active jobs can be cancelled")),
                recoverable: false,
            }),
            None => Err(ErrorInfo {
                code: String::from("NOT_FOUND"),
                message: format!("Job '{}' not found", job_id),
                hint: None,
                recoverable: false,
            }),
        }
    }
}
