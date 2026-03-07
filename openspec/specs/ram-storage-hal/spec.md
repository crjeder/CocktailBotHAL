### Requirement: RamStorageHal implements StorageHal
`RamStorageHal` SHALL implement the `StorageHal` trait and be usable as a drop-in
replacement for any `Stor: StorageHal` generic parameter without requiring `std`.
`RamStorageHal` SHALL live in `examples/dev/` and SHALL NOT be exported from the
library crate. It is a development-only implementation.

#### Scenario: RamStorageHal satisfies StorageHal bound
- **WHEN** `RamStorageHal` is used as the `Stor` generic in `ApiServer` within the dev example
- **THEN** the code compiles without `std` imports

### Requirement: backup returns error on empty store
`RamStorageHal::backup()` SHALL return `Err(ErrorInfo { code: "NO_CONFIG", .. })`
when no config has been stored yet (constructed with `::empty()` or before any
`restore()` call).

#### Scenario: backup on empty store returns NO_CONFIG
- **WHEN** `backup()` is called on a `RamStorageHal::empty()` instance
- **THEN** `Err(ErrorInfo)` is returned with `code == "NO_CONFIG"`

### Requirement: backup returns stored config after restore
After a successful `restore(cfg)`, `backup()` SHALL return `Ok(BackupPayload)`
where `payload.data` equals `cfg`.

#### Scenario: backup reflects most recent restore
- **WHEN** `restore(cfg_a)` is called, then `restore(cfg_b)` is called
- **THEN** `backup()` returns `payload.data == cfg_b`

### Requirement: backup payload includes checksum
`backup()` SHALL compute a CRC32 checksum of the JSON-serialised `AdminConfig`
and include it as `payload.checksum`.

#### Scenario: checksum matches data
- **WHEN** `backup()` is called on a non-empty store
- **THEN** `crc32_hex(serialize(payload.data)) == payload.checksum`

### Requirement: new() pre-seeds a default config
`RamStorageHal::new()` SHALL initialise the store with a default `AdminConfig`
containing `token: "dev"`, an empty `liquids` list, a minimal `glass_types` list,
and `admin_password: ""`. This allows development builds
to boot into `Idle` state without a provisioning step.

#### Scenario: new() makes backup succeed immediately
- **WHEN** `backup()` is called on a `RamStorageHal::new()` instance
- **THEN** `Ok(BackupPayload)` is returned without any prior `restore()` call

### Requirement: empty() starts with no stored config
`RamStorageHal::empty()` SHALL initialise the store with `None`, causing `backup()`
to return `Err` until `restore()` is called. Used in tests that exercise the
`Provisioning` boot path.

#### Scenario: empty() makes backup fail until restore
- **WHEN** `backup()` is called on a `RamStorageHal::empty()` instance without
  any prior `restore()` call
- **THEN** `Err(ErrorInfo)` is returned
