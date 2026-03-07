## MODIFIED Requirements

### Requirement: job-create signature
`DispenseHal::create_job` SHALL accept exactly four parameters:
1. `job_id: String` — server-generated unique identifier (see job_id generation requirement)
2. `name: String` — human-readable, non-unique label supplied by the client
3. `items: Vec<DispenseItem>` — pre-computed per-ingredient dispenser volumes;
   each `DispenseItem` has `liquid_id: String` and `amount: f32` in the operator's
   abstract volume unit
4. `parallel: bool` — whether channels may dispense simultaneously

The signature SHALL NOT include `require_glass`, `timeout`, `recipe`, `size_ml`,
or `channel_map` parameters. Glass detection is a HAL-internal pre-condition
enforced by the job executor loop before dispensing begins. Volume normalization
is performed by the server and passed in as pre-computed `DispenseItem` values.

#### Scenario: create_job accepts four arguments
- **WHEN** the server layer calls
  `create_job(job_id, name, items: Vec<DispenseItem>, parallel)`
- **THEN** the HAL accepts the call without additional parameters

#### Scenario: DispenseItem carries pre-computed volume
- **GIVEN** a recipe `{vodka: 2 parts, lime: 1 part}` for a glass of `volume = 90`
- **WHEN** the server builds the `DispenseItem` list
- **THEN** `items` contains `[{liquid_id: "vodka", amount: 60.0}, {liquid_id: "lime", amount: 30.0}]`

### Requirement: Queue flush on config mutation
Any config mutation (`PATCH /config` or `POST /config/restore`) SHALL execute a
pre-flight sequence before applying the change:
1. Cancel all jobs in `Queued` or `WaitingForGlass` state immediately. Each
   cancelled job SHALL emit a `job_update` SSE event with `state: "cancelled"`.
2. If a job is `Active` (in `Working` or `Error { recovery: PutGlassBack }`),
   wait for it to reach a terminal state (`Done`, `Cancelled`, or `Error`) before proceeding.
3. After pre-flight completes, apply the config change.

The response body of the config mutation SHALL include a `cancelled_job_ids`
array listing the IDs of jobs that were cancelled by the pre-flight.

#### Scenario: Queued jobs are cancelled before config applies
- **WHEN** `PATCH /config` is called with three jobs in the queue
- **THEN** all three jobs transition to `Cancelled`, their IDs appear in
  `cancelled_job_ids`, and the new config is applied after cancellation

#### Scenario: Active job is awaited before config applies
- **WHEN** `PATCH /config` is called while one job is `Active` and one is `Queued`
- **THEN** the queued job is cancelled immediately, the request waits for the
  active job to finish, and the config is applied only after the active job is done

#### Scenario: cancelled_job_ids is empty when queue is idle
- **WHEN** `PATCH /config` is called with no jobs queued or running
- **THEN** the response includes `"cancelled_job_ids": []`

#### Scenario: SSE events emitted for flushed jobs
- **WHEN** the pre-flight cancels queued jobs
- **THEN** a `job_update` SSE event is emitted for each cancelled job with
  `state: "cancelled"`

## ADDED Requirements

### Requirement: JobState Active variant
`JobState` SHALL define an `Active` variant that replaces `Running`. `Active`
indicates the job is the current job — either in `Working`, `WaitingForGlass`,
or recoverable `Error` state. Clients requiring detailed active-job information
SHALL read it from `RobotState` (via `GET /status` or SSE), not from `JobState`.

#### Scenario: Active job shows JobState::Active
- **WHEN** a job is the current job being executed
- **THEN** `DispenseHal::job_status(job_id)` returns `JobStatus { state: Active, .. }`

#### Scenario: Running variant does not exist
- **WHEN** `src/hal/mod.rs` is inspected
- **THEN** `JobState::Running` is not defined

### Requirement: Cleaning entry clears the job queue
When `CleaningHal::start_cleaning()` is called, the HAL SHALL cancel all jobs
in `Queued` or `WaitingForGlass` state before transitioning to `Cleaning` state.
Jobs currently `Active` SHALL be cancelled immediately (dispensing is interrupted).

#### Scenario: start_cleaning cancels all queued jobs
- **WHEN** `start_cleaning()` is called with two queued and one active job
- **THEN** all three jobs are cancelled and `RobotState::Cleaning` is entered
