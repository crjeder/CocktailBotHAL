## Why

The SSE poll loop detects new and changed jobs but silently drops jobs that
disappear from `list_jobs()`. A client that last saw a job at `running 80%`
never learns it finished — leaving the display in a stale state indefinitely.

## What Changes

- The `SseServer` poll loop gains a "departed jobs" pass: for every job in
  `prev` that is absent from `current`, one final `job_update` event is emitted
  with the last known payload (job_id, name, state, progress_pct).
- `API.yaml` `/events` description is updated to document terminal `job_update`
  behaviour.

## Capabilities

### New Capabilities

_(none — this is a behaviour fix within an existing capability)_

### Modified Capabilities

- `sse-wiring`: new requirement — emit a terminal `job_update` when a job
  departs from `list_jobs()`.

## Impact

- **`src/server/sse.rs`** — poll loop addition (~10 lines).
- **`API.yaml`** — `/events` description prose update only; no schema changes.
- No HAL trait changes, no breaking changes, no version bump required.
