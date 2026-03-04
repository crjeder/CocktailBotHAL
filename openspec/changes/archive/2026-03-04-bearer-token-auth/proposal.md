## Why

`API.yaml` declares Bearer token authentication as a global security requirement,
but the server currently dispatches every request to handlers without parsing or
validating the `Authorization` header. Any caller — authorized or not — can issue
commands, including power-cycling the robot or creating dispense jobs.

## What Changes

- Add token validation middleware in `src/server/mod.rs` that runs before route
  dispatch on every request.
- Extract the configured token from `RobotConfig` (or a compile-time constant as
  an interim measure) and compare it to the incoming `Authorization: Bearer <token>`
  header.
- Reject unauthenticated requests with `401 Unauthorized` and a JSON error body
  before any HAL method is called.
- Add a `token` field to `RobotConfig` to make the accepted token configurable at
  runtime via the config API.

## Capabilities

### New Capabilities

- `bearer-auth`: Parse `Authorization` header on every incoming request, validate
  the Bearer token, and return 401 if missing or invalid — before dispatching to
  any handler.

### Modified Capabilities

- `esp32-hal-impl`: `RobotConfig` gains a `token: heapless::String<64>` field;
  `Esp32Hal::get_active_config()` must return a config that includes this field.

## Impact

- `src/server/mod.rs` — auth check inserted in the request dispatch loop.
- `src/hal/mod.rs` — `RobotConfig` struct gains a `token` field.
- `src/esp32/mod.rs` — `Esp32Hal` config stub updated to include `token`.
- `API.yaml` — no schema change needed; Bearer auth is already declared.
- No new crate dependencies required (`heapless` already available in
  embedded Rust ecosystem; check `Cargo.toml` before adding).
