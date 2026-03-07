## Context

The robot has a `SensorHal::glass_state()` method and a `GlassSensorState` type, but nothing currently reads sensor state before or during dispensing. `RobotState` is a flat enum with no payload — clients cannot distinguish *why* the robot is waiting or what job it is working on without a separate `GET /v1/dispense/jobs/{id}` call. The `Prepared` state was added but never assigned semantics. `Cleaning` has no access controls and no queue-clearing behavior.

This change makes the robot self-governing around glass presence: the HAL implementation handles all waiting, validation, and timeout logic internally. The server layer and clients are passive observers.

## Goals / Non-Goals

**Goals:**
- `RobotState` carries job context and timeout countdowns in active states — a single `GET /status` or SSE event gives clients everything they need.
- HAL implementation autonomously waits for glass, validates type (if capable), and emits state transitions without server-layer orchestration.
- Configurable timeouts for glass placement and drink collection.
- `Cleaning` is admin-gated and terminally exits to power-off only.
- Physical button capabilities are declared so clients can adapt their UI.

**Non-Goals:**
- Implementing actual glass sensor hardware on ESP32 (stubs suffice; real wiring is a separate hardware bring-up task).
- Implementing the cancel button or power button interrupt handlers on ESP32.
- Changing the `DispenseHal` trait signature or job creation flow.
- Adding glass presence polling to the SSE server directly — the existing state poll is sufficient.

## Decisions

### D1: Data-carrying `RobotState` enum (not a wrapper struct)

**Decision:** `RobotState` becomes a data-carrying enum where active variants embed job context.

**Alternatives considered:**
- *`RobotStatus` wrapper struct* (`state: RobotState` + `active_job: Option<ActiveJobInfo>`): additive, less breaking. Rejected because it requires clients to check two fields and leaves the relationship between `state` and `active_job` implicit. The enum variant *is* the state — carrying its data directly is more correct.
- *Separate `GET /v1/status/job` endpoint*: rejected, adds a round-trip where none is needed.

**Rationale:** A single-job robot's machine state IS the job state when active. Data-carrying variants make this explicit in the type system and in the JSON shape. Rust's `serde` tagged-union serialization handles this cleanly.

### D2: Glass logic lives entirely in the HAL implementation

**Decision:** The server layer (`handle_create_job`, etc.) does not call `SensorHal`; the HAL implementation's job executor loop owns all glass polling and state transitions.

**Alternatives considered:**
- *Server layer checks sensor before `create_job`*: requires the dispense handler to take a `SensorHal` generic parameter, adding coupling. Rejected — the HAL interface already composes all sub-HALs internally (e.g. `Esp32Hal`).
- *Client polls sensor and decides when to submit*: defeats the goal of autonomous operation.

**Rationale:** The HAL is the right place for hardware-gated safety logic. The server layer stays a thin HTTP adapter. `Esp32Hal` already holds references to all sub-components.

### D3: `WaitingForGlass` is a pre-job state; mid-pour removal is `Error` (recoverable)

**Decision:** `RobotState::WaitingForGlass` is only entered from `Idle` (before dispensing begins). If the glass is removed during `Working`, the robot enters `Error { recovery: PutGlassBack }`.

**Rationale:** `WaitingForGlass` is an expected, normal flow step. Mid-pour glass removal is abnormal — it warrants an `Error` signal. The distinction helps clients show the right UI ("please place your glass" vs "something went wrong"). `DrinkReady` timeout also yields `Error { recovery: RemoveGlass }` for the same reason: the blocked robot is an exceptional condition.

### D4: Two separate timeouts in `AdminConfig`

**Decision:** `glass_wait_timeout_secs` (default 60) governs waiting for glass placement and mid-pour recovery. `drink_ready_timeout_secs` (default 300) governs `DrinkReady`. Both accept `0` for "wait indefinitely."

**Rationale:** The two scenarios have different natural durations. Placing a glass is an intentional immediate action; collecting a drink can be deferred (party context). Sharing one timeout would force an awkward compromise.

### D5: Timeout restarts on each new glass detection event

**Decision:** In `WaitingForGlass` and `Error { recovery: PutGlassBack }`, the countdown resets each time the sensor detects a glass (even if wrong size) rather than running from job creation.

**Rationale:** Penalizing users for sensor events they can't control (e.g., a glass briefly touching the sensor) is bad UX. Restarting gives the user the full window after each attempt.

### D6: `Cleaning` exit is power-off only

**Decision:** `CleaningHal::stop_cleaning()` is retained but its documented effect is "cleaning acknowledged, ready for shutdown." There is no `Cleaning → Idle` transition.

**Rationale:** Cleaning requires physical hardware reconfiguration (swapping liquid containers). Returning to `Idle` without physical intervention could cause the robot to dispense cleaning solution. Power-off + power-on through `SelfTest` is the safe gate.

### D7: ESP32 stub returns `present: true` (optimistic default)

**Decision:** When no glass sensor hardware is wired, `Esp32Sensors::glass_state()` returns `GlassSensorState { present: true, glass_type: None, confidence: 0.0 }`.

**Alternatives considered:**
- *Return `present: false`*: would make the robot wait forever on hardware without a sensor. Rejected.

**Rationale:** Optimistic default means the robot behaves as if glass is always present — correct for development and hardware without a sensor. `confidence: 0.0` signals to any observer that the reading is not from real hardware.

## Risks / Trade-offs

- **Breaking change to `RobotState`**: Any client or test that pattern-matches on `RobotState` variants will fail to compile. Mitigation: version bump to `0.6.0`, update all mock/stub implementations in the same PR.
- **HAL implementation owns sensor timing**: The job executor must poll `glass_state()` at a reasonable rate (e.g., 250ms). Polling too fast wastes CPU; too slow makes the robot unresponsive to glass placement. Mitigation: define a `GLASS_POLL_INTERVAL_MS` constant in the HAL implementation, not the trait.
- **`timeout_remaining_secs` is approximate**: The SSE loop polls at 500ms; the countdown decrements each second in the HAL. Clients may see a 0.5–1.5s jitter in the displayed countdown. Mitigation: document that `timeout_remaining_secs` is informational, not a precise clock.
- **`WaitingForGlass` state blocks the queue**: The queue freezes while waiting for glass. For short queues (max 8 per `Capabilities::max_queue_depth`) this is acceptable. Mitigation: document in spec; no new mechanism needed.

## Migration Plan

1. Update `src/hal/mod.rs` with new types (`GlassWaitReason`, `RecoveryAction`) and `RobotState` variants.
2. Fix all compile errors: mock, ESP32 stubs, status handler, SSE server.
3. Add new `AdminConfig` fields with `#[serde(default)]` — backward-compatible with stored configs.
4. Update `API.yaml` to reflect the new tagged-union `RobotState` schema.
5. Bump crate version to `0.6.0` in `Cargo.toml`.

No database migrations. No network protocol changes beyond the JSON schema of `GET /status` and SSE events.

**Rollback:** Revert the commit. The `#[serde(default)]` fields on `AdminConfig` mean stored configs remain valid either way.
