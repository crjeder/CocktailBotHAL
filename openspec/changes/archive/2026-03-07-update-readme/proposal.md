## Why

The current README.md is a stale bullet-list stub that predates the library's
current architecture. New contributors and hardware implementors have no written
guide to the crate's structure, key types, or how to implement the HAL traits.

## What Changes

- Rewrite `README.md` following [makeareadme.com](https://www.makeareadme.com/)
  conventions, covering: project overview, architecture, key types and HAL
  traits reference, build/installation commands, **usage example** (minimal HAL
  implementation in code), **contributing** guide, **project status** note
  (pre-1.0, active), **support** pointer, and license.

## Capabilities

### New Capabilities

_(none — this is a documentation-only change)_

### Modified Capabilities

_(none — no spec-level requirements change)_

## Impact

- `README.md` only — no Rust source, API, or dependency changes.
