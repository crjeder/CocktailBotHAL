## 1. Extend RobotConfig with token field

- [x] 1.1 Add `token: String` field to `RobotConfig` in `src/hal/mod.rs` with a `#[serde(default)]` attribute so existing JSON without the field deserializes without error
- [x] 1.2 Verify `RobotConfig` still derives `Serialize, Deserialize` and compiles

## 2. Check heapless availability

- [x] 2.1 Inspect `Cargo.toml` for existing `heapless` dependency or commented-out entry
- [x] 2.2 If not present, add `heapless` at a version compatible with the embassy snapshot in use; if `no_std` target requires it, gate behind `#[cfg(not(feature = "std"))]` as needed

## 3. Add constant-time comparison helper

- [x] 3.1 Add a `fn tokens_equal(a: &str, b: &str) -> bool` in `src/server/mod.rs` that compares byte-by-byte without early exit (XOR accumulator pattern)
- [x] 3.2 Add `/// # Panics` and `///` doc comment explaining the constant-time rationale

## 4. Implement auth check in server dispatch

- [x] 4.1 Define `const DEFAULT_TOKEN: &str = "changeme";` at the top of `src/server/mod.rs`
- [x] 4.2 In the request dispatch function, before the `match (method, path)` expression, call `ConfigHal::get_active_config()` to retrieve the active token
- [x] 4.3 Select the effective token: use `RobotConfig::token` if non-empty, else fall back to `DEFAULT_TOKEN`
- [x] 4.4 Extract the `Authorization` header from the parsed request; strip the `"Bearer "` prefix
- [x] 4.5 Call `tokens_equal()` to compare extracted token to effective token; if false or header absent, write a `401 Unauthorized` JSON response and return without dispatching
- [x] 4.6 Run `cargo fmt` and verify the file compiles

## 5. Update Esp32Hal config stub

- [x] 5.1 In `src/esp32/mod.rs`, update `get_active_config()` to return a `RobotConfig` with `token: String::new()` (empty, triggering the default fallback)
- [x] 5.2 Confirm the ESP32 feature-gated code still uses only `core` and `alloc` (no `std` imports)

## 6. Verify end-to-end

- [x] 6.1 Build with `cargo build --features esp32` and confirm no errors or warnings
- [x] 6.2 Manually test: send a request without the header → expect 401; send with `Authorization: Bearer changeme` → expect normal response
- [x] 6.3 Run `cargo fmt` and commit
