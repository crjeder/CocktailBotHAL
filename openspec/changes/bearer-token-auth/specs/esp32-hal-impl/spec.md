## MODIFIED Requirements

### Requirement: ConfigHal stub implementation
`Esp32Hal` SHALL implement `ConfigHal`. `get_active_config()` SHALL return a
default `RobotConfig` with an empty liquids list and `token` set to an empty
string. `update_active_config()` SHALL return `Ok(())`.

#### Scenario: get_active_config returns default config
- **WHEN** `get_active_config()` is called
- **THEN** a `RobotConfig` is returned with `liquids` as an empty Vec and
  `token` as an empty string

#### Scenario: update_active_config returns Ok
- **WHEN** `update_active_config(cfg)` is called with any config
- **THEN** the method returns `Ok(())`
