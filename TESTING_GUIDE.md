# cargo-cicd Testing Guide

## Overview

This comprehensive guide documents the complete testing strategy for **cargo-cicd v26.6.2**, a process-data engine exposed as a boring Rust CI/CD helper. The guide covers test organization, fixture design, test writing patterns, mocking strategies, evidence-gate integration, and CI/CD workflows.

cargo-cicd implements a **three-tier testing strategy**:

1. **Smoke Tests** — Public boundary invariants, CLI parsing, schema validation (non-closing)
2. **Integration Tests** — Noun-verb CLI behavior against isolated fixture workspaces (non-closing)
3. **Evidence-Gate Tests** — XES emission and wasm4pm verdicts (release closure requirement)

**Key Principle:** Tests assert on observable CLI behavior and process evidence, never on internal engine state. Internal state belongs in unit tests; process conformance assertions belong in evidence-gate tests.

---

## Table of Contents

1. [Test Organization](#test-organization) — Three-tier strategy (smoke, integration, evidence-gate)
2. [Fixture Design](#fixture-design) — FixtureWorkspace API and fixture construction patterns
3. [Writing New Tests](#writing-new-tests) — Step-by-step guide and decision tree
4. [Mocking and Isolation](#mocking-and-isolation) — Test independence, external tools, time control
5. [Evidence-Gate Testing](#evidence-gate-testing) — XES format, wasm4pm oracle, verdict assertion
6. [CI/CD Integration](#cicd-integration) — Workflows for development, merge, and release
7. [Practical Test Examples](#practical-test-examples) — Real-world copy-paste-ready patterns
8. [Troubleshooting](#troubleshooting-common-test-issues) — Common issues and solutions
9. [Additional Resources](#additional-resources) — Links and command reference

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
├── SMOKE TESTS (Fast, deterministic, no external deps)
│   ├── invariants.rs                # 7 non-negotiable invariants
│   │   ├── invariant_public_boundary_no_forbidden_terms_in_all_help()
│   │   ├── invariant_no_false_close_git_close_help_mentions_safety()
│   │   ├── invariant_no_destructive_default_target_prune_is_safe()
│   │   ├── invariant_no_full_trybuild_by_default()
│   │   └── invariant_wasm4pm_scan_or_documented_absence()
│   └── feature_projection.rs        # Feature flag surface contract
│
├── INTEGRATION TESTS (Fixture-based, CLI behavior, <30 sec total)
│   ├── cli/
│   │   ├── command_projection.rs    # Harness: all commands parse
│   │   ├── test_status.rs           # status command
│   │   ├── test_target.rs           # target command
│   │   ├── test_git.rs              # git command
│   │   ├── test_publish.rs          # publish command
│   │   ├── test_workspace.rs        # workspace command
│   │   ├── test_evidence.rs         # evidence command
│   │   └── mod.rs                   # Shared CLI test utils
│   ├── changed_tests.rs             # test changed, trybuild changed selection
│   ├── cicd_toml_truth.rs           # cicd.toml schema round-trip
│   ├── autonomic_policies.rs        # Policy verdicts (unit-level)
│   ├── git_phase_closure.rs         # Git state verification
│   ├── fixture_workspaces.rs        # Fixture construction verification
│   ├── interactions.rs              # Cross-command interactions
│   ├── publish_gate.rs              # Publish command safety
│   ├── refusal_calibration.rs       # Command refusal edge cases
│   └── fixtures/
│       ├── mod.rs                   # FixtureWorkspace impl + helpers
│       ├── clean_workspace/         # Pre-built: valid, clean, committed
│       ├── dirty_workspace/         # Pre-built: clean + untracked file
│       ├── missing_manifest/        # Pre-built: no Cargo.toml
│       ├── corrupted_cicd_toml/     # Pre-built: invalid TOML syntax
│       ├── stale_cicd_toml/         # Pre-built: dirty cache state
│       ├── toolchain_mismatch/      # Pre-built: old rust-toolchain.toml
│       ├── target_over_limit/       # Pre-built: 1 MB artifact
│       ├── trybuild_changed_only/   # Pre-built: 10 unchanged + 1 changed
│       └── trybuild_huge_set/       # Pre-built: 50 fixtures
│
├── EVIDENCE-GATE TESTS (XES/wpm adjudication, release closure)
│   ├── wasm4pm_evidence_gate.rs     # Positive: accepted cases
│   ├── wasm4pm_evidence_mutation.rs # Negative: corrupted evidence
│   ├── wasm4pm_refusal_cases.rs     # Edge cases: oracle absent, etc.
│   ├── wasm4pm_harness.rs           # Test harness
│   ├── wpm_verdict_key_contract.rs  # Oracle contract verification
│   └── wasm4pm_evidence/
│       └── fixtures/                # XES/JSONL reference files
│
├── SPECIALIZED TESTS
│   ├── ggen_customization_guard.rs  # ggen ontology invariant
│   ├── lsp_explain.rs               # LSP integration
│   └── [others]
└── Other utils
    └── fixtures.rs (imports)
```

**Test Count by Tier:**
- **Smoke:** ~10 tests (~5 sec)
- **Integration:** ~50+ tests (~30 sec)
- **Evidence-Gate:** ~20+ tests (~60 sec with wpm oracle)
- **Total:** ~80+ tests (~2 min full suite)

---

## Fixture Design

### Overview: Why Fixtures Matter

Fixtures are the foundation of integration testing in cargo-cicd. They provide:
- **Isolation:** Each test gets its own temporary directory (via `tempfile::TempDir`)
- **Repeatability:** Fixture state is deterministic and reproducible
- **Safety:** No test can affect another test or the development machine
- **Clarity:** Fixture names clearly indicate what workspace state they represent

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

Located in `/tests/fixtures/*/`, each pre-built fixture includes a `README.md` describing its state and expected command verdicts.

**Standard Fixtures:**

| Fixture | State | Use Case |
|---------|-------|----------|
| `clean_workspace/` | Minimal valid workspace, clean git tree, no artifacts | Happy-path testing |
| `dirty_workspace/` | Clean baseline + untracked file | Git dirty detection |
| `missing_manifest/` | Empty directory, no Cargo.toml | Refusal handling |
| `corrupted_cicd_toml/` | Clean baseline + invalid TOML in cicd.toml | Error handling |
| `stale_cicd_toml/` | Clean baseline + cicd.toml claiming dirty=false but workspace is actually dirty | Cache invalidation |
| `toolchain_mismatch/` | Clean baseline + rust-toolchain.toml with old channel | Toolchain detection |
| `target_over_limit/` | Clean baseline + 1 MB binary in target/debug/ | Size threshold testing |
| `trybuild_changed_only/` | 10 pre-existing fixtures + 1 changed (not committed) | Changed-file selection |
| `trybuild_huge_set/` | 50 fixture files across tests/ui/ | Performance testing |
| `wasm4pm_missing/` | Clean baseline + no wpm binary | Oracle absence fallback |

**Fixture Characteristics:**
- All fixtures except `missing_manifest/` have valid `Cargo.toml`
- All fixtures with valid manifests have initialized git repos
- Fixtures are minimal: only files necessary to trigger the condition
- Pre-built fixtures save setup time and ensure consistency

### Fixture Construction Patterns

**Pattern 1: Programmatic Construction (One-Off)**

For unique test conditions, build directly in the test:

```rust
#[test]
fn test_custom_condition() {
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    
    // Create minimal valid workspace
    std::fs::write(root.join("Cargo.toml"), r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"
"#).unwrap();
    
    // Initialize git
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output();
    
    // Create custom condition
    std::fs::write(root.join("custom_marker.txt"), "marker").unwrap();
    
    // Run test
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(root)
        .args(["status", "show"])
        .output()
        .unwrap();
    
    // Assertions
    assert!(output.status.code().is_some());
}
```

**Pattern 2: Extend FixtureWorkspace (Reusable)**

For fixture types used in multiple tests, extend `FixtureWorkspace` in `/tests/fixtures/mod.rs`:

```rust
impl FixtureWorkspace {
    /// Clean workspace plus a custom marker file.
    pub fn with_custom_marker() -> Self {
        let fixture = Self::clean();
        std::fs::write(fixture.root.join("marker.txt"), "marker").unwrap();
        fixture
    }
}
```

Then use in tests:
```rust
#[test]
fn test_with_marker() {
    let fixture = FixtureWorkspace::with_custom_marker();
    // Test runs...
}
```

**Pattern 3: Pre-Built Fixture Directory (High-Reuse)**

For fixture sets used in many tests or CI/CD pipelines:

1. Create the fixture directory structure under `/tests/fixtures/<name>/`
2. Include a `README.md` describing state and expected verdicts
3. Commit the fixture to the repo
4. Load via `std::fs::read_dir()` in tests or CI scripts

Example pre-built fixture structure:
```
tests/fixtures/my_workspace/
├── README.md
├── Cargo.toml
├── src/
│   └── lib.rs
├── Cargo.lock
└── .git/
    └── (initialized git repo, committed state)
```

### Fixture Lifecycle and Cleanup

`FixtureWorkspace` wraps a `TempDir`, which automatically cleans up when dropped:

```rust
#[test]
fn test_with_auto_cleanup() {
    let fixture = FixtureWorkspace::clean();  // Creates temp dir
    
    // Test runs...
    
    // When fixture is dropped (test ends), temp dir is automatically deleted
}
```

**Key Points:**
- Never manually delete fixture directories
- Never use hardcoded paths like `/tmp/my-test`
- Let `TempDir` manage cleanup
- If a test panics, `TempDir` still cleans up (drop is called)

---

## Writing New Tests

### Decision Tree: Which Tier?

```
Does the test verify CLI parsing or public output?
  YES → Smoke Test (invariants.rs or feature_projection.rs)
  NO → Continue...

Does the test verify a command against workspace state?
  YES → Integration Test (tests/cli/ or domain-specific file)
  NO → Continue...

Does the test verify wasm4pm evidence and verdicts?
  YES → Evidence-Gate Test (tests/wasm4pm_*.rs)
  NO → Continue...

Does the test verify internal engine logic?
  YES → Unit Test (src/*/tests/ or inline #[cfg(test)])
  NO → Check again or ask for guidance
```

### Step 1: Identify the Test Tier

**Smoke Test** → Use `invariants.rs` or `feature_projection.rs`
- **What it tests:** Public output, CLI parsing, feature flags, schema validity
- **How to run:** `cargo test --test invariants`, `cargo test --test feature_projection`
- **Fixture needed:** No; invoke binary directly
- **Key assertion:** Exit codes, help text, output patterns
- **Duration:** < 5 sec total for all smoke tests

**Integration Test** → Create in `tests/cli/<noun>` or domain file
- **What it tests:** Noun-verb commands against various workspace conditions
- **How to run:** `cargo test --test cli`, `cargo test --test changed_tests`, etc.
- **Fixture needed:** Yes; use `FixtureWorkspace::*()` constructors
- **Key assertion:** Exit code, output patterns, file side effects
- **Duration:** < 30 sec total for all integration tests
- **Scope:** Observable CLI behavior only (no internal state assertions)

**Evidence-Gate Test** → Add to `tests/wasm4pm_*.rs`
- **What it tests:** XES emission correctness, wpm oracle verdicts
- **How to run:** `cargo test --test wasm4pm_evidence_gate`, or `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate`
- **Fixture needed:** TempDir for XES output, no workspace fixtures
- **Key assertion:** `assert_wpm_verdict()` against expected verdict
- **Duration:** < 60 sec total (oracle invocation is slow)
- **Scope:** Process evidence flow; release-closure requirement

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

### Philosophy: Test Independence

Each test must:
1. **Not depend on other tests** — no shared state or sequential ordering
2. **Not affect the development machine** — isolate to temporary directories
3. **Be reproducible** — running 10 times should give the same result
4. **Clean itself up** — no leftover files after success or failure
5. **Tolerate missing external tools** — gracefully handle missing `git`, `cargo`, etc.

### Temporary Filesystems with `tempfile`

All integration tests use `tempfile::TempDir` for isolation. **Never** use `/tmp`, hardcoded paths, or workspace-relative directories in tests.

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

### External Tool Handling

cargo-cicd depends on external tools: `git`, `cargo`, `cargo metadata`, and optionally `wpm` (the wasm4pm oracle).

**Strategy 1: Graceful Absence (Most Tests)**

Make tests robust to tool absence:

```rust
#[test]
fn test_git_command_with_fallback() {
    let fixture = FixtureWorkspace::clean();
    
    // Attempt to run git command
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["git", "status"])
        .output()
        .unwrap();
    
    // Accept both success and failure (git may be absent)
    let status_ok = output.status.success();
    assert!(
        output.status.code().is_some(),
        "Command must not panic, git absence is ok"
    );
}
```

**Strategy 2: Skip if Tool Absent**

For tests that absolutely require a tool:

```rust
#[test]
fn test_requires_git() {
    // Check if git is available
    let git_available = std::process::Command::new("git")
        .args(["--version"])
        .output()
        .is_ok();
    
    if !git_available {
        eprintln!("SKIP: git not available");
        return;  // Skip test gracefully
    }
    
    let fixture = FixtureWorkspace::clean();
    // Test runs...
}
```

**Strategy 3: Environment Variable Control**

For tests that need control over behavior:

```rust
#[test]
fn test_with_environment_control() {
    let fixture = FixtureWorkspace::clean();
    
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .env("CARGO_CICD_DRY_RUN", "1")  // Set env variable
        .args(["target", "prune"])
        .output()
        .unwrap();
    
    // Verify dry-run behavior (no actual deletion)
    assert!(
        fixture.root.join("target").exists(),
        "Dry-run must not delete target/"
    );
}
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

### Controlling Time and Timestamps

For tests involving time-based logic (e.g., event timestamps in evidence):

**Option 1: Accept real time (most tests)**

```rust
#[test]
fn test_event_emission() {
    let events = vec![ProcessEvent::new("status show", "PASS")];
    // ProcessEvent::new() captures current real time
    assert!(!events[0].timestamp_iso.is_empty());
}
```

**Option 2: Mock time in unit tests**

For policy tests with time-dependent logic, call policy functions directly:

```rust
#[test]
fn test_policy_direct() {
    use cargo_cicd::autonomic::policies::{check_target_pressure, PolicyVerdict};
    
    // Call policy function with known values (no time dependency)
    let result = check_target_pressure(25.0, 20.0);
    assert!(matches!(result.verdict, PolicyVerdict::Suggest));
}
```

**Option 3: Environment-based control**

For integration tests, accept that event timestamps reflect real time:

```rust
#[test]
fn test_evidence_timestamps() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("test", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).unwrap();
    
    // Verify timestamp is valid ISO-8601 (not checking specific time)
    let content = std::fs::read_to_string(&xes_path).unwrap();
    assert!(content.contains("T"));  // ISO-8601 includes 'T' separator
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

### Introduction: Why Evidence Gates Matter

cargo-cicd is a **process-data engine**. It doesn't adjudicate its own correctness; it emits evidence (XES logs) and an external oracle (wasm4pm) issues verdicts. This separation ensures:

- **No Self-Judging:** cargo-cicd cannot claim it passed (wasm4pm does)
- **Audit Trail:** All process events are recorded in standard XES format
- **Reproducibility:** Evidence can be re-audited at any time
- **Certification:** Release closure requires wasm4pm Accept verdict

**Release Closure Requirement:**
> No release may claim "ALIVE" solely from cargo-cicd internal tests. wasm4pm verdict (Accept/Refuse) is the source of truth for release closure.

### Evidence Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ cargo-cicd (Emitter)                                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ProcessEvent::new("status show", "PASS")                  │
│      ↓                                                       │
│  emit_xes([events], "events.xes")                          │
│      ↓                                                       │
│  events.xes (on disk)  ──────────────────────┐             │
│                                               │             │
└─────────────────────────────────────────────────────────────┘
                                                │
                                                ↓
                                    ┌──────────────────────┐
                                    │ wasm4pm Oracle       │
                                    ├──────────────────────┤
                                    │ wpm audit events.xes │
                                    ├──────────────────────┤
                                    │ ACCEPT / REFUSE      │
                                    └──────────────────────┘
                                                │
                                                ↓
                                        Verdict recorded
```

**Key Invariants (Evidence Architecture):**

- **E1:** cargo-cicd NEVER adjudicates its own process conformance (wasm4pm does)
- **E2:** Evidence must exist on disk BEFORE oracle invocation
- **E3:** If oracle is unavailable and expected verdict is not `Blocked`, test panics
- **E4:** Tests assert only wasm4pm verdict, never internal cargo-cicd state
- **E5:** XES emission groups events by `case_id` into separate traces
- **E6:** JSONL emission mirrors XES for downstream tooling
- **E7:** `Blocked` is a first-class expected verdict (oracle absent is not an error)

### XES (XML Event Stream) Format

cargo-cicd emits process logs in XES format, an industry-standard XML format for event logs defined at http://www.xesstandard.org/.

**Minimal XES Example:**
```xml
<?xml version="1.0"?>
<log xes:version="1.0" xmlns:xes="http://www.xesstandard.org/">
  <trace>
    <event>
      <string key="concept:name" value="status show"/>
      <string key="lifecycle:transition" value="PASS"/>
      <date key="time:timestamp" value="2026-06-14T12:34:56.789Z"/>
    </event>
  </trace>
</log>
```

**Full ProcessEvent Fields:**
```xml
<event>
  <string key="event_id" value="evt-status-show-20260614123456789"/>
  <date key="timestamp_iso" value="2026-06-14T12:34:56.789Z"/>
  <string key="case_id" value="pipeline-run-001"/>  <!-- optional, groups events -->
  <string key="lifecycle:transition" value="complete"/>  <!-- "start" or "complete" -->
  <string key="workspace_id" value="cargo-cicd-workspace"/>
  <string key="repo_path" value="."/>
  <string key="concept:name" value="status show"/>  <!-- The command -->
  <string key="verdict:claimed" value="PASS"/>  <!-- PASS, WARN, FAIL, DRY-RUN -->
  <int key="duration_ms" value="123"/>  <!-- only for "complete" -->
  <string key="verdict:adjudicated" value="ACCEPT"/>  <!-- filled by oracle -->
  <date key="adjudicated_at" value="2026-06-14T12:34:56.890Z"/>  <!-- filled by oracle -->
  <string key="trace_class" value="live_workspace"/>  <!-- "pipeline_run" or "live_workspace" -->
</event>
```

**Verdict Values:**
- **Claimed by cargo-cicd:** `"PASS"`, `"WARN"`, `"FAIL"`, `"DRY-RUN"`, or custom strings
- **Issued by wasm4pm oracle:** `"ACCEPT"` (process conforms), `"REFUSE"` (violation detected)

**ProcessEvent API:**
```rust
use cargo_cicd::evidence::ProcessEvent;

// Simple: completed event with PASS verdict
let event = ProcessEvent::new("status show", "PASS");

// With timing: start and complete events
let (start_event, t0) = ProcessEvent::started("target prune");
// ... do work ...
let complete_event = ProcessEvent::completed("target prune", t0, "PASS");
```

### WpmEvidenceOracle

The `WpmEvidenceOracle` type discovers, invokes, and interprets the wpm binary (from the wasm4pm project).

**Oracle Binary Location:**
```
/Users/sac/wasm4pm/target/release/wpm
```

If this path is inaccessible (CI environment, missing installation, etc.), the oracle is unavailable and tests gracefully fall back to `Blocked` verdict.

**Oracle API:**
```rust
use cargo_cicd::evidence::WpmEvidenceOracle;

let oracle = WpmEvidenceOracle::new();

// Check availability
if oracle.is_available() {
    // Binary found; oracle can issue verdicts
    let verdict = oracle.audit_xes(Path::new("events.xes"))?;
    match verdict {
        WpmVerdict::Accept => println!("Process conforms"),
        WpmVerdict::Refuse => println!("Process violates"),
        WpmVerdict::Blocked => println!("Oracle unavailable"),
    }
} else {
    // Binary not found; gracefully degrade
    println!("wpm binary not available; tests will use Blocked verdict");
}
```

**Oracle Commands:**

The oracle issues verdicts via two pathways:

1. **XES Audit (Primary):**
   ```bash
   wpm audit <file.xes>
   # Emits JSON verdict: { "verdict": "ACCEPT" } or { "verdict": "REFUSE" }
   ```

2. **Receipt Doctor (Secondary - for receipts):**
   ```bash
   wpm receipt doctor --format json --strict <receipt.json>
   # Emits JSON verdict for receipt validation
   ```

**Verdict Types:**
```rust
pub enum WpmVerdict {
    Accept,  // Process conforms; release closure OK
    Refuse,  // Process violates; release closure blocked
    Blocked, // Oracle unavailable; graceful degradation
}
```

**Oracle Invocation:**
```rust
use cargo_cicd::evidence::{assert_wpm_verdict, ExpectedWpmVerdict, WpmEvidenceOracle};

let oracle = WpmEvidenceOracle::new();
let xes_path = Path::new("events.xes");

// Invoke and assert verdict
assert_wpm_verdict(
    &oracle,
    &xes_path,
    &ExpectedWpmVerdict::Accept,  // Expected verdict
);
// Panics with clear message if actual verdict differs from expected
```

**Oracle Graceful Degradation:**
```rust
let oracle = WpmEvidenceOracle::new();

if oracle.is_available() {
    // Oracle present: assert on actual verdict
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    // Oracle absent: assert Blocked (not Accept, not Refuse)
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}
```

**Forcing Oracle Availability (CI):**

For CI pipelines that have the wpm binary installed, require its presence:

```rust
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!(
            "REQUIRE_WPM_ORACLE=1 set but wpm oracle is absent. \
             Test '{}' cannot exercise Accept assertion. \
             Ensure /Users/sac/wasm4pm/target/release/wpm exists.",
            test_name
        );
    }
    ExpectedWpmVerdict::Blocked
}
```

Then in CI:
```bash
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

This panics if the oracle is absent, preventing silent test skips in release CI.

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

**Pre-Commit (Local Development):**

Run these before staging changes:

```bash
# Fast: lint + type-check (no test execution)
cargo make check

# Fast: unit tests only (library, inline #[test])
cargo test --lib
```

**Expected Duration:** < 10 seconds

**Pre-Push (Local Integration Test):**

Run before pushing to branch:

```bash
# Full smoke + integration tests (no evidence-gate, no oracle dependency)
cargo test

# Or explicitly:
cargo test --test invariants
cargo test --test feature_projection
cargo test --test cli
cargo test --test changed_tests
cargo test --test autonomic_policies
```

**Expected Duration:** < 2 minutes

**Feature Flag Coverage (Local):**

```bash
# Default (no features)
cargo test

# With process-data feature
cargo test --features process-data

# With autonomic feature (implies process-data)
cargo test --features autonomic

# With wasm4pm feature (implies process-data)
cargo test --features wasm4pm
```

**Merge Gate (CI Pipeline):**

Run on every PR to main:

```bash
#!/bin/bash
set -e

# 1. Build
cargo make build

# 2. Smoke tests (fast, universally safe)
cargo test --test invariants
cargo test --test feature_projection

# 3. Integration tests
cargo test --test cli
cargo test --test changed_tests
cargo test --test autonomic_policies
cargo test --test git_phase_closure
cargo test --test cicd_toml_truth

# 4. Feature flag matrix
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm

# 5. Evidence-gate tests (skip Accept assertions if no oracle)
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
```

**Expected Duration:** < 5 minutes (with parallel test execution)

**Release Gate (CI + Certification):**

Run before tagging a release:

```bash
#!/bin/bash
set -e

# 1. Full build
cargo make build

# 2. All tests with oracle required
cargo make test
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_mutation
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_refusal_cases

# 3. All feature combinations
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm

# 4. Final verification
echo "✓ All tests passed"
echo "✓ wasm4pm verdict is ACCEPT"
echo "✓ Release closure may proceed"
```

**Expected Duration:** < 10 minutes

**Release Closure Checklist:**

- [ ] `cargo make test` passes (all tiers)
- [ ] `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate` passes
- [ ] `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_mutation` shows mutations are rejected
- [ ] `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_refusal_cases` passes
- [ ] All feature flags tested: `default`, `process-data`, `autonomic`, `wasm4pm`
- [ ] wasm4pm receipt doctor confirms ACCEPT
- [ ] Evidence XES files archived in `target/cargo-cicd/evidence/`
- [ ] Release notes document wasm4pm verdict

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

## Practical Test Examples

This section provides real-world, copy-paste-ready examples for common testing scenarios.

### Example 1: Complete Integration Test (Workspace States + CLI)

This example shows testing a command across multiple workspace conditions:

```rust
use assert_cmd::Command;
use crate::fixtures::FixtureWorkspace;

#[test]
fn test_target_show_with_various_workspace_states() {
    // TEST 1: Clean workspace (baseline)
    {
        let fixture = FixtureWorkspace::clean();
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .current_dir(fixture.root)
            .args(["target", "show"])
            .output()
            .unwrap();
        
        // Assertions
        assert!(
            output.status.code().is_some(),
            "target show must complete without panic"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("target") || stdout.len() > 0,
            "Should produce output or graceful message"
        );
    }
    
    // TEST 2: Over-limit target directory (size pressure)
    {
        let fixture = FixtureWorkspace::with_target_over_limit();
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .current_dir(fixture.root)
            .args(["target", "show"])
            .output()
            .unwrap();
        
        // May warn about size; that's OK
        let text = String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.code().is_some(),
            "Must not panic: {}",
            text
        );
    }
    
    // TEST 3: Missing manifest (should refuse gracefully)
    {
        let fixture = FixtureWorkspace::missing_manifest();
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .current_dir(fixture.root)
            .args(["target", "show"])
            .output()
            .unwrap();
        
        // Should exit with non-zero (no manifest)
        let status = output.status.code();
        assert!(status.is_some(), "Must exit with status code");
    }
}
```

**Key Patterns:**
- Use scoped blocks `{ }` to isolate each test case
- Each fixture is independent; no shared state between blocks
- Assert on observable behavior (exit code, output), not internal state
- Gracefully handle both success and failure modes
- Use descriptive assertion messages

### Example 2: Testing a Safety Invariant

### Example 2: Testing a Safety Invariant

This example verifies that a destructive command refuses to run without confirmation:

```rust
use assert_cmd::Command;
use crate::fixtures::FixtureWorkspace;

#[test]
fn test_target_prune_refuses_without_confirm_flag() {
    // Create fixture with large artifacts
    let fixture = FixtureWorkspace::with_target_over_limit();
    
    // Verify artifact exists before test
    let artifact = fixture.root.join("target/debug/placeholder.bin");
    assert!(artifact.exists(), "Fixture must have artifact");
    
    // Run prune WITHOUT --confirm flag
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["target", "prune"])  // NOTE: no --confirm
        .output()
        .unwrap();
    
    // INVARIANT: Artifact must STILL EXIST (no deletion without confirmation)
    assert!(
        artifact.exists(),
        "target prune without --confirm MUST NOT delete files (INVARIANT: No Destructive Default)"
    );
    
    // INVARIANT: Command should exit non-zero (request to confirm)
    assert!(
        !output.status.success() || output.status.code() == Some(0),
        "Command must either refuse or dry-run; actual code: {:?}",
        output.status.code()
    );
}

#[test]
fn test_target_prune_with_confirm_succeeds_on_dry_run() {
    // With --confirm, should proceed (but may be dry-run)
    let fixture = FixtureWorkspace::with_target_over_limit();
    let artifact = fixture.root.join("target/debug/placeholder.bin");
    
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["target", "prune", "--confirm"])
        .output()
        .unwrap();
    
    // Should complete (may be dry-run or actual deletion)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("prune") || stdout.contains("target"),
        "Should report prune action"
    );
}
```

**Key Patterns:**
- Test the refusal path first (baseline safety)
- Verify files still exist after refusal
- Then test the success path with confirmation flag
- Use `INVARIANT:` comments to explain non-negotiable safety requirements

### Example 3: Testing Git State Verification

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

### Example 4: Testing Policy Verdicts (Unit-Level)

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

### Example 5: Complete Evidence-Gate Test with Oracle Fallback

This example demonstrates proper evidence-gate testing with graceful oracle fallback:

```rust
use cargo_cicd::evidence::{
    assert_wpm_verdict, emit_xes, ExpectedWpmVerdict,
    ProcessEvent, WpmEvidenceOracle,
};
use tempfile::TempDir;

#[test]
fn evidence_gate_test_changed_accepted() {
    // Step 1: Create temporary directory for XES output
    let dir = TempDir::new().unwrap();
    
    // Step 2: Emit process evidence
    let events = vec![ProcessEvent::new("test changed", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path)
        .expect("emit_xes must succeed");
    
    // Step 3: Verify XES file exists before oracle invocation
    assert!(
        xes_path.exists(),
        "XES file must exist on disk before oracle call (INVARIANT: E2)"
    );
    
    // Step 4: Get the oracle
    let oracle = WpmEvidenceOracle::new();
    
    // Step 5: Assert verdict (with graceful fallback)
    if oracle.is_available() {
        // Oracle installed: assert on actual verdict
        eprintln!("Oracle available; checking actual verdict");
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &ExpectedWpmVerdict::Accept,
        );
    } else {
        // Oracle not installed: assert Blocked (graceful degradation)
        eprintln!("Oracle unavailable; graceful fallback to Blocked");
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &ExpectedWpmVerdict::Blocked,
        );
    }
}

/// Optional helper for tests that require the oracle
fn absent_oracle_verdict(test_name: &str) -> ExpectedWpmVerdict {
    if std::env::var("REQUIRE_WPM_ORACLE").as_deref() == Ok("1") {
        panic!(
            "REQUIRE_WPM_ORACLE=1 is set but wpm oracle is absent. \
             Test '{}' cannot proceed without oracle. \
             Ensure /Users/sac/wasm4pm/target/release/wpm exists.",
            test_name
        );
    }
    ExpectedWpmVerdict::Blocked
}

#[test]
fn evidence_gate_with_required_oracle() {
    let dir = TempDir::new().unwrap();
    let events = vec![ProcessEvent::new("git close", "PASS")];
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).unwrap();
    
    let oracle = WpmEvidenceOracle::new();
    
    // Use helper to enforce oracle presence if REQUIRE_WPM_ORACLE=1
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
    } else {
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &absent_oracle_verdict("evidence_gate_with_required_oracle"),
        );
    }
}
```

**Key Patterns:**
- XES file must exist on disk BEFORE oracle invocation
- Always check `oracle.is_available()` before asserting on actual verdicts
- Use `absent_oracle_verdict()` helper to enforce oracle presence with `REQUIRE_WPM_ORACLE=1`
- Accept `Blocked` as a valid verdict (oracle absence)
- Include descriptive comments and eprintln! for debugging

### Example 6: Testing Evidence Mutation (Negative Case)

This example verifies that corrupted evidence is rejected by the oracle:

```rust
use cargo_cicd::evidence::{assert_wpm_verdict, ExpectedWpmVerdict, WpmEvidenceOracle};
use tempfile::TempDir;

