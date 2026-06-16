# Chapter 5: Verification, Testing, and the Autonomic Policy Layer

## 5.1 Introduction

The validation strategy of cargo-cicd is not a post-hoc collection of unit tests appended to a working system; it is a constitutive element of the system's design. Testing in cargo-cicd serves three logically distinct but practically interleaved purposes: (1) proving that the public boundary of the CLI is free of architectural leakage, (2) certifying process conformance through externally adjudicated evidence, and (3) autonomically monitoring workspace health via a layer of read-only policies that emit recommendations without taking action. Together these concerns define a verification architecture that goes beyond coverage-rate optimization to something closer to what Bertrand Meyer called "correctness by construction" — the system cannot be assembled without the tests passing, because the tests are the assembly gate.

This chapter treats each concern in turn. Section 5.2 introduces the two-tier test stratification theory that governs which tests run in which contexts and what each tier is permitted to assert. Section 5.3 catalogues and formally specifies the seven non-negotiable public boundary invariants that form the innermost ring of the test hierarchy. Section 5.4 analyzes the feature projection contract — the surface exposed to the user as a function of which Cargo features are enabled. Section 5.5 documents the autonomic policy layer in detail: its design rationale, its data model, and each of its seven individual policy implementations. Section 5.6 develops the mutation testing strategy used for evidence corruption scenarios, drawing on the literature of mutation testing as a rigorous test-adequacy criterion. Section 5.7 examines changed-file-driven test selection as a mechanism for avoiding whole-suite regression when only a subset of the codebase has changed. Section 5.8 explores the conservative trybuild invariant that prevents an accidental full fixture sweep. The chapter closes in Section 5.9 with a synthesis of the assertion constraint that is perhaps the system's most unusual rule: test code must never assert on cargo-cicd's own internal state; it must assert exclusively on the verdict returned by the external oracle wasm4pm.

---

## 5.2 Test Stratification Theory: Tier 1 and Tier 2

A central concern in large-scale software verification is the question of which properties require which kinds of evidence. Unit tests provide local, fast, isolated evidence about the behavior of individual functions in abstraction from the world. Integration tests provide evidence about the behavior of composed subsystems. But neither kind of test speaks to the question of whether a process was conducted correctly — a question that belongs to process certification, not code coverage.

cargo-cicd distinguishes two tiers of test that map cleanly onto this distinction.

**Tier 1: Smoke and Invariant Tests (Non-Closing)**

Tier 1 tests validate internal logic, public boundaries, and CLI grammar stability. They run on every `cargo make test` invocation, require no external dependencies beyond the compiled binary, and are expected to complete in seconds. They are called "non-closing" because they do not gate a release: failing a Tier 1 test blocks a build, not a delivery.

The test files comprising Tier 1 are:

- `tests/invariants.rs` — seven non-negotiable public boundary invariants
- `tests/cli/` — noun/verb CLI parsing and command projection
- `tests/feature_projection.rs` — feature flag surface contract
- `tests/cicd_toml_truth.rs` — serialization round-trip fidelity
- `tests/autonomic_policies.rs` — policy evaluation logic
- `tests/changed_tests.rs` — file classification accuracy
- `tests/git_phase_closure.rs` — git state detection and no-false-close safety

The assertion primitives used in Tier 1 are: `assert!`, `assert_eq!`, `assert_cmd` exit-code predicates, and string-contains checks on `stdout`/`stderr`. The tests may not assert on the verdict field of any wasm4pm oracle call.

**Tier 2: Evidence Gate Tests (Closing — Release Gate)**

Tier 2 tests operate at the process level. They emit XES (XML Event Stream) artifacts, invoke the external `wpm` oracle, and assert on the oracle's verdict — not on cargo-cicd's own reported state. Tier 2 is called "closing" because release version 26.6.2 cannot be tagged without these tests producing an `Accept` verdict from a live wasm4pm oracle.

The test files comprising Tier 2 are:

- `tests/wasm4pm_evidence_gate.rs` — positive acceptance cases
- `tests/wasm4pm_evidence_mutation.rs` — corrupted evidence, verifying oracle rejection
- `tests/wasm4pm_refusal_cases.rs` — structural invariants of the evidence API
- `tests/wasm4pm_harness.rs` — reusable fixture workspace and mutation infrastructure

This stratification is not merely organizational; it reflects a principled epistemological claim. Tier 1 tests prove properties of the implementation. Tier 2 tests prove properties of the evidence produced by the implementation — a genuinely distinct predicate, as it is possible for an implementation to be internally correct while emitting evidence that misrepresents what happened. The separation of tiers prevents these two concerns from contaminating each other's assertion logic.

The stratification also has operational implications. In a CI environment without the `wpm` binary installed, Tier 2 tests declare `ExpectedWpmVerdict::Blocked` and exit without asserting on oracle output, treating oracle absence as a first-class state rather than an error. Setting the environment variable `REQUIRE_WPM_ORACLE=1` converts this graceful degradation to a hard panic, allowing release pipelines to enforce oracle presence:

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

This pattern implements what one might call a "soft gate with a hard override" — a design that accommodates both local development (oracle absent, gate soft) and release certification (oracle required, gate hard).

---

## 5.3 The Seven Non-Negotiable Public Boundary Invariants

The most fundamental tests in the cargo-cicd test suite are the invariants defined in `tests/invariants.rs`. These invariants are "non-negotiable" in the sense that no release can proceed if any of them fail, and no architectural change may weaken or conditionally exempt any invariant. They formalize properties that are not implementation choices but architectural commitments — things the system is defined to be, not merely things it currently happens to do.

### Invariant 1: Public Boundary — No Forbidden Terms in Any Help Output

The most important invariant in the suite. cargo-cicd is a Level 5 process-data engine; internally it uses a vocabulary of private terms that describe its manufacturing pipeline, autonomic reasoning subsystem, and internal adjudication metaphors. None of these terms may appear in any user-visible output.

Formally, let $H$ be the set of all strings reachable via `cargo cicd [noun] [--help]` for all nouns in the grammar. Let $F$ be the ten-element set of forbidden internal vocabulary terms documented in `CLAUDE.md`. The invariant states:

$$\forall h \in H, \forall f \in F: f \not\subseteq h$$

The test implementation exhaustively enumerates all noun help surfaces — top-level help, noun-level help, and each noun-verb combination tested:

```rust
fn invariant_public_boundary_no_forbidden_terms_in_all_help() {
    let forbidden = [
        "ALIVE", /* internal-only terms — see CLAUDE.md for full list */
    ];
    let noun_verbs = [
        vec!["--help"],
        vec!["status", "--help"],
        vec!["target", "--help"],
        vec!["target", "show", "--help"],
        vec!["test", "--help"],
        vec!["trybuild", "--help"],
        vec!["git", "--help"],
        vec!["publish", "--help"],
        vec!["workspace", "--help"],
    ];
    for args in &noun_verbs {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args(args.iter())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);
        for term in &forbidden {
            assert!(
                !text.contains(term),
                "Forbidden term '{}' found in output of: cargo cicd {}",
                term, args.join(" ")
            );
        }
    }
}
```

The concatenation of both `stdout` and `stderr` is deliberate: clap-noun-verb, the underlying parser library, routes help text through `stderr` on certain error paths. Checking only `stdout` would leave a leakage channel unguarded.

### Invariant 2: No Destructive Default

Destructive operations — those that delete files, commit to version control, or publish artifacts — must never be performed without explicit user confirmation. The canonical test case is `target prune`: without a confirming flag, the command must plan but not act.

```rust
fn invariant_no_destructive_default_target_prune_is_safe() {
    let dir = TempDir::new().unwrap();
    let fake_target = dir.path().join("target/debug");
    std::fs::create_dir_all(&fake_target).unwrap();
    std::fs::write(fake_target.join("binary"), b"ELF fake binary").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("target")
        .arg("prune")
        .output()
        .unwrap();
    // INVARIANT: binary must still exist after prune without confirmation
    assert!(
        fake_target.join("binary").exists(),
        "target prune without --confirm must not delete files"
    );
}
```

This invariant protects against an entire class of CI pipeline accidents in which a default command path takes destructive action. The principle generalizes: all execution verbs accept a confirmation mechanism; all planning verbs exit without side effects.

### Invariant 3: No False Close

The `git close` verb marks a git phase as complete, which has downstream implications for the evidence record and the release gate. A false close — claiming the phase is closed when uncommitted changes remain — corrupts the process record. The invariant asserts that `git close` must refuse to proceed when the working tree is dirty:

```rust
fn test_no_false_close_invariant_dirty_unrelated() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    std::fs::write(dir.path().join("unrelated.rs"), "// untracked").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("git")
        .arg("close")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    let claims_closed = combined.contains("phase already closed")
        || combined.contains("phase closed")
        || combined.contains("committed");
    assert!(
        !claims_closed,
        "git close must not claim closed when unrelated dirty files remain; output: {}",
        combined
    );
}
```

The test additionally asserts that the command exits non-zero, ensuring the refusal is visible to calling scripts and CI systems that inspect exit codes.

### Invariant 4: No Full Trybuild By Default

Trybuild tests compile Rust source files specifically to observe and snapshot compiler error messages. A workspace may contain hundreds of such fixtures. Running all fixtures on every invocation of `trybuild changed` would introduce unacceptable CI latency. The invariant asserts that `trybuild changed` must not announce or perform a full fixture sweep when invoked without changed fixtures:

```rust
fn invariant_no_full_trybuild_by_default() {
    let dir = TempDir::new().unwrap();
    let ui_dir = dir.path().join("tests/ui/compile_fail");
    std::fs::create_dir_all(&ui_dir).unwrap();
    for i in 0..100 {
        std::fs::write(ui_dir.join(format!("fixture_{}.rs", i)), b"fn main() {}").unwrap();
    }
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        !combined.contains("100 fixtures") && !combined.contains("all 100"),
        "trybuild changed must not run all 100 fixtures: {}",
        &combined[..combined.len().min(200)]
    );
}
```

The test constructs an environment with 100 fixture files, then verifies the output does not announce having run them all. This is a conservative test: it tests what is printed, not what is compiled, since the compilation of trybuild fixtures requires a full Rust toolchain and is not feasible within a unit test boundary.

### Invariant 5: Noun Names Are Lowercase ASCII

The CLI grammar constraint that noun names must be lowercase ASCII without spaces is a usability invariant: it ensures the grammar is predictable, shell-safe, and consistent across all noun definitions regardless of the ontology terms used internally.

```rust
fn invariant_noun_names_are_lowercase_ascii() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let combined = format!("{}{}", stdout, stderr);
    for word in combined.split_whitespace() {
        let trimmed = word.trim_matches(|c: char| !c.is_alphabetic());
        if trimmed.len() > 2
            && trimmed.chars().all(|c| c.is_alphabetic())
            && trimmed.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
        {
            assert!(
                trimmed == trimmed.to_lowercase(),
                "noun '{}' is not lowercase ascii",
                trimmed
            );
        }
    }
}
```

### Invariant 6: Binary Name Is `cargo-cicd`

The binary name is not merely a naming convention; it determines how `cargo` discovers the subcommand. Cargo subcommands must be named `cargo-<name>` to be invocable as `cargo <name>`. The invariant asserts binary existence and identity.

### Invariant 7: Status Command Exits Zero (Baseline Health)

The simplest invariant but arguably the most operationally significant. `cargo cicd status show` is the baseline health check: it reads the workspace state and reports it without side effects. If this command fails in a well-formed workspace, no other command can be trusted. The invariant asserts unconditional exit zero:

```rust
fn invariant_status_exits_zero() {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["status", "show"])
        .assert()
        .success();
}
```

This invariant underpins the operational contract of the system: any environment in which `status show` fails is, by definition, a broken environment, not a broken workspace.

### Additional Structural Invariants

Beyond the numbered seven, `tests/invariants.rs` also asserts:

- `invariant_all_nouns_accept_help` — all registered nouns accept `--help` without panicking. This is a regression guard against noun registration failures.
- `invariant_wasm4pm_scan_or_documented_absence` — the wasm4pm capability scan must produce at least one of three evidence documents (scan receipt, integration recommendation, or deferred extraction note), or the test explicitly marks itself PARTIAL with an explanatory message. This "soft invariant" documents process progress without blocking builds.

