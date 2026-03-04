## Context

The async HTTP server (`src/server/mod.rs`) reads an HTTP request, matches the
path and method, and forwards to a handler. There is no middleware layer — the
match expression goes directly from parsed request to handler call. The server
runs on bare embassy-net TCP sockets under `no_std + alloc`, so there is no
framework-level filter mechanism to hook into.

`RobotConfig` holds runtime configuration. `ConfigHal::get_active_config()`
retrieves it. This is the natural home for a configurable token.

## Goals / Non-Goals

**Goals:**
- Reject any request that lacks a valid `Authorization: Bearer <token>` header
  with `401 Unauthorized` before any HAL method executes.
- Make the accepted token runtime-configurable via `RobotConfig`.
- Add a compile-time default token so the robot is usable before config is set.
- Keep the change self-contained in the server dispatch loop and config struct.

**Non-Goals:**
- Multi-user or role-based access control.
- Token rotation or expiry.
- HTTPS / TLS (out of scope for embedded target at this stage).
- Authenticating the legacy Rocket API (`src/main.rs`, `src/api/mod.rs`).

## Decisions

### 1. Where to check: dispatch loop in `src/server/mod.rs`

**Decision:** Insert an auth check at the top of the request-dispatch function,
before the `match (method, path)` expression.

**Alternatives considered:**
- Per-handler check — would require duplicating logic in every handler and
  makes it easy to forget in future handlers. Rejected.
- Separate middleware struct — adds abstraction for a single concern; overkill
  for an embedded server with a single dispatch function. Rejected.

### 2. Token storage: `RobotConfig::token` field

**Decision:** Add `token: String` to `RobotConfig`. On embedded targets with
`no_std`, use a fixed-capacity `heapless::String<64>`. If `heapless` is not
already in `Cargo.toml`, check commented-out entries first; if absent, add it.

**Alternatives considered:**
- Compile-time constant only — not configurable at runtime, operator can never
  change the token without reflashing. Rejected for production use.
- Separate `AuthConfig` struct — extra indirection for a single field. Rejected.

### 3. Fallback: compile-time default token

**Decision:** Define `const DEFAULT_TOKEN: &str = "changeme"` in
`src/server/mod.rs`. If `RobotConfig::token` is empty, the server falls back to
the default. This ensures the robot responds out of the box.

**Alternatives considered:**
- Refuse all requests if token is unconfigured — blocks first-time setup.
  Rejected.

### 4. Header parsing: manual byte scan

**Decision:** Parse the `Authorization` header value with a simple `strip_prefix`
check on the raw header string. No regex or external parsing crate needed.

Expected header format:
```
Authorization: Bearer <token>
```

If the header is absent or the prefix does not match `Bearer `, return 401.

## Risks / Trade-offs

- **Default token is a known value** → Document clearly that operators MUST change
  it via `PATCH /v1/config` before deploying.
- **Token transmitted in plain text** → Acceptable given no TLS at this stage;
  local network only. Note in API docs.
- **`heapless` version** → Must match the version used by other embassy crates in
  the workspace. Check `Cargo.toml` and embassy snapshot before pinning.
- **Token comparison timing** → Use constant-time comparison (`subtle` crate or
  manual byte-by-byte loop without early exit) to avoid timing-based token
  enumeration. This is a low-risk attack on a local robot network, but is good
  practice. If `subtle` is not available, a simple length + XOR accumulator loop
  is sufficient.