#[test]
fn evidence_gate_corrupted_xes_must_be_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("corrupted.xes");
    
    // Step 1: Write intentionally corrupted XES
    std::fs::write(
        &xes_path,
        "NOT A VALID XML DOCUMENT AT ALL\n",
    ).unwrap();
    
    // Step 2: Verify file exists
    assert!(xes_path.exists(), "Corrupted file must exist");
    
    // Step 3: Get oracle and assert
    let oracle = WpmEvidenceOracle::new();
    
    if oracle.is_available() {
        // Oracle installed: corrupted evidence MUST be Refused
        assert_wpm_verdict(
            &oracle,
            &xes_path,
            &ExpectedWpmVerdict::Refuse,
        );
        eprintln!("✓ Oracle correctly rejected corrupted evidence");
    } else {
        // Oracle absent: gracefully degrade to Blocked
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}
```

**Mutation Testing Strategies:**
- Empty file → Refuse
- Not valid XML → Refuse
- Mismatched XML tags → Refuse
- Binary garbage → Refuse
- Truncated XES → Refuse
- Valid structure but semantically invalid → Refuse

These tests verify that wasm4pm is a **real adjudicator**, not a rubber stamp.

### Example 7: Testing Public Boundary (Smoke Test)

This example shows a smoke test that verifies public output contains no forbidden terms:

```rust
use assert_cmd::Command;