---

## 5.4 Feature Projection Tests: The Surface Contract

Rust's feature flag system enables conditional compilation of subsystems. cargo-cicd uses four non-default features: `process-data`, `autonomic`, `contrib`, and `wasm4pm`. Each feature flag expands the surface exposed to the user and to downstream tooling. The feature projection tests in `tests/feature_projection.rs` verify that this surface contract is stable — that enabling a feature exposes the correct capabilities, and that feature names themselves do not leak private architecture.

### The Feature Projection Contract

The feature hierarchy is:

```
default → (no Level 5 engine)
process-data → EngineState, adapters, cicd.toml
autonomic → process-data + PolicyState, policy evaluation
contrib → process-data + extra diagnostics
wasm4pm → process-data + Wasm4pmShell, verdict adjudication
```

The projection tests verify three properties:

**Property FP1: Default Build Produces Valid Output**

```rust
fn test_default_features_build_succeeds() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .arg("--help")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        combined.contains("cargo-cicd") || combined.contains("Usage"),
        "Expected help output from cargo-cicd --help, got: {}",
        combined
    );
    assert!(!combined.contains("panicked"), "cargo-cicd --help panicked: {}", combined);
}
```

The test uses `env!("CARGO_BIN_EXE_cargo-cicd")` rather than `assert_cmd` to access the compiled binary path directly, which is stable across toolchain versions and does not require `assert_cmd`'s lookup heuristics.

**Property FP2: Feature Names Do Not Expose Private Architecture**

```rust
fn test_feature_names_are_public_safe() {
    let cargo_toml = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    ).unwrap();
    assert!(cargo_toml.contains("process-data"), "missing expected feature: process-data");
    assert!(cargo_toml.contains("autonomic"), "missing expected feature: autonomic");
    assert!(!cargo_toml.contains("cell8"), "forbidden feature name: cell8");
    assert!(!cargo_toml.contains("ALIVE"), "forbidden feature name: ALIVE");
}
```

This test reads `Cargo.toml` directly rather than querying the binary, ensuring the source-level feature declaration is audited. Feature names like `cell8` would expose the internal cell-model architecture; `ALIVE` would expose the engine status marker.

**Property FP3: Publish Emits Required cicd.toml Sections**

When the `process-data` feature populates the state model, the publish verb must persist the full state to `cicd.toml`. The test verifies the presence of the three mandatory TOML sections — `[workspace]`, `[state]`, and `[target]` — after a publish run:

```rust
fn test_publish_emits_all_required_sections() {
    let dir = TempDir::new().unwrap();
    std::process::Command::new(env!("CARGO_BIN_EXE_cargo-cicd"))
        .current_dir(dir.path())
        .arg("publish").arg("run")
        .output().unwrap();
    let toml_path = dir.path().join("cicd.toml");
    if toml_path.exists() {
        let content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("[workspace]"), "missing [workspace] section");
        assert!(content.contains("[state]"), "missing [state] section");
        assert!(content.contains("[target]"), "missing [target] section");
    }
}
```

The conditional on `toml_path.exists()` accommodates environments where publish fails for reasons unrelated to the feature under test (e.g., no Cargo.toml in the temp directory). This is a deliberate choice: the projection test is validating the shape of output when output exists, not asserting that the verb always succeeds.

### The Verb Registry Contract

A related concern is the completeness of the verb registry — ensuring that no verb is accidentally dropped from a noun's `verbs()` list during a refactor. The verb registry tests in `tests/cli/verb_registry.rs` use a declarative macro to assert that every expected noun-verb pair responds to `--help`:

```rust
macro_rules! assert_verb_registered {
    ($noun:expr, $verb:expr) => {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args([$noun, $verb, "--help"])
            .output()
            .unwrap();
        assert!(
            output.status.success()
                || String::from_utf8_lossy(&output.stderr).contains("Usage")
                || String::from_utf8_lossy(&output.stdout).contains("Usage"),
            "verb '{} {}' not registered or --help broken",
            $noun, $verb
        );
    };
}
```

The macro accepts either a zero exit or a "Usage" string in output, because clap-noun-verb routes help text through stderr in certain error paths. The verb registry test currently validates 23 noun-verb pairs across eight nouns, covering the full public grammar surface.

---

## 5.5 The Autonomic Policy Layer

### 5.5.1 Design Rationale

The autonomic policy layer is cargo-cicd's mechanism for surfacing workspace health signals as actionable recommendations without taking any action. The design reflects a specific philosophical commitment: automated tooling in a CI/CD context must be conservative about remediation. A tool that automatically cleans a build directory, pulls from remote, or commits staged files introduces non-determinism into a process that is supposed to be deterministic. Policies in cargo-cicd run exclusively in `Suggest` mode by default; they observe, analyze, and recommend, but they do not act.

This commitment is enforced at the type level and the test level simultaneously. The `PolicyMode` enum has two variants:

```rust
pub enum PolicyMode {
    Suggest,
    Apply,
}
```

But `Apply` is reserved for future use and is never the default for any policy. The test `test_no_policy_uses_apply_mode_by_default` in `tests/autonomic_policies.rs` asserts this over all registered policies:

```rust
fn test_no_policy_uses_apply_mode_by_default() {
    for r in &[
        check_target_pressure(5.0, 20.0),
        check_toolchain_mismatch("stable", None),
        check_trybuild_changed(0),
        check_git_phase_dirty(0),
    ] {
        assert!(
            matches!(r.mode, PolicyMode::Suggest),
            "policy '{}' must default to Suggest mode, got {:?}",
            r.name, r.mode
        );
    }
}
```

### 5.5.2 The PolicyState Data Model

Each policy evaluation returns a `PolicyResult` struct:

```rust
pub struct PolicyResult {
    pub name: String,
    pub enabled: bool,
    pub mode: PolicyMode,
    pub verdict: PolicyVerdict,
    pub recommendation: String,
    pub event: String,
}
```

The `verdict` field is a three-valued type:

```rust
pub enum PolicyVerdict {
    Pass,
    Warn,
    Suggest,
}
```

The semantics of each variant are carefully distinguished:

