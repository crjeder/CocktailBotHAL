## ADDED Requirements

### Requirement: Queue flush on config mutation
Any config mutation (`PATCH /config` or `POST /config/restore`) SHALL execute a
pre-flight sequence before applying the change:
1. Cancel all jobs in `Queued` state immediately. Each cancelled job SHALL emit a
   `job_update` SSE event with `state: "cancelled"`.
2. If a job is in `Running` state, wait for it to reach a terminal state
   (`Done`, `Cancelled`, or `Error`) before proceeding.
3. After pre-flight completes, apply the config change.

The response body of the config mutation SHALL include a `cancelled_job_ids`
array listing the IDs of jobs that were cancelled by the pre-flight.

#### Scenario: Queued jobs are cancelled before config applies
- **WHEN** `PATCH /config` is called with three jobs in the queue
- **THEN** all three jobs transition to `Cancelled`, their IDs appear in
  `cancelled_job_ids`, and the new config is applied after cancellation

#### Scenario: Running job is awaited before config applies
- **WHEN** `PATCH /config` is called while one job is `Running` and one is `Queued`
- **THEN** the queued job is cancelled immediately, the request waits for the
  running job to finish, and the config is applied only after the running job is done

#### Scenario: cancelled_job_ids is empty when queue is idle
- **WHEN** `PATCH /config` is called with no jobs queued or running
- **THEN** the response includes `"cancelled_job_ids": []`

#### Scenario: SSE events emitted for flushed jobs
- **WHEN** the pre-flight cancels queued jobs
- **THEN** a `job_update` SSE event is emitted for each cancelled job with
  `state: "cancelled"`
