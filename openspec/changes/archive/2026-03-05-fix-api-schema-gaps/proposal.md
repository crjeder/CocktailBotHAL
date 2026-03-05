## Why

The `API.yaml` schemas and the Rust types in `src/hal/mod.rs` have diverged:
fields are missing, one field was replaced by a runtime-computed value, a
calibration model was over-specified for a single hardware paradigm, and minor
structural inconsistencies (a single-field nesting, an unreachable enum variant)
add noise. Fixing this now, before StorageHal and SSE are implemented, avoids
compounding the mismatch.

## What Changes

- **BREAKING** Remove `RobotConfig.part_ml` — cocktail volume is now determined
  at job-creation time from `glass_types` config + the requested `size`; `part_ml`
  is a computed local variable, not stored config.
- **BREAKING** Remove `RobotConfig.max_channels_per_job` — `Capabilities.simultaneous_channels`
  already covers this constraint.
- **BREAKING** Replace `LiquidCalibration` (three pump-specific fields) with a
  single `factor: f32` — a hardware-agnostic multiplier the HAL implementation
  uses to compensate for liquid density/viscosity differences.
- **BREAKING** Remove `limits` nesting from `Config` schema — `max_total_parts`
  moves to a top-level field on `Config`, matching the flat Rust struct.
- **BREAKING** Remove `booting` from the `State` enum — the HTTP server does not
  respond during boot; this state is unreachable by any API client.
- Add `glass_types: Vec<GlassType>` to `RobotConfig` — each entry maps a size
  name (`short`, `medium`, `long`) to a volume in ml; drives per-job scaling.
- Add `size` (required, enum `short | medium | long`) to `JobCreateRequest` —
  the client selects cocktail size; the server resolves volume from `glass_types`.
- Remove `require_glass` and `timeout` from `DispenseHal::create_job` — these
  are server policy, not client-supplied per-job parameters.

## Capabilities

### New Capabilities

- `cocktail-sizing`: Glass-size-driven volume scaling — config defines available
  glass types with volumes; job requests specify a size; the server computes
  `part_ml = glass_volume / total_parts` and scales each ingredient accordingly.

### Modified Capabilities

- `job-queue`: `JobCreateRequest` gains a required `size` field and loses
  `require_glass`/`timeout`; the job-creation contract changes.

## Impact

- `API.yaml`: `Config`, `LiquidCalibration`, `State`, `JobCreateRequest` schemas all change.
- `src/hal/mod.rs`: `RobotConfig`, `LiquidCalibration`, `DispenseHal::create_job`
  signature change. **Semver: breaking change → bump to v0.3.x or v0.4.0.**
- `src/esp32/`: All stub implementations of `DispenseHal` must be updated to
  match the new `create_job` signature.
- `src/server/handlers/dispense.rs`: Job-creation handler must read `size`,
  resolve `glass_types`, compute scaled ml amounts before calling HAL.