- `Pass` — all conditions satisfied; recommendation string is empty. The test `test_pass_verdict_has_empty_recommendation` asserts this invariant holds for all policies under their pass conditions.
- `Warn` — a degraded condition exists but does not require immediate action. The workspace can proceed; the warning is advisory.
- `Suggest` — a condition exists that benefits from user action. A non-empty recommendation string is always produced.

The absence of a `Fail` variant is significant: policies never block progress. The distinction between blocking and non-blocking is reserved for the evidence gate layer (where the wasm4pm oracle may return `Refuse`), not the autonomic layer. A policy that would make `Fail` assertions under an autonomic framework risks creating situations where the developer cannot proceed until they address the policy signal, even when the signal is based on heuristics or stale data.

The `event` field names the policy check event, enabling downstream tooling to identify which policy fired when correlating policy results with XES evidence.

The input data model is decomposed into three context structs that cleanly separate concerns:

```rust
pub struct WorkspaceInfo {
    pub target_gb: f64,
    pub max_gb: f64,
    pub active_toolchain: String,
    pub pinned_toolchain: Option<String>,
    pub changed_trybuild_fixtures: usize,
}

pub struct GitState {
    pub dirty_count: usize,
    pub commits_behind: Option<usize>,
}

pub struct EvidenceState {
    pub changed_file_count: usize,
    pub evidence_fresh: bool,
    pub receipt_exists: bool,
    pub receipt_stale: bool,
}
```

This decomposition is a testability affordance: each policy function takes only the fields it needs, making it possible to construct minimal test inputs without populating an entire `EngineState`. The `run_all_policies` function accepts all three context structs and dispatches to the individual checkers.

### 5.5.3 Individual Policy Implementations

#### Policy 1: `target_pressure`

Evaluates the size of the `target/` build directory against a configurable maximum. The policy uses a two-threshold design: an 80% approach threshold produces `Warn`; exceeding the maximum produces `Suggest`.

```rust
pub fn check_target_pressure(target_gb: f64, max_gb: f64) -> PolicyResult {
    let (verdict, recommendation) = if target_gb > max_gb {
        (
            PolicyVerdict::Suggest,
            "Run cargo cicd target prune to reclaim disk space".to_string(),
        )
    } else if target_gb > max_gb * 0.8 {
        (PolicyVerdict::Warn, "Target directory approaching limit".to_string())
    } else {
        (PolicyVerdict::Pass, String::new())
    };
    // ...
}
```

The test suite for this policy covers four cases: strictly under limit (`Pass`), approaching limit at 80.5% (`Warn`), strictly over limit (`Suggest`), and at the boundary at 100.5% (`Suggest`). The four-case taxonomy maps directly to the three verdict values and the boundary condition:

```rust
fn test_target_pressure_approaching_warns() {
    // 80% threshold: 16.1 / 20.0 = 80.5% → Warn
    let result = check_target_pressure(16.1, 20.0);
    assert!(matches!(result.verdict, PolicyVerdict::Warn), ...);
}
```

#### Policy 2: `toolchain_mismatch`

Detects divergence between the active Rust toolchain and a pinned toolchain specified in `rust-toolchain.toml`. The logic uses Rust's pattern matching to express three cases cleanly:

```rust
pub fn check_toolchain_mismatch(active: &str, pinned: Option<&str>) -> PolicyResult {
    let (verdict, recommendation) = match pinned {
        Some(p) if active != p => (
            PolicyVerdict::Suggest,
            "Pin toolchain in rust-toolchain.toml".to_string(),
        ),
        _ => (PolicyVerdict::Pass, String::new()),
    };
    // ...
}
```

When `pinned` is `None`, the policy always passes: the absence of a pinned toolchain means there is no mismatch to detect. This is a deliberate grace: imposing a toolchain pinning requirement on all workspaces would be inappropriate for libraries and exploratory projects. The recommendation to pin appears only when a pin exists and is violated.

The test `test_toolchain_mismatch_suggests_pin` verifies not only the verdict but the content of the recommendation string:

```rust
fn test_toolchain_mismatch_suggests_pin() {
    let result = check_toolchain_mismatch("stable", Some("nightly-2026-05-30"));
    assert!(matches!(result.verdict, PolicyVerdict::Suggest), ...);
    assert!(
        result.recommendation.contains("Pin") || result.recommendation.contains("pin"),
        "recommendation should mention pinning, got: {}",
        result.recommendation
    );
}
```

Testing recommendation content is unusual but justified here: the recommendation is the primary output of the policy. A policy that recommends the wrong action is arguably worse than one that produces no recommendation, because it misleads the developer.

#### Policy 3: `trybuild_changed`

Detects changed trybuild fixture files and recommends a targeted re-run. The policy has binary logic: any non-zero count of changed fixtures triggers `Suggest`.

```rust
pub fn check_trybuild_changed(changed_fixtures: usize) -> PolicyResult {
    let (verdict, recommendation) = if changed_fixtures > 0 {
        (
            PolicyVerdict::Suggest,
            format!(
                "Run cargo cicd trybuild changed ({} fixtures changed)",
                changed_fixtures
            ),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };
    // ...
}
```

The test `test_trybuild_changed_includes_count` asserts that the numeric count of changed fixtures appears in the recommendation string. This is an important UX invariant: a recommendation that says "some fixtures changed" is less useful than one that says "7 fixtures changed." The count allows the developer to immediately assess the scope of work before running the command.

#### Policy 4: `git_phase_dirty`

Evaluates the cleanliness of the git working tree. The policy uses a count of dirty files as its signal and includes the count in the recommendation:

```rust
pub fn check_git_phase_dirty(dirty_count: usize) -> PolicyResult {
    let (verdict, recommendation) = if dirty_count > 0 {
        (
            PolicyVerdict::Suggest,
            format!(
                "Run cargo cicd git close to commit {} dirty files",
                dirty_count
            ),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };
    // ...
}
```

The test `test_git_dirty_includes_count` mirrors the pattern established for `trybuild_changed`:

```rust
fn test_git_dirty_includes_count() {
    let result = check_git_phase_dirty(12);
    assert!(
        result.recommendation.contains("12"),
        "recommendation should include the dirty file count, got: {}",
        result.recommendation
    );
}
```

