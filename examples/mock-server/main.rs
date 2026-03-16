// examples/mock-server/main.rs
//
// A fully stateful mock server for testing CocktailBotHAL clients.
//
// Exposes the same OpenAPI 3.1.0 REST contract as a real robot, with:
//   - Realistic RobotState transitions (Off → SelfTest → Idle, Working, etc.)
//   - Time-driven job progress and cleaning timers
//   - Runtime glass-state and error injection via POST /mock/control
//
// Usage:
//   cargo run --example mock-server [-- [--port N] [--glass-present] [--glass-absent]
//                                        [--dispense-duration-secs N]]
//
// Clients authenticate with:
//   Bearer token:   "changeme"   (Authorization: Bearer changeme)
//   Admin password: "changeme"   (Authorization: Basic YWRtaW46Y2hhbmdlbWU=)

mod cleaning;
mod config;
mod control;
mod dispense;
mod hasher;
mod sensors;
mod sse;
mod state;
mod status;
mod storage;

use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cocktail_bot_hal::server::http::{read_http_request, write_json};
use cocktail_bot_hal::server::{ApiServer, RobotHal};

use cleaning::MockCleaning;
use config::MockConfig;
use control::MockControl;
use dispense::MockDispense;
use hasher::MockPasswordHasher;
use sensors::MockSensors;
use state::{inject_error, MockState};
use status::MockStatus;
use storage::MockStorage;

// ---------------------------------------------------------------------------
// embassy_time linker stubs
//
// The dispense handler calls embassy_time::Instant::now() to build job IDs.
// Provide the two required symbols here so the example binary links without
// the spin-executor time driver.  `_embassy_time_now` returns real wall-clock
// microseconds; `_embassy_time_schedule_wake` is a no-op (we do not use the
// timer-queue in this example).
// ---------------------------------------------------------------------------

#[no_mangle]
fn _embassy_time_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

#[no_mangle]
fn _embassy_time_schedule_wake(_at: u64, _waker: &core::task::Waker) {}

// ---------------------------------------------------------------------------
// StdTcpStream — embedded-io-async adapter for std::net::TcpStream
//
// Implements Read + Write + Unpin so that ApiServer::handle_request can use
// a regular TcpStream without embassy-net.
// ---------------------------------------------------------------------------

struct StdTcpStream(TcpStream);

#[derive(Debug)]
struct MockIoError;

impl core::fmt::Display for MockIoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "mock IO error")
    }
}

impl core::error::Error for MockIoError {}

impl embedded_io_async::Error for MockIoError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl embedded_io_async::ErrorType for StdTcpStream {
    type Error = MockIoError;
}

impl embedded_io_async::Read for StdTcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, MockIoError> {
        io::Read::read(&mut self.0, buf).map_err(|_| MockIoError)
    }
}

impl embedded_io_async::Write for StdTcpStream {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, MockIoError> {
        io::Write::write(&mut self.0, buf).map_err(|_| MockIoError)
    }

    async fn flush(&mut self) -> Result<(), MockIoError> {
        io::Write::flush(&mut self.0).map_err(|_| MockIoError)
    }
}

// ---------------------------------------------------------------------------
// /mock/control handler
// ---------------------------------------------------------------------------

/// Handle a `POST /mock/control` request.
///
/// Accepted JSON bodies:
/// - `{ "glass": "present" | "absent" }` — set glass sensor state
/// - `{ "inject_error": "<code>" }` — push an error, transition to Error state
async fn handle_mock_control(
    shared: &Arc<Mutex<MockState>>,
    body: &[u8],
    socket: &mut StdTcpStream,
) {
    #[derive(serde::Deserialize)]
    struct ControlBody {
        glass: Option<String>,
        inject_error: Option<String>,
    }

    let body: ControlBody = match serde_json::from_slice(body) {
        Ok(b) => b,
        Err(_) => {
            write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid body", "mock": true}),
            )
            .await
            .ok();
            return;
        }
    };

    if let Some(glass) = &body.glass {
        let present = match glass.as_str() {
            "present" => true,
            "absent" => false,
            _ => {
                write_json(
                    socket,
                    400,
                    &serde_json::json!({
                        "error": "glass must be 'present' or 'absent'",
                        "mock": true
                    }),
                )
                .await
                .ok();
                return;
            }
        };
        shared.lock().unwrap().glass.present = present;
    }

    if let Some(code) = &body.inject_error {
        inject_error(shared, code);
    }

    write_json(socket, 200, &serde_json::json!({"ok": true, "mock": true}))
        .await
        .ok();
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

