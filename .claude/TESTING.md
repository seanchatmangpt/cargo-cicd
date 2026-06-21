# cargo-cicd Testing Guide

This document is the definitive reference for understanding, running, and writing tests in the
`cargo-cicd` repository. Every test must trace to at least one invariant; every invariant must be
enforced by at least one test.

---

## 1. Test Hierarchy

Tests are stratified into two tiers based on their gate type and oracle involvement. The tier
boundary is intentional: Tier 1 runs everywhere, Tier 2 requires release conditions.

### Tier 1 — Unit & Smoke Tests (Non-Closing Gates)

**Purpose:** Validate internal logic, public boundaries, CLI grammar, and feature flag surface.

**Tools used:** `assert_cmd`, `tempfile`, local fixtures, no external processes beyond the binary.

**When they run:** Every `cargo make test` invocation, every CI push, every local change.

**What they prove:** The binary compiles correctly, all nouns and verbs parse, the public boundary
holds, and no invariant is silently broken.

**Files in this tier:**

| File | What it validates |
|---|---|
| `tests/invariants.rs` | 7 non-negotiable public boundary rules (see Section 4) |
| `tests/cli/test_status.rs` | `status show` output, exit codes, dirty-workspace signaling |
| `tests/cli/test_target.rs` | `target show` and `target prune` dry-run safety |
| `tests/cli/test_publish.rs` | `publish run` with complete and incomplete metadata |
| `tests/cli/test_git.rs` | `git status`, `git close`, `git phase` lifecycle |
| `tests/cli/test_workspace.rs` | `workspace doctor` output and missing-manifest handling |
| `tests/cli/test_evidence.rs` | `evidence doctor` and `evidence audit` CLI shape |
| `tests/cli/command_projection.rs` | All noun+verb combinations parse without panicking |
| `tests/cli/verb_registry.rs` | Every registered verb has a handler; no orphan verbs |
| `tests/feature_projection.rs` | Feature flags do not remove or invert existing output facts |
| `tests/feature_projections.rs` | Extended feature flag surface contract |
| `tests/cicd_toml_truth.rs` | `cicd.toml` serialization/deserialization round-trip |
| `tests/autonomic_policies.rs` | Policy evaluation logic for each named policy |
| `tests/changed_tests.rs` | `ChangedFileDetector` classification accuracy |
| `tests/git_phase_closure.rs` | Git state detection and phase tracking correctness |
| `tests/fixture_workspaces.rs` | Fixture workspace builder utilities |
| `tests/ggen_customization_guard.rs` | Regeneration from ontology is idempotent |
| `tests/lsp_explain.rs` | `lsp explain` endpoint shape and response structure |
| `tests/interactions.rs` | User interaction flows and prompt-response cycles |
| `tests/policies.rs` | Policy contract surface (entry shape, verdict categories) |
| `tests/publish_gate.rs` | Publishing gate behavior with various manifest states |

### Tier 2 — Evidence Gate Tests (Closing — Release Gate)

**Purpose:** Verify process conformance through external oracle adjudication. No release ships
without these passing.

**Oracle:** wasm4pm (`wpm` binary). The oracle adjudicates all process evidence.

**When they run:** Release checklist, CI with `REQUIRE_WPM_ORACLE=1`, and locally when `wpm` is
on `PATH`.

**What they prove:** The XES evidence emitted by `cargo-cicd` is accepted by an independent
process oracle. Mutation tests prove the oracle is a real adjudicator, not a rubber stamp.

**Files in this tier:**

| File | What it validates |
|---|---|
| `tests/wasm4pm_evidence_gate.rs` | Happy path: valid evidence → Accept verdict |
| `tests/wasm4pm_evidence_mutation.rs` | Corrupt evidence → Refuse verdict |
| `tests/wasm4pm_refusal_cases.rs` | Edge cases: missing file, empty XES, malformed XML |
| `tests/wasm4pm_harness.rs` | Shared harness utilities for evidence gate tests |
| `tests/wasm4pm_shell.rs` | Shell invocation layer for the wpm oracle |
| `tests/wpm_verdict_key_contract.rs` | Verdict enum structure and key encoding contract |
| `tests/refusal_calibration.rs` | Calibration of oracle sensitivity across mutation types |

