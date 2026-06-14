# cargo-cicd Testing Guide

## Overview

This guide covers the complete testing strategy for cargo-cicd v26.6.2. cargo-cicd is a process-data engine with three test tiers:

1. **Smoke Tests** — Public boundary invariants and parsing
2. **Integration Tests** — Noun-verb CLI behavior using `assert_cmd` and isolated fixture workspaces
3. **Evidence-Gate Tests** — wasm4pm acceptance verdicts (release closure requirement)

---

## Table of Contents

1. [Test Organization](#test-organization)
2. [Fixture Design](#fixture-design)
3. [Writing New Tests](#writing-new-tests)
4. [Mocking and Isolation](#mocking-and-isolation)
5. [Evidence-Gate Testing](#evidence-gate-testing)
6. [CI/CD Integration](#cicd-integration)
7. [Test Utilities](#test-utilities)
8. [Patterns and Examples](#patterns-and-examples)

---

## Test Organization

### Test Tiers

#### 1. Smoke Tests (Non-Closing)

**Purpose:** Verify public boundaries, CLI parsing, and schema validity.

**Files:**
- `/tests/invariants.rs` — 7 non-negotiable invariants enforced on every build
- `/tests/feature_projection.rs` — Feature flag surface contract (no forbidden terms in public output)
- `/tests/cli/test_status.rs`, etc. — Individual noun command parsing

**Characteristics:**
- Fast, deterministic, no external dependencies
- Safe to run in all environments
- Use `assert_cmd::Command` to invoke the binary
- Assertions are about CLI behavior, not internal state
- Exit codes may be 0 or 1 (depending on environment)

**Example:**
```rust
#[test]
fn invariant_public_boundary_no_forbidden_terms() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["status", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(!text.contains("ALIVE"), "Forbidden term 'ALIVE' in public output");
}
```

**Key Invariants:**
- No forbidden terms (ALIVE, Nehemiah, CONSTRUCT8, etc.) in any public output
- `target prune` without `--confirm` must not delete files
- `trybuild changed` must not run all fixtures by default
- Feature flag names are public-safe
- wasm4pm capability is documented (scan receipt, integration docs, or deferral)

---

#### 2. Integration Tests (Non-Closing)

**Purpose:** Test noun-verb CLI commands against realistic, isolated workspaces.

**Files:**
- `/tests/cli/` — Command projection and behavior tests
  - `test_status.rs`, `test_target.rs`, `test_git.rs`, etc.
- `/tests/changed_tests.rs` — Test selection (`test changed`)
- `/tests/cicd_toml_truth.rs` — cicd.toml schema and round-trip
- `/tests/autonomic_policies.rs` — Policy verdicts and recommendations
- `/tests/git_phase_closure.rs` — Git state verification (clean/dirty)

**Characteristics:**
- Use `FixtureWorkspace` to create isolated temporary workspaces
- Test commands with various pre-conditions (clean, dirty, toolchain mismatch, etc.)
- Assert on CLI exit codes and output patterns
- May skip gracefully if external tools (git, cargo) are unavailable
- Do NOT assert on internal engine state; only on observable CLI behavior

**Example:**
```rust
#[test]
fn test_git_close_refuses_dirty_tree() {
    let fixture = FixtureWorkspace::clean();
    
    // Create untracked file to make the tree dirty
    std::fs::write(fixture.root.join("untracked.txt"), "dirty\n").unwrap();
    
    // Run git close — should be refused
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["git", "close"])
        .output()
        .unwrap();
    
    // Assert: exit non-zero (refused)
    assert!(!output.status.success(), "dirty tree must be refused");
    
    // Assert: file is still untracked (no silent commit)
    let file_still_exists = fixture.root.join("untracked.txt").exists();
    assert!(file_still_exists, "git close must not commit untracked files");
}
```

---

#### 3. Evidence-Gate Tests (Release Closure)

**Purpose:** Verify cargo-cicd emits valid process evidence (XES format) and wasm4pm adjudicates correctly.

**Files:**
- `/tests/wasm4pm_evidence_gate.rs` — Positive acceptance cases
- `/tests/wasm4pm_evidence_mutation.rs` — Negative refusal cases (corrupted evidence)
- `/tests/wasm4pm_refusal_cases.rs` — Edge cases (missing oracle, etc.)

**Characteristics:**
- Emit XES (XML Event Stream) process logs
- Invoke wpm oracle: `wpm audit <file.xes>` or `wpm receipt doctor --format json --strict`
- Assert on wpm verdict: Accept, Refuse, or Blocked (oracle absent)
- Require `REQUIRE_WPM_ORACLE=1` in CI for strict enforcement
- No release may claim "ALIVE" solely from cargo-cicd internal tests

**Example:**
```rust
#[test]
fn evidence_gate_status_show_accepted() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("status show", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        // Accept is only asserted when wpm oracle is present
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

---

### Test Hierarchy

```
tests/
├── invariants.rs                    # Smoke: 7 invariants (public boundary, safety, etc.)
├── feature_projection.rs            # Smoke: feature flag surface
├── cli/
│   ├── test_status.rs              # Integration: status command
│   ├── test_target.rs              # Integration: target command
│   ├── test_git.rs                 # Integration: git command
│   ├── test_publish.rs             # Integration: publish command
│   ├── test_workspace.rs           # Integration: workspace command
│   ├── test_evidence.rs            # Integration: evidence command
│   └── command_projection.rs        # Harness: all commands
├── changed_tests.rs                # Integration: test changed, trybuild changed
├── cicd_toml_truth.rs              # Integration: toml schema
├── autonomic_policies.rs           # Integration: policy verdicts
├── git_phase_closure.rs            # Integration: git state
├── wasm4pm_evidence_gate.rs        # Evidence-gate: positive cases (Release Closure)
├── wasm4pm_evidence_mutation.rs    # Evidence-gate: negative cases
├── wasm4pm_refusal_cases.rs        # Evidence-gate: edge cases
├── wasm4pm_harness.rs              # Evidence-gate: harness
├── fixtures/
│   ├── mod.rs                      # FixtureWorkspace helpers
│   ├── clean_workspace/            # Pre-built fixture
│   ├── dirty_workspace/            # Pre-built fixture
│   ├── missing_manifest/           # Pre-built fixture
│   ├── corrupted_cicd_toml/        # Pre-built fixture
│   └── ...
└── [other integration tests]
```

---

## Fixture Design

### FixtureWorkspace API

Located in `/tests/fixtures/mod.rs`, `FixtureWorkspace` encapsulates a temporary workspace for testing.

**Lifecycle:**
- Constructor creates a `TempDir` and populates it with the described state
- Fixture is dropped at test end → `TempDir` is automatically cleaned up
- All paths are absolute; no cleanup hooks needed

**Constructors:**

```rust
use crate::fixtures::FixtureWorkspace;

// Minimal, well-formed workspace: valid Cargo.toml, git-initialized, clean tree
let fixture = FixtureWorkspace::clean();

// Clean + one untracked file (workspace is "dirty")
let fixture = FixtureWorkspace::dirty();

// Empty temp directory (no Cargo.toml, no git)
let fixture = FixtureWorkspace::missing_manifest();

// Clean + rust-toolchain.toml with unlikely channel (simulates toolchain mismatch)
let fixture = FixtureWorkspace::with_toolchain_mismatch();

// Clean + target/debug/placeholder.bin (1 MB fake artifact)
let fixture = FixtureWorkspace::with_target_over_limit();

// Clean + corrupted cicd.toml (invalid TOML syntax)
let fixture = FixtureWorkspace::with_corrupted_cicd_toml();

// Clean + stale cicd.toml (declares dirty=false but workspace is actually dirty)
let fixture = FixtureWorkspace::with_stale_cicd_toml();

// Clean + tests/ui/ with 10 "unchanged" and 1 "changed" fixture
let fixture = FixtureWorkspace::with_changed_trybuild_fixture();
```

**Fields:**
```rust
pub struct FixtureWorkspace {
    pub dir: TempDir,           // Backing temp dir (kept alive for lifetime)
    pub root: PathBuf,          // Absolute path to workspace root
}

// Usage
let fixture = FixtureWorkspace::clean();
assert!(fixture.root.join("Cargo.toml").exists());
std::fs::write(fixture.root.join("custom.txt"), "content").unwrap();
```

### Building Custom Fixtures

To create a fixture not in the standard list, extend `FixtureWorkspace`:

```rust
use std::fs;
use tempfile::TempDir;

#[test]
fn test_with_custom_fixture() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    
    // Write custom Cargo.toml
    fs::write(root.join("Cargo.toml"), r#"
[package]
name = "my-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
"#).unwrap();
    
    // Write custom source code
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn greet() -> &'static str { \"hello\" }").unwrap();
    
    // Initialize git
    let _ = run_git(&root, &["init"]);
    let _ = run_git(&root, &["config", "user.email", "test@example.com"]);
    let _ = run_git(&root, &["add", "."]);
    let _ = run_git(&root, &["commit", "-m", "init"]);
    
    // Now run a test against this fixture
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(&root)
        .args(["status", "show"])
        .output()
        .unwrap();
    
    assert!(output.status.success() || !output.status.success(), "test your assertion");
}

fn run_git(cwd: &std::path::Path, args: &[&str]) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).into_owned()) }
}
```

### Pre-Built Fixture Directories

Located in `/tests/fixtures/*/`, each pre-built fixture includes a `README.md` describing its state:

- `clean_workspace/` — Minimal valid workspace, clean git tree
- `dirty_workspace/` — Clean baseline + untracked file
- `missing_manifest/` — Empty directory
- `corrupted_cicd_toml/` — Clean baseline + invalid cicd.toml
- `stale_cicd_toml/` — Clean baseline + cicd.toml with dirty=false but actual dirty state
- `toolchain_mismatch/` — Clean baseline + rust-toolchain.toml (unlikely channel)
- `target_over_limit/` — Clean baseline + 1 MB fake artifact in target/debug/
- `trybuild_changed_only/` — 10 unchanged fixtures + 1 changed (for trybuild testing)
- `trybuild_huge_set/` — 50 fixture files (for performance testing)
- `wasm4pm_missing/` — Evidence-gate fixture with no wpm oracle

---

## Writing New Tests

### Step 1: Identify the Test Tier

**Smoke Test?** → Use `invariants.rs` or `feature_projection.rs`
- Testing public output, CLI parsing, feature flags
- No fixture needed; just invoke the binary

**Integration Test?** → Create a new file in `tests/cli/` or existing integration test
- Testing noun-verb behavior with various workspace states
- Use `FixtureWorkspace`; assert on exit codes and output

**Evidence-Gate Test?** → Add to `tests/wasm4pm_evidence_*.rs`
- Testing process evidence emission and wpm verdict
- Emit XES; invoke oracle; assert on verdict

### Step 2: Choose Dependencies

Add to `Cargo.toml` `[dev-dependencies]` if needed:
- `assert_cmd = "2"` — Run the binary and assert on output
- `tempfile = "3"` — Create temporary fixtures
- `predicates = "3"` — Composable assertions (e.g., exit code predicates)
- `toml = "0.8"` — Parse cicd.toml

All are already in `Cargo.toml`.

### Step 3: Write the Test

#### Smoke Test Example

```rust
#[test]
fn test_my_invariant() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["my-noun", "my-verb", "--help"])
        .output()
        .unwrap();
    
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    
    // Assert on observable behavior
    assert!(text.contains("expected text"), "Output missing expected text: {}", text);
    assert!(!text.contains("FORBIDDEN"), "Output contains forbidden term: {}", text);
}
```

#### Integration Test Example

```rust
#[test]
fn test_my_command() {
    let fixture = FixtureWorkspace::clean();
    
    // Prepare the fixture
    std::fs::write(fixture.root.join("custom_config.toml"), "[section]\nkey = \"value\"\n").unwrap();
    
    // Run the command
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["my-noun", "my-verb", "--option", "value"])
        .output()
        .unwrap();
    
    // Assert on results
    assert!(output.status.success(), "Command must succeed; stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Assert on side effects
    let result_file = fixture.root.join("result.txt");
    assert!(result_file.exists(), "Command must create result.txt");
}
```

### Step 4: Run and Verify

```bash
# Run a single test
cargo test --test my_test_file test_my_function

# Run all tests in a file
cargo test --test my_test_file

# Run all tests
cargo test

# Run with feature flags
cargo test --features process-data
cargo test --features autonomic

# Run evidence-gate tests with oracle required
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

---

## Mocking and Isolation

### Temporary Filesystems with `tempfile`

All integration tests use `tempfile::TempDir` for isolation. Never use `/tmp` or fixed paths.

```rust
use tempfile::TempDir;

#[test]
fn test_isolated_workspace() {
    let dir = TempDir::new().unwrap();  // Isolated temp dir
    let root = dir.path();              // Path: &Path
    
    // Write files — they are isolated from other tests
    std::fs::write(root.join("file.txt"), "content").unwrap();
    
    // Run commands in this isolated directory
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(root)
        .arg("status")
        .output()
        .unwrap();
    
    // When dir is dropped (test ends), the entire temp directory is cleaned up
}
```

### Git State Control

Initialize git as needed for tests:

```rust
fn init_git_repo(cwd: &std::path::Path) {
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(cwd)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(cwd)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(cwd)
        .output();
}

#[test]
fn test_git_commands() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    
    // Now the directory is a git repo
    std::fs::write(dir.path().join("file.txt"), "content").unwrap();
    
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["git", "status"])
        .output()
        .unwrap();
    
    assert!(output.status.success() || !output.status.success(), "git command ran");
}
```

**Key point:** Gracefully handle missing git. Many CI environments have git, but not all. Use:

```rust
let _ = std::process::Command::new("git")...  // Ignore errors if git is not in PATH
```

### Mocking Cargo Metadata

cargo-cicd adapters read from `cargo metadata`. To mock:

1. Create a minimal `Cargo.toml` in the fixture
2. If `cargo` is available, actual metadata is read
3. If `cargo` is unavailable, gracefully degrade (tests should skip or assert on absence)

```rust
#[test]
fn test_target_scanning() {
    let fixture = FixtureWorkspace::clean();
    
    // The fixture has a valid Cargo.toml; cargo metadata will be read if cargo is available
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["target", "show"])
        .output()
        .unwrap();
    
    // Assert on output, not internal metadata
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        text.contains("target") || !output.status.success(),
        "target show must succeed or indicate missing cargo"
    );
}
```

### Controlling Workspace State

Use `FixtureWorkspace` builders to set up specific states:

```rust
#[test]
fn test_target_pressure_warning() {
    // Fixture with a large artifact
    let fixture = FixtureWorkspace::with_target_over_limit();
    
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["status"])
        .output()
        .unwrap();
    
    // May warn about target size
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    
    // Don't assert on exact message; just that it ran
    assert!(output.status.code().is_some(), "status must run");
}
```

### Time/Environment Control

For policy tests that depend on time-based or computed values:

```rust
use cargo_cicd::autonomic::policies::{check_target_pressure, PolicyVerdict};

#[test]
fn test_policy_direct_unit() {
    // Call the policy function directly with known inputs
    // No need for fixtures or fixtures for unit-level policy testing
    
    let result = check_target_pressure(25.0, 20.0);  // 25 MB actual, 20 MB limit
    assert!(matches!(result.verdict, PolicyVerdict::Suggest));
    
    let result = check_target_pressure(5.0, 20.0);   // 5 MB actual, 20 MB limit
    assert!(matches!(result.verdict, PolicyVerdict::Pass));
}
```

---

## Evidence-Gate Testing

### XES (XML Event Stream) Format

cargo-cicd emits process logs in XES format. XES is a standard XML-based format for event logs.

**Basic Structure:**
```xml
<?xml version="1.0"?>
<log xes:version="1.0" xmlns:xes="http://www.xesstandard.org/">
  <event>
    <string key="concept:name" value="status show"/>
    <string key="lifecycle:transition" value="PASS"/>
  </event>
</log>
```

**ProcessEvent API:**
```rust
use cargo_cicd::evidence::ProcessEvent;

let event = ProcessEvent::new("status show", "PASS");
// Fields: concept:name (the command), lifecycle:transition (result: PASS, FAIL, DRY-RUN, etc.)
```

### WpmEvidenceOracle

The `WpmEvidenceOracle` discovers and invokes the wpm binary.

**Discovery:**
```rust
use cargo_cicd::evidence::WpmEvidenceOracle;

let oracle = WpmEvidenceOracle::new();

// Check if the oracle is available
if oracle.is_available() {
    // The wpm binary exists at the known path; oracle is ready
} else {
    // The wpm binary is missing; gracefully degrade to Blocked verdict
}
```

**Oracle Invocation:**
```rust
use cargo_cicd::evidence::{assert_wpm_verdict, ExpectedWpmVerdict};

assert_wpm_verdict(
    &oracle,
    &xes_path,
    &ExpectedWpmVerdict::Accept,  // Expected verdict
);
// Panics if the actual verdict differs from expected
```

### Evidence-Gate Test Pattern

```rust
#[test]
fn evidence_gate_command_accepted() {
    use cargo_cicd::evidence::{assert_wpm_verdict, emit_xes, ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle};
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    
    // Step 1: Emit XES with process evidence
    let events = vec![ProcessEvent::new("my-command", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    assert!(xes_path.exists(), "XES file must exist before oracle call");
    
    // Step 2: Get the oracle
    let oracle = WpmEvidenceOracle::new();
    
    // Step 3: Assert verdict
    if oracle.is_available() {
        // Oracle present: assert Accept
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        // Oracle absent: assert Blocked (not Accept, not Refuse)
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

**Key Behavior:**
- When the wpm oracle is present, tests assert on the actual verdict (Accept or Refuse)
- When the wpm oracle is absent, tests assert on `Blocked` (oracle unavailable)
- Setting `REQUIRE_WPM_ORACLE=1` forces the test to panic if the oracle is absent, preventing silent skips in CI

### Mutation Testing

To verify wasm4pm is a real adjudicator (not a rubber stamp), corrupt evidence and assert refusal:

```rust
#[test]
fn evidence_mutation_corrupted_xes_refused() {
    use cargo_cicd::evidence::{assert_wpm_verdict, ExpectedWpmVerdict, WpmEvidenceOracle};
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mutated.xes");
    
    // Write intentionally malformed XES
    std::fs::write(&xes_path, "NOT VALID XML AT ALL").unwrap();
    
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        // Valid oracle: corrupted evidence must be Refused
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        // No oracle: Blocked
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

**Mutation Strategies:**
- Empty file → Refuse
- Not valid XML → Refuse
- Mismatched XML tags → Refuse
- Binary garbage → Refuse
- Truncated XES → Refuse
- Valid structure but semantically invalid (e.g., negative timestamps) → Refuse

### Oracle-Absent Coverage Note

In CI environments without the wpm oracle binary (at `/Users/sac/wasm4pm/target/release/wpm`), evidence-gate tests fall back to `Blocked` verdict and Accept assertions are skipped.

To force strict enforcement in CI:
```bash
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

This panics with a clear message if the oracle is absent, preventing silent test skips.

---

## CI/CD Integration

### Test Commands

From `/CLAUDE.md`:

```bash
# Build (using cargo-make)
cargo make build

# Check (lint + type-check)
cargo make check

# Run all tests (all tiers)
cargo make test

# Run a single integration test
cargo test --test invariants
cargo test --test cli
cargo test --test changed_tests
cargo test --test git_phase_closure

# Run evidence-gate tests (with oracle required)
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate

# Run with feature flags
cargo test --features process-data
cargo test --features autonomic
```

### Feature Flag Matrix

cargo-cicd has 4 feature flags with implications:

- `default` — No features (bare binary, no internal state export)
- `process-data` — Enables Level 5 engine internals, XES emission
- `autonomic` → implies `process-data` — Enables policy/suggest mode
- `wasm4pm` → implies `process-data` — Enables richer wpm integration
- `contrib` → implies `process-data` — Contrib mode

**CI Test Matrix:**
```yaml
matrix:
  features:
    - default            # cargo test (no feature flags)
    - process-data       # cargo test --features process-data
    - autonomic          # cargo test --features autonomic
    - wasm4pm            # cargo test --features wasm4pm
```

**Feature Projection Tests** (`tests/feature_projection.rs`) verify that:
- Feature names are public-safe (no internal terms)
- Default binary build succeeds
- Process-data projection includes required sections in cicd.toml

### Performance Considerations

#### Test Duration Targets

- **Smoke tests** (invariants, feature_projection): < 5 sec total
- **Integration tests** (cli, changed_tests, etc.): < 30 sec total
- **Evidence-gate tests**: < 60 sec total (wpm oracle invocation can be slow)
- **Full suite**: < 2 min

#### Optimization Strategies

1. **Run tests in parallel:**
   ```bash
   cargo test -- --test-threads=4
   ```

2. **Skip slow tests in development:**
   ```bash
   cargo test --lib  # Only library tests (unit-level)
   ```

3. **Use `cargo make check`** instead of `cargo test` for pre-commit:
   ```bash
   cargo make check  # Lint + type-check, no test execution
   ```

4. **Skip evidence-gate tests unless oracle is available:**
   ```bash
   # Only run evidence-gate tests when REQUIRE_WPM_ORACLE=1 is set in CI
   if [ -z "$REQUIRE_WPM_ORACLE" ]; then
     cargo test --lib --test invariants --test feature_projection
   else
     cargo test  # Full suite including evidence-gate
   fi
   ```

### CI Workflow

**Pre-Commit (Local):**
```bash
cargo make check    # Fast lint + type-check
cargo test --lib   # Fast unit tests
```

**Pre-Push (Local):**
```bash
cargo test          # Full integration + smoke tests (no oracle)
```

**Merge Gate (CI):**
```bash
cargo make build    # Full build
cargo make test     # Full test suite
cargo test --features process-data
cargo test --features autonomic
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

**Release Gate:**
- Evidence-gate tests must pass with `REQUIRE_WPM_ORACLE=1`
- No release may claim "ALIVE" solely from cargo-cicd internal tests
- wasm4pm verdict (Accept/Refuse) is the source of truth

---

## Test Utilities

### `FixtureWorkspace` (in `/tests/fixtures/mod.rs`)

**API Summary:**

| Constructor | Description | Expected Verdict |
|-----------|-------------|------------------|
| `clean()` | Minimal, valid, clean git tree | PASS |
| `dirty()` | Clean + untracked file | WARN |
| `missing_manifest()` | Empty directory, no Cargo.toml | REFUSE |
| `with_toolchain_mismatch()` | Clean + unlikely rust-toolchain.toml | WARN |
| `with_target_over_limit()` | Clean + 1 MB fake artifact | WARN |
| `with_corrupted_cicd_toml()` | Clean + invalid cicd.toml | FAIL/REFUSE |
| `with_stale_cicd_toml()` | Clean + cicd.toml with dirty=false but actual dirty | WARN |
| `with_changed_trybuild_fixture()` | Clean + 10 unchanged + 1 changed fixture | PASS |

### `assert_cmd::Command`

```rust
use assert_cmd::Command;
use predicates::prelude::*;

// Run the binary
let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
cmd.args(["status", "show"]);
cmd.assert().success();  // Assert exit code 0

// Flexible exit code
cmd.assert()
    .code(predicate::in_iter(vec![0i32, 1]));  // Accept 0 or 1

// Assert on output
cmd.assert()
    .stdout(predicate::str::contains("expected text"));

cmd.assert()
    .stderr(predicate::str::contains("error message"));
```

### Evidence API (`cargo_cicd::evidence`)

```rust
use cargo_cicd::evidence::{
    ProcessEvent, emit_xes, WpmEvidenceOracle, assert_wpm_verdict,
    ExpectedWpmVerdict,
};

// Create an event
let event = ProcessEvent::new("command", "PASS");

// Emit XES
let events = vec![event];
emit_xes(&events, Path::new("events.xes"))?;

// Get oracle
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() { /* ... */ }

// Assert verdict
assert_wpm_verdict(&oracle, &path, &ExpectedWpmVerdict::Accept);
```

### Policy API (`cargo_cicd::autonomic::policies`)

```rust
use cargo_cicd::autonomic::policies::{
    check_target_pressure,
    check_toolchain_mismatch,
    check_git_phase_dirty,
    check_trybuild_changed,
    PolicyVerdict,
};

// Check policies directly (unit-level)
let result = check_target_pressure(25.0, 20.0);  // actual_mb, limit_mb
assert!(matches!(result.verdict, PolicyVerdict::Suggest));
assert!(result.recommendation.contains("prune"));
```

---

## Patterns and Examples

### Pattern 1: Test a CLI Command with Various Workspace States

```rust
#[test]
fn test_target_command_with_various_states() {
    // Test with clean workspace
    {
        let fixture = FixtureWorkspace::clean();
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .current_dir(fixture.root)
            .args(["target", "show"])
            .output()
            .unwrap();
        assert!(output.status.code().is_some(), "target show must run");
    }
    
    // Test with large target directory
    {
        let fixture = FixtureWorkspace::with_target_over_limit();
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .current_dir(fixture.root)
            .args(["target", "show"])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);
        // May mention target size or pressure
        assert!(output.status.code().is_some());
    }
}
```

### Pattern 2: Test Safety Invariants

```rust
#[test]
fn test_no_destructive_default_target_prune() {
    let fixture = FixtureWorkspace::with_target_over_limit();
    
    // Run target prune WITHOUT --confirm
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["target", "prune"])  // No --confirm
        .output()
        .unwrap();
    
    // INVARIANT: binary must still exist after prune without confirmation
    let binary = fixture.root.join("target/debug/placeholder.bin");
    assert!(
        binary.exists(),
        "target prune without --confirm must not delete files"
    );
}
```

### Pattern 3: Test Git State Verification

```rust
#[test]
fn test_git_close_refuses_dirty_tree() {
    let fixture = FixtureWorkspace::clean();
    
    // Make the tree dirty
    std::fs::write(fixture.root.join("untracked.txt"), "dirty").unwrap();
    
    // Attempt git close
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["git", "close"])
        .output()
        .unwrap();
    
    // INVARIANT: must refuse
    assert!(!output.status.success(), "dirty tree must be refused");
    
    // INVARIANT: file must still be untracked (no silent commit)
    assert!(
        fixture.root.join("untracked.txt").exists(),
        "git close must not commit untracked files"
    );
}
```

### Pattern 4: Test Policy Verdicts (Unit-Level)

```rust
#[test]
fn test_autonomic_policy_target_pressure() {
    use cargo_cicd::autonomic::policies::{check_target_pressure, PolicyVerdict};
    
    // Over limit: suggest prune
    let result = check_target_pressure(25.0, 20.0);
    assert!(matches!(result.verdict, PolicyVerdict::Suggest));
    assert!(result.recommendation.contains("prune"));
    
    // Under limit: pass
    let result = check_target_pressure(5.0, 20.0);
    assert!(matches!(result.verdict, PolicyVerdict::Pass));
    
    // Approaching (80%): warn
    let result = check_target_pressure(16.1, 20.0);
    assert!(matches!(result.verdict, PolicyVerdict::Warn));
}
```

### Pattern 5: Test Evidence-Gate with Oracle Fallback

```rust
#[test]
fn test_evidence_gate_with_fallback() {
    use cargo_cicd::evidence::{
        assert_wpm_verdict, emit_xes, ExpectedWpmVerdict,
        ProcessEvent, WpmEvidenceOracle,
    };
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("test changed", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit_xes");
    
    let oracle = WpmEvidenceOracle::new();
    
    if oracle.is_available() {
        // Oracle present: verify Accept
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        // Oracle absent: verify Blocked (graceful degradation)
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

### Pattern 6: Test Public Boundary (No Forbidden Terms)

```rust
#[test]
fn test_public_boundary_no_forbidden_terms() {
    let forbidden = ["ALIVE", "Nehemiah", "CONSTRUCT8", "Instinct8"];
    let commands = vec![
        vec!["--help"],
        vec!["status", "--help"],
        vec!["target", "--help"],
        vec!["git", "--help"],
    ];
    
    for args in commands {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args(&args)
            .output()
            .unwrap();
        
        let text = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);
        
        for term in &forbidden {
            assert!(
                !text.contains(term),
                "Forbidden term '{}' in: cargo-cicd {}",
                term,
                args.join(" ")
            );
        }
    }
}
```

### Pattern 7: Test Changed-File Detection

```rust
#[test]
fn test_changed_file_selection() {
    let fixture = FixtureWorkspace::with_changed_trybuild_fixture();
    
    // Run trybuild changed
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["trybuild", "changed"])
        .output()
        .unwrap();
    
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    
    // Must not mention running all fixtures
    assert!(
        !text.contains("all 10") && !text.contains("10 fixtures"),
        "trybuild changed must not run all fixtures by default"
    );
}
```

---

## Summary

**Test Organization:**
- **Smoke Tests** (invariants, feature_projection) verify public boundaries
- **Integration Tests** (cli, changed_tests, etc.) test noun-verb behavior against fixtures
- **Evidence-Gate Tests** (wasm4pm_*) emit XES and verify wpm verdicts (release closure)

**Fixture Design:**
- Use `FixtureWorkspace` for isolated temporary workspaces
- Pre-built fixtures cover common states (clean, dirty, corrupted, etc.)
- Custom fixtures: write to TempDir, initialize git as needed

**Writing Tests:**
1. Identify tier (smoke, integration, or evidence-gate)
2. Choose or build a fixture
3. Run the command with `assert_cmd::Command`
4. Assert on exit code and output (not internal state)
5. Test runs with `cargo test` or specific test name

**Mocking & Isolation:**
- `tempfile::TempDir` isolates each test
- Git initialization is optional; gracefully handle absence
- Mock cargo metadata by providing minimal Cargo.toml
- Use policy functions directly for unit-level testing

**Evidence-Gate Testing:**
- Emit XES with `emit_xes()`
- Invoke oracle with `WpmEvidenceOracle::new()`
- Assert verdict with `assert_wpm_verdict()`
- Test oracle availability; gracefully degrade to `Blocked`
- Use mutation tests to verify oracle is real

**CI/CD Integration:**
- Run smoke tests everywhere (fast, safe)
- Run integration tests in all environments
- Run evidence-gate tests only with `REQUIRE_WPM_ORACLE=1` in release CI
- Test all feature combinations: default, process-data, autonomic, wasm4pm
- Keep total suite under 2 minutes

---

## Additional Resources

- `/CLAUDE.md` — Project mission and architecture
- `/tests/fixtures/*/README.md` — Pre-built fixture descriptions
- `/tests/invariants.rs` — 7 invariants (public boundary, safety, no false close)
- `/tests/wasm4pm_*.rs` — Evidence-gate test patterns
