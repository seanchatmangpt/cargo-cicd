# TESTING.md — cargo-cicd Agent Reference

## Tier Boundary

| Tier | Gate | Oracle | Runs when |
|------|------|--------|-----------|
| 1 | Non-closing | None | Every `cargo make test`, every CI push |
| 2 | Closing / Release | `wpm` binary | Release checklist, `REQUIRE_WPM_ORACLE=1` CI |

---

## Tier 1 — Test Files

| File | Validates |
|------|-----------|
| `tests/invariants.rs` | 7 public boundary invariants |
| `tests/cli/test_status.rs` | `status show` output, exit codes |
| `tests/cli/test_target.rs` | `target show`, `target prune` dry-run |
| `tests/cli/test_publish.rs` | `publish run` metadata requirements |
| `tests/cli/test_git.rs` | `git status/close/phase` lifecycle |
| `tests/cli/test_workspace.rs` | `workspace doctor` |
| `tests/cli/test_evidence.rs` | `evidence doctor/audit` CLI shape |
| `tests/cli/command_projection.rs` | All noun+verb combinations parse |
| `tests/cli/verb_registry.rs` | Every registered verb has a handler |
| `tests/feature_projection.rs` | Feature flags don't invert output facts |
| `tests/feature_projections.rs` | Extended feature flag surface |
| `tests/cicd_toml_truth.rs` | `cicd.toml` round-trip + determinism (I2) |
| `tests/autonomic_policies.rs` | Policy eval for each named policy |
| `tests/changed_tests.rs` | `ChangedFileDetector` classification |
| `tests/git_phase_closure.rs` | Git state detection |
| `tests/fixture_workspaces.rs` | Fixture builder utilities |
| `tests/ggen_customization_guard.rs` | Ontology regeneration idempotent |
| `tests/lsp_explain.rs` | `lsp explain` response shape |
| `tests/interactions.rs` | User interaction flows |
| `tests/policies.rs` | Policy entry shape, verdict categories |
| `tests/publish_gate.rs` | Publishing gate with varied manifests |

**Missing CLI smoke tests** — create before marking fully covered:

| File | Noun | Validates |
|------|------|-----------|
| `tests/cli/test_analyze.rs` | `analyze` | verb invocations, exit codes, output shape |
| `tests/cli/test_autoarch.rs` | `autoarch` | verb invocations, exit codes, output shape |
| `tests/cli/test_certification.rs` | `certification` | verb invocations, exit codes, output shape |
| `tests/cli/test_sbom.rs` | `sbom` | verb invocations, exit codes, output shape |
| `tests/cli/test_ui.rs` | `ui` | `ui demo`, `ui dashboard`, no-TTY fallback |

## Tier 2 — Evidence Gate Files

| File | Validates |
|------|-----------|
| `tests/wasm4pm_evidence_gate.rs` | Happy path: valid evidence → Accept |
| `tests/wasm4pm_evidence_mutation.rs` | Corrupt evidence → Refuse |
| `tests/wasm4pm_refusal_cases.rs` | Missing file, empty, malformed |
| `tests/wasm4pm_harness.rs` | Shared harness utilities |
| `tests/wasm4pm_shell.rs` | Shell invocation layer for wpm |
| `tests/wpm_verdict_key_contract.rs` | Verdict enum + key encoding |
| `tests/refusal_calibration.rs` | Oracle sensitivity across mutation types |

---

## Running Tests

