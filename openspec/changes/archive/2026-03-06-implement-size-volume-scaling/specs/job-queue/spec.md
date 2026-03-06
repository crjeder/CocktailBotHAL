## MODIFIED Requirements

### Requirement: job-create signature
`DispenseHal::create_job` SHALL accept exactly four parameters:
1. `job_id: String` — server-generated unique identifier (see job_id generation requirement)
2. `name: String` — human-readable, non-unique label supplied by the client
3. `items: Vec<DispenseItem>` — pre-computed per-ingredient dispenser volumes;
   each `DispenseItem` has `liquid_id: String` and `amount: f32` in the operator's
   abstract volume unit
4. `parallel: bool` — whether channels may dispense simultaneously

The signature SHALL NOT include `require_glass`, `timeout`, `recipe`, `size_ml`,
or `channel_map` parameters. Glass detection is a pre-condition enforced by the
server layer before calling the HAL. Volume normalization is performed by the
server and passed in as pre-computed `DispenseItem` values.

#### Scenario: create_job accepts four arguments
- **WHEN** the server layer calls
  `create_job(job_id, name, items: Vec<DispenseItem>, parallel)`
- **THEN** the HAL accepts the call without additional parameters

#### Scenario: DispenseItem carries pre-computed volume
- **GIVEN** a recipe `{vodka: 2 parts, lime: 1 part}` for a glass of `volume = 90`
- **WHEN** the server builds the `DispenseItem` list
- **THEN** `items` contains `[{liquid_id: "vodka", amount: 60.0}, {liquid_id: "lime", amount: 30.0}]`
