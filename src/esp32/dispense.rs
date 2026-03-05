// src/esp32/dispense.rs
//
// ESP32 stub implementation of DispenseHal.

use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::hal::{DispenseHal, ErrorInfo, JobCreated, JobItem, JobStatus};

/// Stub implementation of [`DispenseHal`] for ESP32.
pub struct Esp32Dispense;

impl Esp32Dispense {
    pub fn new() -> Self {
        Esp32Dispense
    }
}

impl DispenseHal for Esp32Dispense {
    async fn create_job(
        &mut self,
        job_id: String,
        _name: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<JobCreated, ErrorInfo> {
        // TODO: wire to hardware — enqueue job to dispense task scheduler
        Ok(JobCreated {
            job_id,
            queue_position: 1,
        })
    }

    async fn list_jobs(&self) -> Vec<JobStatus> {
        // TODO: wire to hardware — return job queue from task scheduler
        Vec::new()
    }

    async fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        // TODO: wire to hardware — look up job status in task scheduler
        Err(ErrorInfo {
            code: String::from("NOT_FOUND"),
            message: alloc::format!("Job '{}' not found (stub)", job_id),
            hint: None,
            recoverable: true,
        })
    }

    async fn cancel_job(&mut self, _job_id: &str) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — cancel job in task scheduler
        Ok(())
    }
}