---

## 2. Test Files Map

### Core Invariant Tests

**`tests/invariants.rs`** — The non-negotiable gate. Enforces all 7 invariants (see Section 4).
This file runs on every CI job and must exit 0 before any release is tagged. The most important
test in this file is `invariant_public_boundary_no_forbidden_terms_in_all_help`, which scans
every `--help` output for internal terms.

### CLI Grammar Tests (`tests/cli/`)

The `tests/cli/` directory validates the noun-verb CLI grammar manufactured from the ontology.
Each file maps to one noun. Tests use `assert_cmd::Command::cargo_bin("cargo-cicd")` to invoke
the real binary and inspect output.

**`test_status.rs`** — Validates that `status show` exits 0, reports git phase, detects dirty
workspaces, and does not panic on minimal Cargo.toml inputs.

**`test_target.rs`** — Validates that `target show` reports directory size correctly and that
`target prune` without `--confirm` reports a plan but deletes nothing (Invariant I4).

**`test_git.rs`** — Validates that `git status` reports branch state, `git close` refuses on
dirty workspaces (Invariant I3), and `git phase` reports the correct lifecycle stage.

**`test_publish.rs`** — Validates that `publish run` requires complete manifest metadata (license,
description, readme) and emits appropriate WARN verdicts when fields are missing.

**`test_workspace.rs`** — Validates that `workspace doctor` runs in single-crate and multi-crate
workspaces, and exits non-0 with a clear message when `Cargo.toml` is missing.

**`test_evidence.rs`** — Validates that `evidence doctor` checks for XES evidence files and that
`evidence audit` invokes the oracle path when `wasm4pm` feature is enabled.

**`command_projection.rs`** — Exercises every noun+verb combination to ensure none panic. This is
a smoke test over the manufactured grammar.

**`verb_registry.rs`** — Verifies that every verb registered in a noun has a callable handler.
Catches grammar/implementation drift introduced after `ggen` regeneration.

> **Missing CLI smoke tests** — The following nouns have no corresponding `tests/cli/` file.
> CLI smoke tests must be created for each before they can be considered fully covered:
>
> | Missing file | Noun | What it should validate |
> |---|---|---|
> | `tests/cli/test_analyze.rs` | `analyze` | `analyze` verb invocations, exit codes, output shape |
> | `tests/cli/test_autoarch.rs` | `autoarch` | `autoarch` verb invocations, exit codes, output shape |
> | `tests/cli/test_certification.rs` | `certification` | `certification` verb invocations, exit codes, output shape |
> | `tests/cli/test_sbom.rs` | `sbom` | `sbom` verb invocations, exit codes, output shape |
> | `tests/cli/test_ui.rs` | `ui` | `ui demo` and `ui dashboard` output, no-TTY fallback |

### State and Serialization Tests

**`tests/cicd_toml_truth.rs`** — Round-trip serialization: builds an `EngineState`, serializes
it to `cicd.toml`, deserializes it back, and asserts field equality. Covers the determinism
invariant (I2) by running `publish run` twice on an identical workspace and comparing SHA-256
checksums of the resulting `cicd.toml`.

**`tests/feature_projection.rs`** and **`tests/feature_projections.rs`** — Validates Invariant I7
(FeatureProjectionConsistency). Runs the same command with and without `process-data`, `autonomic`,
and `wasm4pm` features, then verifies that every fact present in the default output is also present
in the feature-enabled output and not inverted.

### Behavioral Tests

**`tests/changed_tests.rs`** — Validates the `ChangedFileDetector` adapter. Creates repositories
with specific sets of modified files and asserts that classification (`.rs` source, trybuild
fixture, unrelated) is correct.

**`tests/git_phase_closure.rs`** — Validates the `GitStatusAdapter`. Creates repositories in
clean, dirty, staged, and untracked states and asserts correct phase detection.

**`tests/autonomic_policies.rs`** — Validates each named autonomic policy. Constructs
`EngineState` inputs that trigger each policy condition, calls `run_all_policies()`, and asserts
that the correct `PolicyVerdict` (`Pass`, `Warn`, or `Skip`) is returned.

### Evidence Gate Tests (`tests/wasm4pm_*.rs`)