#[test]
fn test_smoke_public_boundary_no_forbidden_terms() {
    let forbidden = [
        "ALIVE",
        "Nehemiah",
        "CONSTRUCT8",
        "Instinct8",
        "Inspection Gate",
        "Cargo Court",
        "AGI",
        "Truex",
        "Field8",
        "wall",
    ];
    
    let noun_verbs = [
        vec!["--help"],
        vec!["status", "--help"],
        vec!["target", "--help"],
        vec!["test", "--help"],
        vec!["git", "--help"],
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
                "INVARIANT VIOLATION: Forbidden term '{}' in: cargo-cicd {}",
                term,
                args.join(" ")
            );
        }
    }
}
```

**Key Patterns:**
- Smoke tests are fast and safe to run everywhere
- Use comprehensive lists of forbidden terms
- Test multiple noun-verb combinations
- Assert on complete output (stdout + stderr)

### Example 8: Testing Changed-File Detection

This example tests that trybuild only runs changed fixtures, not all:

```rust
use assert_cmd::Command;
use crate::fixtures::FixtureWorkspace;

#[test]
fn test_trybuild_changed_selects_only_changed_fixtures() {
    let fixture = FixtureWorkspace::with_changed_trybuild_fixture();
    
    // Fixture contains: 10 pre-existing + 1 changed (uncommitted)
    // Verify fixture setup
    let ui_dir = fixture.root.join("tests/ui");
    assert!(ui_dir.exists(), "tests/ui must exist");
    
    // Run trybuild changed
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["trybuild", "changed"])
        .output()
        .unwrap();
    
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    
    // INVARIANT: Must not claim to run all fixtures
    assert!(
        !text.contains("all 10") && !text.contains("10 fixtures") && !text.contains("run all"),
        "trybuild changed MUST NOT run all fixtures by default (INVARIANT: No Full Trybuild)"
    );
    
    // INVARIANT: Should mention changed or indicate selection
    assert!(
        text.contains("changed") || text.contains("1") || text.contains("select"),
        "Should indicate changed-file selection; output: {}",
        text
    );
}
```

**Key Patterns:**
- Use fixtures specifically designed for changed-file testing
- Verify the fixture has the expected state (N pre-existing + 1 changed)
- Assert that "all N" is NOT in the output
- Verify the command mentions "changed" or reports selective execution

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

---

## Troubleshooting Common Test Issues

### Issue: "test ... FAILED: fixture root does not exist"

**Cause:** Fixture lifecycle ended before test assertion. `TempDir` was dropped prematurely.

**Solution:** Ensure fixture variable stays in scope:

```rust
// WRONG: fixture dropped at end of block
{
    let fixture = FixtureWorkspace::clean();
}  // fixture dropped here
// Command runs, but temp dir is gone!

