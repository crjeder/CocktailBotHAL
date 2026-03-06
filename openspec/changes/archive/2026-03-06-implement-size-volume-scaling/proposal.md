## Why

The dispense handler currently ignores the `size` field from the job-create request
body — the scaling from recipe ratios to actual dispenser volumes is a TODO stub.
The scaling model has also been clarified: `GlassType.volume` is an abstract volume
unit (not necessarily ml), `max_total_parts` is redundant once the total is always
`glass.volume` by construction, and the HAL must receive pre-computed per-ingredient
volumes rather than raw ratio parts.

## What Changes

- **BREAKING** `GlassType.volume_ml` renamed to `GlassType.volume` (unit is
  operator-defined, consistent across all volume values in one robot)
- **BREAKING** `AdminConfig.max_total_parts` and `RobotConfig.max_total_parts`
  removed; the glass volume is the implicit cap
- **BREAKING** `DispenseHal::create_job` signature changes: `items: Vec<JobItem>`
  replaced by `items: Vec<DispenseItem>` where `DispenseItem { liquid_id, amount: f32 }`
  carries the pre-computed per-ingredient volume in dispenser units
- New type `DispenseItem { liquid_id: String, amount: f32 }` added to `hal/mod.rs`
- `handle_create_job` gains a `config: &RobotConfig` parameter; implements full
  normalization: `size` lookup → `volume`, `amount_i = (r_i / Σr) × volume`
- Returns HTTP 422 if the requested `size` is not found in `glass_types`
- `API.yaml` updated: `volume_ml` → `volume`, `max_total_parts` removed from both
  `AdminConfig` and `RobotConfig` schemas
- Mock and test fixtures updated throughout

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `cocktail-sizing`: rename `GlassType.volume_ml` → `GlassType.volume`; remove
  `max_total_parts`; clarify normalization model — `amount = (r_i / Σr) × glass.volume`
  where `glass.volume` is an abstract consistent volume unit, not necessarily ml;
  HAL receives `Vec<DispenseItem>` with pre-computed `amount: f32` per ingredient
- `job-queue`: `DispenseHal::create_job` receives `Vec<DispenseItem>` instead of
  `Vec<JobItem>`; server handler pre-fetches `RobotConfig` and passes it to handler
  for size resolution

## Impact

- `src/hal/mod.rs` — new type `DispenseItem`; remove `max_total_parts` from
  `AdminConfig`/`RobotConfig`; rename field in `GlassType`; update `DispenseHal`
  trait signature (**breaking** — bumps semver minor/major)
- `src/hal/mock.rs` — update `MockDispenseHal::create_job`, `test_robot_config`,
  `test_admin_config`
- `src/server/handlers/dispense.rs` — implement normalization, update signature
- `src/server/mod.rs` — pre-fetch config, pass to `handle_create_job`
- `src/main.rs`, `src/storage/ram.rs` — remove `max_total_parts`, rename field
- `API.yaml` — schema updates for `GlassType`, `AdminConfig`, `RobotConfig`
