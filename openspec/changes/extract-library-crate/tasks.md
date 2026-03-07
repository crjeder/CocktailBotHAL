## 1. Create library crate root

- [x] 1.1 Create `src/lib.rs`: add `extern crate alloc`, `#![allow(dead_code)]`, `pub mod hal`, `pub mod server`; move `#[cfg(test)]` embassy-time stubs (`_embassy_time_now`, `_embassy_time_schedule_wake`) from `main.rs` into `lib.rs`
- [x] 1.2 Update `Cargo.toml`: add `[lib]` section; add `[[example]] name = "dev"` entry; add `[[example]] name = "esp32" required-features = ["esp32"]` entry; remove any implicit binary target reference

## 2. Create dev example

- [x] 2.1 Create `examples/dev/main.rs`: copy all stub HAL structs (`StubControlHal`, `StubStatusHal`, `StubConfigHal`, `StubSensorHal`, `StubDispenseHal`, `StubCleaningHal`, `StubPasswordHasher`) from `src/main.rs`; update imports to `use cocktail_bot_hal::{hal::*, server::*}`
- [x] 2.2 Move `RamStorageHal` into `examples/dev/`: copy `src/storage/ram.rs` content into `examples/dev/storage.rs` and declare it as `mod storage` in `examples/dev/main.rs`; update internal imports
- [x] 2.3 Copy executor setup into `examples/dev/main.rs`: `StaticCell<Executor>`, `StaticCell<StubStatusHal>`, `StaticCell<StubDispenseHal>`, `sse_task`, `async_main`, `fn main()` — update imports

## 3. Create esp32 example

- [x] 3.1 Create `examples/esp32/` directory with all files from `src/esp32/`: `mod.rs`, `control.rs`, `status.rs`, `config.rs`, `storage.rs`, `sensors.rs`, `dispense.rs`, `cleaning.rs`, `hasher.rs`
- [x] 3.2 Update all `use crate::hal::` paths in `examples/esp32/*.rs` to `use cocktail_bot_hal::hal::`
- [x] 3.3 Create `examples/esp32/main.rs`: placeholder entry point with `#[embassy_executor::task]` or `// TODO: #[esp_hal::main]` comment; wire `Esp32Hal` into `ApiServer` and `RobotHal`

## 4. Delete old source files

- [x] 4.1 Delete `src/main.rs`
- [x] 4.2 Delete `src/storage/` directory (`src/storage/mod.rs`, `src/storage/ram.rs`)
- [x] 4.3 Delete `src/esp32/` directory and all sub-files

## 5. Verify and fix

- [x] 5.1 Run `cargo check` — fix any missing `pub` exports or broken import paths in the library
- [x] 5.2 Run `cargo test` — all existing tests pass
- [x] 5.3 Run `cargo check --features esp32` — no errors or warnings
- [x] 5.4 Run `cargo build --example dev` — dev example builds successfully

## 6. Documentation and cleanup

- [x] 6.1 Run `cargo fmt`
- [x] 6.2 Update `CLAUDE.md`: revise repository structure section; change `cargo run` to `cargo run --example dev`; remove esp32 from `src/`; note `examples/` directory
- [x] 6.3 Update `claude-progress.txt`
- [ ] 6.4 Commit with message `refactor: extract library crate, move impls to examples/`