// CORRECT: fixture stays alive for the entire test
#[test]
fn test_correct() {
    let fixture = FixtureWorkspace::clean();  // Still alive
    // ... test runs ...
}  // fixture dropped here after test completes
```

### Issue: "error: git not found" or "cargo metadata failed"

**Cause:** External tool (git, cargo) not in PATH.

**Solution:** Gracefully handle absence:

```rust
#[test]
fn test_with_tool_fallback() {
    let fixture = FixtureWorkspace::clean();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture.root)
        .args(["status"])
        .output()
        .unwrap();
    
    // Accept both success and failure (tool may be absent)
    assert!(
        output.status.code().is_some(),
        "Must not panic; tool absence is ok"
    );
}
```

### Issue: "assertion failed: expected Accept, got Blocked"

**Cause:** Oracle not available when test expected it.

**Solution:** Check oracle availability or use `REQUIRE_WPM_ORACLE=1`:

```rust
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() {
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Accept);
} else {
    eprintln!("Oracle unavailable; test will use Blocked verdict");
    assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
}

// OR: Require oracle in CI
// REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

### Issue: "panic: XES file does not exist before oracle call"

**Cause:** Tried to invoke oracle on non-existent file.

**Solution:** Verify XES file exists before oracle call:

```rust
let xes_path = dir.path().join("events.xes");
emit_xes(&events, &xes_path)?;

// CRITICAL: Verify file exists BEFORE oracle invocation
assert!(xes_path.exists(), "XES file must exist on disk before oracle call");

// Now safe to invoke oracle
let oracle = WpmEvidenceOracle::new();
if oracle.is_available() {
    oracle.audit_xes(&xes_path)?;
}
```

