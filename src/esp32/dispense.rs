// src/esp32/dispense.rs
//
// ESP32 stub implementation of DispenseHal.

use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::hal::{DispenseHal, ErrorInfo, JobItem, JobStatus};

/// Stub implementation of [`DispenseHal`] for ESP32.
pub struct Esp32Dispense;

impl Esp32Dispense {
    pub fn new() -> Self {
        Esp32Dispense
    }
}

impl DispenseHal for Esp32Dispense {
    fn create_job(
        &mut self,
        client_job_id: String,
        _items: Vec<JobItem>,
        _require_glass: bool,
        _parallel: bool,
        _timeout: Duration,
    ) -> Result<String, ErrorInfo> {
        // TODO: wire to hardware — enqueue job to dispense task scheduler
        let job_id = alloc::format!("stub-job-{}", client_job_id);
        Ok(job_id)
    }

    fn list_jobs(&self) -> Vec<JobStatus> {
        // TODO: wire to hardware — return job queue from task scheduler
        Vec::new()
    }

    fn job_status(&self, job_id: &str) -> Result<JobStatus, ErrorInfo> {
        // TODO: wire to hardware — look up job status in task scheduler
        Err(ErrorInfo {
            code: String::from("NOT_FOUND"),
            message: alloc::format!("Job '{}' not found (stub)", job_id),
            hint: None,
            recoverable: true,
        })
    }

    fn cancel_job(&mut self, _job_id: &str) -> Result<(), ErrorInfo> {
        // TODO: wire to hardware — cancel job in task scheduler
        Ok(())
    }
}
