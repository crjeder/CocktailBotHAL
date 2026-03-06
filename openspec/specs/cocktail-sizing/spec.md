## ADDED Requirements

### Requirement: Glass type configuration
`AdminConfig` SHALL include a `glass_types: Vec<GlassType>` field where each
`GlassType` has an `id: String` (e.g. `"short"`, `"medium"`, `"long"`) and a
`volume_ml: f32`. The active glass types determine the available cocktail size
targets used during job execution. `glass_types` is admin-configurable and
persisted to non-volatile storage as part of `AdminConfig`.

#### Scenario: Glass types are present in active config
- **WHEN** `GET /config` is called
- **THEN** the response includes a `glass_types` array with at least one entry

#### Scenario: Glass types can be updated via PATCH
- **WHEN** `PATCH /config` is called with an updated `glass_types` array
- **THEN** subsequent `GET /config` responses reflect the new glass types

#### Scenario: Glass types survive power cycle
- **WHEN** `PATCH /config` is called, then the robot is power-cycled
- **THEN** `GET /config` after reboot returns the same `glass_types`

### Requirement: Job size field
`JobStatus` SHALL include a `size_ml: u16` field indicating the total target volume
(in millilitres) for the job, computed at job-creation time from the active glass type
and the recipe's proportions.

#### Scenario: size_ml is set on job creation
- **WHEN** `DispenseHal::create_job` is called
- **THEN** the returned `JobCreated` and subsequent `JobStatus` both carry `size_ml > 0`

#### Scenario: size_ml reflects glass type at creation time
- **WHEN** a job is created with `glass_type = Large`
- **THEN** `size_ml` is greater than the `size_ml` of an equivalent job created with `glass_type = Small`

### Requirement: part_ml computation
The server layer SHALL compute each ingredient's absolute volume as:

```
part_ml = (ingredient.parts / total_parts) * size_ml
```

where `total_parts` is the sum of all `parts` values in the recipe, and `size_ml` is
the job's total target volume derived from the active glass type. Fractional millilitres
SHALL be rounded to the nearest integer (round-half-up).

#### Scenario: part_ml sums to size_ml
- **GIVEN** a recipe where ingredients sum to `total_parts`
- **WHEN** a job is created with `size_ml = S`
- **THEN** the sum of all computed `part_ml` values equals `S` (rounding tolerance: ±1 ml)

#### Scenario: proportions are preserved
- **GIVEN** a two-ingredient recipe with parts `[1, 3]`
- **WHEN** a job is created with `size_ml = 200`
- **THEN** the computed volumes are `50 ml` and `150 ml` respectively

### Requirement: Liquid calibration factor
`LiquidConfig` SHALL include a `calibration_factor: f32` field (default `1.0`).
The actual dispensed volume for a channel SHALL be `part_ml * calibration_factor`,
allowing per-channel compensation for pump flow-rate variance.

#### Scenario: calibration_factor defaults to 1.0
- **WHEN** a `LiquidConfig` is deserialized without a `calibration_factor` field
- **THEN** `calibration_factor` defaults to `1.0`

#### Scenario: calibration_factor scales dispensed volume
- **GIVEN** `calibration_factor = 0.95` for a channel
- **WHEN** the computed `part_ml` for that channel is `100`
- **THEN** the HAL is instructed to dispense `95 ml` for that channel