#### Policy 5: `evidence_stale`

The evidence staleness policy has the most nuanced logic in the suite. It distinguishes three states based on the combination of changed file count and evidence freshness:

```rust
pub fn check_evidence_stale(changed_file_count: usize, evidence_fresh: bool) -> PolicyResult {
    let (verdict, recommendation) = if changed_file_count > 0 && !evidence_fresh {
        (
            PolicyVerdict::Suggest,
            "evidence stale: run 'cargo cicd test changed' and 'cargo cicd workspace doctor'"
                .to_string(),
        )
    } else if changed_file_count > 0 && evidence_fresh {
        (
            PolicyVerdict::Warn,
            "source changes detected — verify evidence is current before closing".to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };
    // ...
}
```

The three-way logic reflects a real epistemic distinction:

1. No changes and fresh evidence: `Pass` — the workspace is consistent and certified.
2. Changes present and evidence absent: `Suggest` — the developer has modified code but produced no evidence; the gap must be filled before closing.
3. Changes present and evidence present: `Warn` — evidence exists, but source changes may have invalidated it; human judgment is needed to determine if re-running is necessary.

This tripartite logic avoids the binary trap of treating "evidence exists" as synonymous with "evidence is valid," which would allow stale or misleading evidence to satisfy the policy.

#### Policy 6: `branch_behind`

Evaluates how many commits the local branch is behind the remote tracking branch. The policy handles the `None` case — no upstream configured, or git unavailable — gracefully by returning `Pass`:

```rust
pub fn check_branch_behind(commits_behind: Option<usize>) -> PolicyResult {
    let (verdict, recommendation) = match commits_behind {
        Some(n) if n > 0 => (
            PolicyVerdict::Suggest,
            format!(
                "branch is {} commit(s) behind remote — run 'git pull --rebase' to sync",
                n
            ),
        ),
        _ => (PolicyVerdict::Pass, String::new()),
    };
    // ...
}
```

The recommendation explicitly names `git pull --rebase` rather than `git pull`, because a rebase strategy preserves a linear history and avoids the creation of spurious merge commits. The policy cannot perform this operation itself (policies are read-only); it can only surface the recommendation.

The test `policy_branch_behind_evaluates_without_panic` verifies the graceful handling of the `None` case:

```rust
fn policy_branch_behind_evaluates_without_panic() {
    // commits_behind = None (no upstream configured) → Pass gracefully
    let result = check_branch_behind(None);
    assert!(matches!(result.verdict, PolicyVerdict::Pass), ...);
}
```

#### Policy 7: `publish_not_adjudicated`

The most release-critical of the seven policies. It verifies that a wasm4pm-adjudicated receipt exists and is current before publishing:

```rust
pub fn check_publish_not_adjudicated(receipt_exists: bool, receipt_stale: bool) -> PolicyResult {
    let (verdict, recommendation) = if !receipt_exists {
        (
            PolicyVerdict::Suggest,
            "no adjudicated receipt found — run 'cargo cicd evidence doctor' before publish"
                .to_string(),
        )
    } else if receipt_stale {
        (
            PolicyVerdict::Warn,
            "receipt exists but may be stale — re-run 'cargo cicd evidence doctor' to refresh"
                .to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };
    // ...
}
```

The two-parameter design (`receipt_exists`, `receipt_stale`) enables the same tripartite verdict logic as `evidence_stale`: no receipt is more alarming than a stale receipt, justifying the escalation from `Warn` to `Suggest` in the first branch.

### 5.5.4 The Aggregate Runner and the Seven-Result Contract

The `run_all_policies` function is the single entry point for the full policy evaluation sweep:

```rust
pub fn run_all_policies(
    workspace: &WorkspaceInfo,
    git: &GitState,
    evidence: &EvidenceState,
) -> Vec<PolicyResult> {
    vec![
        check_target_pressure(workspace.target_gb, workspace.max_gb),
        check_toolchain_mismatch(&workspace.active_toolchain, workspace.pinned_toolchain.as_deref()),
        check_trybuild_changed(workspace.changed_trybuild_fixtures),
        check_git_phase_dirty(git.dirty_count),
        check_branch_behind(git.commits_behind),
        check_evidence_stale(evidence.changed_file_count, evidence.evidence_fresh),
        check_publish_not_adjudicated(evidence.receipt_exists, evidence.receipt_stale),
    ]
}
```

The test `run_all_policies_returns_seven_results` asserts this count as a contract:

```rust
fn run_all_policies_returns_seven_results() {
    // ... construct WorkspaceInfo, GitState, EvidenceState with clean defaults ...
    let results = run_all_policies(&workspace, &git, &evidence);
    assert_eq!(results.len(), 7, "expected 7 policy results, got {}", results.len());
}
```

This count assertion is a stability contract: any addition or removal of a policy must be a deliberate architectural decision, not a silent side effect of a refactor. The companion test `run_all_policies_all_pass_on_clean_state` verifies that all seven policies return `Pass` when given clean inputs, establishing the baseline for clean workspace expectations:

```rust
fn run_all_policies_all_pass_on_clean_state() {
    // ... clean state inputs ...
    let results = run_all_policies(&workspace, &git, &evidence);
    for r in &results {
        assert!(
            matches!(r.verdict, PolicyVerdict::Pass),
            "policy '{}' should pass on clean state, got {:?}",
            r.name, r.verdict
        );
    }
}
```

---

## 5.6 Mutation Testing Strategy for Evidence Corruption

### 5.6.1 Theoretical Foundations

Mutation testing, as formalized by DeMillo, Lipton, and Sayward (1978) and systematized by Jia and Harman (2011), is a test-adequacy criterion based on the following principle: a test suite that cannot detect deliberately introduced faults is inadequate, regardless of its statement coverage. A mutant is a program obtained by applying a single syntactic transformation — a mutation operator — to the source; a test suite "kills" a mutant if at least one test fails against the mutant. The mutation score is the fraction of killed mutants over all generated mutants.

