// src/server/handlers/dispense.rs
//
// Handlers for POST/GET /v1/dispense/jobs and
// GET/POST /v1/dispense/jobs/{job_id}

use alloc::string::String;
use alloc::vec::Vec;
use embedded_io_async::Write;
use embassy_time::Duration;
use serde::Deserialize;

use crate::hal::JobItem;
use crate::server::http::{self, HttpRequest};
use crate::server::RobotHal;

/// Default job timeout (30 seconds) when not specified in the
/// request.
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// POST /v1/dispense/jobs — create and queue a new dispensing job.
pub async fn handle_create_job<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    request: &HttpRequest,
    socket: &mut W,
)
{
    #[derive(Deserialize)]
    struct Body
    {
        client_job_id: String,
        items: Vec<JobItem>,
        #[serde(default)]
        require_glass: bool,
        #[serde(default)]
        parallel: bool,
        #[serde(default)]
        timeout_ms: Option<u64>,
    }

    let body: Body = match http::parse_body(request)
    {
        Ok(b) => b,
        Err(_) =>
        {
            http::write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid request body"}),
            )
            .await
            .ok();
            return;
        }
    };

    let timeout = Duration::from_millis(
        body.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
    );

    match hal.dispense.create_job(
        body.client_job_id,
        body.items,
        body.require_glass,
        body.parallel,
        timeout,
    )
    {
        Ok(job_id) =>
        {
            http::write_json(
                socket,
                202,
                &serde_json::json!({"job_id": job_id}),
            )
            .await
            .ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/dispense/jobs — list job queue and history.
pub async fn handle_list_jobs<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    socket: &mut W,
)
{
    let jobs = hal.dispense.list_jobs();
    http::write_json(
        socket,
        200,
        &serde_json::json!(jobs),
    )
    .await
    .ok();
}

/// GET /v1/dispense/jobs/{job_id} — job status with progress.
pub async fn handle_job_status<W: Write + Unpin>(
    hal: &RobotHal<'_>,
    job_id: &str,
    socket: &mut W,
)
{
    match hal.dispense.job_status(job_id)
    {
        Ok(status) =>
        {
            http::write_json(
                socket,
                200,
                &serde_json::json!(status),
            )
            .await
            .ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/dispense/jobs/{job_id} — cancel a job.
pub async fn handle_cancel_job<W: Write + Unpin>(
    hal: &mut RobotHal<'_>,
    job_id: &str,
    socket: &mut W,
)
{
    match hal.dispense.cancel_job(job_id)
    {
        Ok(()) =>
        {
            http::write_accepted(socket).await.ok();
        }
        Err(e) =>
        {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