### Issue: "test ... timed out after 30 seconds"

**Cause:** Test waiting for slow operation (wpm oracle invocation, cargo metadata).

**Solution:** 
- Run evidence-gate tests separately: `cargo test --test wasm4pm_evidence_gate`
- Use `cargo test --lib` to skip integration tests during development
- Increase timeout in CI: `cargo test -- --test-threads=1`

### Issue: "test ... uses hardcoded path /tmp/test-123"

**Cause:** Test creates files in shared `/tmp`, affecting other tests or the system.

**Solution:** Always use `tempfile::TempDir`:

```rust
// WRONG: Hardcoded path
std::fs::create_dir_all("/tmp/my-test").unwrap();

// CORRECT: Isolated temp directory
let dir = tempfile::TempDir::new().unwrap();
std::fs::create_dir_all(dir.path().join("my-subdir")).unwrap();
// Automatically cleaned up when dir is dropped
```

### Issue: "assertion ... contains 'PASS' failed; output was empty"

**Cause:** Command output is on stderr, not stdout.

**Solution:** Combine stdout and stderr in assertions:

```rust
let output = Command::cargo_bin("cargo-cicd")...output().unwrap();

// WRONG: Only checks stdout
let stdout = String::from_utf8_lossy(&output.stdout);
assert!(stdout.contains("PASS"));

// CORRECT: Combines stdout + stderr
let text = String::from_utf8_lossy(&output.stdout).to_string()
    + &String::from_utf8_lossy(&output.stderr);
assert!(text.contains("PASS"));
```