See Section 5 for detailed coverage. These tests follow a strict assertion rule: **never assert
on cargo-cicd internal state; only assert on wasm4pm verdict**.

---

## 3. Running Tests

### Basic: Run All Tests

```sh
cargo make test
```

This is the canonical command. It runs the full test suite including all Tier 1 tests. Tier 2
(evidence gate) tests run as part of this command when `wpm` is on PATH; when `wpm` is absent,
they fall back to `ExpectedWpmVerdict::Blocked` gracefully.

### Specific Test Suites by Name

```sh
# Invariants only (fastest gate)
cargo test --test invariants

# CLI grammar tests
cargo test --test cli

# Serialization round-trip
cargo test --test cicd_toml_truth

# Autonomic policies
cargo test --test autonomic_policies

# Changed file classification
cargo test --test changed_tests

# Git phase detection
cargo test --test git_phase_closure

# Feature flag surface
cargo test --test feature_projection
cargo test --test feature_projections

# Evidence gate — happy path
cargo test --test wasm4pm_evidence_gate

# Evidence gate — mutations (negative cases)
cargo test --test wasm4pm_evidence_mutation

# Evidence gate — refusal edge cases
cargo test --test wasm4pm_refusal_cases
```

### Running a Single Test Function

```sh
# Run one function from a test file
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help

# Run one function with output visible
cargo test --test invariants invariant_status_exits_zero -- --nocapture

# Run tests matching a pattern
cargo test --test cli test_status
```

### With Feature Flags

Feature flags change which code paths are compiled and which evidence paths are exercised. Always
test the feature combinations that will ship:

```sh
# Core Level 5 engine
cargo test --features process-data

# Autonomic policy layer
cargo test --features autonomic

# Community contribution tooling
cargo test --features contrib

# wasm4pm oracle integration
cargo test --features wasm4pm

# All non-default features together
cargo test --features autonomic,wasm4pm,contrib

# Single test with features
cargo test --test wasm4pm_evidence_gate --features wasm4pm
```

### Offline Mode (No wpm Oracle)

When the `wpm` binary is not on PATH, all evidence gate tests fall back to
`ExpectedWpmVerdict::Blocked`. This is the expected behavior for local development and CI
environments without wasm4pm installed.

```sh
# Offline — evidence gate tests will use Blocked fallback, not fail
cargo test --test wasm4pm_evidence_gate

# Verify oracle is absent (expected Blocked fallback)
cargo test --test wasm4pm_evidence_gate -- --nocapture 2>&1 | grep -i blocked

# Force failure if oracle is absent (for release gate CI)
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

The `REQUIRE_WPM_ORACLE=1` environment variable causes evidence gate tests to panic immediately
if `wpm` is not available, rather than silently falling back to Blocked. Use this in release
gate CI jobs where the oracle is expected to be present.

---

## 4. The 7 Non-Negotiable Invariants

These invariants are enforced by `tests/invariants.rs`. Every one must pass before any release is
tagged. They represent the minimum correctness contract for the public interface.

### I1 — PublicBoundary (`invariant_public_boundary_no_forbidden_terms_in_all_help`)

**What it checks:** Scans the full `--help` output of every noun, every verb, and the top-level
`cargo-cicd --help` for any of the 10 forbidden internal terms:

- `ALIVE` — Level 5 engine status marker
- `Inspection Gate` — Manufacturing subsystem identity
- `wall` — Jargon from manufacturing pipeline
- `Nehemiah` — Code name for manufacturing layer
- `Field8` — Internal capacity measurement
- `Instinct8` — Autonomic reasoning subsystem
- `Cargo Court` — Internal adjudication metaphor
- `AGI` — AI system classification
- `Truex` — Internal truth engine
- `CONSTRUCT8` — Manufacturing directive system

**How to debug if it fails:**

```sh
# Identify which command leaks the term
cargo run -- --help | grep -i ALIVE
cargo run -- status --help | grep -i "Cargo Court"

# Search source for the term
rg "ALIVE" src/
rg "Nehemiah" src/ templates/

