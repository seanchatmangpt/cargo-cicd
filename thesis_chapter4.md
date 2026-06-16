# Chapter 4: Implementation and Evaluation

## 4.1 Implementation Overview

cargo-cicd is implemented entirely in Rust and targets a minimum supported Rust version (MSRV) of 1.86, declared explicitly in `Cargo.toml` via the `rust-version` field and enforced at every CI run. The project is structured as a Cargo workspace with three members: the primary `cargo-cicd` binary crate at the workspace root and two supporting library crates, `cargo-cicd-core` and `cargo-cicd-lsp`, under the `crates/` subdirectory.

### 4.1.1 Dependency Selection

The dependency footprint has been deliberately constrained to a small set of well-audited crates. Table 4.1 summarises the direct runtime dependencies.

**Table 4.1 — Runtime Dependencies**

| Crate | Version | Role |
|---|---|---|
| `clap` | 4 (derive) | CLI argument parsing |
| `clap-noun-verb` | 26.6.2 | Noun-verb CLI grammar layer |
| `serde` | 1 (derive) | Serialisation/deserialisation |
| `toml` | 0.8 | cicd.toml reading and writing |
| `anyhow` | 1 | Error propagation |
| `walkdir` | 2 | Target directory traversal |
| `serde_json` | 1 | JSON output for evidence receipts |

Development-time dependencies add `assert_cmd` (2), `tempfile` (3), and `predicates` (3) for integration testing, plus `toml` (0.8) for fixture TOML verification.

`clap-noun-verb` is a local crate developed in tandem with cargo-cicd. It implements the noun-verb command grammar that forms the public CLI surface: each top-level noun (`status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`) is registered with a set of verb subcommands, and the framework handles default-verb injection so that bare nouns resolve to their primary verb without user intervention.

### 4.1.2 Feature Flags

The codebase exposes four non-default feature flags that gate internal subsystems:

**Table 4.2 — Feature Flags**

| Feature | Implies | When enabled |
|---|---|---|
| `process-data` | — | Level 5 engine internals, cicd.toml I/O, XES emission |
| `autonomic` | `process-data` | Policy evaluation, suggest-mode recommendations |
| `wasm4pm` | `process-data` | Richer wasm4pm runtime integration seam |
| `contrib` | `process-data` | Contributor utilities and debugging aids |

This layered implication graph allows CI to test each surface in isolation. A build with no features is a fully functional CLI; enabling `process-data` activates the `EngineState` aggregate and all adapters; enabling `autonomic` additionally activates policy evaluation.

### 4.1.3 Workspace Structure

```
cargo-cicd/
├── src/
│   ├── main.rs                  # Entry point, default-verb injection
│   ├── engine/                  # EngineState and per-dimension State types
│   ├── adapters/                # One adapter per external data source
│   ├── nouns/                   # CLI noun modules (status, target, test, …)
│   ├── policies/                # Autonomic policy implementations
│   ├── evidence.rs              # ProcessEvent and XES emission
│   └── cicd_toml.rs             # cicd.toml schema and deserialization
├── crates/
│   ├── cargo-cicd-core/
│   └── cargo-cicd-lsp/
├── tests/                       # Integration tests (16 test binaries)
│   ├── fixtures/                # FixtureWorkspace helpers
│   ├── invariants.rs
│   ├── autonomic_policies.rs
│   ├── wasm4pm_evidence_gate.rs
│   └── …
├── ontology/cargo-cicd.ttl      # OWL/RDF ontology (manufacturing input)
├── queries/                     # SPARQL queries for ggen pipeline
├── templates/                   # Tera templates for code generation
└── .github/workflows/           # CI pipeline definitions
```

---

## 4.2 Test Hierarchy

The project enforces a three-tier test hierarchy. Each tier has a distinct scope and a distinct role in the release process.

**Table 4.3 — Test Tier Summary**

| Tier | Files | Tools | Release-blocking? |
|---|---|---|---|
| Tier 1: Unit and smoke | `invariants.rs`, `cli/`, `feature_projection.rs`, `autonomic_policies.rs` | `assert_cmd`, `tempfile` | No |
| Tier 2: Integration | `cicd_toml_truth.rs`, `changed_tests.rs`, `git_phase_closure.rs`, `ggen_customization_guard.rs` | `assert_cmd`, `tempfile`, `walkdir` | No |
| Tier 3: Evidence gate | `wasm4pm_evidence_gate.rs`, `wasm4pm_evidence_mutation.rs`, `wasm4pm_refusal_cases.rs` | `wpm` oracle, XES | **Yes** |

