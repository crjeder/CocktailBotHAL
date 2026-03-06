## Why

The robot currently uses a single shared Bearer token for all API access. This
means any API client — including end-user cocktail apps — holds a credential
that also grants access to destructive admin operations (PATCH /config,
POST /storage/config, POST /control/reset, etc.). A compromised user client can
reconfigure the hardware or wipe stored settings.

The admin interface needs a separate, stronger authentication mechanism so that
operational endpoints (dispense, status) can be exposed to client apps while
administrative endpoints remain protected by credentials known only to the
operator.

## What Changes

- Add an `admin_password` field to `RobotConfig` (hashed with Argon2 or
  bcrypt; stored as a hash string, never in plaintext).
- Admin endpoints require HTTP Basic Auth (`admin` / `<password>`), checked
  separately from the Bearer token.
- Non-admin endpoints retain Bearer token auth (unchanged).
- A default admin password (`changeme`) applies when `admin_password` is empty,
  identical to the existing Bearer token fallback.
- `PATCH /config` with `admin_password` field re-hashes and stores the new
  password — no separate "change password" endpoint.
- On ESP32, Argon2 is too expensive; use a configurable hash backend via a new
  `PasswordHasher` trait so the ESP32 HAL impl can substitute a lighter
  algorithm (e.g. PBKDF2-SHA256 with fewer iterations or constant-time
  SHA-256 HMAC).

## Capabilities

### New Capabilities

- `admin-password-auth`: Separate admin credential layer protecting destructive
  endpoints; `PasswordHasher` trait; hash-on-write, verify-on-request.

### Modified Capabilities

- `bearer-token-auth`: Scope narrows to non-admin endpoints only. Admin token
  is no longer valid for config mutation routes.

## Impact

- `src/hal/mod.rs`: Add `admin_password: String` to `RobotConfig`; add
  `PasswordHasher` trait with `hash` and `verify` methods.
- `src/server/mod.rs`: Split auth check — Bearer token for user routes, Basic
  Auth for admin routes; define the admin route set.
- `src/server/http.rs`: Add Basic Auth header parser helper.
- `src/main.rs`: `StubPasswordHasher` using constant-time comparison (no real
  hashing in dev stub).
- `src/esp32/`: Implement `PasswordHasher` with PBKDF2-SHA256 using `ring` or
  `sha2`/`hmac` from `[no_std]` crates.
- `API.yaml`: Document `admin_password` in Config schema; document Basic Auth
  security scheme for admin endpoints; mark admin routes with that scheme.
- Semver: backwards-compatible addition → v0.5.1 (or v0.5.0 if landed before
  the pending `redesign-admin-config-storage` change).
