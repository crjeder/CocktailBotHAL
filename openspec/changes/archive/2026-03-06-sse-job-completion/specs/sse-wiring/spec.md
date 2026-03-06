## ADDED Requirements

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
