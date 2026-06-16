# Code Style Guide for cargo-cicd

This document establishes the code style standards for cargo-cicd, enforced through automated linting and pre-commit hooks.

## 1. Rust Style Baseline

All code MUST follow the **Edition 2021** idioms with Rust 1.85+ features available.

### Formatting Defaults

- **Indentation**: 2 spaces (rustfmt default for Rust 2021)
- **Line length**: 100 characters (enforced)
- **Edition**: 2021

### Formatting Enforcement

Use `rustfmt` with defaults; no custom config needed unless specified below:

```bash
# Format the entire workspace
cargo fmt

# Check formatting without writing
cargo fmt -- --check
```

All code must pass `cargo fmt --check` in CI.

### Clippy Linting

Clippy catches style, performance, and correctness issues. All code must pass:

```bash
# Check all targets
cargo clippy --all-targets --workspace

# Check with all features
cargo clippy --all-targets --all-features

# In CI, Clippy fails on warnings
cargo clippy --all-targets --workspace -- -D warnings
```

**Never** suppress clippy warnings with `#[allow(clippy::*)]` unless documented.

---

## 2. Error Handling

Error handling is critical to cargo-cicd's reliability. Follow these patterns strictly.

### anyhow vs thiserror

- **`anyhow::Result<T>`**: Use in binary code and integration points. Provides flexible error context.
- **`thiserror::Error`**: Use for custom error types in library crates. Provides structured error types.

### Example: Binary Error Handling

```rust
use anyhow::{Context, Result, bail};

// ✓ Good: Use ? operator with context
fn load_manifest(path: &Path) -> Result<Manifest> {
    let contents = std::fs::read_to_string(path)
        .context("failed to read manifest")?;
    toml::from_str(&contents)
        .context("failed to parse manifest")
}

// ✓ Good: Use bail! for early exit with context
fn validate_workspace(manifest: &Manifest) -> Result<()> {
    if manifest.members.is_empty() {
        bail!("workspace has no members; this is invalid");
    }
    Ok(())
}

// ✗ Bad: unwrap/expect in library code
fn load_manifest_bad(path: &Path) -> Manifest {
    let contents = std::fs::read_to_string(path).unwrap(); // WRONG
    toml::from_str(&contents).expect("parse failed")       // WRONG
}
```

### Example: Library Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TargetError {
    #[error("target directory not found: {0}")]
    NotFound(String),

    #[error("failed to read target metadata")]
    Io(#[from] std::io::Error),

    #[error("invalid target state: {reason}")]
    Invalid { reason: String },
}

// Library function returns Result<T, TargetError>
pub fn scan_target(path: &Path) -> Result<TargetState, TargetError> {
    if !path.exists() {
        return Err(TargetError::NotFound(path.display().to_string()));
    }
    // ...
    Ok(state)
}
```

### Error Context Rules

1. **Always wrap errors with context** when delegating to functions:
   ```rust
   // ✓ Good
   let metadata = cargo_metadata::MetadataCommand::new()
       .exec()
       .context("failed to load Cargo metadata")?;
   ```

2. **No unwrap/expect in library code**. Use `?` instead:
   ```rust
   // ✓ Good
   let files = std::fs::read_dir(path)?;

   // ✗ Bad
   let files = std::fs::read_dir(path).unwrap();
   ```

3. **Panics only for unrecoverable invariants**:
   ```rust
   // ✓ OK: This is a genuine invariant check
   assert!(engine_state.workspace.is_some(), "engine not initialized");

   // ✗ Bad: Recoverable error, should return Result
   panic!("file not found"); // Should be anyhow::bail!
   ```

---

## 3. Naming Conventions

Consistent naming improves readability and enables IDE shortcuts.

### Functions: lowercase_snake_case

```rust
// ✓ Good
fn load_workspace_manifest() -> Result<Manifest> { }
fn is_target_stale(path: &Path, threshold: Duration) -> bool { }
fn emit_process_event(event: ProcessEvent) -> Result<()> { }

// ✗ Bad
fn LoadWorkspaceManifest() -> Result<Manifest> { }
fn isTargetStale(path: &Path, threshold: Duration) -> bool { }
```

### Types: PascalCase

```rust
// ✓ Good
pub struct WorkspaceState { }
pub enum TargetKind { Debug, Release }
pub trait CommandRunner { }