# Check generated files
rg "CONSTRUCT8" README.md docs/
```

Replace any leaked term with its public equivalent. Then re-run:

```sh
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
```

### I2 — PublishDeterminism (`invariant_publish_determinism` in `cicd_toml_truth.rs`)

**What it checks:** Runs `publish run` twice on an identical, unchanging workspace and compares
SHA-256 checksums of the resulting `cicd.toml` files. Asserts byte-equality.

**How to debug if it fails:**

The most common cause is a non-deterministic value being serialized into `cicd.toml`:
- Timestamps embedded without normalization
- HashMap iteration order affecting TOML key ordering
- Random event IDs without stable sorting

```sh
# Run twice and diff the outputs
cargo run -- publish run && cp cicd.toml /tmp/cicd1.toml
cargo run -- publish run && cp cicd.toml /tmp/cicd2.toml
diff /tmp/cicd1.toml /tmp/cicd2.toml
```

Fix by normalizing or sorting the non-deterministic field before serialization.

### I3 — NoFalseClose (`invariant_no_false_close_git_close_help_mentions_safety`)

**What it checks:** Verifies that `git close` refuses (exits non-0) when the working tree is
dirty. The help text must mention safety concepts (`dry`, `safe`, `confirm`, or `check`).

**How to debug if it fails:**

```sh
# Verify git close behavior on dirty tree
git status --porcelain  # Should show dirty files

cargo run -- git close  # Must exit non-0
echo $?

# Check help text
cargo run -- git close --help | grep -E "dry|safe|confirm|check"
```

The `GitStatusAdapter` must detect dirty state before `git close` proceeds.

### I4 — NoDestructiveDefault (`invariant_no_destructive_default_target_prune_is_safe`)

**What it checks:** Creates a fake `target/debug/` directory with files, runs `target prune`
without `--confirm`, and asserts that no files were deleted.

**How to debug if it fails:**

```sh
# Test prune behavior manually
mkdir -p /tmp/test-workspace/target/debug
echo "fake binary" > /tmp/test-workspace/target/debug/cargo-cicd
cargo run --manifest-path /tmp/test-workspace/Cargo.toml -- target prune
ls /tmp/test-workspace/target/debug/  # binary must still exist
```

Look for any code path in `target prune` that deletes without checking for a `--confirm` or
`--yes` flag.

### I5 — NoFullTrybuildByDefault (`invariant_no_full_trybuild_by_default`)

**What it checks:** Creates 100 trybuild fixture files in `tests/ui/compile_fail/`, runs
`trybuild changed` with no git changes, and asserts that the output does not mention running
100 fixtures or "all".

**How to debug if it fails:**

```sh
# Check how changed detection works with no git
git diff origin/main --name-only | grep -i trybuild

# Run with output to inspect
cargo run -- trybuild changed 2>&1 | head -20

# Check ChangedFileDetector logic
rg "is_trybuild_fixture" src/adapters/
```

The `ChangedFileDetector` must return an empty set when no git changes are present, and
`trybuild changed` must report "no changed fixtures" rather than running all.

### I6 — NoAssumedWasm4pmCapability (enforced in `feature_projection.rs`)

**What it checks:** When the `wasm4pm` feature is enabled but the `wpm` binary is absent from
PATH, any command with wasm4pm integration must report `PARTIAL` (not panic, not silently
succeed, not fabricate capability).

**How to debug if it fails:**

```sh
# Test with wasm4pm feature but no wpm binary
PATH="" cargo test --features wasm4pm --test feature_projection

# Check the Wasm4pmShell availability detection
rg "is_available" src/integrations/wasm4pm_shell.rs
rg "PARTIAL" src/
```

Ensure `Wasm4pmShell::is_available()` is called before any oracle invocation, and that the
PARTIAL signal is emitted when the binary is absent.

### I7 — FeatureProjectionConsistency (`invariant_feature_projection_consistency`)

**What it checks:** Runs a command with no features, captures its output facts (exit code, status
lines), then runs the same command with each feature flag and asserts that every fact from the
default run is still present and identical. Feature flags must only add records, never remove or
invert existing facts.

**How to debug if it fails:**

```sh
# Compare output with and without features
cargo run -- status show > /tmp/default.txt
cargo run --features process-data -- status show > /tmp/with-features.txt
diff /tmp/default.txt /tmp/with-features.txt

