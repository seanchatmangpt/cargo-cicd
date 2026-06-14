# cargo-cicd Developer Troubleshooting Guide

**For developers working on cargo-cicd v26.6.2**

A comprehensive guide to debugging, testing, development environment setup, and performance profiling for cargo-cicd developers. This guide covers practical techniques for isolating and fixing issues in the Level 5 process-data engine.

---

## Table of Contents

1. [Logging and Tracing](#logging-and-tracing)
2. [Test Debugging](#test-debugging)
3. [Common Issues](#common-issues)
4. [Development Environment Setup](#development-environment-setup)
5. [Performance Profiling](#performance-profiling)
6. [Quick Reference](#quick-reference)

---

## Logging and Tracing

### Debug Prints and Inspection

cargo-cicd uses **direct `eprintln!` and `dbg!`** for development debugging (no structured logging framework yet). This is intentional for v26.6.2 to avoid dependency bloat on a CI/CD tool.

#### Add Debug Output to Adapters

Adapters are the clearest place to add tracing. Each adapter owns one external source and transforms it into internal state:

**Example: Debug a GitStatusAdapter output**

```rust
// File: src/adapters/git_status.rs (or src/adapters/git.rs)

pub fn read_git_state() -> Result<GitState> {
    let branch = {
        let out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
        eprintln!("[DEBUG] git branch: {}", b);  // Add this
        b
    };

    let (dirty, staged, untracked) = {
        let out = Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;
        let mut dirty = Vec::new();
        let mut staged = Vec::new();
        let mut untracked = Vec::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.len() < 3 {
                continue;
            }
            let x = line.chars().next().unwrap_or(' ');
            let y = line.chars().nth(1).unwrap_or(' ');
            let file = line[3..].to_string();
            match (x, y) {
                ('?', '?') => untracked.push(file),
                (' ', _) => dirty.push(file),
                (_, ' ') => staged.push(file),
                _ => {
                    staged.push(file.clone());
                    dirty.push(file);
                }
            }
        }
        eprintln!("[DEBUG] git status: dirty={}, staged={}, untracked={}", 
                  dirty.len(), staged.len(), untracked.len());  // Add this
        (dirty, staged, untracked)
    };

    Ok(GitState {
        branch,
        dirty,
        staged,
        untracked,
        ahead: 0,
        behind: 0,
    })
}
```

### Trace EngineState Mutations

`EngineState` is the aggregate root of all runtime state. To inspect it:

#### Use dbg! Macro

```rust
// In any noun verb implementation that populates state:
let mut state = EngineState::default();

// ... populate state from adapters ...

dbg!(&state);  // Pretty-print entire state

// Or inspect specific dimensions:
eprintln!("WorkspaceState: {:#?}", state.workspace);
eprintln!("GitPhaseState: {:#?}", state.git_phase);
eprintln!("TestPlanState: {:#?}", state.test_plan);
```

#### Inspect Adapter Output Separately

Before feeding adapter output to EngineState, print the raw result:

```rust
// File: src/nouns/status.rs or similar
use crate::adapters::GitStatusAdapter;

fn your_verb_impl() -> Result<()> {
    let git_state = GitStatusAdapter::read_git_state()?;
    eprintln!("Raw GitState from adapter: {:#?}", git_state);
    
    // Now populate state
    engine.git_phase.branch = git_state.branch.clone();
    engine.git_phase.dirty_count = git_state.dirty.len();
    
    eprintln!("After population: {:#?}", engine.git_phase);
    
    Ok(())
}
```

### Environment Variables for Conditional Debugging

Use an env var to control debug output without modifying code every time:

```rust
// In your function:
let debug_adapters = std::env::var("CARGO_CICD_DEBUG_ADAPTERS").is_ok();

if debug_adapters {
    eprintln!("[ADAPTER] git status: dirty={}, staged={}", dirty.len(), staged.len());
}
```

Then run:

```bash
CARGO_CICD_DEBUG_ADAPTERS=1 cargo cicd status
```

### Inspect cicd.toml Mutations

cicd.toml is the carrier file. Check what the `CicdTomlWriter` adapter writes:

```bash
# Before running a command:
cat cicd.toml

# Run a command:
cargo cicd status show

# After running, inspect changes:
diff -u <(git show HEAD:cicd.toml) cicd.toml
# or if not in git:
cat cicd.toml
```

Example `cicd.toml` structure to check:

```toml
[workspace]
root = "/home/user/cargo-cicd"
members = [".", "crates/cargo-cicd-core", "crates/cargo-cicd-lsp"]

[state]
last_status = "clean"
last_test_plan = "conservative"

[[events]]
type = "StatusShowEvent"
timestamp = "2026-06-14T10:30:45Z"
passed = true

# Feature state (when autonomic enabled):
[autonomic]
suggest_mode = true
```

### Debug Verb Dispatch

When debugging noun-verb routing, trace how a command is parsed:

```rust
// Add this to main.rs temporarily to debug verb dispatch:
fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().collect();
    eprintln!("[DEBUG] raw argv: {:?}", raw);
    
    let noun = raw.get(1).map(String::as_str).unwrap_or("").to_string();
    let verb = raw.get(2).map(String::as_str).unwrap_or("").to_string();
    eprintln!("[DEBUG] detected noun='{}', verb='{}'", noun, verb);
    
    // ... rest of main ...
}
```

Then run:

```bash
cargo cicd status show 2>&1 | grep DEBUG
```

### Environment Variable Tracing Patterns

For conditional tracing that can be toggled at runtime:

```rust
// Global patterns used throughout the codebase:

// Pattern 1: Simple existence check
if std::env::var("CARGO_CICD_TRACE").is_ok() {
    eprintln!("[TRACE] entering adapter: {:?}", self);
}

// Pattern 2: Severity levels
let trace_level = std::env::var("CARGO_CICD_TRACE_LEVEL")
    .unwrap_or_else(|_| "info".to_string());
if trace_level == "debug" {
    eprintln!("[DEBUG] detailed state: {:#?}", state);
}

// Pattern 3: Component-specific tracing
if std::env::var("CARGO_CICD_TRACE_ADAPTERS").is_ok() {
    eprintln!("[ADAPTER:{}] executing with input: {:?}", 
              adapter_name, input);
}
```

Usage:

```bash
# Trace everything
CARGO_CICD_TRACE=1 cargo cicd status show 2>&1

# Trace only adapters
CARGO_CICD_TRACE_ADAPTERS=1 cargo cicd status show 2>&1

# Debug level tracing
CARGO_CICD_TRACE_LEVEL=debug cargo cicd status show 2>&1
```

---

## Test Debugging

### Quick Test Commands

**Run all tests:**

```bash
cargo test
```

**Run a single test file:**

```bash
cargo test --test invariants
cargo test --test cli
cargo test --test changed_tests
cargo test --test autonomic_policies
cargo test --test git_phase_closure
cargo test --test feature_projection
cargo test --test wasm4pm_harness
```

**Run a specific test function:**

```bash
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
cargo test --test changed_tests test_trybuild_changed_does_not_mention_all_fixtures
```

**Run with output visible (not captured):**

```bash
cargo test --test changed_tests -- --nocapture
```

**Run with features enabled:**

```bash
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
```

### Fixture-Based Test Patterns

Tests use temporary workspaces in two ways:

#### 1. tempfile::TempDir (Isolated, Ephemeral)

Used for single-test isolation:

```rust
#[test]
fn test_git_state_detection() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    
    // Create minimal Cargo workspace
    std::fs::write(root.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    
    // Initialize git
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();
    
    // Run cargo-cicd in that temp directory
    let output = assert_cmd::Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(root)
        .arg("git")
        .arg("status")
        .output()
        .unwrap();
    
    assert!(output.status.success());
}
```

**Advantages:**
- Completely isolated from other tests
- Cleaned up automatically when `TempDir` is dropped
- No global state pollution

#### 2. tests/fixtures/ (Persistent, Reusable)

Pre-created test workspaces under `tests/fixtures/`:

```
tests/fixtures/
├── clean_workspace/          # Minimal, clean Cargo workspace
├── dirty_workspace/          # Has unstaged changes
├── toolchain_mismatch/       # rust-toolchain.toml mismatch
├── missing_manifest/         # No Cargo.toml
├── corrupted_cicd_toml/      # Invalid TOML
└── wasm4pm_missing/          # wpm binary not found
```

To use a fixture:

```rust
#[test]
fn test_with_fixture() {
    let fixture_path = Path::new("tests/fixtures/clean_workspace");
    
    let output = assert_cmd::Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(fixture_path)
        .arg("status")
        .arg("show")
        .output()
        .unwrap();
    
    assert!(output.status.success());
}
```

**Advantages:**
- Reusable across many tests
- Checked into git (immutable test conditions)
- Faster than creating temp workspaces each time

### Inspecting Test Failures

**When a test fails, capture the output:**

```bash
cargo test --test changed_tests test_trybuild_changed_selects_only_changed_fixture -- --nocapture 2>&1 | tee /tmp/test_output.txt
```

**Look for common patterns in the output:**

1. **"Forbidden term found"** → Invariant test caught a public-API leak. Check CLAUDE.md for forbidden terms.
2. **"no such file or directory"** → Adapter failed to find a resource (git, Cargo.toml, etc.). Check working directory.
3. **"assertion failed: ... bytes freed"** → Target pruning claim doesn't match actual deletion. Inspect adapter logic.
4. **"malformed evidence line"** → Evidence file is corrupted or missing a field. See wasm4pm evidence format.

### Debug Test Fixture State

If a test using `tempfile::TempDir` fails, capture the directory before it's cleaned up:

```rust
#[test]
fn test_something_that_might_fail() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    
    // ... setup and run test ...
    
    if test_failed {
        // Copy the temp dir to a persistent location before drop()
        let debug_dir = Path::new("/tmp/cargo-cicd-test-debug");
        let _ = std::fs::remove_dir_all(debug_dir);
        let _ = copy_dir_recursive(root, debug_dir);
        eprintln!("Debug workspace saved to: {}", debug_dir.display());
    }
    
    // TempDir drops and cleans up here
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let new_path = dst.join(&file_name);
        if path.is_dir() {
            copy_dir_recursive(&path, &new_path)?;
        } else {
            std::fs::copy(&path, &new_path)?;
        }
    }
    Ok(())
}
```

### Run Tests with Timeout

Long-running tests (e.g., workspace scans) can hang. Use a timeout:

```bash
timeout 30s cargo test --test changed_tests -- --nocapture
```

### Feature-Gated Tests

Some tests only run with specific feature flags. Check conditional compilation:

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_autonomic_policies_only() {
    // This only runs if compiled with: cargo test --features autonomic
}

#[cfg(not(feature = "wasm4pm"))]
#[test]
fn test_skipped_when_wasm4pm_enabled() {
    // Opposite: test skipped if wasm4pm feature is on
}
```

To test different feature combinations:

```bash
cargo test                                  # default (no features)
cargo test --features process-data
cargo test --features autonomic             # implies process-data
cargo test --features wasm4pm               # implies process-data
cargo test --all-features
```

---

## Common Issues

### Workspace Detection Failures

**Symptom:** "workspace root not found" or "Cargo.toml not detected"

**Root causes:**

1. **Running outside a workspace** — cargo-cicd expects a `Cargo.toml` in the current directory or parent.

2. **Corrupted or missing `Cargo.toml`** — Parser fails on invalid TOML.

3. **Deep nested call** — cargo-cicd walks up from cwd to find workspace root; if not found, errors.

4. **Workspace root detection logic failure** — CargoMetadataAdapter may fail to identify the root correctly.

**Debug steps:**

```bash
# 1. Check where you are:
pwd

# 2. Check if Cargo.toml exists at cwd or parents:
ls -la Cargo.toml              # Current dir
ls -la ../Cargo.toml           # Parent dir
find . -name "Cargo.toml" -type f | head -5

# 3. Validate Cargo.toml syntax:
cargo metadata  # If this fails, your Cargo.toml is broken
# Expected output: JSON describing the workspace

# 4. Check workspace structure:
grep -A 5 '^\[workspace\]' Cargo.toml
grep -A 10 'members' Cargo.toml

# 5. Trace the adapter:
CARGO_CICD_DEBUG_ADAPTERS=1 cargo cicd status show 2>&1 | head -20
```

**Fix:**

- Ensure you're running from workspace root (where top-level `Cargo.toml` lives).
- Validate TOML syntax: `cargo metadata` should output valid JSON.
- For nested workspaces, verify `[workspace] members = [...]` lists all crates.
- Check that members are relative paths that actually exist.

**Example of correct workspace structure:**

```toml
# Cargo.toml at workspace root
[workspace]
members = [
  ".",
  "crates/cargo-cicd-core",
  "crates/cargo-cicd-lsp",
]
resolver = "2"

[package]
name = "cargo-cicd"
version = "26.6.2"
```

**Example of broken workspace:**

```toml
# Missing members field or wrong paths
[workspace]
resolver = "2"

# Wrong: missing or incorrect members
members = ["nonexistent/path", "./crates/wrong-name"]
```

**Quick validation script:**

```bash
#!/bin/bash
# validate_workspace.sh

echo "Checking workspace structure..."
cargo metadata --format-version 1 > /tmp/metadata.json 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Workspace is valid"
    echo "  Root: $(jq -r '.workspace_root' /tmp/metadata.json)"
    echo "  Members: $(jq '.workspace_members | length' /tmp/metadata.json)"
else
    echo "✗ Workspace has errors"
    cat /tmp/metadata.json
fi
```

### Git State Inconsistencies

**Symptom:** `git_phase_state` doesn't match actual git status; cargo cicd git status reports wrong dirty/staged counts

**Root causes:**

1. **Git state changed between adapter calls** — If you modify a file after `git status` is read, the state becomes stale.

2. **Uninitialized git repo** — Commands like `git rev-list --count` fail on non-repos.

3. **Detached HEAD** — Branch name detection fails; ahead/behind counts undefined.

4. **Upstream branch not set** — `@{upstream}` ref doesn't exist.

**Debug steps:**

```bash
# Check raw git status:
git status --porcelain

# Check branch:
git rev-parse --abbrev-ref HEAD

# Check ahead/behind:
git rev-list --left-right --count HEAD...@{upstream}

# Trace adapter output:
CARGO_CICD_DEBUG_ADAPTERS=1 cargo cicd git status 2>&1 | grep -i "branch\|dirty\|staged"
```

**Fix:**

- Ensure you're in a valid git repository: `git rev-parse --git-dir`
- Set an upstream branch: `git branch --set-upstream-to=origin/main`
- Avoid modifying files between adapter calls (read all state atomically if possible).
- Add error handling for detached HEAD in adapter (currently assumes tracking branch exists).

### Feature Flag Interactions

**Symptom:** Test fails only when `--features wasm4pm` is enabled; autonomic policies don't run

**Root causes:**

1. **Feature implication ordering** — `autonomic` implies `process-data`, but the reverse is not true.

2. **Conditional compilation** — Code gated by `#[cfg(feature = "...")]` doesn't exist without the flag.

3. **Missing feature in transitive deps** — A crate depends on `log`, but log feature is not enabled in workspace.

**Debug steps:**

```bash
# Check which features are enabled:
cargo tree --features autonomic 2>&1 | grep -i "process-data\|autonomic"

# Verify feature gates in Cargo.toml:
grep -A 5 '^\[features\]' Cargo.toml

# Check conditional code:
grep -r '#\[cfg(feature' src/

# Build with specific features:
cargo build --no-default-features --features wasm4pm
cargo build --all-features
```

**Fix:**

- Always test with the feature combination you deploy with.
- Document feature implications in code comments.
- For wasm4pm tests, ensure the feature is enabled and wpm binary is discoverable.

### wasm4pm Evidence Format Errors

**Symptom:** Evidence gate tests fail with "malformed evidence" or "wpm receipt doctor refused"

**Root causes:**

1. **Evidence not emitted** — cargo-cicd runs but doesn't write to `target/cargo-cicd/evidence/events.jsonl`.

2. **JSONL format broken** — Evidence file exists but lines are not valid JSON.

3. **Missing required fields** — Event missing `type`, `timestamp`, or other required fields.

4. **wpm binary not found** — Evidence can't be adjudicated without the oracle.

**Debug steps:**

```bash
# Check if evidence was written:
ls -la target/cargo-cicd/evidence/
cat target/cargo-cicd/evidence/events.jsonl | jq .

# Validate each line is JSON:
while IFS= read -r line; do
  echo "$line" | jq . > /dev/null || echo "INVALID: $line"
done < target/cargo-cicd/evidence/events.jsonl

# Check wpm binary location:
which wpm
echo $WPM_BINARY  # Check if env var is set

# Run receipt doctor manually:
wpm receipt doctor --format json --strict target/cargo-cicd/evidence/events.jsonl
```

**Fix:**

- Ensure the `wasm4pm` feature is enabled when testing evidence gates.
- Check that evidence emission code is reached (use `eprintln!` before write).
- Validate JSONL format: each line must be a complete, valid JSON object.
- Place wpm binary at the discovered path (see [Development Environment Setup](#development-environment-setup)).

### Autonomic Policy Failures

**Symptom:** Suggest mode doesn't emit recommendations; policy rules not evaluated

**Root causes:**

1. **Feature not enabled** — Autonomic mode requires `--features autonomic`.

2. **Policy state not populated** — `PolicyState` struct not filled in by adapters.

3. **Policies in enforce mode instead of suggest** — cicd.toml has `enforce = true` instead of `suggest = true`.

4. **Policy condition not met** — Recommendation rules are conditional; if conditions fail, no output.

**Debug steps:**

```bash
# 1. Check feature is enabled:
cargo build --features autonomic
cargo test --features autonomic

# 2. Check feature implication:
cargo tree --features autonomic | grep -i "process-data"
# Should show: autonomic depends on process-data

# 3. Inspect cicd.toml policy config:
grep -A 5 '^\[autonomic\]' cicd.toml
# Expected:
# [autonomic]
# suggest_mode = true

# 4. Test with explicit feature:
CARGO_CICD_DEBUG_ADAPTERS=1 cargo cicd status show --features autonomic 2>&1

# 5. Verify policy modules exist:
ls -la src/policies/
find src/policies/ -name "*.rs" -type f
```

**Fix:**

- Always build with `--features autonomic` when testing policy behavior.
- Ensure `cicd.toml [autonomic] suggest_mode = true` (default is suggest; enforce is rare).
- Add policy implementations in `src/policies/` for each rule.
- Verify that policy conditions are met (e.g., "warn if dirty" requires actual dirty state).

**Testing policies in isolation:**

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_policy_dirty_detection() {
    use cargo_cicd::policies::DirtyPolicy;
    
    let mut state = EngineState::default();
    state.git_phase.dirty_count = 5;  // Set dirty state
    
    let recommendations = DirtyPolicy::evaluate(&state);
    
    assert!(!recommendations.is_empty(), "dirty state should generate recommendations");
    assert!(recommendations[0].contains("clean"), "recommendation should mention cleaning");
}
```

---

## Development Environment Setup

### Rust Toolchain Requirements

**Minimum supported version:** Rust 1.85 (see `rust-version` in Cargo.toml)

```bash
# Install or update Rust:
rustup install 1.85
rustup default 1.85

# Verify:
rustc --version  # Should output 1.85.x or later
cargo --version
```

**Optional: Use a `.rust-version` or `rust-toolchain.toml` file:**

```toml
# File: rust-toolchain.toml
[toolchain]
channel = "1.85"
```

Then `cargo build` automatically uses the right toolchain.

### wasm4pm Binary Location and Discovery

**wasm4pm** is the evidence-gate oracle. Tests that check evidence verdicts need the `wpm` binary.

**Discovery order (in src/evidence/mod.rs):**

1. Environment variable `WPM_BINARY`
2. `/Users/sac/wasm4pm/target/release/wpm` (hardcoded fallback from CLAUDE.md)
3. System `PATH` lookup for `wpm`
4. Not found → tests fall back to `Blocked` verdict

**Setup for development:**

```bash
# Option 1: If you have wasm4pm checked out locally, set the env var:
export WPM_BINARY="/path/to/wasm4pm/target/release/wpm"
cargo test --test wasm4pm_evidence_gate

# Option 2: Create a symlink in your PATH:
mkdir -p ~/local/bin
ln -s /path/to/wasm4pm/target/release/wpm ~/local/bin/wpm
export PATH="$HOME/local/bin:$PATH"
cargo test --test wasm4pm_evidence_gate

# Option 3: Build and install from wasm4pm repo:
cd /path/to/wasm4pm
cargo build --release
# Then either set WPM_BINARY or add to PATH

# Verify discovery:
which wpm
cargo test --test wasm4pm_harness -- --nocapture 2>&1 | grep -i "wpm\|available"
```

**Expected behavior when wpm is NOT found:**

Tests enter graceful `Blocked` verdict path:

```
test evidence_gate_status_show_accepted ... ok
```

The test passes but skips Accept assertions because the oracle is unavailable.

**Force strict oracle requirement in CI:**

```bash
# Set this environment variable to fail fast if wpm is missing:
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

When `REQUIRE_WPM_ORACLE=1` is set, tests panic with a clear message if wpm is unavailable:

```
REQUIRE_WPM_ORACLE=1 is set but the wpm oracle binary is absent. 
Test 'evidence_gate_status_show_accepted' cannot exercise its Accept assertion.
Ensure the wpm binary exists at /Users/sac/wasm4pm/target/release/wpm.
```

**Understanding the Oracle Absence Pattern:**

This is intentional design. Evidence-gate tests require the oracle to close releases because:

- cargo-cicd internal tests cannot self-assert on release safety
- Only the wasm4pm oracle can adjudicate "ALIVE" verdict
- CI pipelines without the oracle gracefully degrade (tests don't fail, but Accept assertions are skipped)
- CI pipelines WITH the oracle can enforce strict evidence requirements

**Debugging wpm Binary Discovery:**

```bash
# Check if env var is set:
echo $WPM_BINARY

# Check if in PATH:
which wpm

# Check if hardcoded path exists:
ls -l /Users/sac/wasm4pm/target/release/wpm 2>&1

# List all ways to provide it:
echo "Option 1: export WPM_BINARY=/path/to/wpm"
echo "Option 2: ln -s /path/to/wpm ~/local/bin/wpm && export PATH=\$HOME/local/bin:\$PATH"
echo "Option 3: Place at /Users/sac/wasm4pm/target/release/wpm (hardcoded fallback)"
```

### Ontology and ggen Setup

**ggen** is the manufacturing pipeline for generating noun modules and test scaffolding from the ontology.

**Key files:**

- `ggen.toml` — ggen configuration
- `ontology/cargo-cicd.ttl` — RDF ontology (Turtle format)
- `queries/` — SPARQL queries for ontology
- `templates/` — Tera templates for code generation

**To regenerate after ontology changes:**

```bash
# Install ggen (if not already):
cargo install ggen

# Run ggen in the workspace root:
ggen

# This regenerates:
# - Noun modules in src/nouns/ (from templates/)
# - CLI test scaffolding in tests/cli/
# - README.md sections marked with <!-- ggen:* -->
```

**Verify ggen didn't break things:**

```bash
# Check for uncommitted changes:
git diff src/nouns/ tests/cli/

# Rebuild to ensure no syntax errors:
cargo build

# Run tests:
cargo test
```

**If ggen output is wrong:**

1. Check the SPARQL query in `queries/` — does it select the right subjects?
2. Check the Tera template in `templates/` — does it have the right variable names?
3. Check `ggen.toml` — does it reference the right query and template files?

Example `ggen.toml` section:

```toml
[[generate]]
query = "queries/nouns.sparql"
template = "templates/noun.tera"
output_dir = "src/nouns"
file_per_subject = true
```

### cargo-make Configuration (if available)

The project may use `cargo-make` for convenient task running (though not strictly required):

```bash
# Install cargo-make:
cargo install cargo-make

# Run common tasks (if Makefile.toml exists):
cargo make build
cargo make check
cargo make test
```

If `Makefile.toml` is not present, just use `cargo` directly:

```bash
cargo build
cargo test
cargo check
```

### Setting Up Your IDE

**VS Code:**

1. Install Rust Analyzer extension.
2. Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

3. Open the workspace root (`/home/user/cargo-cicd`), not a subdirectory.

**CLion / IntelliJ:**

- Just open the workspace root. The IDE auto-detects `Cargo.toml` and configures Rust tooling.

### Local Build and Test Loop

**Fast iteration:**

```bash
# 1. Make a change to src/
vi src/adapters/git.rs

# 2. Rebuild (incremental):
cargo build

# 3. Run a specific test:
cargo test --test changed_tests test_git_state_detection -- --nocapture

# 4. If test fails, inspect the output and go back to step 1.
```

**Cleaner builds (if incremental is confusing):**

```bash
cargo clean
cargo build
cargo test
```

---

## Performance Profiling

### Identify Slow Adapters

**Adapters are isolated, so profile each independently.**

**Add timing to adapter calls:**

```rust
// In src/nouns/your_verb.rs
use std::time::Instant;

fn your_verb_impl() -> Result<()> {
    let mut state = EngineState::default();
    
    // Profile workspace detection:
    let t0 = Instant::now();
    let workspace = CargoMetadataAdapter::read()?;
    eprintln!("[PROFILE] CargoMetadataAdapter: {:.2}ms", t0.elapsed().as_secs_f64() * 1000.0);
    
    // Profile git status:
    let t0 = Instant::now();
    let git_state = GitStatusAdapter::read_git_state()?;
    eprintln!("[PROFILE] GitStatusAdapter: {:.2}ms", t0.elapsed().as_secs_f64() * 1000.0);
    
    // Profile target scan:
    let t0 = Instant::now();
    let target = TargetScannerAdapter::scan_target(&workspace.root)?;
    eprintln!("[PROFILE] TargetScannerAdapter: {:.2}ms", t0.elapsed().as_secs_f64() * 1000.0);
    
    Ok(())
}
```

**Run and compare:**

```bash
cargo build --release
time ./target/release/cargo-cicd status show 2>&1 | grep PROFILE
```

**Common bottlenecks:**

1. **WalkDir::new() on large target dirs** — The `target/` directory can have millions of files. `TargetScannerAdapter` uses `WalkDir` which is slow on massive trees. Consider caching or early exit.

2. **git status --porcelain on large repos** — Each invocation forks `git`. With many files, this is slow. Cache the result if possible.

3. **Cargo metadata parsing** — Running `cargo metadata` spawns cargo and parses large JSON. Cache it.

### Workspace Scan Bottlenecks

**Workspace scanning walks the entire workspace to find crates and test files.**

**Profile the scan:**

```rust
// In ChangedFileDetector or similar:
let t0 = Instant::now();
for entry in walkdir::WalkDir::new(&workspace.root)
    .into_iter()
    .filter_map(|e| e.ok())
{
    // ... process entry ...
}
eprintln!("[PROFILE] Workspace walk: {:.2}ms, entries: {}", 
          t0.elapsed().as_secs_f64() * 1000.0, 
          entry_count);
```

**Optimization strategies:**

1. **Cache the walk result** — Store in a temp file or memory between invocations.

2. **Limit depth** — Don't walk into `.git`, `target/`, or `node_modules`:

```rust
walkdir::WalkDir::new(&root)
    .into_iter()
    .filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target"
    })
```

3. **Parallel iteration** — Use `rayon` for parallel traversal (but adds dependency).

### Memory Usage

**Check memory usage while running a command:**

```bash
/usr/bin/time -v cargo cicd workspace doctor
```

Look for:
- **Maximum resident set size** — Peak memory usage.
- **Page faults** — High faults indicate swap usage (bad for CI/CD).

**If memory usage is high:**

1. Avoid loading entire files into memory. Use streaming/iterators.
2. Profile with `valgrind` or `heaptrack`:

```bash
valgrind --tool=massif cargo cicd status show
```

3. Check for memory leaks in adapters (e.g., unbounded `Vec` growth).

### Benchmark Tests

**Write a benchmark test to track performance regressions:**

```rust
#[test]
#[ignore]  // Run with: cargo test --ignored bench_workspace_scan
fn bench_workspace_scan() {
    let dir = tempfile::TempDir::new().unwrap();
    // Create a realistic workspace structure
    for i in 0..100 {
        let crate_dir = dir.path().join(format!("crate{}", i));
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            format!("[package]\nname=\"crate{}\"\nversion=\"0.1.0\"\n", i)
        ).unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    }
    
    let t0 = std::time::Instant::now();
    let _ = ChangedFileDetector::scan_workspace(dir.path());
    let elapsed = t0.elapsed();
    
    eprintln!("Scanned 100 crates in {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    
    // Assert reasonable threshold:
    assert!(elapsed.as_secs() < 5, "workspace scan too slow: {:.2}s", elapsed.as_secs_f64());
}
```

Run:

```bash
cargo test --ignored bench_workspace_scan -- --nocapture
```

---

## Quick Reference

### Essential Commands Cheat Sheet

```bash
# Build and check
cargo build                    # Build binary
cargo build --release         # Release build for profiling
cargo check                    # Type-check without building
cargo clippy                   # Lint checks

# Test with various scopes
cargo test                     # All tests with default features
cargo test --test NAME        # Run single test file
cargo test --test NAME FUNC   # Run single test function
cargo test -- --nocapture    # Show stdout/stderr

# Test with features
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
cargo test --all-features

# Test with environment variables
CARGO_CICD_DEBUG_ADAPTERS=1 cargo test
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate

# Profile and bench
time cargo cicd status show
/usr/bin/time -v cargo cicd workspace doctor
cargo test --ignored bench_workspace_scan -- --nocapture

# Check git/workspace state
git status --porcelain        # See all changes
git rev-parse --git-dir       # Verify git repo
cargo metadata                # Verify Cargo.toml validity
```

### Common Failure Modes Quick Lookup

| Symptom | Likely Cause | Troubleshooting |
|---------|-------------|-----------------|
| "workspace root not found" | Running outside a workspace | Ensure Cargo.toml exists; check pwd |
| "forbidden term found" | Public API leak | Check CLAUDE.md for forbidden list |
| "malformed evidence" | XES format broken | Validate with jq; check wpm binary |
| "git state inconsistent" | Detached HEAD or no upstream | Run `git rev-parse --abbrev-ref HEAD` |
| "wasm4pm blocked" | wpm binary not found | Check WPM_BINARY env var |
| "test hangs" | Large workspace scan | Use timeout; check WalkDir filters |
| "feature interaction fails" | Missing feature implication | Verify feature flags in Cargo.toml |

---

## Additional Comprehensive Resources

### File Reference

**Core Architecture:**
- `/home/user/cargo-cicd/src/main.rs` — Entry point and default verb injection
- `/home/user/cargo-cicd/src/adapters/mod.rs` — Adapter trait and all implementations
- `/home/user/cargo-cicd/src/engine/mod.rs` — EngineState aggregate root
- `/home/user/cargo-cicd/src/nouns/` — Noun modules (status, target, test, git, publish, workspace, evidence)

**Test Infrastructure:**
- `/home/user/cargo-cicd/tests/invariants.rs` — 7 non-negotiable public boundary invariants
- `/home/user/cargo-cicd/tests/fixtures/mod.rs` — FixtureWorkspace helper builders
- `/home/user/cargo-cicd/tests/fixtures/` — Pre-built test workspaces (clean, dirty, corrupted, etc.)
- `/home/user/cargo-cicd/tests/cli/` — Command projection tests for each noun
- `/home/user/cargo-cicd/tests/wasm4pm_*.rs` — Evidence-gate test suite

**Configuration & Generation:**
- `/home/user/cargo-cicd/Cargo.toml` — Workspace members, features, dependencies, test definitions
- `/home/user/cargo-cicd/CLAUDE.md` — Project mission, forbidden terms, architecture mandates
- `/home/user/cargo-cicd/ggen.toml` — Code generation configuration
- `/home/user/cargo-cicd/ontology/cargo-cicd.ttl` — RDF ontology (Turtle)
- `/home/user/cargo-cicd/queries/` — SPARQL queries for ontology
- `/home/user/cargo-cicd/templates/` — Tera templates for code generation

### Related Documentation

- **CLAUDE.md** — Project mission (Level 5 process-data engine), forbidden terms, commit format, architecture decisions
- **TESTING_GUIDE.md** — Complete testing strategy (smoke, integration, evidence-gate tiers)
- **ARCHITECTURE.md** — Detailed architecture, state dimensions, adapter descriptions
- **README.md** — Public-facing user documentation (generated from ontology)
- **CONTRIBUTING.md** — Contribution workflow and standards

### Adapter Responsibilities Reference

| Adapter | Source | Output | Purpose |
|---------|--------|--------|---------|
| CargoMetadataAdapter | `cargo metadata` | WorkspaceState | Discover workspace root, members, manifest |
| GitStatusAdapter | `git status --porcelain` | GitPhaseState | Read branch, dirty/staged/untracked counts |
| TargetScannerAdapter | Filesystem walk | TargetState | Measure target/ size and file age |
| ChangedFileDetector | git diff + filesystem | ChangedFileState | Find changed .rs files since last commit |
| ToolchainDetector | rust-toolchain.toml | ToolchainState | Detect Rust version and channel |
| TrybuildDetector | Filesystem walk | TrybuildState | Find trybuild fixtures and changed ones |
| CicdTomlWriter | cicd.toml file | Emits events | Write workspace state and process events |

---

## Advanced Debugging Techniques

### Inspecting State at Breakpoints (Without Debugger)

If you don't have a debugger setup, use strategic eprintln! dumps before key operations:

```rust
// In src/nouns/your_command.rs
fn your_verb_impl() -> Result<()> {
    let mut state = EngineState::default();
    
    // Checkpoint 1: After workspace detection
    state.workspace = CargoMetadataAdapter::read()?;
    eprintln!("[CHECKPOINT-1] workspace = {:#?}", state.workspace);
    
    // Checkpoint 2: After git state
    state.git_phase = GitStatusAdapter::read_git_state()?;
    eprintln!("[CHECKPOINT-2] git_phase = {:#?}", state.git_phase);
    
    // Checkpoint 3: After target scan
    state.target = TargetScannerAdapter::scan(&state.workspace.root)?;
    eprintln!("[CHECKPOINT-3] target = {:#?}", state.target);
    
    // Now proceed with business logic
    render_output(&state)?;
    Ok(())
}
```

Then run with output capture:

```bash
cargo cicd status show 2>&1 | tee /tmp/debug.log
# Review checkpoints in the log
```

### Testing State Transitions (State Machine Debugging)

cargo-cicd is fundamentally a state machine: Adapters read external state, populate EngineState, then Nouns render output. To debug state transitions:

```rust
// In tests/your_debug_test.rs
#[test]
#[ignore]  // Run with: cargo test --ignored debug_state_transitions -- --nocapture
fn debug_state_transitions() {
    let fixture = FixtureWorkspace::clean();
    
    // Snapshot 1: Before any commands
    let cmd1 = Command::cargo_bin("cargo-cicd").unwrap()
        .current_dir(fixture.root.clone())
        .arg("status").arg("show")
        .output().unwrap();
    eprintln!("=== SNAPSHOT 1: Clean ===");
    eprintln!("{}", String::from_utf8_lossy(&cmd1.stdout));
    
    // Modify the workspace
    std::fs::write(fixture.root.join("new_file.rs"), "// changed\n").unwrap();
    
    // Snapshot 2: After modification
    let cmd2 = Command::cargo_bin("cargo-cicd").unwrap()
        .current_dir(fixture.root.clone())
        .arg("status").arg("show")
        .output().unwrap();
    eprintln!("=== SNAPSHOT 2: After modification ===");
    eprintln!("{}", String::from_utf8_lossy(&cmd2.stdout));
    
    // Verify state changed
    assert_ne!(cmd1.stdout, cmd2.stdout, "state should change after modification");
}
```

Run:

```bash
cargo test --ignored debug_state_transitions -- --nocapture
```

### Trace Adapter Execution Order

To understand which adapters run and in what order:

```rust
// In each adapter, add a guard struct that logs on drop
pub struct AdapterGuard {
    name: &'static str,
    start: std::time::Instant,
}

impl Drop for AdapterGuard {
    fn drop(&mut self) {
        eprintln!("[ADAPTER] {} took {:.2}ms", 
                  self.name, 
                  self.start.elapsed().as_secs_f64() * 1000.0);
    }
}

// Then in each adapter function:
impl CargoMetadataAdapter {
    pub fn read() -> Result<WorkspaceState> {
        let _guard = AdapterGuard { name: "CargoMetadataAdapter", start: Instant::now() };
        // ... actual implementation ...
    }
}
```

This creates a clear timeline of adapter execution without modifying return values.

### Capture Intermediate Files for Post-Mortem Analysis

When a test fails, capture the workspace state before cleanup:

```bash
#!/bin/bash
# save_test_workspace.sh
# Run a failing test and save the temp workspace

TESTNAME="$1"
SAVEDIR="/tmp/cargo-cicd-debug-${TESTNAME}-$(date +%s)"

# Modify your test temporarily to save on failure:
# Use RUST_BACKTRACE=1 and capture stderr
RUST_BACKTRACE=1 cargo test --test "$TESTNAME" 2>&1 | tee "$SAVEDIR/test.log"

echo "Debug workspace may be in /tmp/cargo-cicd-debug-*/"
```

---

## Advanced Feature Flag Debugging

### Understanding Feature Implication Graph

Features in Cargo.toml form a directed graph:

```toml
[features]
default = []
process-data = []           # Base: Level 5 engine
autonomic = ["process-data"]  # Implies: process-data
contrib = ["process-data"]    # Implies: process-data
wasm4pm = ["process-data"]    # Implies: process-data
```

When you enable `autonomic`, you automatically get `process-data`. But the reverse is NOT true.

### Debug Feature Availability

```bash
# See which features are compiled in:
cargo tree --features autonomic 2>&1 | head -20

# Verify feature gate in code:
grep -r '#\[cfg(feature' src/

# Build with specific feature combos:
cargo build --no-default-features
cargo build --no-default-features --features process-data
cargo build --no-default-features --features autonomic
cargo build --no-default-features --features wasm4pm
cargo build --all-features
```

### Test Feature Isolation

Write a test that verifies feature gates are working:

```rust
#[cfg(feature = "autonomic")]
#[test]
fn test_autonomic_only_when_enabled() {
    // This should only compile with --features autonomic
    use cargo_cicd::autonomic::PolicyState;
    let _state = PolicyState::default();
}

#[cfg(not(feature = "autonomic"))]
#[test]
fn test_autonomic_not_available_by_default() {
    // autonomic::PolicyState should not exist
    // (This test just documents the feature boundary)
}
```

---

## Detailed wasm4pm Evidence Testing

### Understanding XES Format

XES (XML Event Stream) is a process mining standard. cargo-cicd emits events as XES, which wpm then adjudicates.

Example valid XES structure:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="2.0">
  <trace id="cargo-cicd-v26.6.2">
    <event>
      <string key="concept:name" value="status show"/>
      <string key="lifecycle:transition" value="PASS"/>
      <date key="time:timestamp" value="2026-06-14T10:30:45Z"/>
    </event>
  </trace>
</log>
```

### Validating Evidence Manually

```bash
# Check if evidence file exists and is valid XML:
xmllint --noout target/cargo-cicd/evidence/events.xes

# Check if wpm can read it:
wpm audit target/cargo-cicd/evidence/events.xes --format json

# Check receipt doctor directly:
wpm receipt doctor --format json --strict target/cargo-cicd/evidence/events.jsonl
```

### Creating Evidence-Only Tests

Test evidence emission without waiting for the full command:

```rust
#[test]
fn test_emit_evidence_only() {
    use cargo_cicd::evidence::{ProcessEvent, emit_xes};
    use tempfile::TempDir;
    
    let dir = TempDir::new().unwrap();
    let events = vec![
        ProcessEvent::new("status show", "PASS"),
        ProcessEvent::new("git status", "PASS"),
    ];
    
    let xes_path = dir.path().join("events.xes");
    emit_xes(&events, &xes_path).expect("emit must succeed");
    
    // Validate the file exists and is well-formed
    assert!(xes_path.exists(), "XES file must exist");
    let content = std::fs::read_to_string(&xes_path).unwrap();
    assert!(content.contains("<?xml"), "Must be valid XML");
    assert!(content.contains("<log"), "Must have log element");
}
```

---

## Workspace Scan Optimization Checklist

If workspace scans are slow, follow this checklist:

- [ ] Profile with `time` to get baseline (e.g., `time cargo cicd status show`)
- [ ] Check if you're scanning the entire workspace or just changed files
- [ ] Verify WalkDir filters exclude `.git`, `target/`, `node_modules/`
- [ ] Consider caching workspace metadata between invocations
- [ ] Profile individual adapters with the `[PROFILE]` markers above
- [ ] Check if git operations are slow (`git status --porcelain` on large repos)
- [ ] If scanning large target dirs, consider `find` with depth limits instead of WalkDir
- [ ] Use release builds for profiling: `cargo build --release && time ./target/release/cargo-cicd ...`

Example optimization: limit walk depth

```rust
// Bad: walks entire workspace including target, .git
for entry in walkdir::WalkDir::new(&root) { }

// Good: excludes slow directories
for entry in walkdir::WalkDir::new(&root)
    .into_iter()
    .filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "target" && name != "node_modules"
    })
    .filter_map(|e| e.ok())
{ }
```

---

## Debugging Clap/Noun-Verb Interactions

cargo-cicd uses `clap-noun-verb` for CLI parsing. Debugging command-line parsing:

### Check How Bare Nouns are Mapped

The `inject_default_verbs()` function in `main.rs` maps bare nouns to default verbs:

```
cargo cicd status    -> cargo cicd status show
cargo cicd publish   -> cargo cicd publish run
cargo cicd workspace -> cargo cicd workspace doctor
```

If your bare noun command doesn't work, check:

1. Is it listed in the `match` statement in `main.rs`?
2. Does the noun implement `NounCommand::run_direct()`?
3. Is the default verb implemented?

### Test Clap Parsing Separately

```rust
#[test]
fn test_clap_parsing_status_bare() {
    use clap::Parser;
    
    // Simulate: cargo cicd status
    let args = vec!["cargo-cicd", "status"];
    let cmd = YourCliParser::try_parse_from(&args);
    
    // Should either parse successfully or explain why not
    match cmd {
        Ok(parsed) => eprintln!("Parsed successfully: {:#?}", parsed),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

---

## Last Updated

**Last Updated:** 2026-06-14  
**Tested with:** cargo-cicd v26.6.2, Rust 1.85+
**Related Docs:** TESTING_GUIDE.md, ARCHITECTURE.md, CLAUDE.md
