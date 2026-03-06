## ADDED Requirements

### Requirement: SseServer runs as embassy task
The `SseServer` SHALL be spawned as a dedicated `#[embassy_executor::task]`
alongside `ApiServer` in `src/main.rs`. Both tasks SHALL run concurrently on the
same embassy executor. The SSE task SHALL hold `&'static` references to
`StatusHal` and `DispenseHal` instances stored in `StaticCell` or equivalent
static storage.

#### Scenario: SSE task starts with ApiServer
- **WHEN** the binary starts
- **THEN** both the `ApiServer` task (port 80) and the `SseServer` task (port 9000) are spawned and accepting connections

#### Scenario: SSE task holds static HAL refs
- **WHEN** `src/main.rs` is inspected
- **THEN** `SseServer` is constructed with `&'static Stat` and `&'static Disp` references, not scoped borrows

### Requirement: Single display client
The `SseServer` SHALL accept one SSE client at a time on port 9000. When a client
disconnects, the server SHALL immediately begin accepting the next connection.

#### Scenario: Second connection is held until first disconnects
- **WHEN** one client is connected and a second client attempts to connect on port 9000
- **THEN** the second connection is accepted only after the first client disconnects

#### Scenario: Reconnection after drop
- **WHEN** the SSE client disconnects (e.g., display reboots)
- **THEN** the server re-enters the accept loop and the display can reconnect immediately

### Requirement: Initial snapshot on connect
Upon accepting a connection the `SseServer` SHALL send the current `RobotState`
as a `state_change` event and all active/queued `JobStatus` entries as `job_update`
events before entering the polling loop.

#### Scenario: Display receives state on connect
- **WHEN** the display connects to port 9000
- **THEN** the first event received is `state_change` with the current robot state

#### Scenario: Display receives active jobs on connect
- **WHEN** a job is running at connect time
- **THEN** a `job_update` event for that job is sent immediately after the `state_change` event

### Requirement: SSE job_update uses name field
The `job_update` SSE event payload SHALL use the field key `name` (not `client_job_id`).

#### Scenario: job_update payload contains name
- **WHEN** a `job_update` event is emitted
- **THEN** the JSON payload contains the key `"name"` with the human-readable job label

### Requirement: SSE requires no authentication
The SSE endpoint on port 9000 SHALL accept connections without an
`Authorization` header. No bearer token validation is performed.

#### Scenario: Unauthenticated client receives stream
- **WHEN** a client connects to port 9000 without an Authorization header
- **THEN** the server responds with `HTTP/1.1 200 OK` and begins streaming events

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
