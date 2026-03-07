## Context

`README.md` is the first file a potential hardware implementor or API client
author reads. It currently contains a pre-architecture bullet list that does not
reflect the library crate, the trait-based HAL design, the server module, or the
build commands. All content decisions are editorial — there are no code changes.

## Goals / Non-Goals

**Goals:**
- Make the README accurate and complete enough that a Rust developer can
  understand the crate's purpose and start implementing a HAL trait without
  reading source code first.
- Document the most important public types (`RobotState`, HAL traits,
  `ApiServer`, `RobotConfig`, `JobItem`, `Capabilities`) and their roles.
- Include build / run commands so contributors don't need to guess.

**Non-Goals:**
- Auto-generating docs from rustdoc — the README is hand-written prose.
- Covering every public symbol — that is `cargo doc`'s job.
- Adding badges, CI links, or deployment instructions not yet implemented.

## Decisions

### Structure of the README

The README will have these top-level sections in order, following
[makeareadme.com](https://www.makeareadme.com/) conventions:

1. **Title + one-line description** — crate identity at a glance.
2. **Project Status** — pre-1.0, active development note (makeareadme.com §Project Status).
3. **What it does** — 2–3 sentence prose overview.
4. **Architecture** — explains the two modules (`hal`, `server`), the
   trait-based dispatch pattern, and why static generics are used instead of
   `dyn Trait`.
5. **Key Types** — a table or subsection per major type with its role.
6. **HAL Traits** — one row per trait with its responsibility.
7. **Installation / Build & Run** — verbatim commands from CLAUDE.md
   (makeareadme.com §Installation).
8. **Usage** — a minimal code example showing a stub HAL implementation wired
   into `ApiServer` (makeareadme.com §Usage). This is the highest-value addition:
   a concrete snippet readers can copy and adapt.
9. **Implementing the HAL** — a short guide for hardware vendors with numbered
   steps (struct → impl trait → wire into ApiServer).
10. **Support** — pointer to GitHub Issues for questions and bug reports
    (makeareadme.com §Support).
11. **Contributing** — statement of openness to contributions; note on running
    `cargo test` and `cargo fmt` before PRs (makeareadme.com §Contributing).
12. **License** — GPL v3 statement (makeareadme.com §License).

_Why this order?_ Readers scan top-to-bottom. High-level motivation before
implementation details matches the mental model of both new contributors and
hardware vendors.

### Source of truth for types and traits

`src/hal/mod.rs` is the canonical source. The README will describe types as
they exist there today; it will not invent or anticipate future changes.

## Risks / Trade-offs

- [Drift] README may fall out of sync as the HAL evolves → Mitigation: keep
  descriptions at the trait level (stable), not at method signature level
  (volatile). Method signatures live in `src/hal/mod.rs` and `cargo doc`.
- [Verbosity] Detailed type docs may duplicate `///` comments → Mitigation:
  README gives *role* descriptions only; rustdoc gives parameter details.