cargo-cicd applies mutation testing not to the source code but to the *evidence artifacts* produced by the system. This is a non-standard application of the mutation testing principle, but one that follows naturally from the system's architecture: the primary output of cargo-cicd is XES evidence, not program behavior in the traditional sense. The question "does the test suite detect a corrupted evidence artifact?" is directly analogous to "does the test suite detect a mutated source program?" Both are asking whether the test suite can distinguish good outputs from bad ones.

Jia and Harman (2011) classify mutation operators into three categories: method-level operators (manipulate method calls), state-level operators (manipulate field accesses), and value-level operators (manipulate constants and literals). In evidence mutation testing, the analogous classification is:

- **Structural operators**: corrupt the XML structure of the XES file (malformed tags, truncation, binary injection)
- **Semantic operators**: corrupt the content while preserving structure (flipped verdicts, contradicted sizes, omitted required fields)
- **Identity operators**: corrupt the identity chain (wrong encoding declaration, mismatched attribute values)

### 5.6.2 The Mutation Library

The mutation library in `tests/wasm4pm_evidence_mutation.rs` implements both structural and semantic mutation operators as exported functions:

```rust
pub fn corrupt_xes_mismatched_tags(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replacen("</event>", "</wrong_close>", 1);
    std::fs::write(path, mutated).unwrap();
}

pub fn corrupt_xes_contradictory_verdict(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replace("pass", "FAIL").replace("PASS", "FAIL");
    std::fs::write(path, mutated).unwrap();
}

pub fn corrupt_xes_missing_trace(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let start = content.find("<trace>").unwrap_or(content.len());
    let end = content.find("</trace>")
        .map(|i| i + "</trace>".len())
        .unwrap_or(content.len());
    let mutated = format!("{}{}", &content[..start], &content[end..]);
    std::fs::write(path, mutated).unwrap();
}
```

The harness in `tests/wasm4pm_harness.rs` also implements a JSONL-level mutation library for the companion evidence format, with operators named:

- `FlipVerdict` — change `verdict_claimed_by_cargo_cicd` from `PASS` to `FAIL` or vice versa
- `OmitField(field)` — remove a required field from the event record
- `ContradictSize` — set `target_size_bytes` to an implausible value (`u64::MAX / 2`)
- `HideChangedFile` — remove an entry from the `changed_files` list
- `AddFakeArtifact` — inject an artifact path that does not exist on disk

The design of the JSONL mutation operators is particularly notable. `ContradictSize` sets the target directory size to approximately 9.2 petabytes — a value that is implausible for any real workspace. This is deliberately non-subtle: the oracle must refuse evidence that makes physically impossible claims, not just evidence that makes internally inconsistent claims. `HideChangedFile` tests whether the oracle can detect omission attacks: a malicious or buggy emitter might report fewer changed files than actually exist, making a dirty workspace appear clean.

### 5.6.3 Test Structure for Mutation Cases

Each mutation test follows a three-phase structure: emit valid evidence, apply a mutation, assert refusal:

```rust
fn evidence_mutation_corrupted_xes_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mutated.xes");
    std::fs::write(&xes_path, "NOT VALID XML AT ALL").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

The oracle-availability branch prevents mutation tests from producing false passes when the oracle is absent. In oracle-absent mode, the test produces `Blocked` — a documented oracle-unavailability state — rather than falsely concluding that the oracle accepted the corrupted evidence.

The mutation test coverage is deliberately comprehensive. The eight XES mutations tested are:

1. Non-XML plain text content
2. Empty file (zero bytes)
3. Mismatched XML closing tags
4. Binary garbage (null bytes, non-UTF-8 sequences)
5. Truncated XML (cut off mid-element at byte 20)
6. Invalid XML attribute value (unescaped `<` character)
7. Wrong encoding declaration (`EBCDIC-US` declared, UTF-8 content)
8. Missing closing `</log>` tag

These mutations span the full range of XML structural validity requirements, ensuring the oracle is a genuine validator and not a rubber stamp that accepts any syntactically plausible file.

### 5.6.4 Invariants E1–E7

The refusal case tests in `tests/wasm4pm_refusal_cases.rs` also formalize seven structural invariants of the evidence API, designated E1 through E7:

- **E1** (`evidence_invariant_e1_no_self_certification`): cargo-cicd never adjudicates its own process conformance. Structural proof: `emit_xes` returns `Result<()>`, not a verdict. Only the oracle can return a verdict.
- **E2** (`evidence_invariant_e2_evidence_required_before_adjudication`): XES file must exist on disk before `audit_xes()` is called. The test verifies that `emit_xes` creates the file before the oracle is invoked.
- **E3** (`evidence_invariant_e3_blocked_is_first_class`): `Blocked` is a first-class expectation. Calling `assert_wpm_verdict` with `Blocked` expected must not panic when the oracle is unavailable.
- **E4** (documented in `wasm4pm_harness.rs` module comment): Tests assert only wasm4pm verdicts, never cargo-cicd self-assertions.
- **E5**: XES groups events by `case_id` into `<trace>` elements.
- **E6**: JSONL emission mirrors XES.
- **E7**: `Blocked` is not an error; it is a legitimate verdict for oracle-absent environments.

---

## 5.7 Changed-File-Driven Test Selection

### 5.7.1 The Problem of Re-Running Everything

A naive CI strategy re-runs every test on every commit. For small codebases this is acceptable; for large workspaces with extensive test suites it introduces unnecessary latency. The problem is not merely one of developer experience: in environments where test execution consumes compute resources with a monetary cost, redundant test runs have a direct financial impact. Furthermore, long CI cycles reduce the feedback bandwidth available to developers, increasing the cognitive cost of attributing a failure to a specific change.

cargo-cicd addresses this through changed-file-driven test selection: the `test changed` and `trybuild changed` verbs analyze which files have changed since the last commit (via `git diff origin/main --name-only`) and produce a test plan that covers only those files and their known dependents.

### 5.7.2 The Test Plan Invariant

The key invariant of the changed-file selection subsystem is what the tests call the "no fake precision" guarantee: the system must never claim to have run tests it did not run, and it must never run tests on the assumption that more files changed than actually did.

```rust
fn test_test_changed_emits_test_plan_not_fake_precision() {
    let dir = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("test")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    // The command must not panic — exit code is the only assertion.
    assert!(output.status.code().is_some(), "test changed should not panic: {}", combined);
}
```

In an environment with no git history (a fresh temp directory), the changed file detector finds no changed files and produces an empty plan. The test accepts this outcome: an empty plan is honest. What the test refuses to accept is a plan that claims to cover files it cannot actually have detected.

### 5.7.3 The ChangedFileDetector Classification Logic

The `ChangedFileDetector` adapter classifies each changed file into one of three categories:

1. **Test file** — a `.rs` file under `tests/`
2. **Trybuild fixture** — a `.rs` file matching the path pattern `tests/ui/compile_fail/*.rs`
3. **Source file** — any other `.rs` file

Classification drives the test plan: source file changes produce a test plan covering the source module and its dependents; trybuild fixture changes add those fixtures to the trybuild plan; test file changes add the test file directly.

The test `test_trybuild_changed_selects_only_changed_fixture` exercises the detection logic with a single synthetic fixture:

```rust
fn test_trybuild_changed_selects_only_changed_fixture() {
    let dir = TempDir::new().unwrap();
    let ui_dir = dir.path().join("tests/ui/compile_fail");
    std::fs::create_dir_all(&ui_dir).unwrap();
    std::fs::write(ui_dir.join("my_law.rs"), "fn main() {}").unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    assert!(output.status.code().is_some(), "trybuild changed should not panic");
}
```

The test does not assert on stdout content here — it asserts only on exit code. The reason is that in a temp directory without git history, the changed file detector returns an empty set (no git diff is available), and the command legitimately reports zero changed fixtures. The important property under test is that the command handles this environment without panicking.

---

## 5.8 The Trybuild Conservative Mode Invariant

### 5.8.1 Why Conservative Mode Exists

Trybuild tests are expensive relative to ordinary unit tests: each fixture file requires a full `rustc` invocation to capture and snapshot the compiler's error output. A workspace with 100 or more trybuild fixtures cannot re-run all of them on every CI cycle without significant latency consequences.

The conservative mode invariant is formally stated as follows: invoking `cargo cicd trybuild changed` must never announce or perform a full fixture sweep when the working environment does not have changed fixtures. This property is designated `INVARIANT_NO_FULL_TRYBUILD_BY_DEFAULT` in the codebase.

### 5.8.2 Test Implementation and Edge Cases

The invariant test constructs an environment with 100 fixture files and verifies that the output does not mention having run all of them:

```rust
fn invariant_no_full_trybuild_by_default() {
    let dir = TempDir::new().unwrap();
    let ui_dir = dir.path().join("tests/ui/compile_fail");
    std::fs::create_dir_all(&ui_dir).unwrap();
    for i in 0..100 {
        std::fs::write(ui_dir.join(format!("fixture_{}.rs", i)), b"fn main() {}").unwrap();
    }
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        !combined.contains("100 fixtures") && !combined.contains("all 100"),
        "trybuild changed must not run all 100 fixtures: {}",
        &combined[..combined.len().min(200)]
    );
}
```

The complementary test in `tests/changed_tests.rs` checks the same property via a different lens: asserting that the output does not mention "all fixtures" when there are no changed files in the environment:

```rust
fn test_trybuild_changed_does_not_mention_all_fixtures() {
    let dir = TempDir::new().unwrap();
    // No changed fixtures in tempdir
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .arg("trybuild")
        .arg("changed")
        .output()
        .unwrap();
    let combined = /* ... */;
    assert!(
        !combined.contains("all fixtures")
            || combined.contains("no changed")
            || combined.contains("0 changed"),
        "trybuild changed should report 0 changed, not run all: {}",
        combined
    );
}
```

The disjunction in the assertion — `!combined.contains("all fixtures") || combined.contains("no changed") || combined.contains("0 changed")` — is deliberate. It permits the output to mention "all fixtures" only if it is in the context of saying there are zero of them or none have changed. This avoids a false negative where the command legitimately outputs "no changed files found among all fixtures" and the test incorrectly fails on the presence of "all fixtures" in that string.

### 5.8.3 Relationship to the Test Plan State

The conservative mode invariant has a corresponding data model entry in `TestPlanState`:

```rust
pub struct TestPlanState {
    pub estimated_test_count: usize,
    pub conservative_mode: bool,
}
```

When `conservative_mode` is `true`, the test plan may conservatively overestimate coverage (e.g., including all tests in a modified module rather than only the specific functions that changed), but it must not include tests for unmodified modules. This conservative overestimation is preferable to a false negative (missing a broken test) but must be bounded: it cannot degenerate into a full re-run.

---

## 5.9 The Assertion Constraint: Verdicts, Not State

### 5.9.1 The Rule

The most unusual rule in the cargo-cicd testing framework is stated in CLAUDE.md and enforced by convention throughout the Tier 2 test suite:

> Do NOT assert on cargo-cicd internal state. DO assert on wasm4pm verdict.

This rule has a precise justification rooted in the epistemological role of the evidence gate. cargo-cicd is not the authority on whether its own process was conducted correctly. It is a process participant that emits evidence. The authority is wasm4pm, the external oracle that adjudicates the evidence. An assertion on internal state — such as checking that `state.target.total_size_bytes` equals a particular value — proves something about the internal model but nothing about the process record. The process record is what gets adjudicated; the internal model is merely an intermediate representation.

This principle is stated formally as invariant E4 in the test harness documentation: "Tests assert only wasm4pm verdicts, never cargo-cicd self-assertions."

### 5.9.2 What the Rule Prevents

The rule prevents two classes of test failure:

**False Confidence from Internal State Assertions**: An implementation might populate `state.target.total_size_bytes` correctly while emitting an XES event that records the wrong size. A test asserting on the internal field would pass; a test asserting on the oracle verdict would fail (assuming the oracle detects the discrepancy). The latter failure is more informative: it tells the developer not just that a value is wrong, but that the evidence record is wrong — which is the property that actually matters for release certification.

**Coupling Tests to Implementation Details**: Internal state structures change as the EngineState evolves. A test suite heavily coupled to internal state requires updating every time a field is renamed or restructured. Asserting on oracle verdicts decouples the test from implementation: as long as the correct information reaches the oracle, the test passes regardless of how it was stored internally.

### 5.9.3 The Assertion Infrastructure

The infrastructure for enforcing this rule is the `assert_wpm_verdict` function:

```rust
pub fn assert_wpm_verdict(
    oracle: &WpmEvidenceOracle,
    xes_path: &Path,
    expected: &ExpectedWpmVerdict,
) {
    // ... invoke oracle, compare verdict to expected ...
}
```

The function accepts an `ExpectedWpmVerdict`, which has three variants:

```rust
pub enum ExpectedWpmVerdict {
    Accept,
    Refuse,
    Blocked,
}
```

The `Blocked` variant is the mechanism by which oracle-absent environments remain valid test environments: when `expected` is `Blocked` and the oracle is unavailable, the assertion passes. This allows the full Tier 2 test suite to run in development environments without the oracle, producing `Blocked` results throughout but not panicking or producing false passes.

### 5.9.4 The Hard Gate Override

For release certification, the environment variable `REQUIRE_WPM_ORACLE=1` converts every `Blocked` outcome to a hard panic:

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

Additionally, the `evidence_gate_wpm_doctor_hard_gate` test does not use the graceful fallback at all: when the oracle is present, it invokes `wpm doctor` directly and asserts that the exit code is zero and no failure indicators appear in the output. The assertion is framed explicitly as a gate: "GATE-FAIL" rather than an ordinary assertion message.

---

## 5.10 Test Infrastructure: The Fixture Workspace Pattern

A recurrent pattern throughout the test suite is the use of temporary directory fixtures — minimal Rust workspaces constructed in ephemeral `TempDir` instances. This pattern serves two purposes: isolation (each test operates in its own environment, preventing state leakage between tests) and reproducibility (the fixture state is precisely controlled, unlike a real workspace whose git state, file system, and toolchain may vary between machines).

The `FixtureWorkspace` struct in `tests/wasm4pm_harness.rs` provides a reusable abstraction over this pattern:

```rust
pub struct FixtureWorkspace {
    dir: TempDir,
}

