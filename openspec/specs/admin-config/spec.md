## Requirements

### Requirement: AdminConfig type
The system SHALL define an `AdminConfig` struct containing the admin-owned
configuration fields: `token: String`, `liquids: Vec<LiquidConfig>`,
`glass_types: Vec<GlassType>`, and `max_total_parts: u16`.
`AdminConfig` is the type persisted to non-volatile storage and passed to
`StorageHal::restore` and `ConfigHal::update_admin_config`.

#### Scenario: AdminConfig contains all admin-owned fields
- **WHEN** an `AdminConfig` value is constructed
- **THEN** it includes `token`, `liquids`, `glass_types`, and `max_total_parts`

#### Scenario: AdminConfig does not contain Capabilities
- **WHEN** `AdminConfig` is serialized to JSON
- **THEN** the output does not include a `capabilities` key

### Requirement: Capabilities contains version
The `Capabilities` struct SHALL include a `version: String` field carrying
the hardware firmware version (e.g. `"1.5"`). `version` SHALL NOT be present
in `AdminConfig`.

#### Scenario: Capabilities includes version
- **WHEN** the hardware implementation provides its `Capabilities`
- **THEN** `Capabilities.version` is a non-empty string

#### Scenario: version is not admin-configurable
- **WHEN** an admin submits a `PATCH /config` or `POST /config/restore` request
- **THEN** the `version` field in `Capabilities` is unchanged

### Requirement: GET /config returns merged view
`GET /config` SHALL return a JSON object merging `AdminConfig` and `Capabilities`
into a single flat response. Clients SHALL NOT be required to call separate
endpoints for admin config and capabilities.

#### Scenario: GET /config response includes both admin and hardware fields
- **WHEN** a client calls `GET /config`
- **THEN** the response body contains `liquids`, `glass_types`, `max_total_parts`,
  `token`, and `capabilities` (containing `version`, `level_reporting`,
  `glass_typing`, `simultaneous_channels`, `max_queue_depth`)

### Requirement: PATCH /config auto-persists and pre-flights
`PATCH /config` SHALL:
1. Execute pre-flight (flush queued jobs, wait for running job to finish).
2. Apply the new `AdminConfig` to RAM.
3. Persist the new `AdminConfig` to non-volatile storage.
4. Respond `200 OK` with the cancelled job IDs (may be empty).

#### Scenario: PATCH /config with idle queue applies immediately
- **WHEN** `PATCH /config` is called and no jobs are queued or running
- **THEN** the config is updated in RAM and flash, and `200 OK` is returned

#### Scenario: PATCH /config flushes queued jobs
- **WHEN** `PATCH /config` is called and jobs are in the queue
- **THEN** all queued jobs are cancelled, the response body lists the cancelled
  job IDs, and the new config is applied

#### Scenario: PATCH /config waits for running job
- **WHEN** `PATCH /config` is called and a job is currently running
- **THEN** the request waits until the running job reaches a terminal state,
  then applies the config change

### Requirement: GET /config/backup returns BackupPayload
`GET /config/backup` SHALL return a `BackupPayload` containing:
- `data: AdminConfig` — the current admin-owned configuration
- `checksum: String` — CRC32 hex of the JSON-serialised `data`
- `backed_up_at: String` — ISO 8601 UTC timestamp of when the backup was created

#### Scenario: Backup payload includes checksum and timestamp
- **WHEN** `GET /config/backup` is called
- **THEN** the response includes `data`, `checksum`, and `backed_up_at`

#### Scenario: Checksum matches data
- **WHEN** a client re-serialises the returned `data` and computes its CRC32
- **THEN** the result matches the returned `checksum`

### Requirement: POST /config/restore validates and applies backup
`POST /config/restore` SHALL:
1. Accept `{ data: AdminConfig, checksum: String }`.
2. Verify the checksum matches the CRC32 of the serialised `data`; return
   `422 Unprocessable Entity` on mismatch.
3. Execute pre-flight (flush queued jobs, wait for running job).
4. Write `AdminConfig` to non-volatile storage and to RAM.
5. If the robot is in `Provisioning` state, transition to `Idle`.
6. Respond `200 OK`.

#### Scenario: Restore with valid checksum succeeds
- **WHEN** `POST /config/restore` is called with a valid `data` and matching `checksum`
- **THEN** the config is written to flash and RAM, and `200 OK` is returned

#### Scenario: Restore with invalid checksum is rejected
- **WHEN** `POST /config/restore` is called with a `checksum` that does not match `data`
- **THEN** the server returns `422 Unprocessable Entity` and does not modify config

#### Scenario: Restore exits Provisioning state
- **WHEN** the robot is in `Provisioning` state and `POST /config/restore` succeeds
- **THEN** the robot state transitions to `Idle`

### Requirement: Provisioning state on missing config
The robot SHALL enter `RobotState::Provisioning` on boot if `StorageHal::backup()`
returns an error (no stored config, or checksum mismatch on load).
In `Provisioning` state, all endpoints except the admin config endpoints and
`GET /status` SHALL return `503 Service Unavailable`.

#### Scenario: Robot enters Provisioning on first boot
- **WHEN** the robot boots and no valid config exists in non-volatile storage
- **THEN** `GET /status` returns `state: "provisioning"`

#### Scenario: Dispense endpoint blocked in Provisioning
- **WHEN** the robot is in `Provisioning` state and a client calls `POST /dispense/jobs`
- **THEN** the server returns `503 Service Unavailable`

#### Scenario: Admin endpoints available in Provisioning
- **WHEN** the robot is in `Provisioning` state
- **THEN** `GET /config`, `PATCH /config`, `GET /config/backup`, and
  `POST /config/restore` return normal responses

### Requirement: StorageHal trait — backup and restore
`StorageHal` SHALL define exactly two async methods:
- `backup(&self) -> Result<BackupPayload, ErrorInfo>` — reads stored `AdminConfig`
  from non-volatile storage and returns it wrapped in a `BackupPayload`.
- `restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>` — writes
  `AdminConfig` to non-volatile storage.

#### Scenario: StorageHal::backup returns stored config
- **WHEN** a config has been previously stored and `backup()` is called
- **THEN** `Ok(BackupPayload)` is returned with `data` matching the stored config

#### Scenario: StorageHal::backup errors on empty storage
- **WHEN** no config has been stored and `backup()` is called
- **THEN** `Err(ErrorInfo)` is returned

#### Scenario: StorageHal::restore writes config
- **WHEN** `restore(cfg)` is called
- **THEN** a subsequent `backup()` call returns `data` equal to `cfg`
