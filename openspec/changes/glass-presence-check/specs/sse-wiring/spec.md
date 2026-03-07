## MODIFIED Requirements

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

## ADDED Requirements

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