# A feature flag must not add a new exit code or remove a status line
```

If a feature flag causes a previously-passing command to fail or a previously-reported fact to
disappear, the feature implementation has a consistency violation.

---

## 5. Evidence Gate Tests

### Evidence Gate Concepts

The evidence gate enforces Invariant E1: **cargo-cicd never adjudicates itself**. Only the
external `wpm` oracle issues verdicts. The gate tests prove this contract holds in three
dimensions: happy path (Accept), mutation (Refuse), and edge cases (Blocked).

**Key rule for all evidence gate tests:**

```rust
// WRONG — asserting on internal cargo-cicd state
assert_eq!(state.target.size, expected_size);

// CORRECT — asserting only on wasm4pm verdict
assert_eq!(wpm_verdict, WpmVerdict::Accept);
```

### Happy Path Test Flow (`wasm4pm_evidence_gate.rs`)

Each happy path test follows this pattern:

1. Create a `TempDir` for isolation
2. Construct a `ProcessEvent` with a valid command and `"PASS"` verdict
3. Emit the event to XES via `emit_xes(&events, &xes_path)`
4. Assert the XES file exists on disk (Invariant E2)
5. Create a `WpmEvidenceOracle`
6. If the oracle is available, assert `Accept`; if absent, assert `Blocked`

```rust
#[test]
fn evidence_gate_status_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("status show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");

    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### Mutation Test Patterns (`wasm4pm_evidence_mutation.rs`)

Mutation tests prove the oracle is a real adjudicator, not a rubber stamp. Each test emits valid
XES, corrupts it in a specific way, then asserts `Refuse`:

| Mutation | Test Name | What is corrupted |
|---|---|---|
| Corrupted XML | `evidence_mutation_corrupted_xes_refused` | Entire file replaced with non-XML |
| Empty file | `evidence_mutation_empty_xes_refused` | File truncated to zero bytes |
| Mismatched tags | `evidence_mutation_mismatched_tags_refused` | Closing tags do not match opening |
| Missing verdict | `evidence_mutation_missing_verdict_refused` | `verdict_claimed` attribute removed |
| Fabricated Accept | `evidence_mutation_fabricated_verdict_refused` | verdict changed to `ACCEPT` manually |

The pattern for any mutation test:

