---
artifact: COMBINATORIAL_MAXIMALIST_TEST_PLAN
date: 2026-06-02
version: 26.6.2
---

# Combinatorial Maximalist Test Plan

## Overview

The test unit is not a command. The test unit is a **capability**. The proof unit is a
**combination**. The completion unit is a **receipt**.

This plan enumerates 11 capability dimensions, defines three coverage bands, names seven proof
families, and states the manufacturing rule that derives tests from source law.

---

## Manufacturing Rule

```
Source law (INVARIANTS.md)
  → Capability matrix (CAPABILITY_TEST_MATRIX.md)
    → Fixture ledger (NEGATIVE_FIXTURE_LEDGER.md)
      → Test code
        → Receipt (pass/fail assertion with named law)
```

No test may be written that does not trace to at least one row in the capability matrix. No
capability matrix row may exist that does not trace to at least one invariant. Tests that exist
only for coverage percentage are forbidden artifacts.

---

## 11 Capability Dimensions

### D1 — Command Surface

The enumerated commands and sub-commands under test:

- `status` (bare)
- `status show`
- `target show`
- `target prune`
- `test changed`
- `trybuild changed`
- `git status`
- `git close`
- `publish run`
- `workspace doctor`

### D2 — Feature Flags

Cargo feature combinations:

- `(none)` — base, default features only
- `process-data` — event emission enabled
- `autonomic` — suggest/plan/apply modes
- `wasm4pm` — graduation bridge active
- `process-data,autonomic` — combined
- `process-data,wasm4pm` — combined
- `autonomic,wasm4pm` — combined
- `process-data,autonomic,wasm4pm` — all

### D3 — Workspace Shape

- Single-crate workspace
- Multi-crate Cargo workspace
- Missing `Cargo.toml` (no manifest)
- Virtual workspace (workspace-level manifest only)
- Nested workspace (crate within a workspace)

### D4 — Toolchain State

- Stable toolchain (no `rust-toolchain.toml`)
- Nightly toolchain, pinned via `rust-toolchain.toml`
- `rust-toolchain.toml` present but file specifies wrong/missing channel
- Toolchain binary absent from PATH
- Toolchain mismatch (file says nightly, active is stable)

### D5 — Git State

- Clean tree (no uncommitted changes)
- Dirty tracked (modified tracked files)
- Untracked files present
- Staged but uncommitted changes
- Ahead of remote (unpushed commits)
- Behind remote (unpulled commits)
- Detached HEAD

### D6 — Target State

- No `target/` directory
- Small `target/` (under limit)
- `target/` over configured size limit
- Stale incremental profiles only (no release artifacts)
- Release artifacts present in `target/release/`

### D7 — Test State

- No tests in workspace
- Unit tests only, all passing
- Integration tests present
- Changed source file (triggers changed-test plan)
- Changed test file only (no source change)
- Tests failing

### D8 — Trybuild State

- No trybuild fixtures
- Fixtures present, none changed
- Changed fixture source (`.rs` file modified)
- Changed expected output (`.stderr` file modified)
- Snapshot mismatch (fixture compiles but stderr differs)
- Huge fixture set (>500 fixtures)

### D9 — CicdToml State

- `cicd.toml` absent
- `cicd.toml` present and valid
- `cicd.toml` present but stale (inputs changed since last write)
- `cicd.toml` present but corrupted (unparseable)
- `cicd.toml` partial (valid TOML, missing required sections)

### D10 — Autonomic Mode

- Autonomic disabled (default)
- Suggest mode (propose actions, do not execute)
- Plan mode (generate full plan, do not execute)
- Apply-forbidden (apply attempted but blocked by law)

### D11 — wasm4pm State

- wasm4pm binary not found on PATH
- Scan-only (binary found, no file exchange)
- File exchange enabled
- Shell-out mode
- Local adapter registered
- Deferred (capability discovered but not yet classified)

---

## 3 Coverage Bands

### Band 1 — Singleton

Each dimension value is exercised at least once in isolation. All other dimensions are set to
their baseline (clean, small, absent, default).

**Target count:** one test per non-baseline dimension value across all 11 dimensions.

### Band 2 — Pairwise (2-wise)

Every pair of dimension values is covered by at least one test. The standard pairwise covering
array is generated from the 11 dimensions above using the IPOG algorithm. This is the minimum
acceptable coverage for any release.

**Target count:** approximately 80–120 tests after deduplication with Band 1.

### Band 3 — Critical 3-wise

Manually selected three-way combinations that represent dangerous interactions where pairwise
coverage would miss failure modes. See CAPABILITY_TEST_MATRIX.md for the enumerated critical
3-wise rows.

**Target count:** 5 explicitly named cases (expandable by law addition).

---

## 7 Proof Families

### PF1 — Refusal Proof

The system refuses an unsafe or invalid request. The test asserts a non-zero exit code **and**
a specific error message containing a named law or reason. A refusal with a generic message is
not a valid proof.

### PF2 — Determinism Proof

Running the same command twice on unchanged inputs produces identical output (stdout, exit code,
and any generated files). The test runs the command, hashes outputs, runs again, and compares
hashes.

### PF3 — Conservation Proof

A destructive-adjacent operation (prune, close, publish) does not destroy artifacts that must
be preserved. The test asserts the preserved artifact exists and is byte-identical after the
operation.

### PF4 — Isolation Proof

Enabling a feature flag does not change facts visible to a command that does not use that flag.
The test runs a command with and without the flag and diffs the relevant output subset.

### PF5 — Boundary Proof

No internal implementation term appears in any public-facing surface (stdout, stderr, help
text). The test captures full output and scans for each forbidden term in the invariant list.

### PF6 — Discovery Proof

A capability is only exercised when it is first discovered and classified. The test asserts that
a capability path is not entered when the capability binary/adapter is absent, even when the
corresponding feature flag is enabled.

### PF7 — Honest Partial Proof

When a capability is partially available (binary present, some exchange modes absent), the
command reports PARTIAL rather than fabricating a success or a failure. The test asserts the
PARTIAL signal and the presence of an honest reason.

---

## Notes on Test Authoring

- Every test must name the proof family it belongs to in a comment.
- Every test must name the invariant(s) it exercises.
- A test that exercises no invariant is a cosmetic artifact and must not be committed.
- Smoke tests (Band 1 singleton, basic exit-0) are valid receipts only when they trace to an
  invariant.
