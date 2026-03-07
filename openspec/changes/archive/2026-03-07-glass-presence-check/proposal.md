## Why

The robot currently has no glass presence logic: it will dispense into thin air if no glass is placed, and has no way to communicate glass-related waiting states to clients. Adding HAL-internal glass detection with SSE-observable state makes dispensing safe without requiring any client-side orchestration — the robot handles it autonomously.

## What Changes

- **BREAKING** `RobotState` becomes a data-carrying enum: `WaitingForGlass`, `Working`, `DrinkReady`, and `Error` all carry job context and/or timeout countdowns.
- `Prepared` state is **removed** (was unused).
- New types: `GlassWaitReason` (NoGlass | TooSmall), `RecoveryAction` (PutGlassBack | RemoveGlass | CallResetErrors | None).
- `JobState::Running` replaced by `JobState::Active` (detail lives in `RobotState`).
- `AdminConfig` gains two configurable timeouts: `glass_wait_timeout_secs` and `drink_ready_timeout_secs`.
- `Capabilities` gains `has_cancel_button` and `has_power_button` fields; `glass_typing` is retained (indicates whether the sensor can identify glass type for size validation).
- Glass type validation at job start: reject if detected glass is smaller than requested, warn via SSE if larger.
- `Cleaning` state is admin-only, clears the job queue on entry, and has no valid transition back to `Idle` — the only exit is power-off.
- Mid-pour glass removal triggers a recoverable `Error` (code `GLASS_REMOVED`) with a timeout; drink left uncollected triggers `Error` (code `DRINK_ABANDONED`) requiring physical glass removal.
- All glass-related state transitions are HAL-internal; the server layer is a passive observer via `StatusHal::state()` and SSE.

## Capabilities

### New Capabilities

- `glass-aware-state-machine`: Full state machine for the robot including glass-presence states, timeout countdowns, recovery actions, and state transition rules. Covers `RobotState`, `GlassWaitReason`, `RecoveryAction`, and the complete set of valid transitions.

### Modified Capabilities

- `async-hal-traits`: **BREAKING** — `RobotState` enum shape changes (data-carrying variants); `JobState::Running` → `JobState::Active`; new supporting types added to `hal/mod.rs`.
- `admin-config`: New fields `glass_wait_timeout_secs` (default 60) and `drink_ready_timeout_secs` (default 300) added to `AdminConfig`; new fields `has_cancel_button` and `has_power_button` added to `Capabilities`.
- `job-queue`: `Cleaning` state now clears the job queue on entry; `JobState::Running` replaced by `JobState::Active`.
- `sse-wiring`: New SSE state-change events carry the enriched `RobotState` payload including glass-wait reason and timeout countdowns.

## Impact

- `src/hal/mod.rs`: Breaking changes to `RobotState`, `JobState`, `Capabilities`, `AdminConfig`; new types `GlassWaitReason`, `RecoveryAction`.
- `src/hal/mock.rs`: Mock `StatusHal` must return new `RobotState` variants.
- `src/server/handlers/status.rs`: Serialize new `RobotState` shape.
- `src/server/handlers/cleaning.rs`: Gate `start_cleaning` on admin auth; clear job queue on entry.
- `src/server/sse.rs`: Emit enriched state-change events.
- `src/esp32/`: Stub `glass_state()` returns `present: true` (optimistic no-sensor default).
- `API.yaml`: Update `RobotState` schema to tagged union; add new fields to `AdminConfig` and `Capabilities`.
- Crate version bump required (breaking HAL trait change): `0.5.0` → `0.6.0`.