```rust
#[test]
fn evidence_mutation_<name>_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mutated.xes");

    // Either write corrupt content directly, or:
    let events = vec![ProcessEvent::new("some command", "PASS")];
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    corrupt_xes_<mutation_type>(&xes_path);  // apply mutation

    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### Refusal Cases and `ExpectedWpmVerdict::Blocked` (`wasm4pm_refusal_cases.rs`)

The refusal cases file tests edge conditions that the happy path and mutation tests do not cover:

- **Missing file:** Calling `audit_xes` on a non-existent path. When oracle is available, must
  return `Refuse`; when absent, returns `Blocked`.
- **Empty XES:** Zero-byte XES file must be `Refuse` (not `Accept` with empty evidence).
- **Corrupted XML:** Non-XML content must be `Refuse`.

`ExpectedWpmVerdict::Blocked` is a first-class expectation, not an error. It means: "the oracle
is unavailable; this test is not skipped, but the Accept/Refuse assertion is suspended." This
allows the full test suite to pass in offline environments (laptops, CI without wpm).

When writing a test that needs full oracle coverage in release gate CI:

```rust
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!(
            "REQUIRE_WPM_ORACLE=1 is set but wpm oracle binary is absent. \
             Test '{}' cannot exercise its Accept assertion.",
            test_name
        );
    }
    ExpectedWpmVerdict::Blocked
}
```

---

## 6. Writing New Tests

### Test Fixture Pattern (TempDir + assert_cmd)

All CLI tests that need an isolated workspace use `tempfile::TempDir`:

```rust
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_my_command_exits_zero() {
    let dir = TempDir::new().unwrap();

    // Set up minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-workspace"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();

    // Invoke the binary
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["my-noun", "my-verb"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expected text"));
}
```

Key rules for fixture tests:
- Always use `TempDir` — never use the real workspace as a test fixture
- Always set `current_dir` to the temp directory
- Use `output()` when you need to inspect stdout/stderr; use `.assert().success()` for exit-only
- Keep fixtures minimal — add only the files the test actually needs

### Never Assert on Internal State — Only wpm Verdict

For any test touching evidence emission or wasm4pm integration:

```rust
// NEVER do this:
let state = EngineState::from_workspace();
assert_eq!(state.git_phase.dirty_files.len(), 0);  // internal state check

// ALWAYS do this:
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}
```

This rule exists because `cargo-cicd` is forbidden from adjudicating itself (Invariant E1).
Internal state checks would couple tests to implementation details that the oracle is meant to
adjudicate independently.

### Regression Test Template

When a bug is fixed, add a regression test immediately:

```rust
/// Regression: <describe the bug> — fixed in <commit or PR reference>.
///
/// Previous behavior: <what went wrong>.
/// Correct behavior: <what should happen>.
#[test]
fn regression_<noun>_<verb>_<description>() {
    let dir = TempDir::new().unwrap();

    // Set up the exact workspace state that triggered the bug
    // ... minimal reproduction ...

    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["noun", "verb"])
        .output()
        .unwrap();

    // Assert the bug is fixed
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "command must not fail: {}", stdout);
    assert!(!stdout.contains("wrong text"), "must not contain old behavior");
    assert!(stdout.contains("correct text"), "must show correct output");
}
```

Place regression tests in the most specific test file for the affected noun. If the bug spans
multiple nouns, add to `tests/invariants.rs` or `tests/cli/command_projection.rs`.

### Feature-Gated Test Template

For tests that only make sense when a specific feature flag is enabled:

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_autonomic_policy_emits_recommendation_when_target_over_limit() {
    use cargo_cicd::engine::EngineState;
    use cargo_cicd::autonomic::policies::run_all_policies;

    // Build a synthetic EngineState that triggers the policy
    let mut state = EngineState::default();
    state.target.total_size_bytes = 200 * 1024 * 1024 * 1024; // 200 GB

    let policies = run_all_policies(&state);
    let target_policy = policies
        .iter()
        .find(|p| p.policy_name == "target_pressure")
        .expect("target_pressure policy must be registered");

    assert_eq!(target_policy.verdict, PolicyVerdict::Warn);
    assert!(
        target_policy.recommendation.contains("target prune"),
        "recommendation must suggest target prune: {}",
        target_policy.recommendation
    );
}
```

Run the feature-gated test:

```sh
cargo test --features autonomic test_autonomic_policy_emits_recommendation_when_target_over_limit
```

For wasm4pm-gated tests, combine with the oracle fallback pattern:

```rust
#[cfg(feature = "wasm4pm")]
#[test]
fn test_evidence_emitted_with_wasm4pm_feature() {
    let dir = TempDir::new().unwrap();
    // ... setup ...

    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### Adding a Test for a New Policy

When adding a new autonomic policy:

1. Add the policy implementation in `src/policies/<policy_name>.rs`
2. Register it in `src/autonomic/policies.rs`
3. Write a test in `tests/autonomic_policies.rs`:

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_<policy_name>_policy_detects_<condition>() {
    let mut state = EngineState::default();
    // Set state fields that trigger the policy condition
    // ...

    let policies = run_all_policies(&state);
    let policy = policies
        .iter()
        .find(|p| p.policy_name == "<policy_name>")
        .expect("<policy_name> policy must be registered");

    assert_eq!(policy.verdict, PolicyVerdict::Warn);
}

#[cfg(feature = "autonomic")]
#[test]
fn test_<policy_name>_policy_passes_when_<condition_not_met>() {
    let state = EngineState::default();  // default state has no triggering condition

    let policies = run_all_policies(&state);
    let policy = policies
        .iter()
        .find(|p| p.policy_name == "<policy_name>")
        .expect("<policy_name> policy must be registered");

    assert!(
        matches!(policy.verdict, PolicyVerdict::Pass | PolicyVerdict::Skip),
        "policy must not warn when condition is absent"
    );
}
```

---

## 7. Test Debugging

### Run a Single Test with `--nocapture`

By default, Rust tests suppress stdout. Use `--nocapture` to see all output:

```sh
# See all output from a single test
cargo test --test invariants invariant_status_exits_zero -- --nocapture

# See output from all tests in a file
cargo test --test cli -- --nocapture 2>&1 | head -100

# Run one specific function from the cli module
cargo test --test cli test_status -- --nocapture
```

