## Context

CocktailBotHAL is a headless embedded REST API. There is no browser session,
no cookie store, and no interactive login flow. The robot's two categories of
callers are:

1. **Client apps** — end-user cocktail applications that create dispense jobs
   and read status. These currently authenticate with a shared Bearer token.
2. **The operator** — the person who sets up the hardware. They configure
   liquids, manage storage, and control power. This role needs separate,
   stronger credentials.

The current single-token model means any client app credential also grants
full admin access. This is the gap the change closes.

## Goals / Non-Goals

**Goals:**
- Add a password-based credential for admin endpoints only.
- Keep Bearer token auth for all non-admin endpoints unchanged.
- Store only the password hash — never the plaintext.
- Support `no_std` environments (ESP32); hash algorithm must be swappable.
- Default password (`changeme`) when none is set, consistent with existing
  token fallback behaviour.

**Non-Goals:**
- Session tokens, JWTs, or cookie-based auth.
- HTTPS/TLS (out of scope for this change; assumed to be handled at the network
  layer or a future change).
- Role-based access control beyond the binary admin / non-admin split.
- Password complexity enforcement.
- Audit logging.

## Decisions

### D1 — HTTP Basic Auth for admin endpoints

Admin requests supply credentials in the `Authorization: Basic <base64>` header
where the base64 payload is `admin:<password>`.

**Why Basic Auth?** It is stateless (fits the embedded HTTP model), widely
supported by CLI tools (`curl -u admin:pass`), and requires no new token
management. The username is always `admin` — there is only one admin account.

**Why not a second Bearer token?** A bearer token for admin would look
identical to the user token from the client's perspective, creating confusion
and the same single-secret problem.

### D2 — `PasswordHasher` trait

```rust
pub trait PasswordHasher {
    /// Hash `password` and return the storable hash string.
    fn hash(&self, password: &str) -> Result<String, ErrorInfo>;

    /// Return `true` iff `password` matches `stored_hash`.
    fn verify(&self, password: &str, stored_hash: &str) -> bool;
}
```

The server generic parameter list gains one more type: `H: PasswordHasher`.
`RobotHal` and `ApiServer` get the same treatment.

**Why a trait?** Hashing on ESP32 must use a `no_std` crate (e.g. `sha2` +
`hmac`). On the development stub, a constant-time plain comparison is
sufficient. The trait boundary keeps the server code platform-agnostic.

### D3 — Password stored as hash in `RobotConfig::admin_password`

`admin_password: String` in `RobotConfig` holds a hash string (format:
`pbkdf2$<iterations>$<salt_hex>$<hash_hex>` or similar). An empty string
means "use the default password" — same pattern as `token`.

`PATCH /config` with a non-empty `admin_password` field triggers re-hashing:
```
incoming plain password → PasswordHasher::hash() → store as hash string
```

**Why embed in `RobotConfig`?** Config is already persisted via `StorageHal`.
No separate storage path is needed. The hash travels with the config backup.

**Why not Argon2?** Argon2's memory-hard nature requires heap allocation
(hundreds of KB) during verification, which is incompatible with the 520 KB
total RAM of the ESP32-S3. PBKDF2-SHA256 with 10 000 iterations is a
reasonable `no_std`-compatible default; the trait allows tuning per platform.

### D4 — Admin route set is a compile-time list

```rust
const ADMIN_ROUTES: &[(&str, &str)] = &[
    ("PATCH",  "/v1/config"),
    ("GET",    "/v1/storage/config"),
    ("POST",   "/v1/storage/config"),
    ("POST",   "/v1/control/power"),
    ("POST",   "/v1/control/power-save"),
    ("POST",   "/v1/control/reset"),
    ("POST",   "/v1/control/reload-config"),
    ("POST",   "/v1/cleaning/start"),
    ("POST",   "/v1/cleaning/stop"),
];
```

Any `(method, path)` pair in `ADMIN_ROUTES` requires Basic Auth. All others
require Bearer token. `GET /v1/status`, `GET /v1/config`, `GET /v1/sensors/*`,
and all `GET /v1/dispense/*` remain Bearer-token-gated (read-only, non-admin).

**Rationale:** Read-only and dispense endpoints are delegated to the operator's
client app. Destructive or configuration-mutating endpoints are admin-only.

### D5 — Auth dispatch order

```
1. Parse method + path.
2. Is this route in ADMIN_ROUTES?
   Yes → extract Basic Auth header → verify password hash.
        → 401 if missing or wrong.
   No  → extract Bearer token → verify against RobotConfig::token.
        → 401 if missing or wrong.
3. Dispatch to handler.
```

Config is loaded once at the start of `handle_connection` (same as current).

### D6 — Base64 decoding is done without `std`

The `base64` crate supports `no_std`. It is added to `Cargo.toml`
(`base64 = { version = "0.22", default-features = false, features = ["alloc"] }`).
A checked entry may already exist in Cargo.toml comments.

### D7 — `StubPasswordHasher` for dev/test

```rust
pub struct StubPasswordHasher;

impl PasswordHasher for StubPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, ErrorInfo> {
        Ok(alloc::format!("stub${}", password))
    }
    fn verify(&self, password: &str, stored_hash: &str) -> bool {
        stored_hash == alloc::format!("stub${}", password)
    }
}
```

This is constant-time-safe (no short-circuit, same length strings compared) and
makes tests deterministic without a real hash function.

## Risks / Trade-offs

- **Basic Auth over plain HTTP sends credentials on every request.** Mitigation:
  document in `API.yaml` that TLS is required for production; add a note in
  `CLAUDE.md`. TLS termination is out of scope.
- **Single admin account with no lockout.** A brute-force attack is limited by
  the robot's HTTP throughput (~50 req/s on ESP32); not a concern for a local
  network device.
- **Config roundtrip exposes the hash.** `GET /config` will include
  `admin_password` (the hash). Mitigation: redact the field in `GET /config`
  response (return `"***"` or `""`) so the hash is not accidentally leaked to
  non-admin clients. Admin can still back it up via `GET /storage/config`.
- **`PasswordHasher` trait adds a 8th generic to `ApiServer`.** This increases
  the monomorphisation surface. Accepted — the pattern already exists for the
  7 HAL generics.

## Migration Plan

1. Add `PasswordHasher` trait and `admin_password: String` to `src/hal/mod.rs`.
2. Update `RobotHal` / `ApiServer` with the 8th `Hasher` generic in
   `src/server/mod.rs`; add `ADMIN_ROUTES`; split auth dispatch.
3. Add Basic Auth parser to `src/server/http.rs`.
4. Add `StubPasswordHasher` to `src/main.rs`; add `admin_password` to stub
   config.
5. Add `base64` dependency to `Cargo.toml`.
6. Implement `Esp32PasswordHasher` (PBKDF2-SHA256) in `src/esp32/`.
7. Update `API.yaml`: add `admin_password` to Config schema (redacted in GET);
   document Basic Auth security scheme; annotate admin routes.
8. Run `cargo fmt` and `cargo check` (both feature sets).
