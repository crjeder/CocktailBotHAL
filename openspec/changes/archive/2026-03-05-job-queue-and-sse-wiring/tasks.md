## 1. HAL Types (src/hal/mod.rs)

- [x] 1.1 Add `max_queue_depth: u8` field to `Capabilities` struct
- [x] 1.2 Add `JobCreated { job_id: String, queue_position: u8 }` struct with `#[derive(Debug, Clone, Serialize)]`
- [x] 1.3 Rename `JobStatus::client_job_id` → `JobStatus::name`; update serde rename if needed
- [x] 1.4 Change `DispenseHal::create_job` return type from `Result<String, ErrorInfo>` to `Result<JobCreated, ErrorInfo>`
- [x] 1.5 Bump crate version in `Cargo.toml` to `0.3.0` (breaking change)

## 2. Stub HAL Updates (src/main.rs)

- [x] 2.1 Update `StubDispenseHal::create_job` to return `Ok(JobCreated { job_id: ..., queue_position: 1 })`
- [x] 2.2 Update `StubDispenseHal::list_jobs` — rename `client_job_id` → `name` in any returned `JobStatus` literals
- [x] 2.3 Add `max_queue_depth: 5` to `StubConfigHal::get_active_config` Capabilities

## 3. job_id Generation (src/server/handlers/dispense.rs)

- [x] 3.1 Implement `generate_job_id(name: &str) -> String` using `embassy_time::Instant::now()`: sanitize name (replace non-`[A-Za-z0-9 _-]` with `_`, truncate to 32 chars), append `-`, encode `day + month*3` as 2 hex digits, encode time-of-day deciseconds as 4 hex digits
- [x] 3.2 Update `handle_create_job` to parse `name` from request body (not `client_job_id`)
- [x] 3.3 Pass generated `job_id` and parsed `name` to `hal.create_job(...)`
- [x] 3.4 Return HTTP 201 with `{ "job_id": ..., "queue_position": ... }` on success
- [x] 3.5 Match `ErrorInfo.code == "QUEUE_FULL"` and return HTTP 503 `{ "error": "queue full" }`

## 4. SSE Event Update (src/server/sse.rs)

- [x] 4.1 Rename `client_job_id` → `name` in the `emit_job_event` JSON payload

## 5. SSE Task Wiring (src/main.rs)

- [x] 5.1 Declare `static STUB_STATUS: StaticCell<StubStatusHal>` and `static STUB_DISPENSE_SSE: StaticCell<StubDispenseHal>` for the SSE read path
- [x] 5.2 Add `#[embassy_executor::task] async fn sse_task(net_stack, status, dispense)` that constructs and runs `SseServer`
- [x] 5.3 In `async_main`, initialise the static cells and spawn `sse_task` via the spawner before entering the `ApiServer` run loop
- [x] 5.4 Update the `async_main` task signature to receive `embassy_executor::Spawner` so it can spawn `sse_task`

## 6. API.yaml

- [x] 6.1 Add `max_queue_depth` (integer) to the `Capabilities` schema
- [x] 6.2 Rename `client_job_id` → `name` in `JobStatus` schema
- [x] 6.3 Update `JobCreateRequest` schema: rename `client_job_id` → `name`
- [x] 6.4 Update `JobCreateResponse` schema: add `queue_position` (integer, 1-based); keep `job_id`
- [x] 6.5 Document 503 response on `POST /v1/dispense/jobs`

## 7. Verification

- [x] 7.1 `cargo check` passes with zero warnings
- [x] 7.2 `cargo check --features esp32` passes with zero warnings
- [x] 7.3 `cargo fmt` applied; no formatting diff
