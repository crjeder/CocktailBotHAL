## MODIFIED Requirements

### Requirement: Glass type configuration
`AdminConfig` SHALL include a `glass_types: Vec<GlassType>` field where each
`GlassType` has an `id: String` (e.g. `"short"`, `"medium"`, `"long"`) and a
`volume: f32`. The `volume` field is an abstract, operator-defined volume quantity;
its unit (ml, cl, oz, etc.) is not specified by the protocol — it MUST be
consistent across all `glass_types` entries and all `LiquidCalibration.factor`
values within a single robot. The active glass types determine the available
cocktail size targets used during job execution. `glass_types` is
admin-configurable and persisted to non-volatile storage as part of `AdminConfig`.

#### Scenario: Glass types are present in active config
- **WHEN** `GET /config` is called
- **THEN** the response includes a `glass_types` array with at least one entry,
  each having `id` and `volume` fields

#### Scenario: Glass types can be updated via PATCH
- **WHEN** `PATCH /config` is called with an updated `glass_types` array
- **THEN** subsequent `GET /config` responses reflect the new glass types

#### Scenario: Glass types survive power cycle
- **WHEN** `PATCH /config` is called, then the robot is power-cycled
- **THEN** `GET /config` after reboot returns the same `glass_types`

### Requirement: Ingredient volume normalization
The server layer SHALL compute each ingredient's absolute dispenser volume as:

```
amount_i = (r_i / Σr) × glass.volume
```

where `r_i` is the client-supplied `parts` value for ingredient `i`, `Σr` is the
sum of all `parts` values in the recipe, and `glass.volume` is the target volume
for the requested size looked up from `AdminConfig.glass_types`. The result
`amount_i` is a `f32` in the same abstract volume unit as `glass.volume`. The
server SHALL pass `Vec<DispenseItem>` (not `Vec<JobItem>`) to
`DispenseHal::create_job`; each `DispenseItem` carries `liquid_id: String` and
`amount: f32`. The HAL is responsible for applying `LiquidCalibration.factor` to
convert from abstract volume to hardware commands.

#### Scenario: Volumes sum to glass.volume
- **GIVEN** a recipe where ingredients have `r_parts` summing to `Σr`
- **WHEN** a job is created for a glass with `volume = V`
- **THEN** the sum of all `DispenseItem.amount` values equals `V`
  (within f32 precision)

#### Scenario: Proportions are preserved
- **GIVEN** a two-ingredient recipe with `parts = [1, 3]`
- **WHEN** a job is created for a glass with `volume = 200`
- **THEN** the computed amounts are `50.0` and `150.0` respectively

#### Scenario: Empty items array is rejected
- **WHEN** `POST /v1/dispense/jobs` is called with an empty `items` array
- **THEN** the server returns HTTP 422

#### Scenario: Unknown size is rejected
- **WHEN** `POST /v1/dispense/jobs` is called with `size` not present in
  `AdminConfig.glass_types`
- **THEN** the server returns HTTP 422

### Requirement: Liquid calibration factor
`LiquidConfig` SHALL include a `calibration: LiquidCalibration` field where
`LiquidCalibration` has `factor: f32` (default `1.0`). The HAL SHALL apply the
factor when converting `DispenseItem.amount` (abstract volume units) to hardware
commands. The factor's physical interpretation is hardware-specific (e.g.,
seconds-per-unit for a pump, grams-per-unit for a scale).

#### Scenario: calibration factor defaults to 1.0
- **WHEN** a `LiquidConfig` is deserialized without a `calibration` field
- **THEN** `calibration.factor` defaults to `1.0`

#### Scenario: calibration factor is applied by the HAL
- **GIVEN** `calibration.factor = 0.95` for a channel
- **WHEN** the computed `amount` for that channel is `100.0`
- **THEN** the HAL uses `95.0` (in hardware units) for that channel

## REMOVED Requirements

### Requirement: max_total_parts safety limit
**Reason**: Redundant — by construction, `Σ(amount_i) = glass.volume`, so the
maximum total dispenser volume is bounded by the largest `GlassType.volume`
configured by the admin. A separate `max_total_parts` cap adds no protection
that `glass_types` configuration does not already provide.
**Migration**: Remove `max_total_parts` from `AdminConfig` and `RobotConfig`
structs and from `PATCH /config` / `GET /config` API payloads.

### Requirement: Job size field
**Reason**: `JobStatus.size_ml` was specced but never implemented. With the
volume unit now abstract (not necessarily ml), the field name is misleading.
A future `JobStatus.total_volume` field may be added when `JobStatus` is
extended, but it is out of scope for this change.
**Migration**: No API migration needed — the field was never in the wire format.