// ✗ Bad
pub struct workspace_state { }
pub enum target_kind { }
```

### Constants: SCREAMING_SNAKE_CASE

```rust
// ✓ Good
const DEFAULT_TARGET_DIR: &str = "target";
const MAX_CONCURRENT_JOBS: usize = 16;
pub const VERSION: &str = "26.6.2";

// ✗ Bad
const default_target_dir: &str = "target";
const MaxConcurrentJobs: usize = 16;
```

### Type Parameters & Generics

Use single letters for simple generics:

```rust
// ✓ Good: Simple, obvious usage
pub struct Cache<T> { items: Vec<T> }
pub fn process<T, E>(input: T) -> Result<(), E> where E: std::error::Error { }

// ✓ Better: More descriptive when complexity warrants
pub fn collect_by<Item, Key, F>(items: Vec<Item>, keyfn: F) -> Map<Key, Vec<Item>>
where
    F: Fn(&Item) -> Key,
{ }
```

### Intentionally Unused Variables

Prefix with underscore to indicate intentionality:

```rust
// ✓ Good: Explicitly unused, will be used in the future
fn register_handler(_context: &AppContext) {
    // Not used yet, reserved for future expansion
}

// ✓ Good: Unused in some code paths
#[allow(dead_code)]
fn debug_print_state(state: &EngineState) { }
```

---

## 4. Comments & Documentation

### Public API Documentation (///)

All public items MUST have doc comments:

```rust
/// Load and parse the workspace manifest from the given path.
///
/// Returns the parsed manifest or an error if the file cannot be read
/// or is invalid TOML.
///
/// # Arguments
/// * `path` - Path to the Cargo.toml file
///
/// # Examples
/// ```
/// let manifest = load_manifest(Path::new("Cargo.toml"))?;
/// ```
pub fn load_manifest(path: &Path) -> Result<Manifest> { }

/// Represents the state of all targets in the workspace.
///
/// This struct aggregates metadata about debug, release, and test targets,
/// used by the status noun for reporting.
#[derive(Debug, Clone)]
pub struct TargetState {
    pub targets: Vec<Target>,
}
```

### Implementation Comments (//)

Comment the "why", not the "what":

```rust
// ✓ Good: Explains intent
// Strip the "cicd" prefix from argv because Cargo's external subcommand protocol
// inserts it; we need clean argv for clap parsing.
if args.get(1).map(|s| s.as_str()) == Some("cicd") {
    args.remove(1);
}

// ✗ Bad: Echoes the code
// Remove the element at index 1
args.remove(1);
```

### No Comment Noise

Avoid comments that restate the code:

```rust
// ✗ Bad: Echo comments
let path = Path::new("/target");  // Set path to /target
let count = items.len();           // Get the count
if x > 5 { /* x is greater than 5 */ }

// ✓ Good: Omit obvious comments
let path = Path::new("/target");
let count = items.len();
if x > 5 { /* Skip if processing too many items */ }
```

### Doc Comment Examples

Include runnable examples in doc comments:

```rust
/// Find all Rust source files in the workspace.
///
/// # Examples
/// ```
/// use cargo_cicd_core::workspace::find_rust_files;
/// let files = find_rust_files(Path::new("."))?;
/// assert!(!files.is_empty());
/// ```
pub fn find_rust_files(root: &Path) -> Result<Vec<PathBuf>> { }
```

---

## 5. Imports

Organize imports into three groups: standard library, external crates, internal crates. Separate groups with blank lines.

### Import Organization

```rust
// ✓ Good: Organized by origin
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::engine::EngineState;
use crate::policies::PolicyState;

// ✗ Bad: Random order
use crate::engine::EngineState;
use std::path::Path;
use anyhow::Result;
use serde::Serialize;
use walkdir::WalkDir;
```

### Avoid Glob Imports

Never use glob imports except in test modules:

```rust
// ✗ Bad: Pollutes namespace, unclear what's imported
use prelude::*;
use std::collections::*;

// ✓ Good: Explicit imports
use std::collections::{HashMap, HashSet};

// ✓ OK in test modules only
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

### No Unused Imports

Clippy enforces this. Always remove unused imports:

```bash
# Clippy warns on unused imports
cargo clippy --all-targets

# rustfmt does not remove unused imports; use a tool or manual review
```

---

## 6. Testing Conventions

### Test Module Structure

