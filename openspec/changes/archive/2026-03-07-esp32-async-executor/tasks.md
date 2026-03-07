## 1. Resolve dependency versions

- [x] 1.1 Identify the latest compatible set: `esp-hal`, `esp-hal-embassy`, `esp-wifi`, `esp-alloc` that work together with `embassy-executor 0.9` and `embassy-net 0.8`
- [x] 1.2 Determine the correct hardware timer to use for `esp-hal-embassy::init()` (systimer vs timg0) for the target chip variant

## 2. Update Cargo.toml

- [x] 2.1 Add `esp-hal`, `esp-hal-embassy`, `esp-wifi`, `esp-alloc` as optional dependencies with pinned versions
- [x] 2.2 List the four new deps under the `esp32` feature in `[features]`
- [x] 2.3 Remove `arch-spin` from the `embassy-executor` features entry under the `esp32` feature (keep it for dev/default path)
- [x] 2.4 Run `cargo check` (dev, default features) and confirm no regressions

## 3. Rewrite examples/esp32/main.rs entry point

- [x] 3.1 Replace `fn main()` + `StaticCell<Executor>` + `Executor::run()` with `#[esp_hal::main] async fn main(spawner: Spawner)`
- [x] 3.2 Add `esp_hal::init(esp_hal::Config::default())` as first statement
- [x] 3.3 Add `esp_alloc::heap_allocator!(size: 72 * 1024)` before any heap allocation
- [x] 3.4 Add `esp_hal_embassy::init(/* timer */)` with a `todo!()` stub for timer selection
- [x] 3.5 Remove `#[embassy_executor::task] async fn async_main(spawner: Spawner)` — merge its body into `main`

## 4. Wire embassy-net stack stub

- [x] 4.1 Add stub call to `esp_wifi::init(...)` with `todo!("configure wifi peripherals")` placeholder
- [x] 4.2 Build a stub `embassy-net` `Stack` with `todo!("configure SSID/password and IP")` placeholder
- [x] 4.3 Pass the stack to `ApiServer::run(stack).await` (replacing the current `// TODO` comment)
- [x] 4.4 Update the `sse_task` signature/body to accept and use the network stack (or a reference to it)

## 5. Verify and clean up

- [ ] 5.1 Run `cargo build --example esp32 --features esp32` — confirm it compiles (panics at `todo!()` are acceptable)
      NOTE: requires xtensa toolchain (`espup install`) and `--target xtensa-esp32s3-none-elf`.
      Host cross-compilation without the toolchain is not supported after this change.
- [x] 5.2 Run `cargo build --example dev` — confirm dev example is unaffected
- [x] 5.3 Run `cargo test` — confirm library tests still pass (113 passed)
- [x] 5.4 Run `cargo fmt` and fix any formatting issues
- [x] 5.5 Update `TODO.md` to mark ESP32 bring-up entry point task as complete
- [x] 5.6 Add a CHANGELOG.md entry under `[Unreleased]`