### Enable Verbose Adapter Logging with `RUST_LOG=debug`

The adapters (git, toolchain, target scanner) log their activity at `debug` level. Enable this to
see what each adapter is doing during a failing test:

```sh
RUST_LOG=debug cargo test --test changed_tests -- --nocapture 2>&1

# Or for a single adapter
RUST_LOG=cargo_cicd::adapters::git_status=debug cargo test --test git_phase_closure -- --nocapture
```

Common adapter debug signals to look for:
- `GitStatusAdapter` — reports the raw `git status --porcelain` output and parsed dirty files
- `ChangedFileDetector` — reports which files `git diff` returned and how they were classified
- `TargetScannerAdapter` — reports the walkdir traversal count and total bytes
- `ToolchainDetector` — reports the raw `rustc --version` output

### Inspect XES Evidence Output

When evidence gate tests fail, inspect the actual XES files written:

```sh
# Run a test that emits evidence and keep the temp dir
# (use env var if the test supports it, or add a manual path)
EVIDENCE_DIR=/tmp/cicd-evidence cargo test --test wasm4pm_evidence_gate -- --nocapture

# Check what was written
ls -la /tmp/cicd-evidence/
cat /tmp/cicd-evidence/events.xes

# Manually audit with wpm
wpm audit /tmp/cicd-evidence/events.xes
# Output: Accept / Refuse / Blocked
```

The standard evidence output directory during normal operation is:
```
target/cargo-cicd/evidence/evt-*.xes
target/cargo-cicd/evidence/evt-*.jsonl
```

```sh
# After running cargo-cicd normally
ls -la target/cargo-cicd/evidence/
wpm audit target/cargo-cicd/evidence/evt-*.xes
```

### Debugging a Failing Invariant

When an invariant fails, the error message includes the command that triggered it:

```
thread 'invariant_public_boundary_no_forbidden_terms_in_all_help' panicked at:
Forbidden term 'ALIVE' found in output of: cargo cicd status --help
```

Steps to investigate:

1. Run the exact command manually:
   ```sh
   cargo run -- status --help 2>&1 | grep -n "ALIVE"
   ```

2. Search source for the term:
   ```sh
   rg "ALIVE" src/ templates/ docs/
   ```

3. Check if the term is coming from a dependency message or panic:
   ```sh
   RUST_BACKTRACE=1 cargo run -- status --help 2>&1
   ```

4. Fix the source and re-run the invariant:
   ```sh
   cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
   ```

### Comparing Feature-Flag Output for I7 Failures

When `invariant_feature_projection_consistency` fails, the most useful debugging approach is
direct side-by-side comparison:

```sh
# Capture default output
cargo run -- status show > /tmp/no-features.txt 2>&1

# Capture feature-enabled output
cargo run --features process-data -- status show > /tmp/with-process-data.txt 2>&1
cargo run --features autonomic -- status show > /tmp/with-autonomic.txt 2>&1

# Compare — lines present in default must be present in feature-enabled
comm -23 <(sort /tmp/no-features.txt) <(sort /tmp/with-process-data.txt)
```

---

## 8. Capability Test Matrix Reference

The Capability Test Matrix in `docs/testing/CAPABILITY_TEST_MATRIX.md` defines the primary 12
scenarios and 5 critical 3-wise cases that the test suite must cover.

### Primary 12 Scenarios

Each scenario specifies workspace shape, git state, toolchain, target state, and expected outcome:

| Scenario | Key Assertion |
|---|---|
| `status` on clean single-crate | exit 0, all-green report |
| `status` on dirty-tracked | exit 0, dirty warning present |
| `status show` with valid cicd.toml | exit 0, structured output |
| `target show` on over-limit workspace | exit 0, size warning |
| `target prune` default (no confirm) | exit 0, plan only, no deletion |
| `test changed` with changed source | exit 0, conservative changed plan |
| `trybuild changed` with changed fixture | exit 0, changed-only plan |
| `git status` on dirty-tracked | exit 0, dirty state reported |
| `git close` on clean workspace | exit 0, no-op pass |
| `git close` on dirty-unrelated | exit non-0, refuse with named law |
| `publish run` on ready workspace | exit 0, cicd.toml written |
| `workspace doctor` on missing manifest | exit non-0, explains missing Cargo.toml |

