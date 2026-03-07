## MODIFIED Requirements

### Requirement: AdminConfig type
The system SHALL define an `AdminConfig` struct containing the admin-owned
configuration fields: `token: String`, `liquids: Vec<LiquidConfig>`,
`glass_types: Vec<GlassType>`, `admin_password: String`,
`glass_wait_timeout_secs: u32`, and `drink_ready_timeout_secs: u32`.
`AdminConfig` is the type persisted to non-volatile storage and passed to
`StorageHal::restore` and `ConfigHal::update_admin_config`.

`glass_wait_timeout_secs` SHALL default to `60` when absent from stored config
(backward-compatible via `#[serde(default)]`).
`drink_ready_timeout_secs` SHALL default to `300` when absent from stored config.
A value of `0` for either field means "wait indefinitely."

#### Scenario: AdminConfig contains all admin-owned fields
- **WHEN** an `AdminConfig` value is constructed
- **THEN** it includes `token`, `liquids`, `glass_types`, `admin_password`,
  `glass_wait_timeout_secs`, and `drink_ready_timeout_secs`

#### Scenario: AdminConfig does not contain Capabilities
- **WHEN** `AdminConfig` is serialized to JSON
- **THEN** the output does not include a `capabilities` key

#### Scenario: Timeout fields default on missing config
- **WHEN** a stored `AdminConfig` JSON without `glass_wait_timeout_secs` or `drink_ready_timeout_secs` is deserialized
- **THEN** `glass_wait_timeout_secs` is `60` and `drink_ready_timeout_secs` is `300`

#### Scenario: Zero timeout means wait indefinitely
- **WHEN** `glass_wait_timeout_secs` is `0`
- **THEN** the robot waits for glass placement without ever cancelling the job due to timeout

### Requirement: GET /config returns merged view
`GET /config` SHALL return a JSON object merging `AdminConfig` and `Capabilities`
into a single flat response. Clients SHALL NOT be required to call separate
endpoints for admin config and capabilities.

#### Scenario: GET /config response includes both admin and hardware fields
- **WHEN** a client calls `GET /config`
- **THEN** the response body contains `liquids`, `glass_types`, `token`,
  `glass_wait_timeout_secs`, `drink_ready_timeout_secs`, and `capabilities`
  (containing `version`, `level_reporting`, `glass_typing`, `simultaneous_channels`,
  `max_queue_depth`, `has_cancel_button`, `has_power_button`)

### Requirement: Capabilities contains version
The `Capabilities` struct SHALL include: `version: String`, `level_reporting: LevelReporting`,
`glass_typing: bool`, `simultaneous_channels: u8`, `max_queue_depth: u8`,
`has_cancel_button: bool`, and `has_power_button: bool`.

`has_cancel_button`: `true` if the hardware has a physical button that cancels
the active job when pressed. Clients SHOULD hide the software cancel button when
`true` and show it when `false`.

`has_power_button`: `true` if the hardware has a physical button that powers
off the robot when held. This is a separate capability from `has_cancel_button`
even if implemented on the same physical button (long vs. short press).

#### Scenario: Capabilities includes all fields
- **WHEN** the hardware implementation provides its `Capabilities`
- **THEN** all seven fields are present: `version`, `level_reporting`, `glass_typing`,
  `simultaneous_channels`, `max_queue_depth`, `has_cancel_button`, `has_power_button`

#### Scenario: has_cancel_button false prompts UI cancel button
- **WHEN** a client reads `capabilities.has_cancel_button == false`
- **THEN** the client MAY display a software cancel button to allow job cancellation

#### Scenario: version is not admin-configurable
- **WHEN** an admin submits a `PATCH /config` or `POST /config/restore` request
- **THEN** the `version` field in `Capabilities` is unchanged
