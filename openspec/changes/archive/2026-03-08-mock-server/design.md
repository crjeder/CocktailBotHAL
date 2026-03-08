## Context

The existing `dev` example wires real `ApiServer` code to stub HAL implementations that return hardcoded empty responses. It proves the HTTP layer compiles and routes correctly, but it cannot simulate realistic robot behavior: dispensing jobs never progress, glass state never changes, state transitions never happen. Client developers hitting this server cannot test their state-machine handling, error flows, or SSE event consumption.

The `mock-server` example must serve the identical REST contract while running entirely on a host machine, with a live state engine that produces convincing, time-driven responses.

## Goals / Non-Goals

**Goals:**
- All API endpoints respond with valid, schema-conformant JSON per `API.yaml`
- `RobotState` transitions happen automatically over time (e.g., `Working` → `DrinkReady` after simulated dispense duration)
- Dispensing jobs track progress (`JobState::Active` with progress %) and complete
- Cleaning programs start and finish on a timer
- Glass sensor state is controllable at runtime (CLI flag, environment variable, or control endpoint)
- Config persists in RAM across requests (same pattern as `RamStorageHal` in `dev`)
- SSE endpoint emits state-change events as transitions occur
- Runs with `cargo run --example mock-server` with no extra setup

**Non-Goals:**
- Does not implement actual liquid physics or hardware simulation
- Does not persist state across process restarts
- Does not require a real network interface (binds to `127.0.0.1:8000` or configurable port)
- Does not modify `src/hal/mod.rs` or any library code
- Does not replace the `dev` example (both coexist)

## Decisions

### 1. State engine: shared `Arc<Mutex<MockState>>` driven by a background task

**Decision:** All HAL trait implementations share a single `Arc<Mutex<MockState>>` struct. A dedicated async background task ticks the state machine forward on a timer (e.g., every 100ms), advancing job progress and firing state transitions.

**Rationale:** The embassy spin executor used in `dev` has no native timer support on host. A background task with `embassy_time::Timer::after` provides time-driven transitions without requiring `tokio`. The spin executor runs tasks cooperatively; a ticker task suffices.

**Alternatives considered:**
- Spawn an OS thread with `std::thread::sleep` — works but mixes threading models with the async executor
- Use `tokio` runtime — adds a significant dependency; overkill when embassy-executor already present

### 2. Executor: reuse embassy spin executor (same as `dev` example)

**Decision:** Use `embassy-executor` with `arch-spin` (already in `Cargo.toml`) and `StaticCell<Executor>`, identical to `examples/dev/main.rs`.

**Rationale:** No new executor dependency. The spin executor is sufficient for host testing; the mock server is not latency-sensitive.

### 3. Control surface: CLI arguments + optional `/mock/control` HTTP endpoint

**Decision:** Accept CLI flags (`--glass-present`, `--glass-absent`, `--inject-error <code>`, `--dispense-duration-secs <n>`) to set initial state. Optionally expose a `POST /mock/control` endpoint (not in `API.yaml`, clearly documented as mock-only) for runtime control.

**Rationale:** CLI flags cover the most common CI scenario (start server with known state). The control endpoint enables interactive testing and scripted sequences without restarting the process.

**Alternatives considered:**
- Environment variables only — harder to discover, no runtime control
- Config file — more setup friction for a test tool

### 4. HAL split: one struct per HAL trait (same pattern as `examples/esp32/`)

**Decision:** Define separate structs (`MockControl`, `MockStatus`, `MockDispense`, etc.) that all hold a clone of `Arc<Mutex<MockState>>`. Wire them into `ApiServer` in `main.rs`.

**Rationale:** Mirrors the ESP32 example structure, keeping the pattern consistent and the code reviewable. Avoids a god-struct that implements all traits.

### 5. Networking: `embassy-net` with host TCP (same as `dev`)

**Decision:** Reuse the host networking approach from `examples/dev/main.rs` — bind a TCP listener, pass sockets to `ApiServer::run`.

**Rationale:** No new networking dependency. The mock server is a test tool, not a production service.

## Risks / Trade-offs

- **Spin executor and wall-clock timing:** The spin executor busy-loops; time-driven transitions depend on `embassy_time::Timer` which uses the stub time backend in host builds. Need to verify the spin driver actually advances time or provide a real driver. → _Mitigation:_ use `embassy-executor`'s `time-tick-hz` feature or confirm the spin executor integrates with system time on host; fall back to a separate std thread for ticking if needed.

- **Mutex contention:** The background ticker and HTTP handlers all lock `MockState`. Under load this could serialize requests. → _Mitigation:_ The mock is a test tool; throughput is not a goal. A `std::sync::Mutex` is fine.

- **SSE and the spin executor:** SSE requires a task that holds a socket open and writes events. The spin executor must cooperatively yield between the SSE writer and the ticker. → _Mitigation:_ SSE task polls a channel for events emitted by the ticker task; no blocking calls.

- **`/mock/control` endpoint is not in `API.yaml`:** Clients might accidentally depend on it. → _Mitigation:_ Document clearly in binary help text and README that it is mock-only. Prefix with `/mock/` to make it visually distinct.

## Migration Plan

No migration needed. This is a purely additive new example binary. Existing CI, `cargo test`, and `cargo check` are unaffected.

To use: `cargo run --example mock-server [-- --port 8080 --glass-present]`

## Implementation Notes (post-build)

**Ticker — std::thread, not embassy timer (Decision 1 revised):**
`embassy_time::Timer` requires a registered time driver. The `arch-spin` executor
provides one for host targets, but wiring it correctly requires using the full
embassy executor — which prevents blocking std I/O in the same thread.
Instead, the ticker runs in a `std::thread::spawn` loop with `std::thread::sleep(100ms)`,
completely independent of the async executor. This is simpler and has no scheduler
interaction issues.

**Time stubs — inline in main.rs (new):**
The dispense handler calls `embassy_time::Instant::now()` to generate job IDs.
Two `#[no_mangle]` symbols (`_embassy_time_now`, `_embassy_time_schedule_wake`)
are defined in `main.rs` — `_embassy_time_now` returns wall-clock microseconds
via `SystemTime::now()`. No embassy executor needed in the example.

**HTTP loop — `futures::executor::block_on` per request (Decision 2 revised):**
Each incoming `TcpStream` is handled with `futures::executor::block_on`, reusing
the `futures` dev-dependency already in `Cargo.toml`. No embassy executor is used
in the mock server — the async HAL methods are all lock+return with no `.await`
suspension, so `block_on` completes synchronously.

**`ApiServer::handle_request` added (new pub method):**
A new `pub async fn handle_request(&mut self, req: &HttpRequest, socket: &mut S)`
method was added to `ApiServer` in `src/server/mod.rs`. The existing
`handle_connection` now calls it after parsing the request. This lets the mock
server parse the request, intercept `/mock/` paths, and delegate everything else.

**SSE not wired (deferred):**
`GET /v1/events` is served by `SseServer`, a separate struct from `ApiServer`.
Integrating SSE into the mock server requires additional wiring; deferred to a
future change.
