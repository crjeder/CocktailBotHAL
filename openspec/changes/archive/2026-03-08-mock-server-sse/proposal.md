## Why

The `mock-server` example currently returns 404 for `GET /v1/events` — clients that rely on SSE for real-time state updates cannot be tested against the mock. Adding SSE closes the last gap between the mock server's REST API coverage and the real robot's.

## What Changes

- Add `GET /v1/events` SSE streaming to `examples/mock-server/`
- Each connected client receives an initial state snapshot, then `state_change` and `job_update` events as the mock state machine advances
- SSE clients are served on a background `std::thread` per connection — consistent with the ticker thread already in the mock server
- No library code changes (`src/` is untouched); no new dependencies

## Capabilities

### New Capabilities
- `mock-server-sse`: SSE event streaming on `GET /v1/events` for the mock server — emits `state_change` and `job_update` events driven by the existing `MockState` ticker

### Modified Capabilities
<!-- No existing spec-level requirements change. -->

## Impact

- New file: `examples/mock-server/sse.rs` — standalone SSE client handler using `std::net::TcpStream`
- Modified: `examples/mock-server/main.rs` — intercepts `GET /v1/events` before `ApiServer::handle_request`, spawns SSE thread
- No changes to `src/`, `API.yaml`, or `Cargo.toml`
- `src/server/sse.rs` (embassy/ESP32 SSE path) is unaffected