Use the standard Rust pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_manifest_with_valid_file() {
        // Arrange
        let path = Path::new("tests/fixtures/Cargo.toml");

        // Act
        let result = load_manifest(path);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_manifest_with_missing_file() {
        // Arrange
        let path = Path::new("/nonexistent/Cargo.toml");

        // Act
        let result = load_manifest(path);

        // Assert
        assert!(result.is_err());
    }
}
```

### Test Naming

Test names describe what they test, not arbitrary numbers:

```rust
// ✓ Good: Descriptive names
#[test]
fn test_target_state_includes_debug_target() { }

#[test]
fn test_git_phase_detects_uncommitted_changes() { }

#[test]
fn test_policy_suggests_nothing_when_clean() { }

// ✗ Bad: Non-descriptive
#[test]
fn test_1() { }

#[test]
fn test_target() { }
```

### Assertion Macros

Use the correct assertion macro:

```rust
// ✓ Good
assert!(x > 0, "expected x to be positive, got {}", x);
assert_eq!(actual, expected, "mismatch in result");
assert_ne!(a, b);

// ✗ Bad: Using unwrap to assert
result.unwrap().assert(condition); // WRONG

// ✗ Bad: Cryptic assertion messages
assert!(x > 0);  // BETTER: Add context
```

### Integration Tests with assert_cmd and tempfile

cargo-cicd uses `assert_cmd` and `tempfile` for integration tests:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_status_shows_clean_workspace() {
    let temp = TempDir::new().unwrap();
    let manifest_path = temp.path().join("Cargo.toml");
    fs::write(&manifest_path, "[package]\nname=\"test\"").unwrap();

    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("status")
        .arg("show")
        .current_dir(temp.path());

    cmd.assert().success();
}
```

---

## 7. Feature Flag Patterns

### Gate Code Behind Features

Use `#[cfg(feature = "...")]` to conditionally include code:

```rust
// ✓ Good: Gate complex features
#[cfg(feature = "advanced")]
pub mod tracing_integration {
    use tracing::{info, span, Level};

    pub fn trace_command_execution(cmd: &str) {
        let span = span!(Level::INFO, "command", cmd);
        let _guard = span.enter();
        info!("executing command");
    }
}
```

### Gate Imports

Gate imports when the feature is unavailable:

```rust
#[cfg(feature = "advanced")]
use tracing::info;

#[cfg(not(feature = "advanced"))]
macro_rules! info {
    ($($tt:tt)*) => {};
}

pub fn process() {
    info!("processing"); // Works whether feature is on or off
}
```

### Provide Stubs for Off-Feature

Ensure the same function signature exists regardless of feature state:

```rust
#[cfg(feature = "advanced")]
pub fn profile_target_build(workspace: &Workspace) -> Result<ProfilingReport> {
    // Real implementation using hdrhistogram, blake3, etc.
    Ok(ProfilingReport { /* ... */ })
}

#[cfg(not(feature = "advanced"))]
pub fn profile_target_build(_workspace: &Workspace) -> Result<ProfilingReport> {
    // Stub: feature not enabled
    Err(anyhow::anyhow!("profiling requires 'advanced' feature"))
}
```

---

## 8. Performance Antipatterns

### Avoid Allocation in Loops

```rust
// ✗ Bad: Allocates in every iteration
for item in items {
    let cloned = item.clone();  // WRONG
    let mut v = Vec::new();     // WRONG
    process(&cloned);
}

// ✓ Good: Allocate once, reuse
let mut buffer = Vec::new();
for item in items {
    buffer.clear();
    process(&item, &mut buffer);
}
```

### Avoid Cloning When References Work

```rust
// ✗ Bad: Unnecessary clone
fn process_items(items: Vec<Item>) {  // Takes ownership, but...
    for item in items.clone() {       // ...then clones again?
        handle(&item);
    }
}

// ✓ Good: Take reference, use iterator
fn process_items(items: &[Item]) {
    for item in items {
        handle(item);
    }
}
```

### Avoid Regex in Hot Paths

```rust
// ✗ Bad: Compiles regex on every call
fn is_test_file(path: &str) -> bool {
    regex::Regex::new(r".*_test\.rs").unwrap().is_match(path)
}

// ✓ Good: Use compile-once or aho-corasick for patterns
fn is_test_file(path: &str) -> bool {
    path.ends_with("_test.rs")
}

// ✓ Good for complex patterns: aho-corasick (behind 'advanced' feature)
#[cfg(feature = "advanced")]
fn find_test_patterns(text: &str) -> Vec<&str> {
    use aho_corasick::AhoCorasick;
    let ac = AhoCorasick::new(["#[test]", "#[tokio::test]"]).unwrap();
    ac.find_iter(text).map(|m| &text[m.start()..m.end()]).collect()
}
```

