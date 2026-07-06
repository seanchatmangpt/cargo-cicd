# Definition of Done — cargo-cicd

**Version:** v26.6.2
**Date:** 2026-06-14
**Authority:** CLAUDE.md, SOLUTION_ARCHITECTURE.md, docs/testing/INVARIANTS.md

---

## Overview

"Done" in cargo-cicd means more than tests passing. Every work item must satisfy the
checklist for its category before it is considered complete. The checklists below are
non-negotiable. Partial completion is not Done.

---

## 1. Feature Completion

A feature is Done when all of the following boxes are checked.

### 1.1 Code

- [ ] Implementation compiles with `cargo build` (no feature flags)
- [ ] Implementation compiles with `--features process-data`
- [ ] Implementation compiles with `--features autonomic`
- [ ] Implementation compiles with `--features wasm4pm`
- [ ] Implementation compiles with `--all-features`
- [ ] `cargo make check` passes (lint + type-check, all feature combinations)
- [ ] Clippy clean: `cargo clippy --all-features -- -D warnings`
- [ ] No `unwrap()` or `expect()` in production paths outside tests
- [ ] Adapter, if added, contains no business logic (pure translation only)
- [ ] Noun verb delegates immediately to domain functions — no logic in `run()`

```sh
cargo make check
cargo clippy --all-features -- -D warnings
```

### 1.2 Tests

- [ ] At least one fixture-based integration test covers the happy path
- [ ] At least one fixture-based integration test covers each error path
- [ ] Invariant `I1` still passes: `cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help`
- [ ] Invariant `I4` still passes: `cargo test --test invariants invariant_no_destructive_default_target_prune_is_safe`
- [ ] Invariant `I5` still passes: `cargo test --test invariants invariant_no_full_trybuild_by_default`
- [ ] Feature projection test still passes: `cargo test --test feature_projection`
- [ ] `cargo make test` passes (all integration tests, no feature flags)

```sh
cargo test --test invariants
cargo test --test feature_projection
cargo make test
```

### 1.3 wasm4pm Evidence

- [ ] The new command emits a `ProcessEvent` via `emit_xes` on completion
- [ ] Evidence is written to `target/cargo-cicd/evidence/` (returned by `evidence_dir()`)
- [ ] An evidence-gate test exists in `tests/wasm4pm_evidence_gate.rs` for the new command
- [ ] Evidence gate test passes with `Accept` when `wpm` is present, `Blocked` when absent
- [ ] `cargo test --test wasm4pm_evidence_gate` passes

```sh
cargo test --test wasm4pm_evidence_gate
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate  # when wpm is available
```

### 1.4 Policy Check (if feature touches autonomic path)

- [ ] New or affected policy runs in `suggest` mode only — no destructive action
- [ ] Policy reads `PolicyState` and emits a `[suggest]`-prefixed recommendation
- [ ] Policy is suppressible via `cicd.toml [policy] disabled = [...]`
- [ ] `cargo test --test autonomic_policies` passes

```sh
cargo test --test autonomic_policies
```

### 1.5 Documentation

