## Why

The robot has no physical UX — every configuration action goes through the admin
API. Config must survive power-off and power-save cycles, yet the current
`StorageHal` trait conflates admin-owned settings with hardware-fixed capabilities,
and the storage/RAM split exposes unnecessary complexity to API consumers. The
design needs to reflect the actual ownership model: hardware determines
capabilities, the admin configures everything else.

## What Changes

- **BREAKING** Split `RobotConfig` into `AdminConfig` (admin-owned, persisted) and
  `Capabilities` (hardware-fixed, read-only). Move `version` into `Capabilities`.
- **BREAKING** Replace `StorageHal::load_storage_config` / `store_storage_config`
  with `backup` / `restore` methods operating on `AdminConfig`.
- **BREAKING** Remove `ControlHal::reload_config` — restore activates immediately,
  making a separate reload step redundant.
- Add a `Provisioning` robot state: entered on first boot when no config exists in
  flash; only admin API endpoints are active until a successful restore.
- Replace `GET /storage/config` + `POST /storage/config` with
  `GET /config/backup` and `POST /config/restore`.
- Both `PATCH /config` and `POST /config/restore` auto-persist to flash and share
  the same pre-flight: flush the job queue (cancel all queued jobs), then wait for
  any running job to finish before applying.
- Backup payload includes a checksum and timestamp; no `valid` field.
- Remove the `overwrite` flag from the write path — restore always overwrites.

## Capabilities

### New Capabilities

- `admin-config`: Admin-owned configuration — `AdminConfig` type, backup/restore
  API, auto-persist on PATCH, provisioning state gate.

### Modified Capabilities

- `job-queue`: Queue flush behaviour on config mutation (pre-flight cancellation).
- `cocktail-sizing`: `RobotConfig` split affects how active config is composed and
  served; `GET /config` response shape changes.

## Impact

- `src/hal/mod.rs`: `RobotConfig` → `AdminConfig` + `Capabilities`; `version`
  moves to `Capabilities`; `StorageHal` trait methods renamed/retyped;
  `ControlHal::reload_config` removed; `RobotState::Provisioning` added.
- `src/server/handlers/config.rs`: handlers replaced for backup/restore endpoints.
- `src/server/mod.rs`: route table updated; provisioning state gate added.
- `src/esp32/storage.rs`: stub updated to new trait.
- `src/main.rs`: `StubStorageHal` updated; `StubControlHal` loses `reload_config`.
- `API.yaml`: routes and schemas updated to match new types and endpoints.
- Semver: breaking changes → v0.5.0.
