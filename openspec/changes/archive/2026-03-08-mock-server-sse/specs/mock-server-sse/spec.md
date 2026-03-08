## ADDED Requirements

### Requirement: SSE endpoint streams state-change events
The mock server SHALL serve `GET /v1/events` as a long-lived SSE stream (Content-Type: `text/event-stream`) with no authentication required. Upon connection it SHALL immediately emit a `state_change` event carrying the current `RobotState` as a JSON payload (tagged-union format, matching the `GET /v1/status` schema). Subsequent `state_change` events SHALL be emitted within 1 second of any `RobotState` transition detected by the 500 ms poll cycle.

#### Scenario: Initial state_change event on connect
- **WHEN** a client connects to `GET /v1/events`
- **THEN** the server responds with HTTP 200, `Content-Type: text/event-stream`, and immediately sends an `event: state_change` frame with the current `RobotState` JSON

#### Scenario: state_change emitted on robot state transition
- **WHEN** the mock state machine transitions (e.g., `Idle` → `Working`)
- **THEN** connected SSE clients receive an `event: state_change` frame within 1 second carrying the new `RobotState`

#### Scenario: No duplicate state_change events
- **WHEN** the robot state does not change between two consecutive polls
- **THEN** no `state_change` event is emitted for that poll interval

---

### Requirement: SSE endpoint streams job-update events
The mock server SHALL emit a `job_update` event on the SSE stream whenever a dispensing job changes its `progress_pct` or lifecycle state (`active` → `done` / `cancelled` / `error`). The payload SHALL include `job_id`, `name`, `state`, and `progress_pct` fields. Upon connection it SHALL emit a `job_update` event for every currently known job (active or recently completed) as part of the initial snapshot.

#### Scenario: Initial job_update events on connect
- **WHEN** a client connects while a dispense job is in progress
- **THEN** the server emits a `job_update` frame for the active job as part of the initial snapshot, including its current `progress_pct`

#### Scenario: job_update emitted as job progresses
- **WHEN** a dispense job's `progress_pct` increases
- **THEN** connected SSE clients receive a `job_update` event with the updated `progress_pct` within 1 second

#### Scenario: Terminal job_update on job completion
- **WHEN** a dispense job transitions to `done` or `cancelled`
- **THEN** connected SSE clients receive a `job_update` event with the terminal state (`state: "done"` or `state: "cancelled"`) and `progress_pct: 100` or final value respectively

---

### Requirement: SSE keepalive prevents proxy timeouts
The mock server SHALL emit an SSE comment line (`: keepalive`) on each connected SSE stream if no event has been sent in the preceding 30 seconds.

#### Scenario: Keepalive sent after 30 s of silence
- **WHEN** the robot remains in a stable state (no transitions) for more than 30 seconds
- **THEN** connected SSE clients receive a `: keepalive` comment at least once per 30 second window

---

### Requirement: Multiple concurrent SSE clients are supported
The mock server SHALL support at least two simultaneous SSE client connections without blocking other API requests.

#### Scenario: SSE client does not block REST requests
- **WHEN** a client is connected to `GET /v1/events` and another client sends `GET /v1/status`
- **THEN** the `GET /v1/status` response is returned normally without waiting for the SSE connection to close

#### Scenario: Two SSE clients receive the same events
- **WHEN** two clients are simultaneously connected to `GET /v1/events` and the robot state changes
- **THEN** both clients receive the `state_change` event

---

### Requirement: SSE stream terminates cleanly on client disconnect
The mock server SHALL detect when an SSE client disconnects (write error on the TCP stream) and stop the background SSE thread for that client, releasing all associated resources.

#### Scenario: SSE thread exits on write failure
- **WHEN** an SSE client closes the connection
- **THEN** the next write attempt fails and the SSE background thread exits without affecting other connections or the main server loop