### Use Parallelism for Fan-Out

```rust
// ✗ Bad: Single-threaded when parallelism available
fn scan_all_targets(workspace: &Workspace) -> Result<Vec<TargetMetadata>> {
    workspace
        .targets
        .iter()
        .map(|t| scan_target(t))
        .collect()
}

// ✓ Good: Use rayon for CPU-bound work (behind 'advanced' feature)
#[cfg(feature = "advanced")]
fn scan_all_targets(workspace: &Workspace) -> Result<Vec<TargetMetadata>> {
    use rayon::prelude::*;
    workspace
        .targets
        .par_iter()
        .map(|t| scan_target(t))
        .collect()
}
```

---

## 9. Automation & Enforcement

### Local: Pre-Commit Hook

cargo-cicd uses a pre-commit hook (in `.claude/hooks/` or `scripts/`) to run:

```bash
#!/bin/bash
set -e

echo "Running cargo fmt..."
cargo fmt --all

echo "Running cargo clippy..."
cargo clippy --all-targets --workspace

echo "Pre-commit checks passed."
```

To install:

```bash
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### CI: GitHub Actions (or equivalent)

The CI pipeline enforces:

1. **Format check** (fail if not formatted):
   ```bash
   cargo fmt --all -- --check
   ```

2. **Clippy (warnings as errors)**:
   ```bash
   cargo clippy --all-targets --workspace -- -D warnings
   ```

3. **Test suite**:
   ```bash
   cargo test --all
   ```

Example CI step:

```yaml
- name: Check formatting
  run: cargo fmt --all -- --check

- name: Run clippy
  run: cargo clippy --all-targets --workspace -- -D warnings

- name: Run tests
  run: cargo test --all
```

### Local: IDE Integration

**VSCode + rust-analyzer**:
- Install "rust-analyzer" extension
- Add to `.vscode/settings.json`:
  ```json
  {
    "[rust]": {
      "editor.defaultFormatter": "rust-lang.rust-analyzer",
      "editor.formatOnSave": true
    },
    "rust-analyzer.checkOnSave.command": "clippy"
  }
  ```

---

## 10. Advanced Guidelines

### Module Organization

Organize by responsibility, not by artifact type:

```
src/
  ├── nouns/              # CLI noun modules (status, target, test, etc.)
  │   ├── status.rs
  │   └── target.rs
  ├── engine/             # Level 5 engine state aggregator
  │   ├── mod.rs
  │   └── state.rs
  ├── adapters/           # External data source adapters
  │   ├── git.rs
  │   └── cargo_metadata.rs
  ├── policies/           # Autonomic policy logic
  │   ├── mod.rs
  │   └── suggest.rs
  ├── cicd_toml.rs        # cicd.toml schema and writers
  └── main.rs
```

### Type Aliases for Clarity

Use type aliases for complex types that appear repeatedly:

```rust
// ✓ Good: Clarifies intent
type PolicySuggestion = Result<Vec<String>>;
type ProcessEventLog = Vec<ProcessEvent>;
type FileChangeMap = HashMap<PathBuf, ChangeKind>;

// Usage
fn suggest_changes(state: &EngineState) -> PolicySuggestion { }
```

### Visibility: Be Explicit

Always declare visibility explicitly:

```rust
// ✓ Good: Clear intent
pub struct PublicType;
pub(crate) struct InternalType;
struct PrivateType;

pub fn public_function() { }
pub(crate) fn internal_function() { }
fn private_function() { }
```

---

## Summary

1. **Format**: `cargo fmt` (2-space, 100-char)
2. **Lint**: `cargo clippy --all-targets` (0 warnings in CI)
3. **Errors**: `anyhow::Result` in binaries, `thiserror::Error` in libraries
4. **Names**: Functions `snake_case`, types `PascalCase`, constants `SCREAMING_SNAKE_CASE`
5. **Comments**: Doc comments (///) for public items; explain WHY, not WHAT
6. **Tests**: Descriptive names, proper assertions, use fixtures
7. **Features**: Gate with `#[cfg]`, provide stubs
8. **Performance**: Avoid loops-alloc, prefer references, parallelize when needed
9. **CI**: Enforce `cargo fmt --check`, `cargo clippy -- -D warnings`, test suite
10. **Imports**: Std → external → internal; no globs (except tests)

All contributions must pass the CI linting gate before merging.
