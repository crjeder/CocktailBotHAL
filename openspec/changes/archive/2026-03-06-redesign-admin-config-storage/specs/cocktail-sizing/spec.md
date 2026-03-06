## MODIFIED Requirements

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
