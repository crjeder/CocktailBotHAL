## 1. HAL Types (src/hal/mod.rs)

- [x] 1.1 Add `AdminConfig` struct with fields: `token`, `liquids`, `glass_types`, `max_total_parts`; derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- [x] 1.2 Move `version: String` from `RobotConfig` into `Capabilities`
- [x] 1.3 Remove `version` and `capabilities`-embedding from `RobotConfig`; keep `RobotConfig` as the merged API-facing type (AdminConfig fields + capabilities field)
- [x] 1.4 Add `BackupPayload` struct: `data: AdminConfig`, `checksum: String`, `backed_up_at: String`; derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- [x] 1.5 Replace `StorageHal::load_storage_config` / `store_storage_config` with `backup(&self) -> Result<BackupPayload, ErrorInfo>` and `restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>`
- [x] 1.6 Remove `ControlHal::reload_config`
- [x] 1.7 Add `RobotState::Provisioning` variant
- [x] 1.8 Update `ConfigHal::update_active_config` to accept `AdminConfig` instead of `RobotConfig`

## 2. ESP32 and Main Stubs

- [x] 2.1 Update `src/esp32/storage.rs`: implement new `StorageHal` trait (`backup`, `restore`)
- [x] 2.2 Update `src/esp32/mod.rs`: remove `reload_config` delegation; update `StorageHal` impl
- [x] 2.3 Update `src/main.rs` `StubStorageHal`: implement new `backup`/`restore` methods (RAM-backed: store `Option<AdminConfig>`)
- [x] 2.4 Update `src/main.rs` `StubControlHal`: remove `reload_config`
- [x] 2.5 Update `src/main.rs` `StubConfigHal::update_active_config` to accept `AdminConfig`

## 3. Server Handlers (src/server/handlers/config.rs)

- [x] 3.1 Remove `handle_storage_read` and `handle_storage_write`
- [x] 3.2 Add `handle_backup`: calls `StorageHal::backup()`, returns `BackupPayload` as JSON
- [x] 3.3 Add `handle_restore`: parses `{ data: AdminConfig, checksum: String }`, verifies CRC32 checksum (422 on mismatch), runs pre-flight, calls `StorageHal::restore` + `ConfigHal::update_admin_config`, transitions state if provisioning
- [x] 3.4 Update `handle_config_patch`: accept `AdminConfig` body, run pre-flight before applying
- [x] 3.5 Extract pre-flight helper (`flush_queue_and_wait`): cancels all queued jobs via `DispenseHal`, waits for running job; returns `Vec<String>` of cancelled job IDs

## 4. Server Routing (src/server/mod.rs)

- [x] 4.1 Replace routes `GET /storage/config` and `POST /storage/config` with `GET /config/backup` and `POST /config/restore`
- [x] 4.2 Remove route `POST /control/reload-config`
- [x] 4.3 Add provisioning gate: check `StatusHal::state()` before dispatching; return `503` for non-admin routes when state is `Provisioning`
- [x] 4.4 Update handler call sites to pass `DispenseHal` generic to config handlers (needed for pre-flight)

## 5. API Specification (API.yaml)

- [x] 5.1 Add `AdminConfig` schema (token, liquids, glass_types, max_total_parts); update `Config` schema to reference it plus capabilities
- [x] 5.2 Add `version` field to `Capabilities` schema; remove `version` from top-level `Config`
- [x] 5.3 Add `BackupPayload` schema: `data: AdminConfig`, `checksum: string`, `backed_up_at: string`
- [x] 5.4 Replace `GET /storage/config` with `GET /config/backup` returning `BackupPayload`
- [x] 5.5 Replace `POST /storage/config` with `POST /config/restore` accepting `{ data: AdminConfig, checksum: string }`; response includes `cancelled_job_ids`
- [x] 5.6 Remove `POST /control/reload-config` path
- [x] 5.7 Update `PATCH /config` request body to use `AdminConfig`; update response to include `cancelled_job_ids`
- [x] 5.8 Add `provisioning` to the `State` enum

## 6. Validation

- [x] 6.1 Run `cargo check` with no features and with `--features esp32`; fix all errors
- [x] 6.2 Run `cargo fmt` and verify no diff
- [x] 6.3 Bump `Cargo.toml` version to `0.5.0`