```sh
# Canonical — full Tier 1 + Tier 2 (Blocked fallback when wpm absent)
cargo make test

# Individual suites
cargo test --test invariants
cargo test --test cli
cargo test --test cicd_toml_truth
cargo test --test autonomic_policies
cargo test --test changed_tests
cargo test --test git_phase_closure
cargo test --test feature_projection
cargo test --test feature_projections
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

# Single function
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
cargo test --test invariants invariant_status_exits_zero -- --nocapture

# Feature flags
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
cargo test --features autonomic,wasm4pm,contrib
cargo test --test wasm4pm_evidence_gate --features wasm4pm

# Force oracle required (release gate CI)
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

---

## 7 Non-Negotiable Invariants (`tests/invariants.rs`)

All 7 must pass before any release tag.

### I1 — PublicBoundary
Test: `invariant_public_boundary_no_forbidden_terms_in_all_help`  
Scans every `--help` output for forbidden terms:
`ALIVE` · `Inspection Gate` · `wall` · `Nehemiah` · `Field8` · `Instinct8` · `Cargo Court` · `AGI` · `Truex` · `CONSTRUCT8`

```sh
# Debug: find leaking command
cargo run -- status --help | grep -i ALIVE
rg "ALIVE" src/ templates/ docs/
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
```

### I2 — PublishDeterminism (`cicd_toml_truth.rs`)
Runs `publish run` twice on identical workspace; asserts SHA-256 byte-equality of `cicd.toml`.

```sh
# Debug: non-deterministic field (timestamps, HashMap order, random IDs)
cargo run -- publish run && cp cicd.toml /tmp/cicd1.toml
cargo run -- publish run && cp cicd.toml /tmp/cicd2.toml
diff /tmp/cicd1.toml /tmp/cicd2.toml
```

### I3 — NoFalseClose
Test: `invariant_no_false_close_git_close_help_mentions_safety`  
`git close` must exit non-0 on dirty working tree. Help text must contain `dry|safe|confirm|check`.

### I4 — NoDestructiveDefault
Test: `invariant_no_destructive_default_target_prune_is_safe`  
`target prune` without `--confirm` must not delete any files.

### I5 — NoFullTrybuildByDefault
Test: `invariant_no_full_trybuild_by_default`  
With 100 fixtures present and no git changes, `trybuild changed` must not mention running all 100.

### I6 — NoAssumedWasm4pmCapability (`feature_projection.rs`)
With `wasm4pm` feature enabled but `wpm` absent from PATH, command must emit `PARTIAL`, not panic.

```sh
PATH="" cargo test --features wasm4pm --test feature_projection
```

### I7 — FeatureProjectionConsistency
Test: `invariant_feature_projection_consistency`  
Every output fact present in default run must be present and identical with any feature flag. Flags add records, never remove or invert.

```sh
# Debug
cargo run -- status show > /tmp/default.txt
cargo run --features process-data -- status show > /tmp/with-features.txt
comm -23 <(sort /tmp/default.txt) <(sort /tmp/with-features.txt)
```

---

## Evidence Gate — Invariants E1–E7

| Invariant | Rule |
|-----------|------|
| E1 | cargo-cicd never adjudicates itself; only wpm issues verdicts |
| E2 | XES file must exist on disk before `audit_xes()` is called |
| E3 | Oracle unavailable + non-Blocked expectation = panic |
| E4 | Tests assert only wpm verdict, never internal cargo-cicd state |
| E5 | XES groups events by `case_id` into `<trace>` elements |
| E6 | JSONL emission mirrors XES |
| E7 | `Blocked` is a first-class expectation, not an error |

**FORBIDDEN in evidence tests:**
```rust
// WRONG
assert_eq!(state.target.size, expected_size);

// CORRECT
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}
```

### Happy Path Pattern

```rust
#[test]
fn evidence_gate_status_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("status show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call"); // E2

    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### Mutation Patterns

| Mutation | Test name suffix | What is corrupted |
|----------|-----------------|-------------------|
| Corrupted XML | `corrupted_xes_refused` | File replaced with non-XML |
| Empty file | `empty_xes_refused` | File truncated to zero bytes |
| Mismatched tags | `mismatched_tags_refused` | Closing tags don't match |
| Missing verdict | `missing_verdict_refused` | `verdict_claimed` attribute removed |
| Fabricated Accept | `fabricated_verdict_refused` | verdict changed to `ACCEPT` manually |

```rust
#[test]
fn evidence_mutation_<name>_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mutated.xes");
    let events = vec![ProcessEvent::new("some command", "PASS")];
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    corrupt_xes_<mutation_type>(&xes_path);

    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### OCEL 2.0 Emission Pattern (new noun handlers)

```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence, AdmittedOcelEvidence};
use wasm4pm_compat::state::{Raw, Admitted};
use wasm4pm_compat::witness::Ocel20;

// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

**FORBIDDEN:**
- Hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs
- Calling `wpm` on `.xes` files in new code (OCEL is the only format)
- Importing from `src/ocel.rs` — DELETE it, use `wasm4pm_compat` imports
- Extending `evidence_xes_v2.rs` — LEGACY, do not touch

Object types in cargo-cicd domain: `Workspace` · `Crate` · `TestRun` · `GitCommit` · `Release` · `Receipt` · `EvidenceFile` · `Policy` · `Toolchain`

Dependency:
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

---

## Writing Tests

### CLI Test Template

```rust
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_my_command_exits_zero() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), r#"[package]
name = "test-workspace"
version = "0.1.0"
edition = "2021"
"#).unwrap();

    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["my-noun", "my-verb"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("expected text"));
}
```

Rules: always `TempDir`, always `current_dir`, minimal fixtures only.

### Regression Test Template

```rust
/// Regression: <describe bug> — fixed in <commit/PR>.
/// Previous: <wrong behavior>. Correct: <expected behavior>.
#[test]
fn regression_<noun>_<verb>_<description>() {
    // minimal reproduction
    // assert bug is fixed
}
```

Place in most specific noun test file; if cross-noun, use `tests/invariants.rs` or `command_projection.rs`.

