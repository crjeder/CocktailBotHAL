## Context

The current `DispenseHal::create_job` returns only a `job_id` string; there is no queue depth limit and no queue position returned to callers. `JobStatus` carries a `client_job_id` field whose name implies it is client-supplied but unique — it is neither. `SseServer` exists in `src/server/sse.rs` but is never started; it polls `StatusHal` and `DispenseHal` every 500 ms and emits `state_change` / `job_update` events to a single display client.

The target deployment has:
- Multiple mobile phones submitting drink orders concurrently (short-lived REST calls)
- One always-on display client consuming SSE (long-lived connection, port 9000)

## Goals / Non-Goals

**Goals:**
- Return `{ job_id, queue_position }` from `create_job`
- Enforce a bounded job queue; return 503 when full
- Expose `max_queue_depth` in `Capabilities`
- Rename `client_job_id` → `name` everywhere (HAL, API, SSE)
- Define a deterministic `job_id` generation scheme (no PRNG)
- Wire `SseServer` as a live embassy task alongside `ApiServer`

**Non-Goals:**
- Multi-client SSE (single display client only)
- Phone clients do not receive SSE (no phone-side progress bar)
- No queue-position updates after creation (phones are fire-and-forget)
- No changes to `StatusHal`, `SensorHal`, `CleaningHal`, or `StorageHal`

## Decisions

### D1 — `create_job` return type: new `JobCreated` struct

**Decision:** Introduce `JobCreated { job_id: String, queue_position: u8 }` and change `DispenseHal::create_job` to return `Result<JobCreated, ErrorInfo>`.

**Rationale:** A tuple `(String, u8)` would work but is anonymous and opaque at call sites. A named struct is self-documenting and forward-compatible (e.g., adding `estimated_wait_secs` later without breaking callers).

**Alternative considered:** Return `queue_position` as a separate trait method (`queue_depth()`) called after `create_job`. Rejected: two calls creates a TOCTOU race and forces clients to make an extra round-trip.

### D2 — Deterministic `job_id` format

**Decision:** `job_id = sanitize(name) + "-" + encode(day + month*3, 2 hex) + encode(time_deciseconds, 4 hex)`.

Example: name `"Marty's Margarita"`, 15 May at 14:23:07.4 →
`day=15, month=5 → 15+5*3=30 → 0x1E`; time = 14*3600+23*60+7)*10+4 = 518074 = 0x7E5BA → last 4 hex = `E5BA` → `"Marty's Margarita-1EE5BA"`.

**Rationale:** No random source required. Uniqueness holds within a single day: the time component has 864 000 possible values (10 per second × 86 400 seconds); two jobs with the same name would need to be submitted in the same 100 ms window — acceptable for a cocktail robot. The day+month component disambiguates across days.

**Where to implement:** Server layer (`src/server/handlers/dispense.rs`) reads `embassy_time::Instant` to derive the time component, then passes the pre-formed `job_id` string as a parameter hint. The HAL stub in `main.rs` uses it directly; real hardware HAL implementors may override if they have a preferred scheme.

**Name sanitization:** Replace any character outside `[A-Za-z0-9 _-]` with `_`; truncate to 32 chars before appending suffix.

### D3 — Queue depth enforcement: HAL responsibility

**Decision:** Queue depth enforcement is the HAL's responsibility. When the queue is full the HAL returns an `ErrorInfo` with `code: "QUEUE_FULL"`. The server handler maps this code to HTTP 503.

**Rationale:** The HAL owns the queue; only the HAL knows the current depth. Duplicating queue-depth tracking in the server layer would create dual state.

**API contract:** `Capabilities.max_queue_depth: u8` is the authoritative bound, fetched via `ConfigHal::get_active_config()`.

### D4 — `name` field (renamed from `client_job_id`)

**Decision:** Rename `client_job_id` → `name` in `JobStatus`, request/response bodies, and SSE event payloads.

**Rationale:** `client_job_id` was confusing: "id" implies uniqueness, "client" implies it is not server-generated. `name` is accurate: it is a human-readable, non-unique label set by the client.

**Breaking change:** All existing HAL trait implementations must update the field. Version bump to `0.3.0` (breaking minor under pre-1.0 semver).

### D5 — SSE task wiring: static HAL references

**Decision:** Move `StubStatusHal` and `StubDispenseHal` to `static` storage using `StaticCell`; both `ApiServer` and the `SseServer` task receive `&'static` references.

**Rationale:** Embassy tasks must be `'static`. `SseServer<'a, Stat, Disp>` holds lifetime-bounded references. Making the HAL instances static satisfies this without introducing `Mutex` overhead for the read-only SSE path. `ApiServer` retains mutable ownership of the HAL structs it needs to mutate (control, config, dispense for writes); SSE only reads `StatusHal` and `DispenseHal`.

**Complication:** `DispenseHal` is used mutably by `ApiServer` (create/cancel jobs) and read-only by `SseServer` (list_jobs). For the stub, this is resolved by having `SseServer` hold a separate `&'static StubDispenseHal` for reads only, while `ApiServer` holds its own mutable stub. In production HAL, the implementor must provide a shared read path (e.g., interior mutability with a `Mutex`).

**Alternative considered:** `embassy_sync::mutex::Mutex<NoopRawMutex, Disp>` shared between both tasks. Rejected for now: adds lock overhead on every SSE poll; suitable if a real HAL needs shared mutable state.

### D6 — SSE auth: none

**Decision:** No bearer token required on port 9000.

**Rationale:** The display is a trusted local device on the same network segment as the robot. SSE events carry no sensitive data (state labels and progress percentages). Adds no meaningful security value for a cocktail robot on a private LAN.

## Risks / Trade-offs

- **Breaking HAL change** → All existing `DispenseHal` implementors (currently only stubs) must update. Mitigated by version bump to `0.3.0` and clear rename in docs.
- **Dual DispenseHal instances in main.rs** → Stub is duplicated for the SSE read path. Acceptable for a stub; production implementors will need interior mutability. Document clearly in `main.rs` TODO.
- **Deterministic job_id collision** → Two orders with identical names within 100 ms → duplicate `job_id`. Extremely unlikely in practice (human UI latency). If it occurs, the second job is still created with the same ID; HAL may detect and suffix with queue position.
- **Embassy time availability** → `embassy_time::Instant::now()` requires the time driver to be initialised. In the spin executor stub, this is initialised automatically. On ESP32, `esp_hal_embassy::init()` must be called before any `Instant` use.
