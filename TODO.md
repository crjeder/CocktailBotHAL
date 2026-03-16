# TODO — CocktailBotHAL

---

## Blockers (project will not compile on target without these)

### ESP32 network stack wiring

`examples/esp32/main.rs` now uses `#[esp_hal::main]` with the correct
async entry point structure. The embassy timer init and the esp-wifi /
embassy-net stack are marked with `todo!()` and must be completed for
actual hardware bring-up. See the `todo!()` comments in that file for
the exact steps:

1. Uncomment the right embassy timer block for your chip variant.
2. Initialise esp-wifi and construct the embassy-net `Stack`.
3. Pass the `Stack` to `ApiServer::run()`.
   (SSE is served as `GET /v1/events` by `ApiServer` — no separate task needed.)

Also: the library uses `serde_json` which requires `std`. For a fully
bare-metal build, switch to `serde_json` alloc feature or `serde-json-core`.

Build command (requires `espup install` toolchain):
```
cargo build --example esp32 --features esp32 --target xtensa-esp32s3-none-elf
```

---

