// examples/mock-server/sse.rs
//
// Standalone SSE client handler for the mock server.
//
// Each connected SSE client gets its own OS thread (via `std::thread::spawn`).
// The thread polls `MockState` every 500 ms and writes `state_change` and
// `job_update` events whenever the robot state or a job changes.
// A `: keepalive` comment is sent if no event has been emitted for 30 seconds.
// The thread exits cleanly when a write to the TCP stream fails (client disconnect).
//
// No authentication is required for SSE (per API.yaml).

use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cocktail_bot_hal::hal::RobotState;

use crate::state::{MockJobState, MockState};

/// How often to poll `MockState` for changes.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Emit a keepalive comment if no event has been sent for this long.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// Snapshot — lightweight copy of the state we track for change detection
// ---------------------------------------------------------------------------

/// Lightweight snapshot of the mutable state we watch for SSE changes.
pub struct SseSnapshot {
    /// Full robot state — compared via `PartialEq` for change detection.
    pub state: RobotState,
    /// Per-job tuple: `(job_id, progress_pct, state_str)`.
    ///
    /// `state_str` is the serialised `JobState` string (e.g. `"active"`,
    /// `"done"`) used to detect lifecycle transitions without requiring
    /// `PartialEq` on `MockJobState`.
    pub jobs: Vec<(String, u8, String)>,
}

fn mock_job_state_str(s: &MockJobState) -> &'static str {
    match s {
        MockJobState::Active => "active",
        MockJobState::Done => "done",
        MockJobState::Cancelled => "cancelled",
        MockJobState::Error(_) => "error",
    }
}

// ---------------------------------------------------------------------------
// Low-level SSE write helpers
// ---------------------------------------------------------------------------

/// Write the HTTP 200 SSE response headers.
fn write_sse_headers(tcp: &mut TcpStream) -> io::Result<()> {
    tcp.write_all(
        b"HTTP/1.1 200 OK\r\n\
          Content-Type: text/event-stream\r\n\
          Cache-Control: no-cache\r\n\
          Connection: keep-alive\r\n\r\n",
    )
}

/// Write one SSE frame: `event: <type>\ndata: <json>\n\n`.
fn write_sse_event(
    tcp: &mut TcpStream,
    event_type: &str,
    data: &serde_json::Value,
) -> io::Result<()> {
    let json = serde_json::to_string(data).map_err(io::Error::other)?;
    write!(tcp, "event: {}\ndata: {}\n\n", event_type, json)
}

/// Write an SSE keepalive comment: `: keepalive\n\n`.
fn write_keepalive(tcp: &mut TcpStream) -> io::Result<()> {
    tcp.write_all(b": keepalive\n\n")
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

/// Lock `shared` briefly, clone `robot_state`, and map jobs to tuples.
fn take_snapshot(shared: &Arc<Mutex<MockState>>) -> SseSnapshot {
    let s = shared.lock().unwrap();
    SseSnapshot {
        state: s.robot_state.clone(),
        jobs: s
            .jobs
            .iter()
            .map(|j| {
                (
                    j.job_id.clone(),
                    j.progress_pct,
                    mock_job_state_str(&j.state).to_string(),
                )
            })
            .collect(),
    }
}

/// Build the `job_update` JSON payload for one job.
///
/// Re-locks the mutex to fetch the human-readable `name` field, which is not
/// stored in `SseSnapshot` to keep it minimal.
fn job_payload(
    shared: &Arc<Mutex<MockState>>,
    job_id: &str,
    progress_pct: u8,
    state_str: &str,
) -> serde_json::Value {
    let name = shared
        .lock()
        .unwrap()
        .jobs
        .iter()
        .find(|j| j.job_id == job_id)
        .map(|j| j.name.clone())
        .unwrap_or_default();
    serde_json::json!({
        "job_id":       job_id,
        "name":         name,
        "state":        state_str,
        "progress_pct": progress_pct,
    })
}

// ---------------------------------------------------------------------------
// Initial snapshot emission
// ---------------------------------------------------------------------------

/// Emit one `state_change` + one `job_update` per job as the initial snapshot.
fn send_initial_snapshot(
    tcp: &mut TcpStream,
    shared: &Arc<Mutex<MockState>>,
    snapshot: &SseSnapshot,
) -> io::Result<()> {
    let state_val = serde_json::to_value(&snapshot.state).map_err(io::Error::other)?;
    write_sse_event(tcp, "state_change", &state_val)?;
    for (job_id, progress_pct, state_str) in &snapshot.jobs {
        let payload = job_payload(shared, job_id, *progress_pct, state_str);
        write_sse_event(tcp, "job_update", &payload)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Poll loop — compare snapshot, emit deltas
// ---------------------------------------------------------------------------

/// Compare a fresh snapshot to `prev` and emit events for any changes.
///
/// Returns `(new_snapshot, emitted)` on success — `emitted` is `true` if at
/// least one SSE frame was written this cycle.  Returns `Err` if a write
/// fails, signalling the SSE thread to exit.
fn poll_and_emit(
    tcp: &mut TcpStream,
    shared: &Arc<Mutex<MockState>>,
    prev: &SseSnapshot,
) -> io::Result<(SseSnapshot, bool)> {
    let current = take_snapshot(shared);
    let mut emitted = false;

    // Emit state_change if the robot state changed.
    if current.state != prev.state {
        let val = serde_json::to_value(&current.state).map_err(io::Error::other)?;
        write_sse_event(tcp, "state_change", &val)?;
        emitted = true;
    }

    // Emit job_update for new or changed jobs.
    for (job_id, progress_pct, state_str) in &current.jobs {
        let changed = match prev.jobs.iter().find(|(id, _, _)| id == job_id) {
            Some((_, old_pct, old_state)) => progress_pct != old_pct || state_str != old_state,
            None => true, // new job appeared
        };
        if changed {
            let payload = job_payload(shared, job_id, *progress_pct, state_str);
            write_sse_event(tcp, "job_update", &payload)?;
            emitted = true;
        }
    }

    // Emit terminal job_update for jobs that departed (completed/cancelled).
    for (job_id, progress_pct, state_str) in &prev.jobs {
        let departed = !current.jobs.iter().any(|(id, _, _)| id == job_id);
        if departed {
            let payload = job_payload(shared, job_id, *progress_pct, state_str);
            write_sse_event(tcp, "job_update", &payload)?;
            emitted = true;
        }
    }

    Ok((current, emitted))
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Serve one SSE client.
///
/// Call this from a `std::thread::spawn` closure. The function blocks until
/// the client disconnects (detected via write failure) or the process exits.
pub fn run_sse_client(mut tcp: TcpStream, shared: Arc<Mutex<MockState>>) {
    if write_sse_headers(&mut tcp).is_err() {
        return;
    }

    let initial = take_snapshot(&shared);
    if send_initial_snapshot(&mut tcp, &shared, &initial).is_err() {
        return;
    }

    let mut prev = initial;
    let mut last_event = Instant::now();

    loop {
        std::thread::sleep(POLL_INTERVAL);

        match poll_and_emit(&mut tcp, &shared, &prev) {
            Ok((new_snapshot, emitted)) => {
                if emitted {
                    last_event = Instant::now();
                } else if last_event.elapsed() >= KEEPALIVE_INTERVAL {
                    if write_keepalive(&mut tcp).is_err() {
                        return;
                    }
                    last_event = Instant::now();
                }
                prev = new_snapshot;
            }
            Err(_) => return, // client disconnected
        }
    }
}