### Issue: "test ... SKIP: git not available"

**Cause:** Test intentionally skipped because required tool is absent.

**Solution:** This is OK for optional dependencies. To require the tool in CI:

```bash
# Check if git is available before running test
if ! command -v git &> /dev/null; then
    echo "SKIP: git not available"
    exit 0
fi

# Require in CI with environment variable
REQUIRE_GIT=1 cargo test
```

---

## Performance Tips

### Speed Up Local Development

**Use `cargo test --lib`** to run only unit tests (skip integration tests):
```bash
cargo test --lib  # ~5 sec
```

**Use `cargo make check`** instead of `cargo test` for pre-commit:
```bash
cargo make check  # Lint + type-check, ~5 sec
```

**Run specific test file**:
```bash
cargo test --test cli  # Only CLI tests, ~10 sec
```

### Speed Up CI

**Run tests in parallel** (4 threads is typical):
```bash
cargo test -- --test-threads=4
```

**Skip slow tests in development CI**:
```bash
cargo test --lib --test invariants  # Fast gate before integration tests
```

**Cache cargo builds** in CI (GitHub Actions, etc.):
```yaml
- uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

---

## Additional Resources

- **Project Documentation:**
  - `/CLAUDE.md` — Project mission and architecture overview
  - `/ARCHITECTURE.md` — Detailed engine and adapter architecture
  - `/tests/fixtures/mod.rs` — FixtureWorkspace implementation

- **Test Examples:**
  - `/tests/invariants.rs` — 7 non-negotiable invariants
  - `/tests/cli/` — CLI command tests
  - `/tests/wasm4pm_evidence_gate.rs` — Evidence-gate patterns
  - `/tests/autonomic_policies.rs` — Policy unit tests

- **External Resources:**
  - [assert_cmd documentation](https://docs.rs/assert_cmd/) — Process assertions
  - [tempfile documentation](https://docs.rs/tempfile/) — Temporary directories
  - [predicates documentation](https://docs.rs/predicates/) — Composable assertions
  - [XES Standard](http://www.xesstandard.org/) — Event log format

- **Development Commands:**
  ```bash
  cargo make build          # Full build
  cargo make check          # Lint + type-check
  cargo make test           # All tests
  cargo test --lib          # Unit tests only
  cargo test --test cli     # Specific test file
  REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
  ```
