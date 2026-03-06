## Context

`SseServer` in `src/server/sse.rs` polls `StatusHal::state()` and
`DispenseHal::list_jobs()` every 500 ms and emits `state_change` and
`job_update` SSE events when it detects changes. The existing diff logic
handles two cases: a job whose `job_id` does not appear in `prev` (new job)
and a job whose `state` or `progress_pct` changed (progress event). It does
not handle the case where a job present in `prev` is absent from `current` —
i.e. the HAL has retired the job from its list.

A connected display client therefore never receives a terminal event for a
finished or cancelled job. The last event it saw was the most recent
in-progress `job_update`, and the display remains stuck in that state until
it reconnects and receives a fresh snapshot.

## Goals / Non-Goals

**Goals:**
- Emit a final `job_update` (with last-known payload) when a job leaves
  `list_jobs()`.
- Keep the event model unchanged: `state_change` and `job_update` only.
- No auth change, no port change, no HAL trait change.

**Non-Goals:**
- Distinguishing *why* a job left (done vs. cancelled vs. error) — the last
  known `state` field carries that information if the HAL updated it before
  removal; if not, the client sees the last-known state, which is the best we
  can do with a polling model.
- Guaranteeing exactly-once delivery — SSE is best-effort over TCP.

## Decisions

### Emit last-known state, not a synthetic terminal state

**Decision:** When a job disappears, re-emit its last `JobStatus` snapshot as
a `job_update` event unchanged.

**Rationale:** The HAL may have already transitioned the job to `done` or
`cancelled` in its final `list_jobs()` response before removing it, or it may
remove it abruptly (reset). Either way, re-emitting the last-known snapshot is
correct and safe. Synthesising `done` when the job simply vanished would be
wrong if the robot was reset mid-job (actual state: `cancelled` or `error`).

**Alternative considered:** Add a new `job_finished` event type. Rejected —
the client already handles `job_update` with terminal states; a second event
type adds API surface with no additional information.

### Detection placement: after the job-changed loop

**Decision:** Scan `prev.jobs` for IDs absent from `current.jobs` in a second
pass, after the existing changed-job loop.

**Rationale:** Keeps the two concerns (progress updates vs. removals) clearly
separated without restructuring the existing loop. The second pass iterates
only `prev.jobs` (typically 0–3 entries on an embedded robot), so the O(n²)
scan is negligible.

## Risks / Trade-offs

- **HAL removes job before final state** — If a HAL removes a job from
  `list_jobs()` without first reporting a terminal `state`, the client will
  receive the last polled state (e.g. `running 80%`). This is a HAL contract
  issue, not an SSE layer issue, and is the best outcome possible under a
  polling model.
- **Race between two polls** — A job could be created and completed within a
  single 500 ms poll interval. The initial-snapshot pass on connect would not
  include it, and the departed-job pass would emit it. Clients will see a
  `job_update` for a job they never saw appear — this is harmless but
  potentially surprising. Documented in the spec scenario.
