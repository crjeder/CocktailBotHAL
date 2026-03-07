## ADDED Requirements

### Requirement: RobotState is a data-carrying enum
`RobotState` in `src/hal/mod.rs` SHALL be a data-carrying Rust enum. Active
variants SHALL embed job context and timeout countdowns so that a single
`StatusHal::state()` call provides complete state information without additional
HAL queries. Stateless variants carry no payload.

Variants and their payloads:

| Variant | Payload |
|---|---|
| `Off` | none |
| `SelfTest` | none |
| `Provisioning` | none |
| `Idle` | none |
| `Cleaning` | none |
| `WaitingForGlass` | `job_id: String`, `reason: GlassWaitReason`, `timeout_remaining_secs: Option<u32>` |
| `Working` | `job_id: String`, `progress_pct: u8` |
| `DrinkReady` | `job_id: String`, `timeout_remaining_secs: Option<u32>` |
| `Error` | `code: String`, `message: String`, `job_id: Option<String>`, `recoverable: bool`, `recovery: RecoveryAction`, `timeout_remaining_secs: Option<u32>` |

`Prepared` SHALL be removed.

#### Scenario: WaitingForGlass carries job_id and reason
- **WHEN** the robot is waiting for a glass to be placed
- **THEN** `StatusHal::state()` returns `RobotState::WaitingForGlass { job_id, reason, timeout_remaining_secs }`

#### Scenario: Working carries progress
- **WHEN** the robot is dispensing
- **THEN** `StatusHal::state()` returns `RobotState::Working { job_id, progress_pct }` where `progress_pct` is in `[0, 100]`

#### Scenario: Error carries recovery hint
- **WHEN** the robot is in an error state
- **THEN** `StatusHal::state()` returns `RobotState::Error { code, message, recovery, .. }` where `recovery` is a `RecoveryAction`

#### Scenario: Prepared variant does not exist
- **WHEN** `src/hal/mod.rs` is inspected
- **THEN** `RobotState::Prepared` is not defined

### Requirement: GlassWaitReason enum
`GlassWaitReason` SHALL be defined in `src/hal/mod.rs` with two variants:
- `NoGlass`: no glass detected at the dispense position.
- `TooSmall { detected_volume: f32, required_volume: f32 }`: a glass was detected but its identified capacity is less than the requested dispense volume. Only produced when `Capabilities::glass_typing` is `true`.

#### Scenario: NoGlass reason when sensor sees nothing
- **WHEN** `SensorHal::glass_state()` returns `present: false`
- **THEN** `WaitingForGlass { reason: GlassWaitReason::NoGlass, .. }`

#### Scenario: TooSmall reason when detected glass is smaller
- **WHEN** `glass_typing` is `true` and the detected glass volume is less than the required dispense volume
- **THEN** `WaitingForGlass { reason: GlassWaitReason::TooSmall { detected_volume, required_volume }, .. }`

### Requirement: RecoveryAction enum
`RecoveryAction` SHALL be defined in `src/hal/mod.rs` with four variants:

| Variant | Meaning | Timeout applies? |
|---|---|---|
| `PutGlassBack` | Glass was removed mid-pour; user must replace it | Yes (`glass_wait_timeout_secs`) |
| `RemoveGlass` | Drink was not collected; user must remove the glass | No (wait indefinitely) |
| `CallResetErrors` | Firmware or HAL error; API call to `reset_errors()` recovers | No |
| `None` | Unrecoverable | No |

#### Scenario: GLASS_REMOVED uses PutGlassBack
- **WHEN** the robot transitions to `Error` because the glass was removed during `Working`
- **THEN** `recovery == RecoveryAction::PutGlassBack`

#### Scenario: DRINK_ABANDONED uses RemoveGlass
- **WHEN** the robot transitions to `Error` because the drink was not collected in time
- **THEN** `recovery == RecoveryAction::RemoveGlass`

### Requirement: Valid state transitions
The HAL implementation SHALL only allow the following state transitions:

```
Off           → SelfTest           (power on)
SelfTest      → Idle               (self-test passed, config present)
SelfTest      → Provisioning       (self-test passed, no stored config)
SelfTest      → Error              (self-test failed)
Provisioning  → Idle               (valid config restored via POST /config/restore)
Provisioning  → Cleaning           (admin starts cleaning)
Idle          → WaitingForGlass    (job created via POST /v1/dispense/jobs)
Idle          → Cleaning           (admin starts cleaning)
WaitingForGlass → Working          (glass present, validated)
WaitingForGlass → Cancelled/Idle   (timeout expires)
Working       → DrinkReady         (dispensing complete)
Working       → Error              (glass removed mid-pour → GLASS_REMOVED)
Working       → Error              (HAL/firmware fault)
DrinkReady    → Idle               (glass removed)
DrinkReady    → Error              (timeout → DRINK_ABANDONED)
Error         → Working            (recovery: PutGlassBack, glass replaced before timeout)
Error         → Idle               (recovery: PutGlassBack, timeout → Cancelled)
Error         → Idle               (recovery: RemoveGlass, glass removed)
Error         → Idle               (recovery: CallResetErrors, reset_errors() called)
Cleaning      → Off                (power off — only valid exit from Cleaning)
```

All other transitions SHALL NOT occur.

#### Scenario: WaitingForGlass transitions to Working on valid glass
- **WHEN** the robot is in `WaitingForGlass` and a valid glass is detected
- **THEN** state transitions to `Working`

