## MODIFIED Requirements

### Requirement: Crate version bumped to 0.2.0
Because the HAL trait interface is a public contract and changing method
signatures is a breaking change, the crate version in `Cargo.toml` SHALL be
updated from `0.5.0` to `0.6.0`.

#### Scenario: Version is 0.6.0
- **WHEN** `Cargo.toml` is inspected
- **THEN** the `[package]` section shows `version = "0.6.0"`

## ADDED Requirements

### Requirement: GlassWaitReason and RecoveryAction are HAL types
`GlassWaitReason` and `RecoveryAction` enums SHALL be defined in
`src/hal/mod.rs` and re-exported as part of the public HAL interface.
Both SHALL derive `Debug`, `Clone`, and `Serialize`.

#### Scenario: Types are publicly accessible
- **WHEN** a downstream crate imports from `cocktail_bot_hal::hal`
- **THEN** `GlassWaitReason` and `RecoveryAction` are accessible without additional imports
