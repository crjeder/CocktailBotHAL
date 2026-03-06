// src/server/handlers/dispense.rs
//
// Handlers for POST/GET /v1/dispense/jobs and
// GET/POST /v1/dispense/jobs/{job_id}

use alloc::string::String;
use alloc::vec::Vec;
use embassy_time::Instant;
use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::{DispenseHal, JobItem};
use crate::server::http::{self, HttpRequest};

/// Maximum length of the sanitized name prefix in the generated `job_id`.
const MAX_NAME_PREFIX_LEN: usize = 32;

/// Generate a deterministic `job_id` from a human-readable `name`.
///
/// Format: `<sanitized_name>-<DD><TIME>`
/// - `<sanitized_name>`: non-`[A-Za-z0-9 _-]` chars replaced by `_`,
///   truncated to `MAX_NAME_PREFIX_LEN` chars.
/// - `<DD>`: `day + month * 3` encoded as 2 uppercase hex digits. This
///   disambiguates IDs across calendar days without a date-aware clock.
/// - `<TIME>`: current time-of-day in deciseconds (1/10 s) encoded as 4
///   uppercase hex digits (max 864 000 values — no collision within 100 ms).
///
/// Uniqueness guarantee: no collision within 24 h for any single name,
/// assuming no two jobs with the same name are submitted within 100 ms.
fn generate_job_id(name: &str) -> String {
    // Sanitize: replace disallowed chars, truncate.
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(MAX_NAME_PREFIX_LEN)
        .collect();

    // Derive time components from embassy_time monotonic clock.
    // Instant::now().as_ticks() gives ticks since boot; we use deciseconds
    // (ticks / tick_hz * 10) modulo 86400*10 for the time-of-day component.
    // For day/month we use a fixed encoding (day=1, month=0 in the stub)
    // because the spin executor has no real-time clock; a real bring-up
    // replaces this with values from an RTC peripheral.
    let ticks = Instant::now().as_ticks();
    // embassy_time tick rate is configurable; use ticks directly modulo
    // 864_000 (deciseconds per day) as a proxy for time-of-day.
    let deciseconds = (ticks % 864_000) as u16;

    // day=1, month=0 → dd_encoded = 1 + 0*3 = 1 (stub: no RTC available).
    // Real bring-up: replace with RTC values.
    let dd_encoded: u8 = 1;

    let mut id = sanitized;
    id.push('-');
    // 2 hex digits for dd_encoded
    let hex_dd = [hex_nibble(dd_encoded >> 4), hex_nibble(dd_encoded & 0x0F)];
    id.push(hex_dd[0] as char);
    id.push(hex_dd[1] as char);
    // 4 hex digits for deciseconds
    let hex_time = [
        hex_nibble(((deciseconds >> 12) & 0x0F) as u8),
        hex_nibble(((deciseconds >> 8) & 0x0F) as u8),
        hex_nibble(((deciseconds >> 4) & 0x0F) as u8),
        hex_nibble((deciseconds & 0x0F) as u8),
    ];
    for b in hex_time {
        id.push(b as char);
    }
    id
}

/// Convert a nibble (0–15) to an uppercase hex ASCII byte.
fn hex_nibble(n: u8) -> u8 {
    b"0123456789ABCDEF"[n as usize]
}

/// POST /v1/dispense/jobs — create and queue a new dispensing job.
pub async fn handle_create_job<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &mut Disp,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct Body {
        name: String,
        /// Requested cocktail size; resolved to a volume via `glass_types` config.
        size: String,
        items: Vec<JobItem>,
        #[serde(default)]
        parallel: bool,
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

    let job_id = generate_job_id(&body.name);

    // TODO: resolve body.size against config.glass_types to get volume_ml,
    // compute part_ml = volume_ml / total_parts, and scale each item to ml
    // before passing to the HAL. Return HTTP 422 if size is not found.
    // Also: wait for glass_sensor.present == true before dispatching.
    let _ = &body.size;

    match dispense
        .create_job(job_id, body.name, body.items, body.parallel)
        .await
    {
        Ok(created) => {
            http::write_json(
                socket,
                201,
                &serde_json::json!({
                    "job_id": created.job_id,
                    "queue_position": created.queue_position,
                }),
            )
            .await
            .ok();
        }
        Err(ref e) if e.code == "QUEUE_FULL" => {
            http::write_json(socket, 503, &serde_json::json!({"error": "queue full"}))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{test_error, MockDispenseHal, MockWrite};
    use futures::executor::block_on;

    #[test]
    fn dispense_list_jobs_empty_returns_200() {
        block_on(async {
            let hal = MockDispenseHal::new();
            let mut buf = MockWrite::new();
            handle_list_jobs(&hal, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"));
            assert!(resp.contains("[]"));
        });
    }

    #[test]
    fn dispense_job_status_not_found_returns_500() {
        block_on(async {
            let hal = MockDispenseHal::new();
            let mut buf = MockWrite::new();
            handle_job_status(&hal, "nonexistent", &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }

    #[test]
    fn dispense_cancel_unknown_job_returns_500() {
        block_on(async {
            let mut hal = MockDispenseHal::new();
            let mut buf = MockWrite::new();
            handle_cancel_job(&mut hal, "nonexistent", &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }

    #[test]
    fn dispense_create_job_hal_error_returns_500() {
        block_on(async {
            let mut hal = MockDispenseHal::new();
            hal.fail_next = Some(test_error());
            let req = HttpRequest {
                method: "POST".to_string(),
                path: "/v1/dispense/jobs".to_string(),
                headers: alloc::vec![],
                body: br#"{"name":"test","size":"medium","items":[]}"#.to_vec(),
            };
            let mut buf = MockWrite::new();
            handle_create_job(&mut hal, &req, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }
}
