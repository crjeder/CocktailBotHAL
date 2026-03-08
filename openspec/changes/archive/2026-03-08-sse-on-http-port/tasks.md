## 1. Refactor src/server/sse.rs

- [x] 1.1 Delete `SseServer` struct and its `run` method
- [x] 1.2 Rename `handle_client` to `handle_sse_stream` and change visibility to `pub(super)`; update its signature to `pub(super) async fn handle_sse_stream<Stat: StatusHal, Disp: DispenseHal, W: Write + Unpin>(status: &Stat, dispense: &Disp, socket: &mut W)`
- [x] 1.3 Remove the `SSE_PORT` constant (no longer needed)
- [x] 1.4 Run `cargo check` to confirm the module compiles in isolation

## 2. Update src/server/mod.rs

- [x] 2.1 Add `("GET", "/v1/events")` to a new `NO_AUTH_ROUTES` constant (routes that skip authentication entirely)
- [x] 2.2 Update the auth gate to skip authentication for routes in `NO_AUTH_ROUTES`
- [x] 2.3 Add `("GET", "/v1/events")` to `PROVISIONING_ALLOWED`
- [x] 2.4 Add `("GET", "/v1/events")` to the `match` dispatch block; call `sse::handle_sse_stream(&self.hal.status, &self.hal.dispense, socket).await`
- [x] 2.5 Run `cargo check` and confirm zero warnings

## 3. Update examples

- [x] 3.1 Remove the `SseServer` import, construction, and spawned task from `examples/dev/main.rs`
- [x] 3.2 Remove the `SseServer` import, construction, and spawned task from `examples/esp32/main.rs`
- [x] 3.3 Run `cargo build --example dev` to confirm the dev example builds cleanly
- [x] 3.4 Run `cargo build --example esp32 --features esp32` to confirm the ESP32 example builds cleanly
  (Note: host build fails with pre-existing `esp-wifi-sys` cross-compilation error unrelated to this change)

## 4. Update API.yaml

- [x] 4.1 Update the `/events` path description to reflect the unified port (remove reference to port 9000; describe as `GET /v1/events` on port 80)
  (Note: API.yaml already described port 80 — no change needed)

## 5. Verify and wrap up

- [x] 5.1 Run `cargo test` — all 113+ tests must pass
- [x] 5.2 Run `cargo fmt` and confirm no formatting changes
- [x] 5.3 Update `openspec/specs/sse-wiring/spec.md` by applying the delta spec (merge MODIFIED requirements into the living spec)
