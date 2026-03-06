## Context

The robot is a headless device. There is no physical interface for configuration —
the admin API is the only control surface. All configuration must survive power-off
and power-save cycles.

The current design stores `RobotConfig` (which includes `Capabilities`) in flash,
conflating admin-owned data with hardware-fixed properties. `StorageHal` exposes a
raw load/store API with an `overwrite` flag whose semantics are undefined. A
separate `reload-config` endpoint exists to copy flash → RAM, creating a two-step
workflow that has no safety guarantees around in-flight jobs.

On first boot there is no defined robot state for "no config yet", meaning the robot
would expose dispense endpoints that silently fail.

## Goals / Non-Goals

**Goals:**
- Separate admin-owned config (`AdminConfig`) from hardware-fixed properties
  (`Capabilities`); `version` belongs in `Capabilities`.
- `GET /config` continues to return the merged view (same client-facing shape).
- `PATCH /config` auto-persists and is safe to call at any time (pre-flight flush).
- Replace raw flash load/store with semantically named backup/restore endpoints.
- Add `Provisioning` robot state for first-boot / factory-reset situations.
- Ensure config mutations never corrupt in-flight dispensing jobs.
- Bump crate to v0.5.0 (breaking HAL changes).

**Non-Goals:**
- Implementing real NVS flash on ESP32 (stub remains; that is a separate task).
- Adding a factory-reset endpoint.
- Supporting partial config updates (PATCH replaces the full `AdminConfig`).
- Versioned config migration (schema compatibility across firmware versions is out
  of scope for now).

## Decisions

### D1 — Split `RobotConfig` into `AdminConfig` + `Capabilities`

`RobotConfig` is split. `Capabilities` is returned solely by the hardware
implementation (it is a method, not a stored value). `GET /config` merges both for
the response. `StorageHal` and `ConfigHal` operate exclusively on `AdminConfig`.

**Why not keep them merged?** Storing capabilities in flash creates a divergence
risk: if firmware changes the hardware capability, the stored value is stale and
misleading. Capabilities are a property of the binary, not of the admin.

### D2 — `StorageHal` operates on `AdminConfig`, methods renamed to `backup`/`restore`

```rust
pub trait StorageHal {
    async fn backup(&self) -> Result<BackupPayload, ErrorInfo>;
    async fn restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>;
}

pub struct BackupPayload {
    pub data: AdminConfig,
    pub checksum: String,   // CRC32 hex of serialised AdminConfig
    pub backed_up_at: String, // ISO 8601 UTC timestamp
}
```

`backup` returns a `BackupPayload` the admin saves locally. `restore` accepts an
`AdminConfig`, writes it to flash, and activates it in RAM.

**Why rename?** `load`/`store` imply raw I/O; `backup`/`restore` communicate intent
(disaster recovery, provisioning) and make it clear this is an admin operation, not
a normal config read.

**Why remove `overwrite`?** The only meaningful protection it offered was accidental
first-write; the `Provisioning` state already handles that by restricting access
until a restore has been completed.

### D3 — Pre-flight for all config mutations: flush queue + wait for running job

Both `PATCH /config` and `POST /config/restore` execute an identical pre-flight
before applying changes:

1. Cancel all `Queued` jobs (emit SSE `job_update` events with state `cancelled`).
2. Wait for any `Running` job to reach a terminal state.
3. Apply config change (write RAM + flash).

This logic lives in a single helper used by both handlers, keeping the behaviour
consistent and testable.

**Why flush instead of reject?** On a headless device there is no operator to drain
the queue manually. Flushing is the only reliable way to ensure the restore
completes in bounded time without requiring a reboot.

**Why wait for the running job?** Stopping a dispense mid-pour risks spills and
inconsistent fill. One-job wait is bounded and safe.

### D4 — `Provisioning` robot state

`RobotState::Provisioning` is added. The robot enters this state on boot when
`StorageHal::backup()` returns an error (empty flash, checksum mismatch). It exits
to `Idle` after a successful `POST /config/restore`.

In `Provisioning`, the server rejects all non-admin endpoints with `503 Service
Unavailable` and a descriptive error. Admin endpoints (`GET /config`,
`PATCH /config`, `GET /config/backup`, `POST /config/restore`, `GET /status`) remain
active.

**Why a named state rather than an error?** It allows clients (and SSE consumers) to
distinguish "robot is broken" from "robot needs provisioning". The `GET /status`
response already returns the state field.

### D5 — `ControlHal::reload_config` removed

The two-step workflow (write flash, then reload) is replaced by restore-activates-
immediately. Removing `reload_config` eliminates a footgun: calling it after a
partial PATCH would silently discard unsaved RAM state.

**Alternative considered:** Keep `reload_config` as a no-op for backwards
compatibility. Rejected — it would leave a dead endpoint in the API that confuses
implementors.

### D6 — Checksum is CRC32 hex of the JSON-serialised `AdminConfig`

Simple, deterministic, no external dependency. The checksum is computed by the
server on write and verified on restore. A mismatch returns HTTP 422 with a
descriptive error.

**Why not SHA256?** CRC32 is sufficient for detecting accidental corruption of a
small config blob. Cryptographic integrity is already provided by the TLS/bearer-
token layer (out of scope for this change).

## Risks / Trade-offs

- **No config migration** → If firmware v2 adds a required `AdminConfig` field,
  restoring a v1 backup will fail deserialisation. Mitigation: document the backup
  format version in `AdminConfig::version` field (TBD in a future change) and use
  `#[serde(default)]` on new optional fields where safe.

- **Flush-on-mutate surprises admins** → An admin PATCHing a single field (e.g.
  token) will silently cancel queued jobs. Mitigation: document clearly in API.yaml
  and return the list of cancelled job IDs in the response body.

- **Single running-job wait is unbounded in theory** → A stuck job could block
  config changes indefinitely. Mitigation: DispenseHal implementations should
  enforce job timeouts (out of scope for this change); the admin can force-reboot as
  a last resort.

- **`ConfigHal` and `StorageHal` are always in sync after a write** → There is no
  staging workflow. An admin cannot preview a config before activating it.
  Mitigation: backup before mutating is the recommended workflow; document it.

## Migration Plan

1. Update `src/hal/mod.rs`: add `AdminConfig`, `BackupPayload`; update `Capabilities`
   with `version`; update `StorageHal` trait; add `RobotState::Provisioning`;
   remove `ControlHal::reload_config`.
2. Update `src/server/handlers/config.rs`: replace storage handlers with
   backup/restore; extract pre-flight helper; add provisioning gate.
3. Update `src/server/mod.rs`: update route table; pass provisioning gate to all
   non-admin handlers.
4. Update `src/esp32/storage.rs` stub to new trait.
5. Update `src/main.rs` stubs.
6. Update `API.yaml`: new routes, updated schemas.
7. Run `cargo fmt` and `cargo check`.
8. Bump `Cargo.toml` version to `0.5.0`.

No database migration or network-layer change required. Flash format changes are
handled by the stub (not yet real); real NVS migration is deferred.