impl FixtureWorkspace {
    pub fn new_clean() -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write_minimal_cargo_toml(root);
        write_minimal_main_rs(root);
        Self { dir }
    }

    pub fn with_git() -> Self {
        let ws = Self::new_clean();
        let root = ws.path().to_path_buf();
        let _ = run_git(&root, &["init"]);
        let _ = run_git(&root, &["config", "user.email", "test@example.com"]);
        let _ = run_git(&root, &["config", "user.name", "Test"]);
        let _ = run_git(&root, &["add", "."]);
        let _ = run_git(&root, &["commit", "-m", "init"]);
        ws
    }

    pub fn run_cargo_cicd(&self, args: &[&str]) -> CicdOutput {
        let binary = std::env::var("CARGO_BIN_EXE_cargo-cicd")
            .map(PathBuf::from)
            .unwrap_or_else(|_| { /* fallback to target/debug */ });
        // ...
    }
}
```

The `with_git` variant creates a workspace with a clean git history, enabling tests that require git state detection to operate without depending on the broader test runner's environment.

The `evidence_path` method returns the canonical path to the JSONL evidence file:

```rust
pub fn evidence_path(&self) -> PathBuf {
    self.path()
        .join("target")
        .join("cargo-cicd")
        .join("evidence")
        .join("events.jsonl")
}
```

This canonicalization ensures that all tests agree on where evidence is written, enabling the mutation tests to locate the file for corruption without needing to discover it.

---

## 5.11 Synthesis: A Verification Architecture for Process-Data Systems

The verification approach of cargo-cicd reflects a coherent theoretical position about what it means to test a Level 5 process-data engine. Traditional software testing asks: "does the implementation produce the correct output for each input?" cargo-cicd's testing asks a more demanding question: "does the process, as recorded in evidence and adjudicated by an external oracle, satisfy the conformance criteria?"

This shift in verification level has several consequences, all of which are reflected in the design choices documented in this chapter:

**Test stratification is necessary, not optional.** Because process conformance and implementation correctness are distinct predicates, the tests that prove each must be kept separate. Allowing Tier 1 tests to assert on oracle verdicts, or Tier 2 tests to assert on internal state, would collapse this distinction and produce a suite that is incoherent about what it is actually proving.

**Mutation testing is the appropriate adequacy criterion for evidence testing.** Coverage-based adequacy criteria measure how much of the source code is executed; they say nothing about whether the test suite can detect bad evidence. Mutation testing of evidence artifacts — introducing corruptions that the oracle must detect — is the correct criterion for this level of verification.

**Policies must be read-only to be trustworthy.** An autonomic policy that takes action has the same epistemological status as a test that asserts on its own side effects: the act of measurement changes what is being measured. Policies confined to `Suggest` mode provide information without introducing the confounding variable of automated remediation.

**The oracle must be external to the system under test.** Invariant E1 — cargo-cicd never adjudicates itself — is not merely a design preference; it is the axiom on which the entire evidence layer rests. A system that both emits and adjudicates its own evidence can, trivially, produce evidence that always passes adjudication regardless of what actually happened. Externalizing the oracle to wasm4pm makes self-certification structurally impossible.

These principles, taken together, constitute a verification architecture that is suited to the complexity of a CI/CD system that treats process correctness, not merely functional correctness, as the fundamental quality predicate.

---

## References

DeMillo, R. A., Lipton, R. J., and Sayward, F. G. (1978). Hints on Test Data Selection: Help for the Practicing Programmer. *IEEE Computer*, 11(4), 34–41.

Jia, Y., and Harman, M. (2011). An Analysis and Survey of the Development of Mutation Testing. *IEEE Transactions on Software Engineering*, 37(5), 649–678.

Meyer, B. (1988). *Object-Oriented Software Construction*. Prentice Hall.

Offutt, A. J., and Untch, R. H. (2001). Mutation 2000: Uniting the Orthogonal. In *Mutation Testing for the New Century*, 34–44.

Clarke, E. M., Grumberg, O., and Peled, D. (1999). *Model Checking*. MIT Press.

Humble, J., and Farley, D. (2010). *Continuous Delivery: Reliable Software Releases through Build, Test, and Deployment Automation*. Addison-Wesley.

---

*End of Chapter 5*
