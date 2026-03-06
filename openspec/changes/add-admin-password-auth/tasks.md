## 1. HAL Types (src/hal/mod.rs)

- [ ] 1.1 Add `PasswordHasher` trait with `hash(&self, password: &str) -> Result<String, ErrorInfo>` and `verify(&self, password: &str, stored_hash: &str) -> bool`; add `/// doc` comments
- [ ] 1.2 Add `admin_password: String` field to `RobotConfig` with `#[serde(default)]`

## 2. HTTP Parsing (src/server/http.rs)

- [ ] 2.1 Add `basic_auth_password(header_value: &str) -> Option<&str>` helper: base64-decode the `Authorization: Basic <b64>` value, strip the `admin:` prefix, return the password slice
- [ ] 2.2 Ensure `HttpRequest::header()` is usable for the `Authorization` header (already exists — verify, no change if sufficient)

## 3. Server Auth Dispatch (src/server/mod.rs)

- [ ] 3.1 Add `ADMIN_ROUTES: &[(&str, &str)]` constant listing all (method, path) pairs that require admin credentials (PATCH /v1/config, GET+POST /v1/storage/config, POST /v1/control/power, POST /v1/control/power-save, POST /v1/control/reset, POST /v1/control/reload-config, POST /v1/cleaning/start, POST /v1/cleaning/stop)
- [ ] 3.2 Add `Hasher: PasswordHasher` as the 8th generic type parameter to `RobotHal` and `ApiServer`; add `hasher: Hasher` field to `RobotHal`
- [ ] 3.3 Update `impl<...> ApiServer<...>` bounds to include `Hasher: PasswordHasher`
- [ ] 3.4 In `handle_connection`: after parsing method+path, check if `(method, path)` is in `ADMIN_ROUTES`; if yes, extract Basic Auth and call `self.hal.hasher.verify(password, &cfg.admin_password)` (with default fallback when hash is empty); return 401 if not verified
- [ ] 3.5 Keep existing Bearer token check for all non-admin routes (no change to that logic)

## 4. Dependency (Cargo.toml)

- [ ] 4.1 Add `base64 = { version = "0.22", default-features = false, features = ["alloc"] }` (check if already commented out; if so, uncomment)

## 5. Dev Stub (src/main.rs)

- [ ] 5.1 Add `StubPasswordHasher` struct implementing `PasswordHasher`: `hash` returns `"stub$<password>"`, `verify` checks `stored_hash == format!("stub${}", password)` in constant-time (no short-circuit via `==` on equal-length strings)
- [ ] 5.2 Add `admin_password: String::new()` to `StubConfigHal`'s default `RobotConfig`
- [ ] 5.3 Wire `StubPasswordHasher` into `RobotHal { ..., hasher: StubPasswordHasher }` in `main`

## 6. ESP32 Implementation (src/esp32/)

- [ ] 6.1 Create `src/esp32/hasher.rs`: implement `Esp32PasswordHasher` using `pbkdf2` + `sha2` crates (`no_std` + `alloc`); hash format `pbkdf2$10000$<salt_hex>$<hash_hex>`; generate random salt via `esp_hal::rng` or a seeded counter
- [ ] 6.2 Add `pub mod hasher;` to `src/esp32/mod.rs`; re-export `Esp32PasswordHasher`
- [ ] 6.3 Wire `Esp32PasswordHasher` into the ESP32 `RobotHal` in `src/esp32/mod.rs`
- [ ] 6.4 Add `pbkdf2 = { version = "0.12", default-features = false }` and `sha2 = { version = "0.10", default-features = false }` to `Cargo.toml` under `[dependencies]` (or uncomment if present); gate with `#[cfg(feature = "esp32")]` if adding as optional deps

## 7. API Specification (API.yaml)

- [ ] 7.1 Add `admin_password` field to the `Config` schema with description noting it is write-only (always returned as `""` in GET responses)
- [ ] 7.2 Add `basicAuth` security scheme (`type: http`, `scheme: basic`) to `components/securitySchemes`
- [ ] 7.3 Annotate all admin routes in the path definitions with `security: [{ basicAuth: [] }]`
- [ ] 7.4 Add a note to `PATCH /config` description that supplying `admin_password` re-hashes and stores the new password

## 8. Validation

- [ ] 8.1 Run `cargo check` with no features; fix all errors
- [ ] 8.2 Run `cargo check --features esp32`; fix all errors
- [ ] 8.3 Run `cargo fmt`; verify no diff
- [ ] 8.4 Manually verify: a PATCH /config request with Bearer token returns 401; same request with correct Basic Auth credentials returns 200
