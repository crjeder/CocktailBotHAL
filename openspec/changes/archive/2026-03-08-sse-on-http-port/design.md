## Context

`SseServer` (src/server/sse.rs) currently accepts on a dedicated TCP port (9000),
reads and discards the incoming HTTP request, then streams events to the client.
`ApiServer` (src/server/mod.rs) handles all REST routes on port 80. Both run as
independent embassy tasks and share only the network stack.

The key constraint: on ESP32 with `no_std + alloc`, every open TCP socket
consumes a fixed slice of the stack's socket pool. Two listening sockets where
one suffices wastes a slot.

## Goals / Non-Goals

**Goals:**

- `GET /v1/events` served by `ApiServer` on port 80
- All existing SSE event semantics preserved (initial snapshot, 500 ms poll,
  30 s keepalive, no authentication required)
- `SseServer` struct and its dedicated task removed from `examples/dev/main.rs`
  and `examples/esp32/main.rs`
- One fewer TCP socket slot consumed at runtime

**Non-Goals:**

- Multiple concurrent SSE clients (still one at a time — but held by the
  single accept loop, not a separate port)
- Push-based events (polling loop is retained)
- Streaming chunks for other routes

## Decisions

### Decision 1: SSE as a long-lived route handler, not a separate task

**Chosen:** Add `("GET", "/v1/events")` to the `match` block in
`ApiServer::handle_request`. The handler calls `sse::handle_sse_stream`, which
writes SSE headers then runs the poll loop until the socket write fails (client
disconnects). The `run` accept loop then moves on to the next connection.

**Alternative considered:** Keep `SseServer` as a struct but bind it to port 80
alongside the API. Rejected — it would require either merging accept loops (complex)
or a second embassy task sharing port 80 (not possible; `accept` owns the socket).

**Why this works:** `handle_request` already accepts `S: Write + Unpin` and does
not `Read` from the socket after parsing. The SSE handler just needs `Write + Unpin`.
The request parsing (path/method) happens before dispatch, so routing is free.

### Decision 2: Bypass authentication for `/v1/events`

**Chosen:** Add `GET /v1/events` to a `NO_AUTH_ROUTES` constant (similar pattern
to `ADMIN_ROUTES`). The auth gate checks this list and skips token verification.

**Alternative:** Require a Bearer token. Rejected — the existing `sse-wiring` spec
explicitly requires unauthenticated access, and the display client is on the same
LAN.

### Decision 3: SSE handler signature

`handle_sse_stream` takes `status: &Stat, dispense: &Disp, socket: &mut S`.
This mirrors the pattern used by all other handlers (each takes only its needed
HAL references). The call site in `handle_request` passes `&self.hal.status` and
`&self.hal.dispense`.

### Decision 4: Keep `sse.rs` module, remove `SseServer` struct

The helper functions (`write_sse_event`, `emit_state_event`, `emit_job_event`,
`write_keepalive`, `write_sse_headers`, `capture_snapshot`, `job_changed`) are
all reusable and stay in `src/server/sse.rs`. Only `SseServer` and its `run`
method are deleted; `handle_client` is renamed `handle_sse_stream` and made
`pub(super)` for dispatch from `mod.rs`.

## Risks / Trade-offs

- **One connection at a time on port 80**: While an SSE client holds the stream
  open, the `accept` loop is blocked serving that socket. REST requests queue in
  the TCP stack's backlog and are served as soon as the SSE client disconnects.
  This is acceptable for a single-display robot; the display is a separate client
  from the app making REST calls. → Mitigation: document that the server is
  single-threaded; consider a timeout if backlog blocking becomes a problem in practice.

- **No provisioning gate for `/v1/events`**: The route must be accessible during
  `Provisioning` state (the display needs state updates during setup). → Mitigation:
  add `/v1/events` to `PROVISIONING_ALLOWED` alongside status and config GET.

## Open Questions

- Should `/v1/events` also be listed in `PROVISIONING_ALLOWED`, or should it
  return 503 while provisioning? (Assumption: yes, add it — display needs state.)
