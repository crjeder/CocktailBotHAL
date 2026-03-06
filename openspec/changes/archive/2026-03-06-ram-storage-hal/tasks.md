## 1. New Module

- [x] 1.1 Create `src/storage/mod.rs` declaring the module and re-exporting `RamStorageHal`
- [x] 1.2 Create `src/storage/ram.rs` with the `RamStorageHal` struct (`stored: Option<AdminConfig>`)
- [x] 1.3 Implement `RamStorageHal::new()` pre-seeded with default `AdminConfig` (token `"dev"`, empty liquids, two glass types `short`/`long`, `max_total_parts: 100`, `admin_password: ""`)
- [x] 1.4 Implement `RamStorageHal::empty()` with `stored: None`
- [x] 1.5 Implement `StorageHal for RamStorageHal`: `backup()` returns `Err(NO_CONFIG)` when `None`, otherwise serialises, checksums via `crc32_hex`, returns `BackupPayload` with epoch `backed_up_at`
- [x] 1.6 Implement `StorageHal for RamStorageHal`: `restore(cfg)` sets `stored = Some(cfg)` and returns `Ok(())`
- [x] 1.7 Add `pub mod storage;` to `src/lib.rs` or `src/main.rs` (whichever is the crate root for this path)

## 2. Wire into main.rs

- [x] 2.1 Remove `StubStorageHal` struct and its `StorageHal` impl from `src/main.rs`
- [x] 2.2 Import `RamStorageHal` and use `RamStorageHal::new()` where `StubStorageHal` was instantiated

## 3. Validation

- [x] 3.1 Run `cargo check` (no features); fix all errors
- [x] 3.2 Run `cargo check --features esp32`; fix all errors
- [x] 3.3 Run `cargo fmt`; verify no diff
