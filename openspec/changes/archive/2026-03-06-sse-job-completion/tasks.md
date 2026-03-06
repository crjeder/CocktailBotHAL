## 1. SSE Poll Loop (src/server/sse.rs)

- [x] 1.1 In `handle_client`, after the existing job-changed loop, add a departed-jobs pass: iterate `prev.jobs` and for each entry whose `job_id` is not present in `current.jobs`, call `emit_job_event(socket, &job).await`; return on error
- [x] 1.2 Set `sent_event = true` when any terminal `job_update` is emitted (so the keepalive timer resets correctly)

## 2. API Specification (API.yaml)

- [x] 2.1 In the `/events` path description, add a note that `job_update` is also emitted as a terminal event when a job departs from `list_jobs()`, carrying the last-known state and progress

## 3. Validation

- [x] 3.1 Run `cargo check`; fix all errors
- [x] 3.2 Run `cargo fmt`; verify no diff
