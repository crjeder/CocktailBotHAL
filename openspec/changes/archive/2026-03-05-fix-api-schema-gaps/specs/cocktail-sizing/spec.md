## ADDED Requirements

### Requirement: Glass type configuration
`RobotConfig` SHALL include a field `glass_types: Vec<GlassType>` where
`GlassType` is a struct with `id: String` and `volume_ml: f32`.
The `id` SHALL correspond to a cocktail size name (e.g. `"short"`, `"medium"`,
`"long"`). At least one glass type MUST be present for the robot to accept
dispensing jobs.

#### Scenario: Config contains glass types
- **WHEN** `ConfigHal::get_active_config()` is called
- **THEN** `RobotConfig.glass_types` is a non-empty list of `GlassType` entries,
  each with a non-empty `id` and a positive `volume_ml`

#### Scenario: Glass type lookup by id
- **WHEN** a job is created with `size = "long"`
- **THEN** the server resolves `volume_ml` from the `glass_types` entry whose `id == "long"`

### Requirement: Job size field drives volume resolution
`JobCreateRequest` SHALL include a required `size` field with enum values
`short | medium | long`. The server SHALL resolve the target glass volume by
looking up `size` in `RobotConfig.glass_types`. If no matching glass type
exists in config, the server SHALL reject the job with HTTP 422.

#### Scenario: Valid size resolves to volume
- **WHEN** a job is submitted with `size = "medium"` and config has
  `{id: "medium", volume_ml: 150.0}`
- **THEN** the server uses `150.0` ml as the total cocktail volume

#### Scenario: Unknown size rejected
- **WHEN** a job is submitted with a `size` value that has no matching entry
  in `glass_types`
- **THEN** the server returns HTTP 422 with an appropriate error message

### Requirement: Per-job part_ml computation
The server SHALL compute `part_ml = glass_volume_ml / total_parts` at
job-creation time, where `total_parts` is the sum of all `JobItem.parts`
values. This value SHALL NOT be stored in config or persisted. Each ingredient
volume in ml is `item.parts * part_ml`.

#### Scenario: Ratio is preserved
- **WHEN** a job is created with items `[{gin, 1}, {tonic, 3}]` and
  `size = "long"` resolving to `200ml`
- **THEN** `part_ml = 50.0`, gin receives `50ml`, tonic receives `150ml`

#### Scenario: Ratio scales with glass size
- **WHEN** the same items are submitted with `size = "short"` resolving to `100ml`
- **THEN** gin receives `25ml`, tonic receives `75ml`, ratio 1:3 is preserved

### Requirement: Liquid calibration factor
`LiquidCalibration` SHALL contain exactly one field: `factor: f32`.
This factor is a hardware-agnostic multiplier that the HAL implementation uses
to compensate for liquid-specific properties (density, viscosity, flow rate).
The interpretation of `factor` is opaque to the API layer and is defined by
each HAL implementation.

#### Scenario: Calibration factor is per-liquid
- **WHEN** `RobotConfig` is retrieved
- **THEN** each `LiquidConfig.calibration.factor` is an independent `f32` value

#### Scenario: Default factor has no effect
- **WHEN** `calibration.factor == 1.0`
- **THEN** the HAL delivers exactly the computed ml amount without adjustment
