## MODIFIED Requirements

### Requirement: job-create signature
`DispenseHal::create_job` SHALL accept the following parameters:
`job_id: String`, `name: String`, `items: Vec<JobItem>`, `parallel: bool`.
The parameters `require_glass` and `timeout` SHALL be removed from the
signature — glass waiting is always enforced by the server layer; timeout
policy is a server-side constant, not a per-job client parameter.

#### Scenario: Successful job creation
- **WHEN** `create_job` is called with valid `job_id`, `name`, `items`, and `parallel`
- **THEN** the HAL returns `Ok(JobCreated { job_id, queue_position })`

#### Scenario: create_job does not accept require_glass or timeout
- **WHEN** a HAL implementation of `DispenseHal` is compiled
- **THEN** the `create_job` method has exactly four parameters:
  `job_id: String`, `name: String`, `items: Vec<JobItem>`, `parallel: bool`

## REMOVED Requirements

### Requirement: require_glass parameter
**Reason**: Glass presence wait is always enforced by the server layer before
dispatching to the HAL. It is not a per-job client choice.
**Migration**: Remove `require_glass` from all `DispenseHal::create_job`
implementations. The server handler is responsible for waiting on the glass
sensor before calling `create_job`.

### Requirement: timeout parameter
**Reason**: Job timeout is server policy (compile-time constant or config
value), not a per-job parameter exposed to API clients.
**Migration**: Remove `timeout` from all `DispenseHal::create_job`
implementations. Apply timeout handling in the server layer.
