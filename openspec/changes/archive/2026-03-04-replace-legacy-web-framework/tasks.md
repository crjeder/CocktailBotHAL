## 1. Add embassy-executor dependency

- [x] 1.1 Run `cargo tree -p embassy-net` to confirm the compatible `embassy-executor` version
- [x] 1.2 Add `embassy-executor` to `Cargo.toml` with `arch-std` and `executor-thread` features (default, non-ESP32 build)
- [x] 1.3 Run `cargo check` to confirm clean compile with the new dependency

## 2. Update entry point and document BSP path

- [x] 2.1 Replace `fn main()` body with documented stub that constructs `ApiServer` and notes the real async call
- [x] 2.2 Add TODO comment block in `src/main.rs` documenting the `#[esp_hal::main]` async entry point pattern for ESP32 bring-up
- [x] 2.3 Note: `#[embassy_executor::main]` is unavailable for arch-spin/arch-std in embassy-executor 0.9.x; ESP32 entry point comes from esp-hal BSP
- [x] 2.4 Run `cargo check` to confirm clean compile

## 3. Fix API.yaml typo

- [x] 3.1 Open `API.yaml` at line 82 and replace `integerlö` with `integer`
- [x] 3.2 Verify no other corrupted characters exist nearby

## 4. Remove Rocket references from documentation

- [x] 4.1 Edit `CLAUDE.md`: remove "Legacy web framework: Rocket 0.4, rocket_contrib 0.4.11" from the tech stack listing; update entry point notes to reflect async embassy entry point
- [x] 4.2 Edit `openspec/config.yaml`: remove Rocket from the tech stack section; note embassy-executor as the new runtime dependency

## 5. Final verification and cleanup

- [x] 5.1 Run `cargo fmt` to apply formatting rules
- [x] 5.2 Run `cargo check` one final time to confirm zero errors and zero unexpected warnings
- [x] 5.3 Commit with message: "Replace legacy Rocket entry point with embassy async main"
