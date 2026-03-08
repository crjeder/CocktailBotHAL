## 1. SSE Module

- [x] 1.1 Create `examples/mock-server/sse.rs` with a `SseSnapshot` struct holding `state: RobotState` and `jobs: Vec<(String, u8, String)>` (job_id, progress_pct, state_str)
- [x] 1.2 Implement `write_sse_headers(tcp: &mut TcpStream)` — writes `HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n`
- [x] 1.3 Implement `write_sse_event(tcp: &mut TcpStream, event_type: &str, data: &serde_json::Value) -> io::Result<()>` — writes `event: <type>\ndata: <json>\n\n`
- [x] 1.4 Implement `write_keepalive(tcp: &mut TcpStream) -> io::Result<()>` — writes `: keepalive\n\n`
- [x] 1.5 Implement `take_snapshot(shared: &Arc<Mutex<MockState>>) -> SseSnapshot` — locks briefly, clones `robot_state` and maps `jobs` to `(job_id, progress_pct, state_str)` tuples
- [x] 1.6 Implement `send_initial_snapshot(tcp, snapshot)` — emits one `state_change` + one `job_update` per job in the snapshot
- [x] 1.7 Implement `poll_and_emit(tcp, shared, prev)` — compares new snapshot to prev, emits `state_change` if `RobotState` differs, emits `job_update` per changed or newly-appeared/departed job; returns new snapshot or `Err` on write failure
- [x] 1.8 Implement `pub fn run_sse_client(tcp: TcpStream, shared: Arc<Mutex<MockState>>)` — writes headers, sends initial snapshot, loops: sleep 500 ms, call `poll_and_emit`; tracks last-event time for keepalive; exits on any write error

## 2. Route Intercept in main.rs

- [x] 2.1 Add `mod sse;` at the top of `examples/mock-server/main.rs`
- [x] 2.2 In the main accept loop, before the `ApiServer::handle_request` branch, add: `if request.method == "GET" && request.path == "/v1/events"` — call `stream.0.try_clone()`, spawn `std::thread::spawn(move || sse::run_sse_client(tcp2, shared.clone()))`, then `return` from the `block_on` closure (do NOT pass to `ApiServer`)
- [x] 2.3 Verify the existing `/mock/` intercept remains first so mock control still works

## 3. Verification

- [x] 3.1 `cargo check --example mock-server` — zero errors and zero warnings
- [x] 3.2 `cargo run --example mock-server` — connect with `curl -N http://127.0.0.1:8000/v1/events` and confirm: initial `state_change` event received, then `state_change` events arrive as the boot sequence runs (`Off` → `SelfTest` → `Idle`)
- [x] 3.3 Issue `POST /v1/dispense/jobs` via curl while SSE is streaming — confirm `job_update` events appear with increasing `progress_pct`, then a terminal `job_update` with `state: "done"`
- [x] 3.4 Open a second `curl -N` SSE connection while the first is active — confirm `GET /v1/status` still responds normally (SSE does not block REST)
- [x] 3.5 Kill one SSE curl client — confirm the server continues serving normally (no crash, no hung thread visible in logs)
- [x] 3.6 `cargo fmt --example mock-server` — no changes needed (code was written formatted)