#### Scenario: WaitingForGlass cancels on timeout
- **WHEN** `glass_wait_timeout_secs > 0` and the timeout expires in `WaitingForGlass`
- **THEN** the active job transitions to `Cancelled` and robot state returns to `Idle`

#### Scenario: Working detects glass removal
- **WHEN** `SensorHal::glass_state()` returns `present: false` during `Working`
- **THEN** robot state transitions to `Error { code: "GLASS_REMOVED", recovery: PutGlassBack }`

#### Scenario: Error PutGlassBack resumes on glass return
- **WHEN** robot is in `Error { recovery: PutGlassBack }` and glass is replaced before timeout
- **THEN** state transitions back to `Working` and dispensing resumes

#### Scenario: Error PutGlassBack cancels on timeout
- **WHEN** robot is in `Error { recovery: PutGlassBack }` and timeout expires
- **THEN** job is `Cancelled` and robot transitions to `Idle`

#### Scenario: DrinkReady transitions to Idle on glass removal
- **WHEN** robot is in `DrinkReady` and `glass_state()` returns `present: false`
- **THEN** robot transitions to `Idle`

#### Scenario: DrinkReady error on timeout
- **WHEN** `drink_ready_timeout_secs > 0` and timeout expires in `DrinkReady`
- **THEN** robot transitions to `Error { code: "DRINK_ABANDONED", recovery: RemoveGlass }`

#### Scenario: DRINK_ABANDONED clears on glass removal
- **WHEN** robot is in `Error { code: "DRINK_ABANDONED" }` and glass is removed
- **THEN** robot transitions to `Idle` (no timeout — waits indefinitely for removal)

### Requirement: Glass type validation at job start
When `Capabilities::glass_typing` is `true`, the HAL implementation SHALL
validate the detected glass type against the requested dispense volume at the
moment dispensing is about to begin (while in `WaitingForGlass`):
- If detected glass volume **<** required volume: remain in `WaitingForGlass` with `reason: TooSmall`.
- If detected glass volume **≥** required volume and volume **>** required: proceed to `Working` and emit an SSE warning event `glass_size_warning`.
- If detected glass volume **==** required volume (within tolerance): proceed to `Working`.

When `glass_typing` is `false`, glass type validation SHALL be skipped entirely.

#### Scenario: Too-small glass blocks dispensing
- **WHEN** `glass_typing` is `true` and detected glass volume is less than required
- **THEN** state stays `WaitingForGlass { reason: TooSmall { .. } }` and dispensing does not begin

#### Scenario: Larger glass allows dispensing with warning
- **WHEN** `glass_typing` is `true` and detected glass volume exceeds required volume
- **THEN** state transitions to `Working` and a `glass_size_warning` SSE event is emitted

#### Scenario: glass_typing false skips validation
- **WHEN** `glass_typing` is `false` and any glass is detected as present
- **THEN** state transitions to `Working` regardless of glass volume

### Requirement: Cleaning is a terminal admin-only state
`Cleaning` state SHALL only be entered via `CleaningHal::start_cleaning()`,
which SHALL be gated by admin authentication at the handler level. `start_cleaning()`
SHALL be valid only when the robot is in `Idle` or `Provisioning` state. On
entry, the HAL SHALL cancel all queued jobs. The only valid exit from `Cleaning`
is power-off (`Off`). There is no `Cleaning → Idle` transition.

#### Scenario: start_cleaning requires admin auth
- **WHEN** `POST /v1/cleaning/start` is called without valid admin credentials
- **THEN** the server returns `401 Unauthorized` and state does not change

#### Scenario: start_cleaning clears job queue
- **WHEN** `POST /v1/cleaning/start` is called with three jobs in the queue
- **THEN** all three jobs are cancelled and `RobotState::Cleaning` is entered

#### Scenario: No transition from Cleaning to Idle
- **WHEN** the robot is in `Cleaning` state
- **THEN** `CleaningHal::stop_cleaning()` does not cause a transition to `Idle`; the state remains `Cleaning` until power-off

#### Scenario: start_cleaning rejected outside Idle and Provisioning
- **WHEN** `POST /v1/cleaning/start` is called while robot is in `Working` state
- **THEN** the server returns `409 Conflict`

### Requirement: Glass polling interval
The HAL implementation SHALL poll `SensorHal::glass_state()` at a fixed interval
of at most 500 ms while in `WaitingForGlass`, `Working`, `DrinkReady`, or
`Error { recovery: PutGlassBack | RemoveGlass }` states. A constant
`GLASS_POLL_INTERVAL_MS` SHALL be defined in the HAL implementation (not in the
trait) and SHALL default to `250`.

#### Scenario: Glass poll constant defined
- **WHEN** the HAL implementation is inspected
- **THEN** a constant `GLASS_POLL_INTERVAL_MS: u64 = 250` (or lower) is defined

#### Scenario: Glass state polled during Working
- **WHEN** the robot is in `Working` state
- **THEN** `SensorHal::glass_state()` is called at least once every 500 ms

### Requirement: Optimistic stub for glass sensor
When no hardware glass sensor is connected, `SensorHal::glass_state()` SHALL
return `GlassSensorState { present: true, glass_type: None, confidence: 0.0 }`.
`confidence: 0.0` signals that the reading is not from real hardware.

#### Scenario: Stub returns present: true
- **WHEN** the ESP32 stub `glass_state()` is called
- **THEN** it returns `Ok(GlassSensorState { present: true, glass_type: None, confidence: 0.0 })`