- [ ] Public-facing help text (`about()` strings) uses only public-safe language
- [ ] No forbidden term appears in any `about()`, error message, or generated output:
  `ALIVE`, `Inspection Gate`, `wall`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`
- [ ] Command added to `docs/reference/commands.md` table (or regenerated via `ggen`)
- [ ] If a new noun/verb: command reference file added under `docs/reference/commands/`
- [ ] CLAUDE.md Architecture section updated if new dimension added to `EngineState`

### 1.6 Commit Format

- [ ] Commit message follows: `feat(core|cli|target|test|git|autonomic|docs|receipts): description`
- [ ] No `--no-verify` bypass of pre-commit hooks

---

## 2. Bug Fix Completion

A bug fix is Done when all of the following boxes are checked.

### 2.1 Reproduction

- [ ] A failing test reproduces the bug before the fix is applied
- [ ] The test uses a fixture from `tests/fixtures/` (or a new fixture that models the bug)
- [ ] Test failure message names the invariant or law that the bug violates

### 2.2 Fix

- [ ] Fix targets the adapter or domain function — not the noun/verb layer
- [ ] Fix does not introduce business logic into any adapter
- [ ] Fix does not change public output format for unrelated commands (I7 compliance)
- [ ] `cargo make check` passes after fix

### 2.3 Regression Test

- [ ] The reproduction test now passes with the fix applied
- [ ] All other invariant tests still pass: `cargo test --test invariants`
- [ ] `cargo make test` passes (full test suite)

### 2.4 Changelog

- [ ] Commit message follows: `feat(core|cli|...): fix <description>`
- [ ] If the bug touched a public-facing surface: crates.io release notes updated

---

## 3. Release Completion

A release is Done when all of the following boxes are checked. These are in addition
to all feature/bug checklist items for changes in the release.

### 3.1 Invariant Tests

- [ ] `cargo test --test invariants` passes (all 4 invariant tests)
  - [ ] `invariant_public_boundary_no_forbidden_terms_in_all_help`
  - [ ] `invariant_no_false_close_git_close_help_mentions_safety`
  - [ ] `invariant_no_destructive_default_target_prune_is_safe`
  - [ ] `invariant_no_full_trybuild_by_default`
  - [ ] `invariant_wasm4pm_scan_or_documented_absence`

```sh
cargo test --test invariants -- --nocapture
```

### 3.2 Feature Flag Matrix

- [ ] `cargo build` (default features) — PASS
- [ ] `cargo build --features process-data` — PASS
- [ ] `cargo build --features autonomic` — PASS
- [ ] `cargo build --features wasm4pm` — PASS
- [ ] `cargo build --all-features` — PASS
- [ ] `cargo test --features process-data` — PASS
- [ ] `cargo test --features autonomic` — PASS

```sh
cargo build
cargo build --features process-data
cargo build --features autonomic
cargo build --features wasm4pm
cargo build --all-features
```

### 3.3 wasm4pm Evidence Gate

- [ ] `cargo test --test wasm4pm_evidence_gate` passes (Accept or Blocked)
- [ ] `cargo test --test wasm4pm_evidence_mutation` passes (all mutation cases Refuse or Blocked)
- [ ] `cargo test --test wasm4pm_refusal_cases` passes (all refusal cases Refuse or Blocked)
- [ ] `evidence_gate_oracle_discover` does not panic
- [ ] When `wpm` binary is available: `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate` passes with Accept

```sh
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

### 3.4 Receipt Doctor Gate

- [ ] `cargo cicd evidence doctor` exits 0 and reports state: Admitted
- [ ] `wpm receipt doctor --format json --strict target/cargo-cicd/evidence/receipts/latest.json` returns `Admitted`
- [ ] `cargo cicd publish run` exits 0 with `ADJUDICATED:accept` in output (or `WARN:oracle_unavailable` if wpm absent)

```sh
cargo cicd evidence doctor
wpm receipt doctor --format json --strict target/cargo-cicd/evidence/receipts/latest.json
cargo cicd publish run
```

### 3.5 crates.io Readiness

- [ ] `Cargo.toml` has all required fields: `name`, `version`, `description`, `license`, `repository`, `readme`, `keywords`, `categories`
- [ ] `README.md` is public-safe and install-focused (no forbidden terms, no private doctrine)
- [ ] Both `LICENSE-MIT` and `LICENSE-APACHE` are present
- [ ] No `path = "..."` dependencies in `[dependencies]`
- [ ] `cargo package --list` reviewed — private paths excluded via `exclude` list
- [ ] `cargo publish --dry-run` passes without errors
- [ ] Working tree is clean: `git status` shows no uncommitted changes
- [ ] Branch is pushed to remote: `git push` completed

```sh
cargo package --list
cargo publish --dry-run
git status
```

### 3.6 Receipts

- [ ] `receipts/` directory contains a release receipt for this version
- [ ] Receipt names version, date, and includes crates.io checklist results
- [ ] Receipt is committed to the repository

### 3.7 MSRV Compliance

- [ ] `rust-toolchain.toml` pins toolchain to Rust 1.85 or later
- [ ] `cargo build` succeeds on the pinned toolchain version

---

## 4. Documentation Completion

A documentation change is Done when all of the following boxes are checked.

### 4.1 CLAUDE.md Sync

- [ ] CLAUDE.md Architecture section reflects any new `EngineState` dimensions
- [ ] CLAUDE.md Nouns list updated if a noun was added or removed
- [ ] CLAUDE.md Build & Test Commands updated if new test commands were added
- [ ] CLAUDE.md Forbidden terms list unchanged (do not add new entries without ADR)

### 4.2 Public/Private Visibility

- [ ] All public docs (`docs/commands/`, `docs/reference/`, `docs/tutorials/`, README) use only public-safe language
- [ ] Internal receipts and architecture notes (`receipts/`, ADRs, CLAUDE.md) may use internal vocabulary
- [ ] Generated command reference files under `docs/reference/commands/` have been re-generated if ontology changed: `ggen`

