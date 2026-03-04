// src/server/handlers/dispense.rs
//
// Handlers for POST/GET /v1/dispense/jobs and /v1/dispense/jobs/{job_id}
//
// TODO: Wire handle_job_status and handle_cancel_job into the route dispatch
// in src/server/mod.rs. Note: these require path parameter extraction —
// see TODO.md for the routing design decision.

use embedded_io_async::Write;

use crate::server::RobotHal;

/// POST /v1/dispense/jobs — create and queue a new dispensing job.
/// Body: JobCreateRequest (liquid_id, parts, require_glass, timeout, ...)
/// TODO: Parse body, call hal.dispense.create_job(...), return 202 with
/// { "job_id": "..." } or error JSON.
pub async fn handle_create_job<R, W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _request: R,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/dispense/jobs")
}

/// GET /v1/dispense/jobs — list job queue and history.
/// TODO: Requires a design decision — DispenseHal has no list_jobs() method.
/// Either add it to the trait in src/hal/mod.rs, or maintain a job registry
/// in ApiServer. See TODO.md for details.
pub async fn handle_list_jobs<W: Write + Unpin>(_hal: &RobotHal<'_>, _socket: &mut W)
{
    todo!("Implement GET /v1/dispense/jobs — requires list_jobs() design decision")
}

/// GET /v1/dispense/jobs/{job_id} — job status with progress.
/// TODO: Extract job_id from path, call hal.dispense.job_status(job_id),
/// serialize JobStatus as JSON.
pub async fn handle_job_status<W: Write + Unpin>(
    _hal: &RobotHal<'_>,
    _job_id: &str,
    _socket: &mut W,
)
{
    todo!("Implement GET /v1/dispense/jobs/{{job_id}}")
}

/// POST /v1/dispense/jobs/{job_id} — cancel a job.
/// TODO: Extract job_id from path, call hal.dispense.cancel_job(job_id),
/// return 202 or error JSON.
pub async fn handle_cancel_job<W: Write + Unpin>(
    _hal: &mut RobotHal<'_>,
    _job_id: &str,
    _socket: &mut W,
)
{
    todo!("Implement POST /v1/dispense/jobs/{{job_id}} (cancel)")
}
