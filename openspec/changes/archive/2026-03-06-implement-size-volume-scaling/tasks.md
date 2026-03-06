## 1. HAL type changes (src/hal/mod.rs)

- [x] 1.1 Rename `GlassType.volume_ml: f32` → `GlassType.volume: f32`; update doc comment to describe abstract volume unit
- [x] 1.2 Add `DispenseItem { liquid_id: String, amount: f32 }` struct with `Serialize + Deserialize + Debug + Clone`; add doc comment
- [x] 1.3 Remove `max_total_parts: u16` from `AdminConfig`
- [x] 1.4 Remove `max_total_parts: u16` from `RobotConfig`
- [x] 1.5 Change `DispenseHal::create_job` fourth parameter from `items: Vec<JobItem>` to `items: Vec<DispenseItem>`
- [x] 1.6 Bump crate version in `Cargo.toml` from `0.5.0` to `0.6.0`

## 2. Mock and test fixtures (src/hal/mock.rs)

- [x] 2.1 Import `DispenseItem` in `mock.rs`; remove `JobItem` from the import (if unused elsewhere in mock)
- [x] 2.2 Update `MockDispenseHal::create_job` signature: `_items: Vec<JobItem>` → `_items: Vec<DispenseItem>`
- [x] 2.3 Update `MockConfigHal::update_active_config`: remove `self.config.max_total_parts = cfg.max_total_parts;` line
- [x] 2.4 Update `test_robot_config()`: rename `volume_ml` → `volume`; remove `max_total_parts` field
- [x] 2.5 Update `test_admin_config()`: rename `volume_ml` → `volume`; remove `max_total_parts` field

## 3. Dispense handler (src/server/handlers/dispense.rs)

- [x] 3.1 Add `config: &RobotConfig` parameter to `handle_create_job`; import `RobotConfig` and `DispenseItem`
- [x] 3.2 Implement size lookup: find `config.glass_types` entry matching `body.size`; return HTTP 422 if not found
- [x] 3.3 Guard against empty `items`: return HTTP 422 `"items must not be empty"` before normalization
- [x] 3.4 Compute normalization: `total_r = items.iter().map(|i| i.parts).sum::<u32>() as f32`; `scale = glass.volume / total_r`; build `Vec<DispenseItem>` mapping each item to `amount = item.parts as f32 * scale`
- [x] 3.5 Pass the `Vec<DispenseItem>` to `dispense.create_job(...)` instead of the raw `body.items`
- [x] 3.6 Remove `let _ = &body.size;` stub
- [x] 3.7 Update existing test `dispense_create_job_hal_error_returns_500`: add `test_robot_config()` fixture, pass config to handler, use non-empty items so normalization reaches the HAL call
- [x] 3.8 Add test `dispense_create_job_unknown_size_returns_422`: send `size: "nonexistent"`, assert HTTP 422
- [x] 3.9 Add test `dispense_create_job_empty_items_returns_422`: send valid size, empty items, assert HTTP 422

## 4. Router wiring (src/server/mod.rs)

- [x] 4.1 In the `("POST", "/v1/dispense/jobs")` arm, pre-fetch config via `self.hal.config.get_active_config().await` and pass `&config` to `handle_create_job`

## 5. Static initializers

- [x] 5.1 `src/main.rs`: rename `volume_ml` → `volume` in all `GlassType` literals; remove `max_total_parts` from `RobotConfig` and `AdminConfig` literals
- [x] 5.2 `src/storage/ram.rs`: rename `volume_ml` → `volume` in `GlassType` literals; remove `max_total_parts` from `AdminConfig` default

## 6. API spec (API.yaml)

- [x] 6.1 Rename `GlassType.volume_ml` → `GlassType.volume`; update description to "Abstract dispenser volume in operator-defined units"
- [x] 6.2 Remove `max_total_parts` property from `AdminConfig` schema
- [x] 6.3 Remove `max_total_parts` property from `RobotConfig` schema

## 7. Verify and finish

- [x] 7.1 `cargo check` passes with zero errors (default features)
- [x] 7.2 `cargo check --features esp32` passes with zero errors
- [x] 7.3 `cargo test` passes (all handler tests green)
- [x] 7.4 `cargo fmt` — no formatting changes outstanding