struct Args {
    port: u16,
    glass_present: bool,
    dispense_duration_secs: u32,
    liquids_file: Option<String>,
}

fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let mut port: u16 = 8000;
    let mut glass_present = false;
    let mut dispense_duration_secs: u32 = 10;
    let mut liquids_file: Option<String> = None;
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "--port" if i + 1 < argv.len() => {
                port = argv[i + 1].parse().unwrap_or(8000);
                i += 2;
            }
            "--glass-present" => {
                glass_present = true;
                i += 1;
            }
            "--glass-absent" => {
                glass_present = false;
                i += 1;
            }
            "--dispense-duration-secs" if i + 1 < argv.len() => {
                dispense_duration_secs = argv[i + 1].parse().unwrap_or(10);
                i += 2;
            }
            "--liquids" if i + 1 < argv.len() => {
                liquids_file = Some(argv[i + 1].clone());
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    Args {
        port,
        glass_present,
        dispense_duration_secs,
        liquids_file,
    }
}

fn load_liquids_file(path: &str) -> Vec<cocktail_bot_hal::hal::LiquidConfig> {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read liquids file {path:?}: {e}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("invalid JSON in liquids file {path:?}: {e}"))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args = parse_args();

    // Load optional liquids override from file.
    let initial_liquids = args.liquids_file.as_deref().map(load_liquids_file);

    // Shared state — all HAL stubs and the ticker share this.
    let shared = MockState::new(
        args.glass_present,
        args.dispense_duration_secs,
        initial_liquids,
    )
    .into_shared();

    // Ticker thread — advances the state machine every 100 ms.
    {
        let shared = shared.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));
            shared.lock().unwrap().tick();
        });
    }

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&addr).expect("failed to bind TCP listener");
    println!("Mock server listening on http://{}", addr);
    println!("  Bearer token:   changeme");
    println!("  Admin password: changeme");
    println!("  Glass present:  {}", args.glass_present);
    println!("  Dispense time:  {}s", args.dispense_duration_secs);

    // Build the ApiServer once; reuse it across all connections.
    let mut server = ApiServer {
        hal: RobotHal {
            control: MockControl {
                state: shared.clone(),
            },
            status: MockStatus {
                state: shared.clone(),
            },
            config: MockConfig {
                state: shared.clone(),
            },
            storage: MockStorage {
                state: shared.clone(),
            },
            sensors: MockSensors {
                state: shared.clone(),
            },
            dispense: MockDispense {
                state: shared.clone(),
            },
            cleaning: MockCleaning {
                state: shared.clone(),
            },
            hasher: MockPasswordHasher,
        },
    };

    // Sequential connection loop — one request at a time.
    // Sufficient for client integration testing.
    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut stream = StdTcpStream(stream);

        // Peek at the request to decide routing.
        let request = match futures::executor::block_on(read_http_request(&mut stream)) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if request.path.starts_with("/mock/") {
            // Mock control surface — not part of the public API.
            futures::executor::block_on(handle_mock_control(&shared, &request.body, &mut stream));
        } else if request.method == "GET" && request.path == "/v1/events" {
            // SSE stream — spawn a background thread per client so the main
            // accept loop stays unblocked.  `try_clone` gives the thread its
            // own file descriptor; the original `StdTcpStream` is dropped.
            if let Ok(tcp2) = stream.0.try_clone() {
                let shared2 = shared.clone();
                std::thread::spawn(move || sse::run_sse_client(tcp2, shared2));
            }
        } else {
            futures::executor::block_on(server.handle_request(&request, &mut stream));
        }
    }
}
