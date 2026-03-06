## Context

`StorageHal` defines two async methods: `backup(&self) -> Result<BackupPayload, ErrorInfo>`
and `restore(&mut self, cfg: AdminConfig) -> Result<(), ErrorInfo>`. The only existing
implementations are `StubStorageHal` (panics) in `src/main.rs` and `Esp32Storage`
(returns `NOT_IMPLEMENTED`) in `src/esp32/storage.rs`. Neither is usable for
development.

The codebase targets `no_std` + `alloc` for ESP32 compatibility, so the RAM
implementation must avoid `std` types.

## Goals / Non-Goals

**Goals:**
- Provide a working `StorageHal` backed by `Option<AdminConfig>` in heap memory.
- Pre-seed the store with a reasonable default `AdminConfig` so development builds
  start in `Idle` state without a restore step.
- Compile cleanly under both `std` (spin executor) and `no_std + alloc` (ESP32).
- Replace `StubStorageHal` in `src/main.rs`.

**Non-Goals:**
- Persistence across process restarts or power cycles (this is a RAM store).
- Thread safety / multi-core access (embassy is single-executor per core).
- Replacing `Esp32Storage` (NVS flash implementation is a separate task).

## Decisions

### D1 — Store as `Option<AdminConfig>` behind a struct

```rust
pub struct RamStorageHal {
    stored: Option<AdminConfig>,
}
```

`backup()` returns `Err` when `stored` is `None` (simulates empty flash).
`restore()` sets `stored = Some(cfg)`.

**Why not `RefCell` or interior mutability?** `restore` takes `&mut self`, matching
the trait signature. Embassy's single-executor model means no concurrent mutation.

### D2 — Pre-seeded default at construction

`RamStorageHal::new()` initialises `stored` with a hardcoded `AdminConfig`
containing sensible development defaults (empty liquids list, default glass types,
token `"dev"`, empty `admin_password`). This means `StorageHal::backup()` succeeds
immediately and the robot starts in `Idle`, not `Provisioning`.

`RamStorageHal::empty()` initialises `stored = None` for tests that explicitly
want to exercise the `Provisioning` boot path.

**Why two constructors?** Separating concerns keeps `src/main.rs` readable
(`::new()`) and test code explicit (`::empty()`).

### D3 — Checksum computed by `crc32_hex` helper (already in `hal/mod.rs`)

`backup()` serialises the stored `AdminConfig` to JSON and passes the bytes to
`crc32_hex` to produce the `BackupPayload::checksum`. The `backed_up_at` timestamp
is fixed to `"1970-01-01T00:00:00Z"` since no real-time clock is available in
the spin-executor development build.

**Why epoch timestamp?** Any hardcoded string is equally fake. Epoch is
unambiguous and signals "not a real clock" without panic.

### D4 — Place in `src/storage/ram.rs`, re-export from `src/storage/mod.rs`

Keeps storage implementations grouped. Future NVS or SD-card implementations can
live alongside without touching `src/main.rs`.

## Risks / Trade-offs

- **Not persistent** → config is lost on reboot in dev builds. This is intentional;
  the pre-seeded default means restarts are seamless during development.
- **Epoch timestamp** → backup payloads from dev builds have an obviously fake
  `backed_up_at`. Clients should not rely on this field for ordering in dev.
- **Default token `"dev"`** → must never be used in production. Mitigation: add a
  doc comment warning on `RamStorageHal::new()`.