### 4.3 Forbidden Term Scan

- [ ] Forbidden terms absent from all files in `docs/commands/`, `docs/reference/`, `docs/tutorials/`, `README.md`, `src/`
- [ ] Verify with: `grep -rE "ALIVE|Inspection Gate|wall|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8" docs/commands/ docs/reference/ docs/tutorials/ README.md src/`
- [ ] `cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help` passes

```sh
grep -rE "ALIVE|Inspection Gate|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8" \
  docs/commands/ docs/reference/ docs/tutorials/ README.md src/ || echo "CLEAN"
```

---

## 5. Adapter Completion

An adapter is Done when all of the following boxes are checked.

### 5.1 Adapter Written

- [ ] Adapter lives in `src/adapters/<source>_adapter.rs`
- [ ] Adapter implements a single external-source query — no multi-source aggregation
- [ ] Adapter contains no business logic: no filtering, interpretation, or decision-making
- [ ] Adapter propagates errors via `anyhow::Result` — no silent swallowing
- [ ] Adapter is registered in `src/adapters/mod.rs`

### 5.2 EngineState Dimension Populated

- [ ] A corresponding `*State` type exists in `src/engine/`
- [ ] `EngineState` struct in `src/engine/mod.rs` includes the new dimension
- [ ] The noun that uses the adapter reads from `EngineState`, not directly from the adapter
- [ ] CLAUDE.md `EngineState` diagram updated to show the new field

### 5.3 Fixture Tests

- [ ] At least one test exercises the adapter via a clean fixture: `FixtureWorkspace::clean()`
- [ ] At least one test exercises the adapter on the failure path (e.g., `FixtureWorkspace::missing_manifest()`)
- [ ] Tests use `tests/fixtures/mod.rs` fixtures — not raw `TempDir` unless no suitable fixture exists
- [ ] Tests pass: `cargo test --test <relevant_test_file>`

### 5.4 No Business Logic

- [ ] Adapter passes the "pure translation" review: given the same external source output, the adapter always produces the same `*State` value
- [ ] Adapter does not invoke other adapters
- [ ] Adapter does not write to `cicd.toml` (that is `CicdTomlWriter`'s sole responsibility)
- [ ] Code review confirms: adapter reads → translates → returns; nothing else

---

## 6. Policy Completion

A policy is Done when all of the following boxes are checked.

### 6.1 Suggest-Mode Only

- [ ] Policy emits a `[suggest]`-prefixed recommendation — never takes action
- [ ] Policy does not modify `cicd.toml`, the filesystem, or any external state
- [ ] `--apply` flag is recognized but documented as not-yet-functional
- [ ] Policy is listed in `docs/explanation/autonomic-policies.md` with trigger conditions and suggestion text

### 6.2 PolicyState Populated

- [ ] Policy reads from `PolicyState` and/or other `EngineState` dimensions
- [ ] Policy result is stored back into `PolicyState` (not emitted directly from the noun)
- [ ] `PolicyState` in `src/engine/` has a field for the new policy verdict

### 6.3 Fixture Verdict Verified

- [ ] A test in `tests/autonomic_policies.rs` or `tests/policies.rs` verifies the policy fires on a triggering fixture
- [ ] A test verifies the policy does NOT fire on a non-triggering fixture
- [ ] Policy is suppressible via `cicd.toml [policy] disabled = ["PolicyName"]`
- [ ] `cargo test --test autonomic_policies` passes

```sh
cargo test --test autonomic_policies
cargo test --test policies
```

### 6.4 cicd.toml Configuration

- [ ] Policy threshold is configurable in `cicd.toml [policy.<name>]` section
- [ ] Default threshold is documented in `docs/explanation/autonomic-policies.md`
- [ ] `cargo test --test cicd_toml_truth` passes (cicd.toml schema unchanged)

```sh
cargo test --test cicd_toml_truth
```

---

## Summary: Quick Reference

| Category | Minimum gate command |
|----------|---------------------|
| Feature | `cargo make test` + `cargo test --test wasm4pm_evidence_gate` |
| Bug Fix | `cargo test --test invariants` + `cargo make test` |
| Release | All gates above + `cargo publish --dry-run` + wasm4pm receipt doctor |
| Documentation | Forbidden term grep + `cargo test --test invariants` |
| Adapter | Fixture tests + `cargo make check` + `cargo make test` |
| Policy | `cargo test --test autonomic_policies` + `cargo test --test cicd_toml_truth` |

No work item is Done until its minimum gate command passes without errors.
