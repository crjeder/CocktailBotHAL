## Requirements

### Requirement: Mock server runs on host without hardware
The mock-server example SHALL start an HTTP server on the host machine (no ESP32) that responds to all endpoints defined in `API.yaml` with schema-conformant JSON.

#### Scenario: Server starts with default configuration
- **WHEN** `cargo run --example mock-server` is executed
- **THEN** the server binds to `127.0.0.1:8000`, logs the listen address, and begins serving requests

#### Scenario: Server starts with custom port
- **WHEN** the server is started with `-- --port 9090`
- **THEN** the server binds to `127.0.0.1:9090`

---

### Requirement: Stateful RobotState transitions
The mock server SHALL maintain a live `RobotState` that transitions automatically in response to API calls and elapsed time, following the same state machine contract as the real robot.

#### Scenario: Boot sequence on startup
- **WHEN** the server starts
- **THEN** `RobotState` begins as `Booting`, transitions to `SelfTest`, then to `Idle` automatically within a few seconds

#### Scenario: Dispense job drives state to Working then DrinkReady
- **WHEN** a valid `POST /v1/dispense` request is received while in `Idle` or `Prepared`
- **THEN** `RobotState` transitions to `Working`, progresses the job to completion, then transitions to `DrinkReady`

#### Scenario: Cleaning drives state to Cleaning then Idle
- **WHEN** a valid `POST /v1/cleaning/start` request is received while in `Idle`
- **THEN** `RobotState` transitions to `Cleaning` and returns to `Idle` after the simulated cleaning duration

#### Scenario: State is consistent between status poll and SSE
- **WHEN** a client polls `GET /v1/status` and simultaneously listens to SSE events
- **THEN** both reflect the same current `RobotState` without contradiction

---

### Requirement: Dispensing jobs have realistic progress
The mock server SHALL track dispensing jobs with time-driven progress, matching `JobStatus` schema fields (`job_id`, `state`, `progress_percent`, `items`).

#### Scenario: Job progresses from 0 to 100 percent
- **WHEN** a dispense job is created
- **THEN** subsequent `GET /v1/dispense/{job_id}` responses show `progress_percent` increasing from 0 to 100 over the simulated duration

#### Scenario: Completed job returns JobState Done
- **WHEN** a dispense job reaches 100% progress
- **THEN** `GET /v1/dispense/{job_id}` returns `state: "done"`

#### Scenario: Cancelled job returns JobState Cancelled
- **WHEN** `DELETE /v1/dispense/{job_id}` is called while a job is active
- **THEN** `GET /v1/dispense/{job_id}` returns `state: "cancelled"` and `RobotState` returns to `Idle`

---

### Requirement: Glass sensor state is runtime-controllable
The mock server SHALL allow the glass sensor state (`GlassSensorState`) to be set at startup and changed at runtime, so clients can test glass-detection flows.

#### Scenario: Glass present on startup via CLI flag
- **WHEN** the server is started with `-- --glass-present`
- **THEN** `GET /v1/sensors/glass` returns `{ "state": "present" }`

#### Scenario: Glass absent by default
- **WHEN** the server is started without a glass flag
- **THEN** `GET /v1/sensors/glass` returns `{ "state": "absent" }`

#### Scenario: Glass state changed at runtime via control endpoint
- **WHEN** `POST /mock/control` is called with `{ "glass": "present" }`
- **THEN** `GET /v1/sensors/glass` immediately reflects `{ "state": "present" }`

---

### Requirement: Error injection via control surface
The mock server SHALL support injecting robot errors at runtime to enable client error-handling tests.

#### Scenario: Error injected drives state to Error
- **WHEN** `POST /mock/control` is called with `{ "inject_error": "<error_code>" }`
- **THEN** `RobotState` transitions to `Error` and `GET /v1/status` includes the injected error in its `errors` array

#### Scenario: Error cleared via reset-errors
- **WHEN** `POST /v1/control/reset-errors` is called after an injected error
- **THEN** the error is removed and `RobotState` returns to `Idle`

---

### Requirement: Config persists in RAM across requests
The mock server SHALL store `RobotConfig` in memory and reflect updates in subsequent reads within the same process lifetime.

#### Scenario: Config update is reflected in GET
- **WHEN** `PUT /v1/config` is called with updated admin credentials or liquid config
- **THEN** `GET /v1/config` returns the updated values

---

### Requirement: SSE emits events on state transitions
The mock server SHALL emit Server-Sent Events on `GET /v1/events` whenever `RobotState` changes, using the same event format as the real robot.

#### Scenario: State change triggers SSE event
- **WHEN** `RobotState` transitions (e.g., `Idle` → `Working`)
- **THEN** all connected SSE clients receive a `state` event with the new `RobotState` payload

---

### Requirement: Mock control endpoint is clearly marked as non-production
The `/mock/control` endpoint SHALL not appear in `API.yaml` and SHALL return a header or response field identifying it as a test-only interface.

#### Scenario: Control endpoint not in OpenAPI spec
- **WHEN** `API.yaml` is inspected
- **THEN** no path starting with `/mock` is defined

#### Scenario: Control endpoint identifies itself
- **WHEN** `POST /mock/control` returns a response
- **THEN** the response body includes `"mock": true`