**Tier 1** tests verify the public CLI boundary, the autonomic policy logic, and the feature flag surface contract. These tests run in every CI job and on both supported platforms. They are fast (typically sub-second), use `tempfile::TempDir` for isolation, and never require an external oracle binary.

**Tier 2** tests cover correctness of the cicd.toml carrier format, the changed-test selection algorithm, git phase closure semantics, and the ggen code-generation customisation guard. These tests exercise multi-step workflows and may spawn subprocesses (`git`, `cargo metadata`).

**Tier 3** evidence-gate tests are the release-closing gate. They emit XES (XML Event Stream) evidence files and submit them to the wasm4pm oracle (`wpm`) for adjudication. A release may not proceed if the oracle issues a Refuse verdict. This design decouples cargo-cicd's internal correctness claims from external process-compliance verification.

Cargo.toml declares each integration test as a separate `[[test]]` entry, ensuring that individual suites can be run in isolation with `cargo test --test <name>` and that their build artefacts are cached independently.

---

## 4.3 FixtureWorkspace Testing Strategy

The central abstraction for integration testing is `FixtureWorkspace`, implemented in `tests/fixtures/mod.rs`. Each fixture constructs a minimal but realistic Rust workspace in a `tempfile::TempDir`, populates it with the conditions required to exercise a specific engine verdict, and exposes the workspace root path for use as the `current_dir` of `assert_cmd` invocations. The `TempDir` is owned by the `FixtureWorkspace` struct; when the struct is dropped at the end of a test, the operating system reclaims the temporary directory automatically, providing strong isolation between tests.

### 4.3.1 Available Fixtures

**Table 4.4 — FixtureWorkspace Variants**

| Constructor | Preconditions | Expected Verdict |
|---|---|---|
| `FixtureWorkspace::clean()` | Valid `Cargo.toml`, fully committed, no `target/`, no `cicd.toml` | Pass |
| `FixtureWorkspace::dirty()` | Clean baseline + one untracked file (`untracked.txt`) | Warn (git dirty) |
| `FixtureWorkspace::missing_manifest()` | Empty directory, no `Cargo.toml` | Refuse |
| `FixtureWorkspace::with_toolchain_mismatch()` | Clean + `rust-toolchain.toml` pinning channel `1.50.0` | Warn |
| `FixtureWorkspace::with_target_over_limit()` | Clean + `target/debug/placeholder.bin` (1,048,576 bytes) | Warn (target pressure) |
| `FixtureWorkspace::with_corrupted_cicd_toml()` | Clean + syntactically invalid `cicd.toml` | Fail/Refuse |
| `FixtureWorkspace::with_stale_cicd_toml()` | Clean + `cicd.toml` claiming `dirty = false`, then made dirty | Warn (cache mismatch) |
| `FixtureWorkspace::with_changed_trybuild_fixture()` | Clean + `tests/ui/` containing 10 unchanged + 1 changed fixture | Pass (changed-only) |

### 4.3.2 Isolation Guarantees

Each fixture satisfies four isolation properties:

1. **Filesystem isolation.** The `TempDir` is created in the system temporary directory, never inside the repository. Tests therefore never modify the working checkout.
2. **Git isolation.** Fixtures that require git history initialise a fresh git repository via `git init` within the `TempDir`. The global git configuration of the host is not modified.
3. **State isolation.** Fixtures that place a `cicd.toml` in the workspace do so programmatically, with exactly the state required by the test scenario. There is no shared mutable state between tests.
4. **Drop isolation.** Because `TempDir` implements `Drop`, cleanup occurs even when a test panics. The cargo test harness runs each `#[test]` function in its own thread; a panic in one thread does not prevent other tests from running.

### 4.3.3 Example: Verifying a Dirty Workspace

The following pattern, drawn from the project's own test suite, illustrates the canonical fixture usage:

```rust
#[test]
fn test_dirty_workspace_verdict() {
    let fixture = FixtureWorkspace::dirty();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .arg("status")
        .current_dir(&fixture.root)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("dirty"));
}
```

The `fixture` variable owns the `TempDir`. The `Command::cargo_bin` call resolves the built binary from `CARGO_BIN_EXE_cargo-cicd`, eliminating reliance on the system `PATH`.

---

## 4.4 Invariant Testing

`tests/invariants.rs` encodes the non-negotiable public boundary invariants. These invariants are enforced on every CI push and pull request across all platform and toolchain combinations. The file currently implements four invariant functions, each named with the `invariant_` prefix to make the suite's purpose self-documenting.

