## Context

The `mock-server` example (`examples/mock-server/`) is a fully stateful HTTP server built on `std::net::TcpListener` with `futures::executor::block_on` per request and a `std::thread` ticker for state transitions. It shares state via `Arc<Mutex<MockState>>`.

The real SSE implementation (`src/server/sse.rs`) is a separate `SseServer` struct that uses `embassy_net::TcpSocket` and `embassy_time::Timer` — both incompatible with the mock server's `std` environment.

`GET /v1/events` currently falls through to `ApiServer::handle_request`'s 404 catch-all. The mock server's main loop intercepts `/mock/*` routes before delegating; the same interception point is used here.

`RobotState` has `#[derive(PartialEq, Clone, Serialize)]`. It contains no `f32` fields, so `PartialEq` comparison is correct and safe for change detection.

`MockJob` carries `name`, `progress_pct`, and `MockJobState` — enough to reconstruct a `JobStatus` for `job_update` events.

Per `API.yaml`, `GET /v1/events` has no `security:` requirement — no auth needed.

## Goals / Non-Goals

**Goals:**
- Serve `GET /v1/events` as a real SSE stream from the mock server
- Emit `state_change` events whenever `RobotState` changes
- Emit `job_update` events whenever a job's `progress_pct` or `state` changes, and a terminal event when a job is completed or cancelled
- Send an initial snapshot on connection (current `RobotState` + all known jobs)
- Send `: keepalive` comments every 30 s to prevent proxy/client timeouts
- Support multiple concurrent SSE clients
- Zero changes to `src/` (library) or `API.yaml` or `Cargo.toml`

**Non-Goals:**
- Reuse `src/server/sse.rs` (embassy-specific; incompatible with std TCP)
- Auth on `GET /v1/events` (not required by the API spec)
- Persistent connections surviving mock server restart
- Glass sensor SSE events (not emitted by real server yet either)

## Decisions

### 1. Thread-per-SSE-client (not blocking the main accept loop)

**Decision:** Intercept `GET /v1/events` in the main loop. Call `TcpStream::try_clone()` to give the SSE thread its own file descriptor. Spawn a `std::thread` per client. The main loop continues accepting the next connection immediately.

**Rationale:** The main loop is sequential. SSE clients hold connections open for minutes. Without a thread, the first SSE client would freeze all other API requests. `try_clone()` is stdlib — no new deps. This is consistent with the ticker thread already in the mock server.

**Alternatives considered:**
- Convert main loop to thread-per-connection: requires `Arc<Mutex<ApiServer<...>>>` (hard — 8 generic params, not `Send`-safe without restructuring)
- Separate SSE port (like the real server's port 9000): adds friction for clients, which must connect to two ports; unnecessary given `try_clone()` works fine

### 2. Polling `MockState` at 500 ms intervals (matching `SseServer`)

**Decision:** The SSE thread sleeps 500 ms between polls, reads `MockState` under a brief lock, compares to the previous snapshot, and emits events only on change.

**Rationale:** The ticker fires every 100 ms; polling at 500 ms is sufficient for realistic responsiveness and matches the real `SseServer`'s cadence. Keeping the lock duration short (clone and release) avoids contention with the ticker and HTTP handlers.

**Alternatives considered:**
- Condition variable / channel: ticker would need to notify SSE threads on every state change. Adds complexity; not worth it for a test tool where 500 ms latency is imperceptible.

### 3. Snapshot struct for change detection

**Decision:** Maintain a `SseSnapshot` struct in `sse.rs`:
```rust
struct SseSnapshot {
    state: RobotState,
    jobs: Vec<(String, u8, String)>,  // (job_id, progress_pct, state_str)
}
```
`RobotState::PartialEq` detects machine-level changes. For jobs, compare `progress_pct` and a serialised state string (since `MockJobState` has no `PartialEq`).

**Rationale:** Minimal allocation per poll. `RobotState::PartialEq` is safe (no `f32` in the variants used during active operation). Job state is compared by string to avoid adding `PartialEq` to `MockJobState`.

### 4. New file `examples/mock-server/sse.rs`; minimal change to `main.rs`

**Decision:** All SSE logic lives in a new `sse.rs` module. `main.rs` gains: one `mod sse;` line, a 3-line route intercept (path check + `try_clone` + `thread::spawn`). No other files touched.

**Rationale:** Keeps the routing intercept minimal and keeps SSE logic self-contained and independently reviewable.

## Risks / Trade-offs

- **`try_clone()` on Windows**: `TcpStream::try_clone()` is well-supported on Windows (duplicates the underlying socket handle). No known issues with the project's Win32 dev environment.
- **Thread accumulation from long-lived clients**: Each SSE client holds a thread indefinitely. For a test tool with a handful of clients this is fine; it would be a problem under load. → Acceptable: mock server is single-developer tooling.
- **Ticker lock contention**: SSE threads and the ticker both lock `MockState`. At 500 ms poll intervals and a 100 ms ticker with sub-microsecond lock hold times, contention is negligible.
- **`RobotState` equality and timeout countdown fields**: `DrinkReady` and `WaitingForGlass` carry `timeout_remaining_secs`. The ticker does not currently decrement these in `MockState` (they are `None`). If future work adds countdown ticking, every tick would emit a `state_change` event. → Not a problem now; document as a known future consideration.

## Open Questions

- **`job_update` for queued jobs**: The mock server currently processes one job at a time (no queue). Should `job_update` events be emitted for jobs in `Queued` state if a queue is ever implemented? Leave open; current implementation only has at most one active job.
- **`RobotState` vs `JobStatus` progress redundancy**: Both `state_change` (via `RobotState::Working { progress_pct }`) and `job_update` carry progress for the active job. This is intentional — clients can choose their preferred event — but worth documenting in the mock README.
