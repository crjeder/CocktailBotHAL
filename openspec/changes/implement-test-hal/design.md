## Context

`src/hal/mod.rs` defines 7 async HAL traits. `src/esp32/` contains stub
implementations (all `// TODO: wire to hardware`). `src/hal/tests.rs` already
has 57 unit tests using hand-rolled mocks, but those mocks:

- Are ad-hoc (no shared transaction model)
- Live only in `#[cfg(test)]` blocks inside `src/hal/`
- Cannot be reused for server-handler integration tests

`embedded-hal-mock = "0.7.2"` and `test-case = "3.2"` are present in
`Cargo.toml` as commented-out entries, signalling original intent.

`embedded-hal-mock` provides a transaction-queue pattern:
each mock receives a list of expected operations; calling the mock pops and
verifies the next expectation; `done()` asserts the queue is empty.
Our HAL traits are not embedded-hal traits, so `embedded-hal-mock` cannot
mock them directly — but we apply the same pattern to our own traits.

## Goals / Non-Goals

**Goals:**
- Enable `embedded-hal-mock` and `test-case` as dev-dependencies
- Create `src/hal/mock.rs`: transaction-queue mocks for all 7 HAL traits,
  usable from any test module in the crate
- Write integration tests for the server handler functions using the mock HAL
- Parameterize repetitive tests via `test-case`

**Non-Goals:**
- Mocking embedded-hal GPIO/SPI/I2C pins (no real hardware code yet)
- Replacing the existing 57 unit tests in `src/hal/tests.rs`
- Adding `embedded-hal` itself as a dependency
- Testing the HTTP wire format (that is a separate server-integration concern)

## Decisions

### 1 — Mock location: `src/hal/mock.rs` (not a separate crate)

Transaction-queue mocks belong alongside the traits they implement.
`src/hal/mock.rs` is gated `#[cfg(test)]` and `pub(crate)` so it is
available to every `#[cfg(test)]` block in the crate without exposing
it in the public API.

**Alternative considered:** `tests/mock_hal.rs` (integration-test helper).
Rejected: integration test files cannot share code with `src/` unit tests
without a `lib.rs` re-export, adding unnecessary complexity.

### 2 — Transaction-queue model (same pattern as embedded-hal-mock)

Each mock struct holds a `VecDeque<Expected<Op>>`. Calling a trait method
pops the front expectation, asserts it matches, and returns the pre-loaded
result. `mock.done()` panics if any expectations remain unconsumed.

This matches `embedded-hal-mock`'s API so tests read consistently, and it
exercises the exact call sequence — not just whether a method was called.

**Alternative considered:** Simple `Mutex<Option<Result>>` return values.
Rejected: cannot verify call order or detect excess / missing calls.

### 3 — `embedded-hal-mock` role

The crate is added as a dev-dependency to:
1. Pull in `MockError` (a concrete `Debug + Display` error type useful as
   the `Err` variant in mock return values)
2. Establish a consistent testing idiom shared with any future GPIO-level
   tests once real hardware code is wired in
3. Signal to downstream implementors that embedded-hal-mock is the approved
   mock tooling

We do not use `eh1::pin::Mock` yet — there is no GPIO code to mock.

### 4 — Handler integration tests location: `src/server/handlers/*_test.rs`

Each handler file gets a sibling `*_test.rs` (or inline `#[cfg(test)]` mod)
that constructs the mock HAL and calls the handler directly (bypassing TCP).
This avoids the complexity of a fake TCP stack.

## Risks / Trade-offs

- [Mock drift] Mock types must be kept in sync with trait changes →
  Mitigation: mock impls are in the same file as the traits; compiler
  enforces trait satisfaction at every edit.
- [async in tests] Async trait methods require an executor in tests →
  Mitigation: `embassy-executor`'s `arch-spin` executor works for tests;
  alternatively use `tokio::test` once the host target is confirmed.
  Start with `futures::executor::block_on` from the already-available
  `futures` crate (or inline async test with `#[tokio::test]` if added).
- [VecDeque allocation] Mocks use `std::collections::VecDeque` → only
  compiled in `#[cfg(test)]` on the host (not on ESP32), so `std` is fine.

## Migration Plan

1. Uncomment dev-dependencies in `Cargo.toml`
2. Add `src/hal/mock.rs` with all 7 mock structs
3. Wire `src/hal/tests.rs` to use the new mocks (replace hand-rolled ones)
4. Add handler-level tests in `src/server/handlers/`
5. `cargo test` must pass; `cargo check --features esp32` must still compile

No rollback complexity — all new code is `#[cfg(test)]`-only.

## Open Questions

- **Async test executor**: confirm whether `futures::executor::block_on` is
  sufficient or whether we need to add `tokio` as a dev-dep for `#[tokio::test]`.
- **Coverage target**: how many handler tests are required before this change
  is considered complete? (Suggest: at least one test per handler module.)
