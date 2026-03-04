// src/server/sse.rs
//
// Server-Sent Events server running on a separate TCP port (9000).
// Polls existing HAL traits for state and job changes every 500ms
// and emits typed SSE events to connected clients.

use alloc::format;
use alloc::vec::Vec;
use embassy_net::tcp::TcpSocket;
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::Write;

use crate::hal::{JobStatus, RobotState};
use crate::server::RobotHal;

/// SSE server port.
const SSE_PORT: u16 = 9000;

/// Interval between HAL polls (500ms).
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Send a keepalive comment if no events for 30 seconds.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Snapshot of the state we track for change detection.
struct Snapshot {
    state: RobotState,
    jobs: Vec<JobStatus>,
}

// ====================================================================
// SSE frame helpers
// ====================================================================

/// Write an SSE frame: `event: <type>\ndata: <json>\n\n`
async fn write_sse_event<W: Write + Unpin>(
    socket: &mut W,
    event_type: &str,
    data: &serde_json::Value,
) -> Result<(), ()> {
    let json = serde_json::to_string(data).map_err(|_| ())?;
    let frame = format!("event: {}\ndata: {}\n\n", event_type, json);
    socket.write_all(frame.as_bytes()).await.map_err(|_| ())
}

/// Write an SSE keepalive comment: `: keepalive\n\n`
async fn write_keepalive<W: Write + Unpin>(socket: &mut W) -> Result<(), ()> {
    socket.write_all(b": keepalive\n\n").await.map_err(|_| ())
}

/// Write the HTTP response headers for an SSE stream.
async fn write_sse_headers<W: Write + Unpin>(socket: &mut W) -> Result<(), ()> {
    let headers = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/event-stream\r\n\
                   Cache-Control: no-cache\r\n\
                   Connection: keep-alive\r\n\r\n";
    socket.write_all(headers.as_bytes()).await.map_err(|_| ())
}

// ====================================================================
// Snapshot capture and diffing
// ====================================================================

fn capture_snapshot(hal: &RobotHal<'_>) -> Snapshot {
    Snapshot {
        state: hal.status.state(),
        jobs: hal.dispense.list_jobs(),
    }
}

/// Emit `state_change` event for the current state.
async fn emit_state_event<W: Write + Unpin>(
    socket: &mut W,
    state: &RobotState,
) -> Result<(), ()> {
    write_sse_event(
        socket,
        "state_change",
        &serde_json::json!({"state": state}),
    )
    .await
}

/// Emit a `job_update` event for a single job.
async fn emit_job_event<W: Write + Unpin>(
    socket: &mut W,
    job: &JobStatus,
) -> Result<(), ()> {
    write_sse_event(
        socket,
        "job_update",
        &serde_json::json!({
            "job_id": job.job_id,
            "client_job_id": job.client_job_id,
            "state": job.state,
            "progress_pct": job.progress_pct,
        }),
    )
    .await
}

/// Compare two job lists and return whether any job changed.
fn job_changed(old: &JobStatus, new: &JobStatus) -> bool {
    // Compare serialized state strings and progress.
    // JobState doesn't derive PartialEq, so we compare via
    // the progress_pct and serialize the state for comparison.
    old.progress_pct != new.progress_pct
        || serde_json::to_string(&old.state).ok()
            != serde_json::to_string(&new.state).ok()
}

// ====================================================================
// SSE server
// ====================================================================

/// SSE server that runs on port 9000. Accepts one client at a
/// time. Each connected client receives an initial snapshot
/// followed by change events at 500ms polling intervals.
pub struct SseServer<'a> {
    pub hal: &'a RobotHal<'a>,
}

impl<'a> SseServer<'a> {
    /// Main accept loop — listens on port 9000.
    pub async fn run(&self, net_stack: embassy_net::Stack<'_>) {
        loop {
            let mut rx_buf = [0u8; 1024];
            let mut tx_buf = [0u8; 4096];
            let mut socket =
                TcpSocket::new(net_stack, &mut rx_buf, &mut tx_buf);
            if socket.accept(SSE_PORT).await.is_err() {
                continue;
            }
            // Discard the incoming HTTP request headers; we
            // only care that a connection was established.
            let _ = crate::server::http::read_http_request(&mut socket).await;

            self.handle_client(&mut socket).await;
        }
    }

    /// Serve one SSE client until the connection drops.
    async fn handle_client<W: Write + Unpin>(&self, socket: &mut W) {
        // Send SSE response headers.
        if write_sse_headers(socket).await.is_err() {
            return;
        }

        // Send initial snapshot.
        let mut prev = capture_snapshot(self.hal);

        if emit_state_event(socket, &prev.state).await.is_err() {
            return;
        }
        for job in &prev.jobs {
            if emit_job_event(socket, job).await.is_err() {
                return;
            }
        }

        let mut last_event_time = Instant::now();

        // Poll loop.
        loop {
            Timer::after(POLL_INTERVAL).await;

            let current = capture_snapshot(self.hal);
            let mut sent_event = false;

            // Check for state change.
            if current.state != prev.state {
                if emit_state_event(socket, &current.state).await.is_err() {
                    return;
                }
                sent_event = true;
            }

            // Check for job changes.
            for new_job in &current.jobs {
                let changed = match prev
                    .jobs
                    .iter()
                    .find(|old| old.job_id == new_job.job_id)
                {
                    Some(old_job) => job_changed(old_job, new_job),
                    None => true, // new job appeared
                };

                if changed {
                    if emit_job_event(socket, new_job).await.is_err() {
                        return;
                    }
                    sent_event = true;
                }
            }

            if sent_event {
                last_event_time = Instant::now();
            } else if Instant::now().duration_since(last_event_time)
                >= KEEPALIVE_INTERVAL
            {
                if write_keepalive(socket).await.is_err() {
                    return;
                }
                last_event_time = Instant::now();
            }

            prev = current;
        }
    }
}
