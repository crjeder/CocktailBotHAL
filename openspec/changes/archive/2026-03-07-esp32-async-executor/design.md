## Context

The current `examples/esp32/main.rs` bootstraps embassy-executor using the
`arch-spin` feature, which provides a cooperative spin-loop executor suitable
for host (dev) targets and CI but incompatible with real ESP32 hardware. On
ESP32, the async executor must integrate with the Xtensa/RISC-V interrupt
controller and hardware timers. `esp-hal-embassy` provides this integration,
but it requires the entry point to be annotated with `#[esp_hal::main]` — not
`fn main()` calling `Executor::run()`.

All ESP32 HAL stub structs (`Esp32Control`, `Esp32Status`, etc.) and the
`ApiServer` are already implemented and do not need to change. Only the entry
point wiring changes.

## Goals / Non-Goals

**Goals:**
- Replace the spin-executor bootstrap in `examples/esp32/main.rs` with the
  `#[esp_hal::main]` entry point from `esp-hal`
- Initialise `esp-hal-embassy` with an appropriate hardware timer so that
  embassy time and async tasks work correctly on-chip
- Add stub wiring for `esp-wifi` → `embassy-net` stack with `todo!()` markers
  at real hardware credential/peripheral bindings
- Gate all new dependencies behind the existing `esp32` Cargo feature
- Keep the `dev` example (`examples/dev/main.rs`) and library tests unaffected

**Non-Goals:**
- Implementing real Wi-Fi credentials, SSID, or network configuration
- Replacing the `Esp32*` HAL stub implementations with real drivers
- Changes to `src/` (library crate, HAL traits, HTTP server)
- Supporting other targets (STM32, nRF, etc.)

## Decisions

### D1: Use `#[esp_hal::main]` instead of a custom entry macro

`esp-hal` v0.22+ provides `#[esp_hal::main]` which calls `esp_hal::init()`,
sets up the allocator (via `esp-alloc`), and hands a `Spawner` to the async
body. This is the idiomatic entrypoint for all modern esp-hal projects and is
the approach documented by the esp-rs community.

**Alternative considered**: use `#[entry]` from `riscv-rt`/`xtensa-lx-rt`
directly and bootstrap embassy manually. Rejected because it requires more
boilerplate and is not supported by the current `esp-hal` public API.

### D2: Use `esp-hal-embassy` for timer integration

`esp-hal-embassy::init()` accepts one or more `OneShotTimer<ErasedTimer>`
instances and registers them with the embassy-executor runtime. This is the
only supported way to provide embassy-time on esp-hal targets.

**Alternative considered**: use `embassy-executor` with `arch-riscv32` or
`arch-xtensa` feature. Rejected because these features require manual linker
script setup and do not wire embassy-time automatically; `esp-hal-embassy`
handles this correctly.

### D3: Retain `StaticCell` for SSE sub-task lifetime, remove it for executor

The `StaticCell<Executor>` is only needed for the spin-executor pattern. With
`#[esp_hal::main]`, the executor is managed by the macro. However, the SSE
task still needs `&'static` references to `Esp32Status` and `Esp32Dispense`,
so `StaticCell<Esp32Status>` and `StaticCell<Esp32Dispense>` are retained.

### D4: esp-wifi stack wired with `todo!()` placeholders

The actual Wi-Fi peripheral init (`WifiController`, SSID, password, IP config)
cannot be coded without knowing the deployment environment. The entry point
will call `esp_wifi::init(...)` and build the embassy-net `Stack` with
`todo!("configure wifi credentials and peripheral bindings")` stubs, so the
structure is correct and compilable (behind `#[allow(unreachable_code)]`) while
leaving hardware specifics for the integrator.

**Alternative considered**: omit the network stack entirely and just call
`ApiServer::run(todo!())`. Rejected because it provides less guidance on what
needs to be wired.

### D5: Cargo feature structure unchanged

New dependencies (`esp-hal`, `esp-hal-embassy`, `esp-wifi`, `esp-alloc`) are
added to the `[dependencies]` table with `optional = true` and listed under the
`esp32` feature in `[features]`. The `embassy-executor` dependency drops the
`arch-spin` feature from its `esp32`-feature entry (arch-spin is still needed
for the dev example, so it stays in the default/dev feature path).

## Risks / Trade-offs

- **Version churn**: `esp-hal`, `esp-hal-embassy`, and `esp-wifi` move fast and
  have closely coupled version requirements. The chosen versions must be
  compatible with each other and with `embassy-executor 0.9` / `embassy-net 0.8`.
  → Mitigation: pin exact versions in `Cargo.toml`; document required combination
  in a comment.

- **`todo!()` stubs won't compile to a runnable binary**: The ESP32 example
  after this change will compile but panic at the `todo!()` site on first run.
  → Mitigation: This is intentional and documented with clear `// TODO:` comments.
  The goal is correct structure, not a flashed binary.

- **`no_std` + `alloc` constraint**: All new code in `examples/esp32/` must
  compile without `std`. `esp-alloc` provides the global allocator; the entry
  point must call `esp_alloc::heap_allocator!(size: 72 * 1024)` before any
  heap use. → Mitigation: add the allocator init as the first statement in the
  entry point.

## Open Questions

- **esp-hal version to target**: The esp-rs ecosystem is currently at esp-hal
  0.22.x. Confirm the latest stable version and matching `esp-hal-embassy` /
  `esp-wifi` versions before finalising `Cargo.toml`.
- **Timer selection**: ESP32 has multiple hardware timers. `esp-hal-embassy`
  recommends `systimer` (available on ESP32-S2/S3/C3/C6) or `timg0` (classic
  ESP32). The correct choice depends on the exact chip variant targeted.
