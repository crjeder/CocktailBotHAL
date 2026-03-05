## Context

`API.yaml` and `src/hal/mod.rs` are the dual source of truth for the HAL
contract. They have drifted: the Rust types contain fields not reflected in
the OpenAPI schema, the calibration model encodes pump-specific concerns, and
the `part_ml` config field conflates a runtime computation with stored config.
This change reconciles the two, establishing a clean, hardware-agnostic model
before StorageHal and SSE are built on top of it.

Current crate version: 0.3.0 (post async-hal-traits bump). All changes here
are breaking → bump to **v0.4.0**.

## Goals / Non-Goals

**Goals:**
- `API.yaml` schemas match Rust types exactly, with no missing or phantom fields
- `LiquidCalibration` is a single opaque factor — hardware-agnostic
- Cocktail volume is fully determined by glass type config + requested size
- `DispenseHal::create_job` signature reflects only what the client supplies
- All ESP32 stubs and server handlers compile after the type changes

**Non-Goals:**
- Implementing the glass-sensor-based glass validation flow (future work)
- Implementing the actual volume-scaling logic in the dispense handler (scoped
  here only to the type signature; full scaling logic is follow-on work)
- Changing `StorageHal` or SSE

## Decisions

### 1. `part_ml` becomes a local computation, not stored config

`part_ml = glass_volume_ml / total_parts` is computed in the dispense handler
at job-creation time. It is never stored or serialised. Storing it would
create a redundancy that can go stale; deriving it from `glass_types` and the
job's `size` keeps the config DRY.

*Alternative considered:* Keep `part_ml` as a fallback for when `glass_typing`
is false. Rejected — `glass_types` already provides the authoritative volume
for each named size. The client selects the size; the robot looks it up.

### 2. `LiquidCalibration` collapses to `{ factor: f32 }`

The previous three fields (`ml_per_sec`, `prime_ms`, `viscosity_factor`) were
specific to time-and-pump-based dispensing. The HAL must also support weight-
based and flow-based implementations. A single dimensionless `factor` is all
the API needs to carry — each HAL implementation interprets it according to its
own dispensing model (density g/ml for weight-based, a time multiplier for
time-based, etc.).

*Alternative considered:* Keep the three fields and add more for other modalities.
Rejected — the API would become a union of all possible hardware parameters.

### 3. `JobCreateRequest.size` (required enum) drives volume resolution

The client declares intent (`short | medium | long`). The server resolves the
target volume by looking up `size` in `config.glass_types`. This keeps
volume-resolution logic fully server-side and allows the config to define
non-standard sizes in the future without an API change (just add a new enum
value or, later, make it a free string).

*Alternative considered:* Client sends target volume in ml directly. Rejected —
exposes implementation detail; clients should not need to know glass volumes.

### 4. Remove `require_glass` and `timeout` from the HAL trait signature

Both are server policy, not per-job client parameters. The robot always waits
for a glass to be present before dispensing (`require_glass` is effectively
always `true`). Timeout policy belongs in `RobotConfig` or compile-time
constants — not in the public API surface.

### 5. Remove `booting` from `State` enum

The HTTP server is not running during boot. No client can ever receive this
state value. Keeping it in the API schema would be misleading.

### 6. Flatten `limits` — `max_total_parts` moves to top-level `Config`

The `limits` object had exactly one field. It added a nesting level with no
organisational value. Flattening matches the Rust struct and eliminates the
projection mismatch between serialised API responses and the `RobotConfig`
type.

## Risks / Trade-offs

- **Breaking change to public HAL contract** → Semver bump to v0.4.0; update
  `Cargo.toml` and document in commit message.
- **ESP32 stubs must be updated** → `src/esp32/` contains stub impls of
  `DispenseHal`; `create_job` signature changes. Low risk — stubs are trivial,
  but forgetting them causes a compile error.
- **Dispense handler scaling logic is deferred** → The handler will have the
  correct types after this change but the actual `part_ml` computation and
  per-liquid ml scaling are follow-on work. A `TODO` comment must be left at
  the computation site.

## Open Questions

- Should `size` be a fixed enum (`short | medium | long`) or a free string
  that must match an `id` in `glass_types`? Using a fixed enum is safer for
  now; a free string allows custom sizes but requires validation. **Decision
  deferred — use fixed enum for this change.**