### 4.4.1 Invariant 1: No Forbidden Terms in Public Output

The most comprehensive invariant iterates over all public CLI entry points (top-level `--help` and per-noun `--help`) and asserts that none of the following terms appear in stdout or stderr:

```
ALIVE, Nehemiah, CONSTRUCT8, Instinct8, Inspection Gate,
Cargo Court, AGI, Truex, Field8, wall
```

These terms belong to the internal architecture and must never leak into user-visible output. The test runs `cargo-cicd` with nine distinct argument lists and checks both output streams:

```rust
let text = String::from_utf8_lossy(&output.stdout).to_string()
    + &String::from_utf8_lossy(&output.stderr);
for term in &forbidden {
    assert!(
        !text.contains(term),
        "Forbidden term '{}' found in output of: cargo cicd {}",
        term, args.join(" ")
    );
}
```

A parallel check in `tests/ggen_customization_guard.rs` extends this verification to the file system: it walks `README.md` and all Markdown files under `docs/tutorials/`, `docs/how-to/`, `docs/reference/`, and `docs/explanation/`, asserting the same forbidden-term list. This dual enforcement (CLI output and static documents) closes the gap between runtime behaviour and shipped documentation.

### 4.4.2 Invariant 4: No Destructive Default

`invariant_no_destructive_default_target_prune_is_safe` constructs a temporary directory containing a `target/debug/` tree with a synthetic binary, runs `cargo cicd target prune` without any confirmation flag, and then asserts that the binary still exists:

```rust
assert!(
    fake_target.join("binary").exists(),
    "target prune without --confirm must not delete files"
);
```

This invariant directly encodes the design constraint that no cargo-cicd command may take a destructive action without explicit user confirmation. The `--confirm` flag is required to activate deletion.

### 4.4.3 Invariant 5: No Full Trybuild by Default

The trybuild invariant creates 100 synthetic `tests/ui/compile_fail/fixture_N.rs` files in a temporary workspace and invokes `cargo cicd trybuild changed`. The assertion is negative: the combined output must not contain the strings `"100 fixtures"` or `"all 100"`. The invariant enforces that the changed-only selection algorithm is never bypassed, regardless of the number of fixtures present.

### 4.4.4 Invariant 6: wasm4pm Scan or Documented Absence

The sixth invariant enforces a process-compliance property rather than a code-correctness property. It checks whether at least one of three paths exists: the wasm4pm capability scan receipt, the integration recommendation document, or the deferred-work document. If none exists, the test logs a `PARTIAL` message but does not fail, because the scan workflow may be running concurrently. The invariant is about the process — that capability assessment was performed and its outcome was recorded — not about the binary timing of that recording.

### 4.4.5 ggen Customisation Guard

`tests/ggen_customization_guard.rs` protects the code-generation manufacturing pipeline. It verifies that every `BEGIN ggen:` marker in `README.md` has a matching `END ggen:` marker (and vice versa for `BEGIN custom:`), that the README contains a command table generated from the ontology, and that reference documentation files exist for every public command. The `evidence_emission_not_removed` test specifically guards against accidental removal of the `ProcessEvent` struct or the `emit_xes` function from `src/evidence.rs`.

**Table 4.5 — Invariant Tests at a Glance**

| Test name | What it enforces |
|---|---|
| `invariant_public_boundary_no_forbidden_terms_in_all_help` | 10 forbidden terms absent from all 9 public help surfaces |
| `invariant_no_false_close_git_close_help_mentions_safety` | `git close --help` acknowledges safety (informational) |
| `invariant_no_destructive_default_target_prune_is_safe` | Files survive `target prune` without `--confirm` |
| `invariant_no_full_trybuild_by_default` | 100-fixture workspace does not trigger full run |
| `invariant_wasm4pm_scan_or_documented_absence` | wasm4pm capability was assessed and outcome recorded |

---

## 4.5 wasm4pm Evidence Gate

