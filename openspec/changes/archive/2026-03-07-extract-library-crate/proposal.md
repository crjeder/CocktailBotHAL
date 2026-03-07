## Why

The crate currently conflates two roles: it defines a HAL trait contract (library) and also provides concrete hardware implementations in `src/` (application). Hardware vendors who want to build on top of CocktailBotHAL must fork the whole binary and replace stubs inline, rather than depending on a clean library. Separating them makes the contract explicit and the examples navigable.

## What Changes

- `src/main.rs` (binary entry) is deleted; the crate becomes a `[lib]` only.
- `src/esp32/` moves to `examples/esp32/` — a Cargo `[[example]]` requiring `--features esp32`.
- `src/storage/` (`RamStorageHal`) moves to `examples/dev/` — no longer a library export.
- All inline stub HAL implementations from `main.rs` move to `examples/dev/main.rs`.
- `StubPasswordHasher` moves to `examples/dev/main.rs`.
- `src/lib.rs` is created as the crate root, re-exporting `hal` and `server` only.
- `#[cfg(test)]` embassy-time linker stubs (`_embassy_time_now`, `_embassy_time_schedule_wake`) move from `main.rs` to `src/lib.rs`.
- `[features] esp32` is retained in `Cargo.toml` (still gates `sha2`/`pbkdf2` deps); `required-features = ["esp32"]` added to the esp32 `[[example]]` entry.
- `mod esp32` is removed from the library crate root (no longer in `src/`).

## Capabilities

### New Capabilities

- `dev-example`: A Cargo `[[example]]` named `dev` providing a host-runnable reference implementation: embassy spin executor, all stub HAL implementations, `RamStorageHal`, and `StubPasswordHasher`. Shows hardware vendors how to wire `ApiServer` end-to-end.

### Modified Capabilities

- `esp32-hal-impl`: Implementation moves from `src/esp32/` to `examples/esp32/`; no longer part of the library; compiled only via `cargo build --example esp32 --features esp32`.
- `ram-storage-hal`: `RamStorageHal` moves from `src/storage/` to `examples/dev/`; no longer exported from the library crate.

## Impact

- `Cargo.toml`: Add `[lib]`, remove implicit binary; add two `[[example]]` entries (`dev`, `esp32` with `required-features`).
- `src/lib.rs`: New crate root; declares `pub mod hal` and `pub mod server`; contains `#[cfg(test)]` embassy-time stubs.
- `src/main.rs`: Deleted.
- `src/storage/`: Deleted (moves to `examples/dev/`).
- `src/esp32/`: Deleted (moves to `examples/esp32/`).
- `examples/dev/main.rs`: New file; owns spin executor, all stub impls, `RamStorageHal`.
- `examples/esp32/`: New directory; contains all files from current `src/esp32/`.
- `CLAUDE.md`: Update repository structure section; update build/run instructions.
- `cargo test` is unaffected — tests live in `src/hal/` and `src/server/` (the library).
- No changes to HAL traits, API surface, or `API.yaml`.
- No semver bump required (library API is unchanged; binary is removed, not replaced).
