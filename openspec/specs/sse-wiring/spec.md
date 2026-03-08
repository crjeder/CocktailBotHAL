## ADDED Requirements

### Requirement: SseServer runs as embassy task
The `SseServer` struct and its dedicated embassy task SHALL be removed.
The SSE stream SHALL instead be served by `ApiServer` as a long-lived
`GET /v1/events` route handler. The route handler SHALL be dispatched from
`ApiServer::handle_request` exactly like every other route. No separate TCP
listener or executor task is needed for SSE.

#### Scenario: SSE served by ApiServer on port 80
- **WHEN** a client sends `GET /v1/events` on port 80
- **THEN** `ApiServer` responds with SSE headers and begins streaming events
  without spawning a separate task

#### Scenario: No separate SSE task in examples
- **WHEN** `examples/dev/main.rs` is inspected
- **THEN** there is no spawned `SseServer` task and no `TcpSocket` accepting
  on port 9000

### Requirement: Single display client
The `ApiServer` accept loop SHALL serve one connection at a time. While an SSE
client holds `GET /v1/events` open, subsequent TCP connections on port 80 queue
in the network stack's accept backlog and are served after the SSE client
disconnects.

#### Scenario: Second connection is held until first disconnects
- **WHEN** one SSE client is connected on port 80 and a second client connects
- **THEN** the second connection is accepted by the network stack but not
  processed until the first client disconnects

#### Scenario: Reconnection after drop
- **WHEN** the SSE client disconnects
- **THEN** the `ApiServer` accept loop accepts the next connection immediately

### Requirement: Initial snapshot on connect
Upon accepting a connection the `SseServer` SHALL send the current `RobotState`
(as the full data-carrying enum payload) as a `state_change` event and all
active/queued `JobStatus` entries as `job_update` events before entering the
polling loop.

#### Scenario: Display receives full state on connect
- **WHEN** the display connects to port 9000
- **THEN** the first event received is `state_change` with the complete `RobotState`
  payload, including `job_id`, `reason`, and `timeout_remaining_secs` where applicable

#### Scenario: Display receives active jobs on connect
- **WHEN** a job is active at connect time
- **THEN** a `job_update` event for that job is sent immediately after the `state_change` event

### Requirement: state_change event carries full RobotState payload
The `state_change` SSE event SHALL serialize the complete `RobotState` value,
including all payload fields of the active variant. The JSON SHALL use serde's
`#[serde(tag = "state", rename_all = "snake_case")]` tagged representation so
that the `"state"` key identifies the variant and additional fields appear at
the same level.

Example payloads:
```json
{"state": "idle"}
{"state": "waiting_for_glass", "job_id": "mojito-010042", "reason": {"type": "no_glass"}, "timeout_remaining_secs": 45}
{"state": "working", "job_id": "mojito-010042", "progress_pct": 62}
{"state": "drink_ready", "job_id": "mojito-010042", "timeout_remaining_secs": 287}
{"state": "error", "code": "GLASS_REMOVED", "message": "Glass was removed during dispensing", "job_id": "mojito-010042", "recoverable": true, "recovery": "put_glass_back", "timeout_remaining_secs": 51}
```

#### Scenario: Idle state_change is minimal
- **WHEN** robot is `Idle` and a `state_change` event is emitted
- **THEN** the payload is `{"state": "idle"}` with no additional fields

#### Scenario: WaitingForGlass state_change includes timeout
- **WHEN** robot enters `WaitingForGlass` and a `state_change` event is emitted
- **THEN** the payload includes `job_id`, `reason`, and `timeout_remaining_secs`

#### Scenario: timeout_remaining_secs decrements each second
- **WHEN** the robot is in `WaitingForGlass` with `glass_wait_timeout_secs = 60`
- **THEN** successive `state_change` events show `timeout_remaining_secs` decreasing by 1 per second

### Requirement: glass_size_warning SSE event
When a glass is detected and `glass_typing` is `true` but the glass is larger
than required, the `SseServer` SHALL emit a `glass_size_warning` event before
transitioning to `Working`.

Payload fields: `job_id: String`, `detected_volume: f32`, `required_volume: f32`.

#### Scenario: Oversized glass emits warning event
- **WHEN** detected glass volume exceeds required volume and `glass_typing` is `true`
- **THEN** a `glass_size_warning` event is emitted with `detected_volume` and `required_volume`

#### Scenario: Correct-size glass emits no warning
- **WHEN** detected glass volume equals required volume (within tolerance)
- **THEN** no `glass_size_warning` event is emitted

### Requirement: SSE job_update uses name field
The `job_update` SSE event payload SHALL use the field key `name` (not `client_job_id`).

#### Scenario: job_update payload contains name
- **WHEN** a `job_update` event is emitted
- **THEN** the JSON payload contains the key `"name"` with the human-readable job label

### Requirement: SSE requires no authentication
The `GET /v1/events` route on port 80 SHALL accept connections without an
`Authorization` header. No bearer token or admin password validation SHALL be
performed for this route.

#### Scenario: Unauthenticated client receives stream
- **WHEN** a client sends `GET /v1/events` without an `Authorization` header
- **THEN** the server responds with `HTTP/1.1 200 OK` and begins streaming events

#### Scenario: GET /v1/events is listed in NO_AUTH_ROUTES
- **WHEN** `src/server/mod.rs` is inspected
- **THEN** `("GET", "/v1/events")` appears in the constant that exempts routes
  from authentication

### Requirement: SSE accessible during Provisioning
The `GET /v1/events` route SHALL remain accessible even when the robot is in
`Provisioning` state, so that display clients can observe state transitions
during initial setup.

#### Scenario: Events stream while provisioning
- **WHEN** the robot is in `Provisioning` state and a client sends `GET /v1/events`
- **THEN** the server responds with `HTTP/1.1 200 OK` and begins streaming events
  (not a 503 Service Unavailable)

#### Scenario: GET /v1/events is listed in PROVISIONING_ALLOWED
- **WHEN** `src/server/mod.rs` is inspected
- **THEN** `("GET", "/v1/events")` appears in `PROVISIONING_ALLOWED`

### Requirement: Terminal job_update on job departure
When a job present in the previous poll snapshot is absent from the current
snapshot, the `SseServer` SHALL emit one final `job_update` event with the
last-known `job_id`, `name`, `state`, and `progress_pct` before discarding
the snapshot entry.

#### Scenario: Job completes and is removed by HAL
- **WHEN** `list_jobs()` returns a job in poll N and does not include that job in poll N+1
- **THEN** a `job_update` event is emitted for the departed job using the values from poll N

#### Scenario: Job is cancelled and removed
- **WHEN** a job transitions to `cancelled` and is subsequently removed from `list_jobs()`
- **THEN** the client receives a final `job_update` with `"state": "cancelled"`

#### Scenario: Job created and removed within one poll interval
- **WHEN** a job is created and finishes between two consecutive polls so it never appears in a snapshot
- **THEN** no `job_update` is emitted for that job (it was never in `prev`)

#### Scenario: Departed job does not trigger keepalive reset
- **WHEN** a terminal `job_update` is emitted for a departed job
- **THEN** the keepalive timer is reset as it would be for any other emitted event
