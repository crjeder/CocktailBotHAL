## ADDED Requirements

### Requirement: job-create signature
`DispenseHal::create_job` SHALL accept exactly four parameters:
1. `name: &str` — human-readable, non-unique label supplied by the client
2. `recipe: &Recipe` — the cocktail recipe to dispense (from `generic_cocktail`)
3. `size_ml: u16` — total target volume in millilitres, computed from glass type at call site
4. `channel_map: &[u8]` — mapping from recipe ingredient index to hardware channel index

The signature SHALL NOT include `require_glass` or `timeout` parameters. Glass
detection is a pre-condition enforced by the server layer before calling into the HAL.
Timeouts are managed by the executor / watchdog layer, not the HAL method signature.

#### Scenario: create_job accepts four arguments
- **WHEN** the server layer calls `create_job(name, recipe, size_ml, channel_map)`
- **THEN** the HAL accepts the call without additional parameters

#### Scenario: require_glass is not a HAL parameter
- **GIVEN** the server layer has already verified glass presence via `SensorHal`
- **WHEN** `create_job` is invoked
- **THEN** no `require_glass` flag is passed; glass enforcement is the caller's responsibility

### Requirement: JobCreated return type
`DispenseHal::create_job` SHALL return `Result<JobCreated, ErrorInfo>` where
`JobCreated` is a struct with fields `job_id: String` and `queue_position: u8`.
`queue_position` SHALL be 1-based (1 = next to be dispensed).

#### Scenario: Successful job creation returns position
- **WHEN** `create_job` is called and the queue is not full
- **THEN** the HAL returns `Ok(JobCreated { job_id, queue_position })` where `queue_position >= 1`

#### Scenario: First job in empty queue is position 1
- **WHEN** no jobs are queued or running and `create_job` is called
- **THEN** the returned `queue_position` is `1`

### Requirement: Bounded job queue
`DispenseHal` SHALL enforce a maximum queue depth equal to
`Capabilities.max_queue_depth`. When the queue is at capacity, `create_job`
SHALL return `Err(ErrorInfo { code: "QUEUE_FULL", recoverable: true, .. })`.

#### Scenario: Queue full returns QUEUE_FULL error
- **WHEN** `create_job` is called and the number of active + queued jobs equals `max_queue_depth`
- **THEN** the HAL returns `Err(ErrorInfo)` with `code == "QUEUE_FULL"` and `recoverable == true`

#### Scenario: Queue not full allows creation
- **WHEN** the number of active + queued jobs is less than `max_queue_depth`
- **THEN** `create_job` succeeds and returns `Ok(JobCreated { .. })`

### Requirement: max_queue_depth in Capabilities
The `Capabilities` struct SHALL include a field `max_queue_depth: u8` declaring
the maximum number of jobs (running + queued) the robot accepts simultaneously.

#### Scenario: Capabilities includes max_queue_depth
- **WHEN** `ConfigHal::get_active_config()` is called
- **THEN** the returned `RobotConfig.capabilities.max_queue_depth` is a non-zero value

### Requirement: Deterministic job_id generation
The server layer SHALL generate `job_id` as `<sanitized_name>-<DD><MM3><TIME>` where:
- `<sanitized_name>` is the client-supplied `name` with non-`[A-Za-z0-9 _-]` chars replaced by `_`, truncated to 32 chars
- `<DD>` is `day + month * 3` encoded as 2 uppercase hex digits
- `<TIME>` is the current time of day in deciseconds (1/10 s) encoded as 4 uppercase hex digits
This scheme SHALL produce no collisions within a 24-hour window for any single name.

#### Scenario: job_id embeds sanitized name
- **WHEN** `create_job` is called with `name = "Marty's Margarita"`
- **THEN** the returned `job_id` starts with `"Marty_s_Margarita-"`

#### Scenario: job_id suffix encodes time
- **WHEN** two `create_job` calls are made more than 100 ms apart with the same name
- **THEN** the returned `job_id` values are distinct

### Requirement: job name field
`JobStatus` SHALL have a field `name: String` (renamed from `client_job_id`)
carrying the human-readable, non-unique label supplied by the client at job creation.

#### Scenario: name is preserved in JobStatus
- **WHEN** a job is created with `name = "Jane's Mojito"`
- **THEN** `DispenseHal::job_status(job_id)` returns a `JobStatus` where `name == "Jane's Mojito"`
