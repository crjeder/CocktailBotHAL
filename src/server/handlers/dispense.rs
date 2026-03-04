// src/server/handlers/dispense.rs
//
// Handlers for POST/GET /v1/dispense/jobs and
// GET/POST /v1/dispense/jobs/{job_id}

use alloc::string::String;
use alloc::vec::Vec;
use embassy_time::Duration;
use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::{DispenseHal, JobItem};
use crate::server::http::{self, HttpRequest};

/// Default job timeout (30 seconds) when not specified in the
/// request.
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// POST /v1/dispense/jobs — create and queue a new dispensing job.
pub async fn handle_create_job<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &mut Disp,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct Body {
        client_job_id: String,
        items: Vec<JobItem>,
        #[serde(default)]
        require_glass: bool,
        #[serde(default)]
        parallel: bool,
        #[serde(default)]
        timeout_ms: Option<u64>,
    }

    let body: Body = match http::parse_body(request) {
        Ok(b) => b,
        Err(_) => {
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

    let timeout = Duration::from_millis(body.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS));

    match dispense
        .create_job(
            body.client_job_id,
            body.items,
            body.require_glass,
            body.parallel,
            timeout.into(),
        )
        .await
    {
        Ok(job_id) => {
            http::write_json(socket, 202, &serde_json::json!({"job_id": job_id}))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// GET /v1/dispense/jobs — list job queue and history.
pub async fn handle_list_jobs<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &Disp,
    socket: &mut W,
) {
    let jobs = dispense.list_jobs().await;
    http::write_json(socket, 200, &serde_json::json!(jobs))
        .await
        .ok();
}

/// GET /v1/dispense/jobs/{job_id} — job status with progress.
pub async fn handle_job_status<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &Disp,
    job_id: &str,
    socket: &mut W,
) {
    match dispense.job_status(job_id).await {
        Ok(status) => {
            http::write_json(socket, 200, &serde_json::json!(status))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/dispense/jobs/{job_id} — cancel a job.
pub async fn handle_cancel_job<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &mut Disp,
    job_id: &str,
    socket: &mut W,
) {
    match dispense.cancel_job(job_id).await {
        Ok(()) => {
            http::write_accepted(socket).await.ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}
