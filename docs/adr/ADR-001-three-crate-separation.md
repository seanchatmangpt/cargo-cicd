# ADR-001: Three-Crate Separation

**Status:** Accepted
**Date:** 2026-06-03

## Context

cargo-cicd started as a single crate with CLI parsing, integration wiring, and domain logic mixed together. Verb `run()` methods accumulated subprocess calls, file I/O, parsing, and output formatting in a single body. This made domain logic unreachable by tests without invoking the full binary, and impossible to reuse from alternate surfaces.

## Decision

CLI, integration, and domain logic are separated into three distinct crates with enforced import rules:

- **Crate 1 (CLI):** NounCommand + VerbCommand implementations. May import clap, anyhow, and the integration crate. Must not contain business logic.
- **Crate 2 (Integration):** CliBuilder, VerbArgs, command wiring. Routes calls; no business logic.
- **Crate 3 (Domain):** Pure functions. May import std, anyhow, serde, domain types. Must not import clap or invoke CLI processes.

The dependency arrow flows downward only: CLI → Integration → Domain. Domain never imports CLI or Integration.

## Rationale

Flat `run()` methods that do everything are untestable, unreusable, and impossible to surface from anything other than the CLI. Domain functions called directly from tests validate behavior without subprocess invocation. New surfaces (HTTP, TUI, JSON output) can call the same domain functions without touching the CLI tier.

## Consequences

- Every domain function is independently testable with `#[test]`.
- Adding a new output surface requires only a new Crate 1 implementation.
- `run()` must delegate to a domain function within its first few lines. Growth in `run()` is a defect signal.
- Imports are checkable at compile time — cross-tier imports produce compiler errors.

## Violation

If domain crates import clap or if `run()` bodies accumulate logic, tests must invoke the binary, domain logic cannot be reused, and the test suite becomes integration-only with no unit coverage.
