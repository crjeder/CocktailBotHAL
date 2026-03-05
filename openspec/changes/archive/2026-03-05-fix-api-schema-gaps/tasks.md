## 1. Bump crate version

- [x] 1.1 Update `Cargo.toml` version to `0.4.0`

## 2. Update Rust types in `src/hal/mod.rs`

- [x] 2.1 Replace `LiquidCalibration` struct: remove `ml_per_sec`, `prime_ms`, `viscosity_factor`; add `factor: f32`
- [x] 2.2 Add `GlassType` struct with `id: String` and `volume_ml: f32`
- [x] 2.3 Update `RobotConfig`: remove `part_ml`, remove `max_channels_per_job`, add `glass_types: Vec<GlassType>`
- [x] 2.4 Remove `Booting` variant from `RobotState` enum
- [x] 2.5 Update `DispenseHal::create_job` signature: remove `require_glass: bool` and `timeout: Duration` parameters
- [x] 2.6 Remove unused `use core::time::Duration` import if no longer needed

## 3. Update ESP32 stubs in `src/esp32/`

- [x] 3.1 Update `DispenseHal::create_job` stub impl to match new 4-parameter signature
- [x] 3.2 Update any `RobotConfig` construction in stubs to use `glass_types` instead of `part_ml`/`max_channels_per_job`
- [x] 3.3 Update `LiquidCalibration` construction to use `factor` field

## 4. Update server dispense handler

- [x] 4.1 Update `JobCreateRequest` deserialization struct in the handler to include `size: String` field
- [x] 4.2 Update the `create_job` call site to pass only `job_id`, `name`, `items`, `parallel`
- [x] 4.3 Add a `TODO` comment at the job-creation site marking where `part_ml` computation and glass-type lookup will go

## 5. Update `API.yaml`

- [x] 5.1 Add `GlassType` schema (`id: string`, `volume_ml: number`)
- [x] 5.2 Update `LiquidCalibration` schema: replace all fields with `factor: number`
- [x] 5.3 Update `Config` schema: remove `part_ml`, remove `max_channels_per_job`, remove `limits` object, add `max_total_parts: integer` at top level, add `glass_types` array
- [x] 5.4 Remove `booting` from `State` enum
- [x] 5.5 Update `JobCreateRequest` schema: remove `require_glass` and `timeout`, add `size` as required enum `[short, medium, long]`

## 6. Update `src/main.rs` stubs

- [x] 6.1 Update any hardcoded `RobotConfig` or `LiquidCalibration` literals in `main.rs` to use the new field names

## 7. Verify and format

- [x] 7.1 Run `cargo check` — confirm zero errors and zero warnings
- [x] 7.2 Run `cargo fmt` — apply formatting
- [x] 7.3 Run `cargo check --features esp32` — confirm ESP32 feature also compiles clean
