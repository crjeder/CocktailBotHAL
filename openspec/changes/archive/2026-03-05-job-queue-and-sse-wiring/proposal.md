## Why

The cocktail robot needs to support multiple concurrent clients: mobile phones submit drink orders and receive a queue position at creation time, while a dedicated display shows the active job via SSE. The current API lacks a bounded job queue, returns no queue position on job creation, conflates human-readable job labels with unique IDs, and the SSE server is never started.

## What Changes

- **BREAKING** Rename `client_job_id` → `name` on `JobStatus`, `JobCreated`, and in SSE events
- **BREAKING** `DispenseHal::create_job` return type changes from `Result<String, ErrorInfo>` to `Result<JobCreated, ErrorInfo>`; new `JobCreated { job_id, queue_position }` struct
- Add `max_queue_depth: u8` to `Capabilities`; `create_job` returns 503 when queue is full
- Deterministic `job_id` generation: `<name>-<DD><MM*3><time_1_10s_hex>` — no random source required; unique within 24 h
- Wire `SseServer` as a dedicated embassy task (single display client on port 9000); rename `client_job_id` → `name` in SSE event payload

## Capabilities

### New Capabilities

- `job-queue`: Bounded job queue with capacity declared in `Capabilities.max_queue_depth`; `create_job` returns `JobCreated { job_id, queue_position }` and 503 when full
- `sse-wiring`: `SseServer` spawned as an embassy task alongside `ApiServer`; single display client on port 9000; emits `state_change` and `job_update` events

### Modified Capabilities

- `async-http-server`: `handle_create_job` handler returns `JobCreated` (with `queue_position`), maps queue-full `ErrorInfo` to HTTP 503

## Impact

- `src/hal/mod.rs`: `Capabilities`, new `JobCreated` struct, `DispenseHal::create_job` signature, `JobStatus::client_job_id` renamed to `name` — **semver major bump required**
- `src/server/handlers/dispense.rs`: parse `name` field, return `JobCreated`, emit 503
- `src/server/sse.rs`: rename field in `emit_job_event`
- `src/main.rs`: spawn `SseServer` as second embassy task with `'static` HAL refs
- `API.yaml`: `Capabilities`, `JobCreateRequest`, `JobCreateResponse`, `JobStatus` schemas
