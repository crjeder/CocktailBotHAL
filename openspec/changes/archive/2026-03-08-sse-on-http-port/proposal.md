## Why

The SSE stream is currently served on a separate port (9000), requiring clients
to manage two TCP connections and making network configuration (firewall rules,
mDNS) more complex. Consolidating the SSE endpoint onto the same port as the
REST API (/v1/events on port 80) simplifies client setup and reduces resource
usage on the constrained ESP32.

## What Changes

- Remove the standalone `SseServer` task and its dedicated TCP listener on port 9000
- Add a `GET /v1/events` route to `ApiServer` that serves the SSE stream
- The SSE stream retains all existing behaviour: initial snapshot on connect,
  500 ms poll interval, 30 s keepalive comments, no authentication required
- `examples/dev/main.rs` no longer spawns a separate SSE task
- API.yaml `/events` path description updated to reflect the unified port

## Capabilities

### New Capabilities

_(none — this is a relocation, not a new capability)_

### Modified Capabilities

- `sse-wiring`: Port changes from 9000 to 80; SSE is now a route on `ApiServer`
  rather than a separate `SseServer` task. No-auth requirement and all event
  semantics remain unchanged.

## Impact

- `src/server/mod.rs`: `ApiServer` gains a `/events` route; `SseServer` struct
  removed (or folded into the server module)
- `examples/dev/main.rs`: SSE task spawn removed; no separate port
- `examples/esp32/main.rs`: same cleanup as dev example
- `API.yaml`: `/events` path description updated
- `openspec/specs/sse-wiring/spec.md`: port/task requirements updated