### Critical 3-Wise Cases

These are manually identified dangerous triangles that pairwise coverage would miss:

| Case | What it tests |
|---|---|
| `dirty+trybuild+close` | git close must refuse when trybuild fixture is changed |
| `mismatch+changed+process-data` | toolchain mismatch detected and event emitted |
| `overlimit+release+prune` | release artifacts preserved during incremental prune |
| `corrupted+publish+autonomic` | corrupt cicd.toml causes refusal, not silent overwrite |
| `wasm4pm-missing+feature+publish` | PARTIAL signal when wpm feature on but binary absent |

When writing tests that cover these triangles, each test should:
1. Set up all three state dimensions
2. Invoke the command
3. Assert on the expected outcome **and** the named law that prevents the wrong outcome

---

## 9. CI/CD Gate — What Must Pass Before Release

The following test suites must all exit 0 before a release is tagged. No exceptions.

### Mandatory Pre-Release Gates

```sh
# Gate 1: Invariants (public boundary, safety, determinism)
cargo test --test invariants
# All 7 invariants must pass. A single failure blocks the release.

# Gate 2: Full test suite
cargo make test
# Includes all Tier 1 tests across all noun/verb combinations.

# Gate 3: Feature flag compilation
cargo build --features autonomic,wasm4pm,contrib
# Ensures all feature combinations compile without errors.

# Gate 4: Feature flag consistency
cargo test --features process-data --test feature_projection
cargo test --features autonomic --test autonomic_policies
# Ensures feature flags do not break existing output facts.

# Gate 5: Evidence gate (requires wpm oracle)
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate --features wasm4pm
# If wpm is not available, this gate is skipped; see note below.

# Gate 6: Mutation evidence gate
cargo test --test wasm4pm_evidence_mutation --features wasm4pm
# Proves oracle is a real adjudicator, not a rubber stamp.

# Gate 7: Receipt validation
wpm receipt doctor --format json --strict receipts/*.json
# All receipts in the receipts/ directory must be valid.
```

### Oracle Availability Note

Gate 5 (evidence gate with `REQUIRE_WPM_ORACLE=1`) requires the `wpm` binary on PATH. In CI
environments without wasm4pm installed, this gate will be skipped gracefully (Blocked fallback).

For release tagging, the evidence gate must be run in an environment with `wpm` available. The
release commit must include evidence artifacts in `target/cargo-cicd/evidence/` that have been
accepted by the oracle.

### Shorthand Pre-Release Checklist

```sh
# Run all mandatory gates in sequence
cargo make test \
  && cargo build --features autonomic,wasm4pm,contrib \
  && cargo test --test invariants \
  && cargo test --test wasm4pm_evidence_gate --features wasm4pm \
  && cargo test --test wasm4pm_evidence_mutation --features wasm4pm \
  && echo "All gates passed — safe to tag release"
```

If any command in this chain fails, the release is blocked until the failure is resolved.

### Post-Gate Tagging

After all gates pass:

```sh
# Update CHANGELOG.md with new entries
# Bump version in Cargo.toml and (if present) main.rs

git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore(release): v<VERSION> evidence gate pass"
git tag -a v<VERSION> -m "Release v<VERSION> — evidence adjudicated by wasm4pm"
git push origin main --tags
```

---

## Appendix: Evidence Invariants (E1–E7)

These invariants govern the evidence emission layer in `src/evidence.rs`. They complement the 7
public boundary invariants in `tests/invariants.rs`.

| Invariant | Rule |
|---|---|
| **E1** | cargo-cicd never adjudicates itself; only wasm4pm issues verdicts |
| **E2** | XES file must exist on disk before `audit_xes()` is called |
| **E3** | If oracle unavailable and expected verdict is not `Blocked`, panic |
| **E4** | Tests assert only wasm4pm verdict, never internal cargo-cicd state |
| **E5** | XES groups events by `case_id` into `<trace>` elements |
| **E6** | JSONL emission mirrors XES (same event set, machine-readable) |
| **E7** | `Blocked` is a first-class expectation, not an error |

These invariants are enforced implicitly by the evidence gate test structure. Any test that
violates E1 or E4 (by asserting on internal state) must be corrected before merge.
