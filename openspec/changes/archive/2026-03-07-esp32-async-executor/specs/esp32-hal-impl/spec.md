## ADDED Requirements

### Requirement: esp32 example uses esp-hal async entry point
The ESP32 example entry point SHALL be annotated with `#[esp_hal::main]` and
declared `async fn main(spawner: Spawner)`. It SHALL NOT use `fn main()` with
a `StaticCell<Executor>` spin-executor pattern. The entry point SHALL call
`esp_hal::init(esp_hal::Config::default())` and
`esp_hal_embassy::init(/* timer */)` before spawning any tasks.

#### Scenario: entry point is async with spawner
- **WHEN** the ESP32 example is compiled with `--features esp32`
- **THEN** the `main` function is annotated `#[esp_hal::main]` and has signature
  `async fn main(spawner: embassy_executor::Spawner)`

#### Scenario: spin executor is removed
- **WHEN** the ESP32 example source is inspected
- **THEN** there is no `StaticCell<Executor>` and no `Executor::run()` call in
  `examples/esp32/main.rs`

### Requirement: esp32 example stubs embassy-net stack via esp-wifi
The ESP32 example SHALL contain stub code wiring `esp-wifi` to an `embassy-net`
`Stack`. Unresolved hardware bindings (Wi-Fi SSID, password, peripheral
initialisation) SHALL be marked with `todo!("...")` placeholders. The structure
SHALL be sufficient to show the integrator what needs to be filled in.

#### Scenario: network stack stub is present
- **WHEN** the ESP32 example source is inspected
- **THEN** there is a `todo!()` call indicating where the embassy-net stack
  should be initialised with esp-wifi credentials and peripherals

#### Scenario: ApiServer::run receives network stack
- **WHEN** the ESP32 example source is inspected
- **THEN** `ApiServer::run` is called with (or awaiting) the embassy-net stack,
  not bypassed or left commented out

### Requirement: esp-hal and related dependencies are feature-gated
`Cargo.toml` SHALL declare `esp-hal`, `esp-hal-embassy`, `esp-wifi`, and
`esp-alloc` as optional dependencies and list them under the `esp32` feature.
These dependencies SHALL NOT be compiled for the `dev` example or library tests.

#### Scenario: esp32-gated deps absent from dev build
- **WHEN** `cargo build --example dev` is run (without `--features esp32`)
- **THEN** the build succeeds and none of `esp-hal`, `esp-hal-embassy`,
  `esp-wifi`, or `esp-alloc` are compiled

#### Scenario: esp32-gated deps present for esp32 build
- **WHEN** `cargo build --example esp32 --features esp32` is run
- **THEN** the build succeeds with all four dependencies compiled in
