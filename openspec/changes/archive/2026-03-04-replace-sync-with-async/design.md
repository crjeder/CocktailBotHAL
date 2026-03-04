## Context

All seven HAL traits (`ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`,
`SensorHal`, `DispenseHal`, `CleaningHal`) currently expose synchronous methods.
The async HTTP server (`ApiServer::run`) already runs on embassy, but handler
functions call HAL methods synchronously — blocking the executor while
hardware I/O is outstanding.

`RobotHal` holds each HAL as a `&mut dyn Trait` reference. Native Rust `async fn`
in traits (stable since Rust 1.75 / edition 2021) is not object-safe, so `dyn Trait`
cannot be used directly with async trait methods without additional tooling.

The project runs no_std + alloc. The async executor is embassy-executor 0.9
with the `arch-spin` feature for development; ESP32 deployment uses `esp-hal-embassy`.

## Goals / Non-Goals

**Goals:**
- All HAL trait methods become `async fn`, enabling cooperative await at every
  hardware call site.
- All call sites in `src/server/handlers/` gain `.await`.
- Stub implementations compile with `async fn`.
- The `fn main()` entry point becomes async, driven by the existing embassy spin
  executor.
- Crate version bumped to `0.2.0` (breaking public interface change).

**Non-Goals:**
- Implementing real hardware drivers (stubs remain `todo!()`).
- Changing the HTTP server's network layer or SSE skeleton.
- Introducing `async-trait` or any new dependency.
- Changing `API.yaml` or `openspec/config.yaml`.

## Decisions

### Decision 1 — Native `async fn` in traits + generics on `RobotHal`

**Choice:** Use Rust 1.75+ native `async fn` in traits and replace the seven
`&mut dyn Trait` fields in `RobotHal` with seven generic type parameters.

**Why not `async-trait`?**
`async-trait` boxes every returned future (`Pin<Box<dyn Future>>`), adding a heap
allocation per HAL call. This is unacceptable on memory-constrained embedded targets
and adds a compile-time dependency not already in `Cargo.toml`.

**Why not keep `dyn Trait`?**
Native `async fn` in traits produces opaque, un-nameable future types. Rust does not
(yet) support object-safe async traits without explicit boxing. Keeping `dyn Trait`
would force `async-trait` or equivalent hand-rolled boxing.

**Why generics?**
Embassy's own HAL crates (embedded-hal-async, embassy-hal-internal) use static
dispatch via generics. Monomorphization eliminates vtable overhead and is compatible
with no_std. With seven type parameters the `RobotHal` struct is verbose but
straightforward; a type alias in `server/mod.rs` keeps handler signatures clean.

```rust
// src/server/mod.rs
pub struct RobotHal<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean> {
    pub control: Ctrl,
    pub status: Stat,
    pub config: Cfg,
    pub storage: Stor,
    pub sensors: Sens,
    pub dispense: Disp,
    pub cleaning: Clean,
}
```

`ApiServer` gains the same seven type parameters (or wraps `RobotHal<…>`).

### Decision 2 — Async entry point via `embassy_executor::Executor::run`

**Choice:** Replace `fn main()` with a standard Rust `fn main()` that constructs
an `embassy_executor::Executor`, spawns an async task, and calls `executor.run()`.
Do not use `#[embassy_executor::main]` (unavailable with `arch-spin`; see MEMORY.md).

```rust
fn main() {
    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| { spawner.spawn(async_main(spawner)).unwrap(); });
}
```

`static-cell` is the idiomatic pattern for embassy spin executors. Check
`Cargo.toml` for a commented-out entry before adding it; if absent, add it.

**ESP32 note:** The comment in `src/main.rs` stays — real bring-up replaces
the entire entry point with `#[esp_hal::main]`.

### Decision 3 — HAL method mutability

Methods that were `&self` (read-only) keep `&self`; methods that were `&mut self`
keep `&mut self`. No mutability change is needed; adding `async` does not affect
receiver types.

## Risks / Trade-offs

- **Seven type parameters on `RobotHal`**: Verbose but manageable. A type alias
  `type ConcreteHal = RobotHal<Stub…, Stub…, …>` in `main.rs` keeps the entry
  point readable.
  → Mitigation: define the alias immediately; document the pattern in a comment.

- **`static-cell` dependency**: `static-cell` may not be in `Cargo.toml`. It is
  a tiny crate (no-std compatible) and is the standard embassy pattern.
  → Mitigation: check commented-out dependencies first; add if absent.

- **Trait object flexibility lost**: Downstream crates that stored `Box<dyn HalTrait>`
  must switch to generics or add their own boxing layer.
  → Accepted trade-off; the semver bump communicates the breaking change.

- **`cargo check` may reveal lifetime issues**: Generic `RobotHal` fields no longer
  hold references with explicit lifetimes; stubs are owned values. Actual hardware
  drivers with lifetimes will use generic bounds (e.g., `Ctrl: ControlHal + 'static`).
  → Mitigation: add `'static` bounds where the executor requires them.

## Migration Plan

1. Change trait method signatures in `src/hal/mod.rs` (`async fn`).
2. Update stub impls in `src/main.rs` (`async fn`).
3. Update ESP32 stub impls in `src/esp32/` (`async fn`).
4. Update `RobotHal` / `ApiServer` in `src/server/mod.rs` to use generics.
5. Add `.await` at each HAL call site in `src/server/handlers/`.
6. Convert `fn main()` to the embassy spin executor pattern.
7. Add `static-cell` to `Cargo.toml` if not already present.
8. Bump version to `0.2.0` in `Cargo.toml`.
9. Run `cargo fmt` and `cargo check`; fix any lifetime or bound errors.

No rollback strategy required — this is a local codebase with no deployed consumers.

## Open Questions

- Does `static-cell` appear in the commented-out `Cargo.toml` entries? (Check
  during implementation; add if absent.)
- Will the embassy spin executor require `+ Send` bounds on the task future?
  Embassy's spin executor typically does not require `Send`; confirm at compile time.