The wasm4pm evidence gate is the release-closing mechanism for cargo-cicd. It implements a strict separation between internal test assertions (which verify cargo-cicd's own behaviour) and process-compliance verification (which is delegated entirely to the external `wpm` oracle).

### 4.5.1 Evidence Format and Emission

Process evidence is emitted as XES (XML Event Stream), the standard format specified by the IEEE Process Mining standard. Each invocation of a public verb emits one or more `ProcessEvent` records. The `emit_xes` function in `src/evidence.rs` serialises these records to an XES file at `target/cargo-cicd/evidence/` within the workspace. The XES format was chosen over JSONL or custom CSV because it is natively understood by `wpm`; no translation layer is required.

A minimal evidence emission sequence:

```rust
let events = vec![ProcessEvent::new("status show", "PASS")];
let xes_path = dir.path().join("events.xes");
emit_xes(&events, &xes_path).expect("emit_xes must not fail");
assert!(xes_path.exists(), "XES file must exist before oracle call");
```

### 4.5.2 Oracle Protocol

The `WpmEvidenceOracle` struct encapsulates binary discovery and invocation. At runtime, it resolves the `wpm` binary through three resolution stages: an explicit `WPM_PATH` environment variable, the known installation path `/Users/sac/wasm4pm/target/release/wpm`, and finally the system `PATH` via `which wpm`. This resolution order ensures that CI environments with a centrally installed `wpm` are preferred over developer-local builds.

Once the oracle is resolved, the primary audit command is:

```
wpm audit <file.xes>
```

The secondary receipt doctor command is:

```
wpm receipt doctor --format json --strict <receipt.json>
```

Both commands must return a non-zero-free exit status and must not produce `FAIL` or `REFUSE` in their combined output for the evidence gate to pass.

### 4.5.3 Oracle-Absent Fallback

When the `wpm` binary is absent (as in standard CI runners that do not have wasm4pm installed), each evidence-gate test falls back to an `ExpectedWpmVerdict::Blocked` assertion. This allows the test suite to complete without the oracle while making it transparent that the Accept branch was not exercised. For release closure, pipeline operators set `REQUIRE_WPM_ORACLE=1`; under that environment variable, binary absence causes an immediate panic rather than a silent skip:

```rust
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!(
            "REQUIRE_WPM_ORACLE=1 is set but the wpm oracle binary is absent. \
             Test '{}' cannot exercise its Accept assertion.",
            test_name
        );
    }
    ExpectedWpmVerdict::Blocked
}
```

### 4.5.4 Hard Gate Test

`evidence_gate_wpm_doctor_hard_gate` is a mandatory test that invokes `wpm doctor` directly and asserts both exit code zero and the absence of failure indicators in the combined output. Unlike the per-verb acceptance tests, this test targets the oracle's self-diagnosis facility. If `wpm doctor` reports any internal failure, the release is blocked regardless of the per-verb results.

**Table 4.6 — Evidence Gate Tests**

| Test | Verb under test | Evidence submitted |
|---|---|---|
| `evidence_gate_status_show_accepted` | `status show` | Single PASS event |
| `evidence_gate_target_show_accepted` | `target show` | Single PASS event |
| `evidence_gate_target_prune_accepted` | `target prune` | Single DRY-RUN event |
| `evidence_gate_changed_test_accepted` | `test changed` | Single PASS event |
| `evidence_gate_git_close_accepted` | `git close` | Single PASS event |
| `evidence_gate_publish_run_accepted` | `publish run` | Single PASS event |
| `evidence_gate_workspace_doctor_accepted` | `workspace doctor` | Single PASS event |
| `evidence_gate_oracle_discover` | — | Oracle self-probe (no panic) |
| `evidence_gate_wpm_doctor_hard_gate` | — | `wpm doctor` self-diagnosis |

---

## 4.6 CI/CD Pipeline

The GitHub Actions pipeline is defined in `.github/workflows/ci.yml` and consists of five jobs that run in parallel after checkout.

### 4.6.1 Pipeline Jobs

**fmt-clippy** runs on a 2×1 matrix (`ubuntu-latest`, `macos-latest`). It installs the stable Rust toolchain with the `rustfmt` and `clippy` components and runs `cargo fmt --all -- --check` followed by `cargo clippy --all-targets -- -D warnings` in two passes: once with default features and once with `--all-features`. Clippy warnings are treated as errors.

**test** runs on a 2×2 matrix (2 platforms × 2 toolchains), producing four concurrent runners.

**Table 4.7 — Test Matrix**

| | `ubuntu-latest` | `macos-latest` |
|---|---|---|
| `stable` | ubuntu/stable | macos/stable |
| `1.86` (MSRV) | ubuntu/1.86 | macos/1.86 |

Each runner in the test matrix executes the following sequence:
1. `cargo build --workspace` — verifies the full workspace compiles
2. Individual test suites: `invariants`, `cli`, `cicd_toml_truth`, `autonomic_policies`, `changed_tests`, `git_phase_closure`, `feature_projection`
3. `cargo test --workspace` — runs all tests with default features

Test results are uploaded as artefacts with a 14-day retention period, keyed by platform and toolchain combination.

**feature-matrix** runs on `ubuntu-latest` against four feature combinations: `""` (default), `process-data`, `autonomic`, and `wasm4pm`. Both `cargo build` and `cargo test` are executed for each combination, ensuring that no feature flag introduces a compilation failure or test regression.

**forbidden-terms** is a static analysis job that runs on `ubuntu-latest`. It uses `grep` to scan `src/**/*.rs` for forbidden terms on non-comment lines, scans CLI help strings specifically, and scans public Markdown files (`README.md`, `docs/reference/`, `docs/agents/`). Internal documentation directories (`docs/wasm4pm/`, `docs/release/`, `docs/design/`, `receipts/`) are excluded from the scan.

**workspace-integrity** verifies that `Cargo.lock` is consistent with `Cargo.toml`, that all workspace members resolve via `cargo metadata`, and that the `rust-version` field is present and equal to `1.86`. The MSRV check is implemented as a Python one-liner that parses the JSON output of `cargo metadata`:

```yaml
- name: Verify MSRV is declared and matches rust-version field
  run: |
    MSRV=$(cargo metadata --format-version 1 --no-deps \
      | python3 -c "import sys,json; m=json.load(sys.stdin); ...")
    if [ "$MSRV" != "1.86" ]; then
      echo "::error::MSRV mismatch — expected 1.86, got $MSRV"
      exit 1
    fi
```

---

## 4.7 Performance Characteristics

cargo-cicd is designed to complete a full `cargo cicd status` invocation in under one second on a typical developer workstation. Three design decisions contribute to this target.

### 4.7.1 Single-Invocation Git Queries

All git state is captured with a single `git status --porcelain` invocation. The output is parsed once and cached in `GitPhaseState` for the duration of the session. Multiple adapters (dirty flag, untracked count, changed file list) all read from this single cached parse. The alternative — issuing separate `git ls-files`, `git diff`, and `git status` calls — would multiply subprocess overhead and introduce races on systems with high I/O latency.

### 4.7.2 Bounded Target Directory Traversal

The `TargetScannerAdapter` uses `walkdir` with a `max_depth(3)` limit, avoiding deep traversal of nested build artefacts. The adapter accumulates only the total byte count, not a per-file inventory, which keeps memory usage constant regardless of workspace size. The computed size is persisted in the `[state]` section of `cicd.toml` and is only recomputed when the HEAD commit hash has changed since the last run.

### 4.7.3 cicd.toml State Cache

`cicd.toml` functions as an inter-invocation state cache. On each run, adapters consult the cached value first and re-query the external source only when the cached value may be stale (determined by comparing the recorded HEAD hash to the current HEAD). For expensive operations such as parsing `Cargo.lock` for dependency tree analysis, this caching pattern reduces steady-state cost to a single file read plus a hash comparison.

**Table 4.8 — Performance Design Choices**

| Operation | Naive approach | Chosen approach | Benefit |
|---|---|---|---|
| Git state | Multiple `git ls-files` + `git diff` | One `git status --porcelain` | Fewer subprocesses |
| Target size | Full recursive walk | `walkdir` with `max_depth(3)`, byte sum only | Bounded traversal |
| Cargo metadata | Per-crate `cargo metadata` | Single `cargo metadata --format-version 1` | Linear not quadratic |
| Cross-invocation state | Re-query everything | cicd.toml cache keyed by HEAD hash | Sub-second repeat runs |

---

## 4.8 Evaluation Results

### 4.8.1 Test Suite Scale

The project declares 16 integration test binaries in `Cargo.toml`. The named suites cover the following domains:

**Table 4.9 — Named Integration Test Suites**

| Suite | Domain |
|---|---|
| `feature_projection` | Feature flag surface contract |
| `cli` | CLI command projection |
| `cicd_toml_truth` | cicd.toml schema and write correctness |
| `autonomic_policies` | Autonomic policy verdicts |
| `changed_tests` | Changed-test selection algorithm |
| `git_phase_closure` | Git phase state transitions |
| `invariants` | Non-negotiable public boundary invariants |
| `wasm4pm_harness` | Evidence harness smoke tests |
| `wasm4pm_evidence_gate` | Positive acceptance evidence gate |
| `wasm4pm_evidence_mutation` | Mutation-adversarial evidence gate |
| `wasm4pm_refusal_cases` | Refuse-path evidence gate |
| `ggen_customization_guard` | Code-generation guard |
| `refusal_calibration` | Calibration of refusal thresholds |
| `lsp_explain` | LSP explain surface |
| `fixture_workspaces` | FixtureWorkspace construct correctness |
| (implicit) `interactions` | Cross-noun interaction coverage |

### 4.8.2 Policy Evaluation Coverage

The `autonomic_policies` suite tests four policies across multiple input regions:

**Table 4.10 — Autonomic Policy Test Coverage**

| Policy | Pass condition | Warn condition | Suggest condition |
|---|---|---|---|
| `target_pressure` | size < 80% of limit | 80%–100% of limit | > limit |
| `toolchain_mismatch` | no pinned channel, or channels match | — | channels differ |
| `trybuild_changed` | 0 changed fixtures | — | ≥ 1 changed fixture |
| `git_phase_dirty` | 0 dirty files | — | ≥ 1 dirty file |

A cross-cutting invariant asserts that every policy defaults to `PolicyMode::Suggest`, regardless of verdict. No policy may activate in `Apply` mode without explicit user opt-in. This invariant is encoded as a parametric test:

```rust
#[test]
fn test_no_policy_uses_apply_mode_by_default() {
    for r in &[
        check_target_pressure(5.0, 20.0),
        check_toolchain_mismatch("stable", None),
        check_trybuild_changed(0),
        check_git_phase_dirty(0),
    ] {
        assert!(matches!(r.mode, PolicyMode::Suggest), …);
    }
}
```

A second cross-cutting invariant asserts that all policies are enabled by default and that every `Pass` verdict yields an empty recommendation string.

### 4.8.3 Platform Coverage

The 2×2 test matrix produces four configurations on every push to `main` and every pull request. The MSRV configuration (`1.86`) ensures that no dependency silently raises the minimum compiler requirement. The stable configuration tracks the current release of the Rust toolchain and catches regressions introduced by new Clippy lints or standard library changes.

---

## 4.9 Known Limitations

### 4.9.1 Feature Flag Gating

When built without the `process-data` feature (the default), the Level 5 engine internals are entirely absent from the binary. `EngineState`, all adapters, cicd.toml I/O, policy evaluation, and XES emission are compiled out. This is intentional: the public CLI surface functions correctly in the default configuration, and internal plumbing is opt-in. However, it means that a user installing cargo-cicd from crates.io without specifying features receives a CLI that does not emit process evidence or evaluate autonomic policies.

### 4.9.2 Policy Apply Mode Not Implemented

The `--apply` flag is recognised in the CLI grammar but is not yet functional. All policies operate exclusively in `suggest` mode. Automated remediation — such as running `cargo clean` when `TargetPressurePolicy` fires — is deferred pending additional field testing of the policy calibration thresholds.

### 4.9.3 Serial Test Execution

cargo-cicd runs tests serially. The `--jobs` flag is parsed but ignored. The reason is architectural: because `cicd.toml` is a global file in the workspace root, concurrent test processes modifying it would create a write race. Addressing this limitation would require either per-test cicd.toml namespacing or a locking protocol, both of which are deferred to a post-v26.6.2 release.

### 4.9.4 Git Requirement

All state-tracking features require the workspace to be a git repository. Non-git workspaces receive a degraded experience: git-dependent adapters return empty state, and any test that spawns `git status` will fail. The integration test suite handles this by always initialising a fresh git repository inside the `TempDir`.

### 4.9.5 Platform Coverage

The CI matrix covers Linux (`ubuntu-latest`) and macOS (`macos-latest`). Windows is not tested. Path separator differences and the behaviour of `git status --porcelain` on Windows (particularly with respect to line endings) have not been validated. Users on Windows may encounter path-construction errors in the adapter layer.

### 4.9.6 wasm4pm Oracle Availability

The evidence-gate tests exercise the Accept branch only when the `wpm` binary is present. In standard GitHub Actions runners the binary is absent, so the Accept branch is silently skipped unless `REQUIRE_WPM_ORACLE=1` is set. Release engineers are responsible for configuring a self-hosted runner with `wpm` installed, or for running the evidence-gate suite locally against a known-good oracle installation before tagging a release.

### 4.9.7 No Workspace Federation

cargo-cicd recognises exactly one workspace root per invocation, determined by the current working directory or an explicit `--root` flag. Monorepos that contain multiple Cargo workspaces at different directory levels are not supported. Cross-workspace test dependencies are not modelled in `TestPlanState`, and the cicd.toml carrier does not support workspace federation semantics.