### Feature-Gated Test Template

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_<policy_name>_policy_detects_<condition>() {
    let mut state = EngineState::default();
    // set triggering fields
    let policies = run_all_policies(&state);
    let policy = policies.iter().find(|p| p.policy_name == "<policy_name>").unwrap();
    assert_eq!(policy.verdict, PolicyVerdict::Warn);
}

#[cfg(feature = "autonomic")]
#[test]
fn test_<policy_name>_policy_passes_when_<condition_not_met>() {
    let policies = run_all_policies(&EngineState::default());
    let policy = policies.iter().find(|p| p.policy_name == "<policy_name>").unwrap();
    assert!(matches!(policy.verdict, PolicyVerdict::Pass | PolicyVerdict::Skip));
}
```

For new policy: add `src/policies/<name>.rs`, register in `policies::run_all_policies()`, add both tests above.

### Offline Oracle Fallback

```rust
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!("REQUIRE_WPM_ORACLE=1 set but wpm absent. Test '{}' cannot assert verdict.", test_name);
    }
    ExpectedWpmVerdict::Blocked
}
```

---

## Debugging

```sh
# Verbose output
cargo test --test invariants invariant_status_exits_zero -- --nocapture

# Adapter debug logs
RUST_LOG=debug cargo test --test changed_tests -- --nocapture
RUST_LOG=cargo_cicd::adapters::git_status=debug cargo test --test git_phase_closure -- --nocapture

# Inspect XES evidence
ls -la target/cargo-cicd/evidence/
wpm audit target/cargo-cicd/evidence/evt-*.xes

# Feature-flag comparison (I7)
cargo run -- status show > /tmp/default.txt
cargo run --features process-data -- status show > /tmp/with-pd.txt
comm -23 <(sort /tmp/default.txt) <(sort /tmp/with-pd.txt)
```

Adapter debug signals:
- `GitStatusAdapter` — raw `git status --porcelain` + parsed dirty files
- `ChangedFileDetector` — `git diff` output + file classification
- `TargetScannerAdapter` — walkdir count + total bytes
- `ToolchainDetector` — raw `rustc --version`

---

## Capability Test Matrix

### Primary 12 Scenarios

| Scenario | Key assertion |
|----------|---------------|
| `status` clean single-crate | exit 0, all-green |
| `status` dirty-tracked | exit 0, dirty warning present |
| `status show` with valid cicd.toml | exit 0, structured output |
| `target show` over-limit | exit 0, size warning |
| `target prune` default | exit 0, plan only, no deletion |
| `test changed` with changed source | exit 0, conservative plan |
| `trybuild changed` with changed fixture | exit 0, changed-only plan |
| `git status` dirty-tracked | exit 0, dirty state reported |
| `git close` clean workspace | exit 0, no-op pass |
| `git close` dirty-unrelated | exit non-0, refuse with named law |
| `publish run` ready workspace | exit 0, cicd.toml written |
| `workspace doctor` missing manifest | exit non-0, explains missing Cargo.toml |

### Critical 3-Wise Cases

| Case | Tests |
|------|-------|
| `dirty+trybuild+close` | git close refuses when trybuild fixture changed |
| `mismatch+changed+process-data` | toolchain mismatch detected + event emitted |
| `overlimit+release+prune` | release artifacts preserved during incremental prune |
| `corrupted+publish+autonomic` | corrupt cicd.toml causes refusal, not silent overwrite |
| `wasm4pm-missing+feature+publish` | PARTIAL when wpm feature on but binary absent |

Each 3-wise test must: set up all 3 state dimensions, invoke command, assert expected outcome AND named law.

---

## Pre-Release Gate

```sh
# Gate 1: Invariants
cargo test --test invariants

# Gate 2: Full suite
cargo make test

# Gate 3: Feature compilation
cargo build --features autonomic,wasm4pm,contrib

# Gate 4: Feature consistency
cargo test --features process-data --test feature_projection
cargo test --features autonomic --test autonomic_policies

# Gate 5: Evidence gate (requires wpm)
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate --features wasm4pm

# Gate 6: Mutation evidence gate
cargo test --test wasm4pm_evidence_mutation --features wasm4pm

# Gate 7: Receipt validation
wpm receipt doctor --format json --strict receipts/*.json

# Shorthand
cargo make test \
  && cargo build --features autonomic,wasm4pm,contrib \
  && cargo test --test invariants \
  && cargo test --test wasm4pm_evidence_gate --features wasm4pm \
  && cargo test --test wasm4pm_evidence_mutation --features wasm4pm \
  && echo "All gates passed — safe to tag release"
```

Post-gate tagging:
```sh
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore(release): v<VERSION> evidence gate pass"
git tag -a v<VERSION> -m "Release v<VERSION> — evidence adjudicated by wasm4pm"
git push origin main --tags
```
