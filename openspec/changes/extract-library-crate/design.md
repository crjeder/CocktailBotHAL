## Context

`cocktail_bot_hal` is currently a Cargo binary crate with `fn main()` in `src/main.rs`. Its actual value — the HAL trait definitions, the async HTTP server, and the OpenAPI contract — is library code. The concrete hardware implementations (`src/esp32/`) and the development runtime (`src/main.rs` stubs, `src/storage/`) are application code mixed into the same source tree.

Cargo supports both `[lib]` and `[[example]]` targets in one crate. Examples compile against the library, have their own entry points, and can carry `required-features` guards. This is the idiomatic home for reference implementations in Rust embedded projects.

The `#[cfg(test)]` split in `main.rs` (`mod server` declared twice) and the embassy-time linker stubs exist only because the binary entry point was the only place to put them. Moving to a `[lib]` root removes that tension.

## Goals / Non-Goals

**Goals:**
- Convert the crate from binary to library (`src/lib.rs` as crate root).
- Library exports: `hal` (traits + types + mocks) and `server` (ApiServer, handlers, SSE).
- Move `src/esp32/` → `examples/esp32/` as a reference ESP32 implementation.
- Move `src/storage/` + all inline stubs → `examples/dev/` as a host-runnable reference.
- Keep `cargo test` green with zero changes to test logic.
- Keep `cargo check --features esp32` green.

**Non-Goals:**
- No changes to HAL traits, API behaviour, or `API.yaml`.
- No new functionality.
- No workspace split (single crate, multiple targets).
- No semver bump (library API is unchanged; removing the binary is not a breaking change for a library-first crate).

## Decisions

### Decision 1: Cargo `[[example]]` over workspace split

**Chosen:** Single crate with `[lib]` + `[[example]]` targets.

**Alternatives considered:**
- *Workspace split* (`hal-core` lib + `cocktail-bot-esp32` bin crate): Clean separation but high overhead — two `Cargo.toml` files, cross-crate visibility, version coordination. Premature for a single-team project.
- *`[[bin]]` targets*: Also valid, but Cargo examples are conventionally the right place for "here is how you use this library" code. They are skipped by `cargo build` by default, which matches the intent.

`required-features = ["esp32"]` on the esp32 example prevents it from being compiled without the feature, mirroring the current `#[cfg(feature = "esp32")]` guard.

### Decision 2: `RamStorageHal` belongs in `examples/dev/`, not the library

The library exports only traits and the server. `RamStorageHal` is a convenience implementation useful for development, but hardware vendors on ESP32 will not use it — they write to NVS or flash. Shipping it in the library would imply it is a supported implementation, which it is not.

If it is needed as a shared utility later, it can be extracted to its own module and re-exported deliberately.

### Decision 3: Embassy-time linker stubs stay in `src/lib.rs` under `#[cfg(test)]`

The stubs (`_embassy_time_now`, `_embassy_time_schedule_wake`) are needed when `mod server` is compiled in test mode. Since `server` is now a lib module (always compiled), the stubs live in `src/lib.rs` under `#[cfg(test)]`. This is the same mechanism, just in the correct file.

### Decision 4: No `#[cfg(not(test))]` split on `mod server`

In `main.rs`, `mod server` was declared twice (once under `#[cfg(not(test))]` and once under `#[cfg(test)] #[allow(unused)]`) to avoid unused-import warnings from `use server::ApiServer`. In `lib.rs`, `mod server` is a public module export — no conditional needed.

## Risks / Trade-offs

- **`cargo run` no longer works** → `cargo run --example dev` is the new development invocation. Update `CLAUDE.md` and `README` accordingly.
- **Examples are not tested by `cargo test`** → The stub HAL code in `examples/dev/` has no automated coverage. Acceptable: stubs are intentionally minimal and self-evidently correct.
- **Example import paths use `cocktail_bot_hal::`** → Examples import from the lib. Any item needed by an example must be `pub` in the lib. Current exports (`ApiServer`, `RobotHal`, all HAL types) are already public.
- **`extern crate alloc`** must appear in `src/lib.rs` (currently in `main.rs`). The `#![allow(dead_code)]` attribute also moves to `lib.rs`.

## Migration Plan

1. Create `src/lib.rs` with `extern crate alloc`, `#![allow(dead_code)]`, module declarations for `hal` and `server`, and the `#[cfg(test)]` embassy-time stubs.
2. Create `examples/dev/main.rs` — copy all stub structs, `RamStorageHal`, executor setup, and SSE task from `main.rs`.
3. Create `examples/esp32/` — copy all files from `src/esp32/`, adjusting `use` paths from `crate::hal::` to `cocktail_bot_hal::hal::`.
4. Update `Cargo.toml`: add `[lib]`, add two `[[example]]` entries, remove the implicit binary.
5. Delete `src/main.rs`, `src/esp32/`, `src/storage/`.
6. Run `cargo check`, `cargo test`, `cargo check --features esp32` — fix any path or visibility issues.
7. Update `CLAUDE.md` and `claude-progress.txt`.

Rollback: `git revert` — the change is purely structural with no logic changes.

## Open Questions

None. All decisions were made during the explore session.
