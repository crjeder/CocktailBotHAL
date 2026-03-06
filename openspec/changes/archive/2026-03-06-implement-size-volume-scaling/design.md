## Context

The dispense flow currently has a TODO stub where `body.size` is ignored and
`Vec<JobItem>` (recipe ratios) is passed directly to `DispenseHal::create_job`.
Hardware implementations need pre-computed volumes, not ratios. A design session
clarified the scaling model:

- Client recipe parts (`r_parts`) are **arbitrary ratios** — only their relative
  proportions matter.
- `GlassType.volume` is an **abstract, consistent volume unit** (could be ml, cl,
  oz — operator-defined). All glass volumes and calibration computations in one
  robot MUST use the same unit.
- The server normalises recipe ratios to dispenser volumes:
  `amount_i = (r_i / Σr) × glass.volume`
- The HAL receives pre-computed `amount: f32` per ingredient; it applies
  `LiquidCalibration.factor` internally to convert to hardware commands.
- `max_total_parts` is redundant: `Σ(amount_i) = glass.volume` by construction,
  so the glass configuration itself acts as the cap.

## Goals / Non-Goals

**Goals:**
- Implement the `size` → `volume` lookup and normalization in `handle_create_job`
- Introduce `DispenseItem { liquid_id, amount: f32 }` as the HAL-facing type
- Remove `max_total_parts` from `AdminConfig` and `RobotConfig`
- Rename `GlassType.volume_ml` → `GlassType.volume`
- Return HTTP 422 when the requested size is not in `glass_types`
- Keep `API.yaml` in sync
- Update mock and all test fixtures to compile and pass

**Non-Goals:**
- Glass-presence check before dispatching (separate concern, noted in TODO)
- Fractional `r_parts` from the client — `parts: u32` stays; clients multiply up
- Changing the `LiquidCalibration` model (factor stays as-is)
- Implementing StorageHal persistence

## Decisions

### D1: Two distinct types — `JobItem` (client) and `DispenseItem` (HAL)

`JobItem { liquid_id, parts: u32 }` is the deserialized client request body.
`DispenseItem { liquid_id, amount: f32 }` is what `DispenseHal::create_job` receives.

**Why not add `amount_ml: f32` to `JobItem`?** `JobItem` is `Deserialize` and
represents the wire format. Mixing client fields (`parts`) with server-computed
fields (`amount`) in one struct is confusing and leaks internal concerns into the
API type.

**Why not have the HAL do the math?** Every HAL implementation would need to
duplicate the normalization logic and access to `glass_types`. Keeping it in the
server layer means HAL authors only deal with physical units.

### D2: `handle_create_job` receives `config: &RobotConfig` (pre-fetched)

The router pre-fetches `config` via `ConfigHal::get_active_config()` and passes
the result to the handler. The handler signature becomes:

```rust
pub async fn handle_create_job<Disp: DispenseHal, W: Write + Unpin>(
    dispense: &mut Disp,
    config: &RobotConfig,
    request: &HttpRequest,
    socket: &mut W,
)
```

**Why not add `Cfg: ConfigHal` generic to the handler?** It would require
`handle_create_job` to call `get_active_config().await` itself, adding an await
point and a trait bound. Pre-fetching in the router is simpler, consistent with
other handlers (which already receive only the data they need), and makes the
handler easier to test (just pass a `&RobotConfig` directly).

### D3: `GlassType.volume` — no unit field

The operator is responsible for unit consistency across `glass_types` and
`LiquidCalibration.factor`. Adding an explicit unit enum would complicate the API
without enabling the robot to do anything different — the hardware always maps
`amount × factor` to its physical command regardless of the chosen unit.

### D4: `parts: u32` stays for the client type

Whole-number ratios (e.g., `2:1:1`) are sufficient for cocktail recipes in
practice. If a future recipe requires `1.5 oz`, clients multiply up (`3:2:2`).
Changing to `f32` adds floating-point edge cases to the API surface with no
immediate benefit.

### D5: Semver bump to 0.5.0 → 0.6.0

`DispenseHal::create_job`, `GlassType`, `AdminConfig`, and `RobotConfig` all
change in breaking ways. This warrants a minor-version bump under pre-1.0 semver
(`0.5.x → 0.6.0`).

## Risks / Trade-offs

- **Division by zero** — If `items` is empty, `Σr = 0` and the normalization
  divides by zero. Mitigation: return HTTP 422 `"items must not be empty"` before
  the normalization step.
- **f32 precision** — `amount` values are `f32`; rounding errors accumulate when
  the calibration factor multiplies them. For cocktail volumes this is acceptable
  (±0.1 ml at worst for typical recipes). Mitigation: document the precision in
  the `DispenseItem` doc comment.
- **Existing test fixture** — `dispense_create_job_hal_error_returns_500` currently
  passes an empty items array and no config. After this change the handler needs a
  config with a matching `size`. Mitigation: update the test to use
  `test_robot_config()` and a non-empty items array (or pass items=[] and expect 422).
- **Stored config migration** — `AdminConfig` serialized before this change
  contains `max_total_parts` and `volume_ml`. When the new firmware restores such
  a backup, serde will fail on unknown/missing fields unless `#[serde(default)]`
  or `#[serde(rename)]` is used. Mitigation: add `#[serde(default)]` to
  `max_total_parts` as an ignored field for one release, or document the migration
  step.

## Migration Plan

1. Apply all code changes (single PR).
2. Bump `Cargo.toml` version `0.5.0 → 0.6.0`.
3. API clients must rename `volume_ml` → `volume` in `GlassType` payloads and
   drop `max_total_parts` from any `PATCH /config` bodies.
4. Existing stored configs (RAM-only currently) are transient — no migration
   needed at this stage. When StorageHal is implemented, add a migration shim.
