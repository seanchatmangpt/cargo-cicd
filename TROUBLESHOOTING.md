# cargo-cicd Troubleshooting Guide

**For developers working on cargo-cicd v26.6.2**

This guide covers debugging techniques, test isolation patterns, common failure modes, and development environment setup.

---

## Table of Contents

1. [Logging and Tracing](#logging-and-tracing)
2. [Test Debugging](#test-debugging)
3. [Common Issues](#common-issues)
4. [Development Environment Setup](#development-environment-setup)
5. [Performance Profiling](#performance-profiling)

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

**Debug steps:**

```bash
# Check if you're in a workspace:
pwd
ls -la Cargo.toml

# Trace the workspace detection in CargoMetadataAdapter:
CARGO_CICD_DEBUG_ADAPTERS=1 cargo cicd status show 2>&1 | grep -i "workspace\|manifest"

# Check Cargo.toml syntax:
cargo metadata  # If this fails, Cargo.toml is broken
```

**Fix:**

- Ensure you're running from workspace root (where top-level `Cargo.toml` lives).
- Validate TOML: `toml-cli validate Cargo.toml` (or use `cargo metadata`).
- For nested workspaces, verify `[workspace] members = [...]` is properly configured.

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

**Debug steps:**

```bash
# Check feature:
cargo build --features autonomic

# Check cicd.toml policy config:
grep -A 5 '^\[autonomic\]' cicd.toml

# Check if PolicyState is populated:
dbg!(&state.policies);

# Verify policy rules are implemented:
ls -la src/policies/
```

**Fix:**

- Build with `--features autonomic` and test.
- Ensure `cicd.toml [autonomic] suggest_mode = true` (default is true, but explicit is safer).
- Add policy implementations for each rule in `src/policies/`.

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

**Discovery order:**

1. Environment variable `WPM_BINARY`
2. `/Users/sac/wasm4pm/target/release/wpm` (hardcoded fallback from CLAUDE.md)
3. System `PATH`

**Setup for development:**

```bash
# If you have wasm4pm checked out elsewhere, set the env var:
export WPM_BINARY="/path/to/wasm4pm/target/release/wpm"

# Or create a symlink:
mkdir -p ~/local/bin
ln -s /path/to/wasm4pm/target/release/wpm ~/local/bin/wpm
export PATH="$HOME/local/bin:$PATH"

# Verify discovery:
cargo test --test wasm4pm_harness -- --nocapture 2>&1 | grep "wpm"
```

**If wpm is not found, tests print:**

```
BLOCKED: wpm binary not discoverable
```

This is intentional — evidence-gate tests require the oracle to close releases (no self-assertion on release safety).

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

## Additional Resources

**Files referenced in this guide:**

- **Main modules:** `/home/user/cargo-cicd/src/main.rs`, `/home/user/cargo-cicd/src/adapters/mod.rs`, `/home/user/cargo-cicd/src/engine/mod.rs`
- **Tests:** `/home/user/cargo-cicd/tests/` (invariants, changed_tests, wasm4pm_harness, etc.)
- **Test fixtures:** `/home/user/cargo-cicd/tests/fixtures/`
- **Configuration:** `/home/user/cargo-cicd/Cargo.toml`, `/home/user/cargo-cicd/CLAUDE.md`, `/home/user/cargo-cicd/ggen.toml`
- **Ontology:** `/home/user/cargo-cicd/ontology/cargo-cicd.ttl`

**Related documentation in the codebase:**

- `CLAUDE.md` — Project mission, forbidden terms, commit format, architecture overview.
- `README.md` — Public-facing usage; generated from ontology via ggen.
- `src/adapters/mod.rs` — Overview of all adapters and their responsibilities.

---

**Last Updated:** 2026-06-14  
**Tested with:** cargo-cicd v26.6.2, Rust 1.85+
