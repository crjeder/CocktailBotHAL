## Why

The `StorageHal` trait (redesigned in `redesign-admin-config-storage`) has no
concrete implementation. `StubStorageHal` in `src/main.rs` panics on every call
(`todo!()`), making development and testing impossible. A RAM-backed implementation
gives every developer and the CI environment a working `StorageHal` that survives
the process lifetime (not power cycles) and unblocks integration of the full config
flow without requiring real flash hardware.

## What Changes

- Add `RamStorageHal` struct in a new `src/storage/` module implementing the
  `StorageHal` trait (`backup`, `restore`) against an in-memory store
  (`Option<AdminConfig>`).
- Replace the `todo!()` `StubStorageHal` in `src/main.rs` with `RamStorageHal`.
- The RAM store is pre-seeded with a default `AdminConfig` at construction time so
  the robot boots into `Idle` (not `Provisioning`) in development builds.

## Capabilities

### New Capabilities

- `ram-storage-hal`: A concrete, no_std-compatible RAM-backed `StorageHal`
  implementation for development and testing.

### Modified Capabilities

<!-- none — no spec-level requirement changes -->

## Impact

- New file: `src/storage/mod.rs` (or `src/storage/ram.rs`)
- `src/main.rs`: `StubStorageHal` replaced with `RamStorageHal`
- No API, HAL trait, or dependency changes required
