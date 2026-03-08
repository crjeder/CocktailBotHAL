## MODIFIED Requirements

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
