# Code Style & Patterns

Conventions and patterns used throughout cargo-cicd.

## Rust Style Guide

cargo-cicd follows standard Rust conventions:

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting (don't modify)
cargo fmt -- --check
```

The project uses `.rustfmt.toml` defaults (if present) or Rust stable defaults.

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Or with cargo-make
cargo make check
```

Fix warnings before submitting PRs; don't suppress them with `#[allow]` unless there's good reason.

## Naming Conventions

### Modules

- **Plural for collections:** `src/adapters/`, `src/policies/`, `src/nouns/`
- **Singular for singletons:** `src/engine/`, `src/session.rs`
- **Snake_case:** `my_module_name`

```rust
// Good
mod adapters;
mod engine;
mod state;
mod nouns;

// Bad
mod adapter;      // should be plural for a dir of adapters
mod Engine;       // should be snake_case
```

### Types (Structs, Enums, Traits)

- **PascalCase**
- **Descriptive names** — avoid abbreviations unless obvious
- **Suffix with type category** when helpful:
  - `*State` — state types
  - `*Adapter` — adapter types
  - `*Event` — event types
  - `*Error` — error types (rarely needed; prefer `anyhow::Error`)

```rust
// Good
pub struct EngineState;
pub struct GitStatusAdapter;
pub struct TestChangedEvent;
pub enum WorkspaceError;

// Avoid
pub struct ES;            // Too abbreviated
pub struct Adapter;       // Too generic
pub struct ev;            // Wrong case
```

### Functions and Methods

- **snake_case**
- **Verb-based for actions:** `scan()`, `validate()`, `emit()`
- **Adjective/noun-based for queries:** `is_clean()`, `has_changes()`, `latest_receipt()`
- **Prefix `try_` for fallible operations** that can fail gracefully

```rust
// Good
pub fn scan(root: &Path) -> anyhow::Result<LintState> { }
pub fn is_healthy(&self) -> bool { }
pub fn try_load_cicd_toml(root: &Path) -> anyhow::Result<CicdToml> { }

// Avoid
pub fn Scan();          // Wrong case
pub fn scan_or_error(); // Use Result<T> instead
pub fn process();       // Too vague; what process?
```

### Constants

- **SCREAMING_SNAKE_CASE**
- **Define at module level or in `const` blocks**

```rust
// Good
const DEFAULT_THREAD_COUNT: usize = 4;
const EVIDENCE_DIR: &str = "target/cargo-cicd/evidence/";

// Avoid
const default_count: usize = 4;  // Wrong case
const DEFLT_THRD_CNT: usize = 4; // Abbreviated
```

### Lifetimes

- **Use descriptive names when ambiguous** (rare; most lifetimes are elided)
- **Single quotes** prefix: `'a`, `'static`

```rust
// Good
fn parse<'input>(input: &'input str) { }

// Common case (lifetime elision is fine)
fn parse(input: &str) { }
```

## Module Organization

### Standard Module Structure

```
src/
├── main.rs                  # Binary entry point, CLI setup
├── lib.rs                   # Public library interface
├── nouns/                   # CLI commands (plural)
│   ├── mod.rs               # Re-exports
│   ├── status.rs
│   ├── target.rs
│   └── ...
├── adapters/                # External source adapters
│   ├── mod.rs
│   ├── git_status_adapter.rs
│   ├── target_scanner_adapter.rs
│   └── ...
├── engine/                  # Core state engine (singular)
│   ├── mod.rs
│   └── state.rs
├── state/                   # State type definitions
│   ├── mod.rs
│   ├── workspace_state.rs
│   ├── git_state.rs
│   └── ...
├── policies/                # Autonomic policies (if enabled)
│   ├── mod.rs
│   └── ...
├── cicd_toml.rs             # cicd.toml schema and I/O
└── evidence.rs              # Process evidence (XES format)
```

### Module Re-exports

In `src/adapters/mod.rs`:
```rust
pub mod git_status_adapter;
pub mod target_scanner_adapter;

pub use git_status_adapter::GitStatusAdapter;
pub use target_scanner_adapter::TargetScannerAdapter;
```

This allows:
```rust
// Instead of
use crate::adapters::git_status_adapter::GitStatusAdapter;

// You can do
use crate::adapters::GitStatusAdapter;
```

### File Organization Within a Module

```rust
// src/adapters/git_status_adapter.rs
use anyhow::{Context, Result};
use std::path::Path;

// 1. Type definitions and structs
pub struct GitStatusAdapter;

#[derive(Debug)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
}

// 2. Implementation blocks (impl YourType)
impl GitStatusAdapter {
    pub fn scan(root: &Path) -> Result<GitStatus> {
        // Implementation
    }
}

// 3. Tests (if module tests exist)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scan_clean_repo() {
        // Test code
    }
}
```

## Comments and Documentation

### Public API Documentation

Use `///` doc comments for public items. Include examples for complex types.

```rust
/// Scans the workspace for git status information.
///
/// Returns a `GitStatus` containing the current branch,
/// number of commits ahead/behind the upstream, and
/// dirty/untracked file counts.
///
/// # Arguments
/// * `root` - Path to the workspace root
///
/// # Returns
/// * `Ok(GitStatus)` - Successfully scanned
/// * `Err(...)` - Failed to read git metadata
///
/// # Examples
/// ```ignore
/// let status = GitStatusAdapter::scan(Path::new("."))?;
/// println!("Branch: {}", status.branch);
/// ```
pub fn scan(root: &Path) -> anyhow::Result<GitStatus> {
    // Implementation
}
```

### Internal Comments

Use `//` for implementation details that need explanation.

```rust
// If we're on a detached HEAD, branch name is empty
if branch.is_empty() {
    return Err(anyhow!("Cannot publish from detached HEAD"));
}

// Adapters must be called in order: git status first,
// then workspace state (depends on git status),
// then engine state (depends on all others).
```

### Avoid Over-Commenting

Don't comment the obvious:

```rust
// BAD: Comment is redundant with code
// Increment counter
counter += 1;

// GOOD: No comment needed for obvious code
counter += 1;

// GOOD: Comment explains *why*, not *what*
// Skip the current crate if it's already been analyzed
// (prevents double-counting in workspace with duplicates)
if analyzed.contains(&crate_name) {
    continue;
}
```

### TODO and FIXME Comments

Use for known issues, but include context:

```rust
// TODO: Replace with async I/O when Tokio is available (Issue #123)
// Currently blocking all workspace scans while reading target/ directory

// FIXME: This panics on invalid UTF-8 in filenames; use OsStr instead
```

## Error Handling

Use `anyhow::Result<T>` for fallible functions:

```rust
use anyhow::{Context, Result};

pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    
    let config = toml::from_str(&content)
        .context("Invalid TOML syntax in config")?;
    
    Ok(config)
}
```

**Pattern: context() chains**
```rust
std::fs::read_to_string(path)
    .context("Failed to read config")?

// vs

std::fs::read_to_string(path)
    .map_err(|e| anyhow!("Failed to read config: {}", e))?
```

Prefer `.context()` — it's cleaner.

## Type Aliases and Type Bounds

### When to Use Type Aliases

```rust
// For complex generic types
pub type BoxedAdapter = Box<dyn SomeAdapterTrait>;

// For error types (though anyhow::Result is preferred)
pub type Result<T> = std::result::Result<T, anyhow::Error>;

// For legibility in bounds
pub type StringError = String;
```

### Avoid

```rust
// Too many single-use aliases
pub type MyNumber = i32;  // Just use i32

// Misleading aliases
pub type Percentage = u32;  // Use a newtype instead if semantics matter
pub struct Percentage(u32);
```

## Trait Design

### Nouns as Traits

Each noun implements `NounCommand`:

```rust
pub trait NounCommand {
    fn name() -> &'static str;
    fn about() -> &'static str;
}

impl NounCommand for StatusNoun {
    fn name() -> &'static str { "status" }
    fn about() -> &'static str { "Display workspace status" }
}
```

### Adapters Don't Need Traits (Usually)

Adapters are typically standalone:

```rust
pub struct GitStatusAdapter;

impl GitStatusAdapter {
    pub fn scan(root: &Path) -> Result<GitStatus> { }
}
```

Unless multiple adapters share behavior, keep them concrete.

### Use Composition Over Inheritance

```rust
// GOOD: Composition
pub struct EngineState {
    pub git_state: GitStatus,
    pub workspace_state: WorkspaceState,
}

// AVOID: Deep trait hierarchies (Rust doesn't encourage inheritance)
pub trait AdapterBase { }
pub trait GitAdapter: AdapterBase { }
pub trait TargetAdapter: AdapterBase { }
```

## Visibility and Encapsulation

### Public API (`pub`)

```rust
// Expose noun commands
pub struct StatusNoun;
pub impl NounCommand for StatusNoun { }

// Expose major state types
pub struct EngineState;

// Expose key adapters
pub struct GitStatusAdapter;
```

### Crate-Private (`pub(crate)`)

```rust
// Implementation details of adapters
pub(crate) struct GitStatusImpl;

// Internal event types
pub(crate) enum InternalEvent;
```

### Private (`no pub keyword`)

```rust
// Helper functions, internal types
fn parse_branch_name(raw: &str) -> String { }

struct ParseError { }
```

## Pattern: Adapters

Standard adapter pattern:

```rust
pub struct MyAdapter;

impl MyAdapter {
    /// Scan external source and return internal state type.
    pub fn scan(root: &Path) -> anyhow::Result<MyState> {
        // 1. Read external source
        // 2. Translate to internal types
        // 3. Return MyState
        Ok(MyState::default())
    }
}
```

Rules:
- Static methods (no `self`)
- Return `anyhow::Result<StateType>`
- No business logic; just translation
- Idempotent (same input → same output)

## Pattern: Nouns

Standard noun pattern:

```rust
pub struct MyNoun;

impl NounCommand for MyNoun {
    fn name() -> &'static str { "my-noun" }
    fn about() -> &'static str { "Description" }
}

impl MyNoun {
    pub fn new() -> Self { Self }
    
    /// Default verb (called if user types just "cargo cicd my-noun")
    pub fn run_direct() -> anyhow::Result<()> {
        Self::show()
    }
    
    fn show() -> anyhow::Result<()> {
        let root = std::env::current_dir()?;
        let engine = EngineState::new(&root)?;
        
        // Read from engine, display results
        println!("Results: {}", engine.my_state.count);
        Ok(())
    }
}
```

Rules:
- Implement `NounCommand` trait
- Provide `new()` constructor
- `run_direct()` for default verb
- Verbs as private methods
- Read-only access to EngineState

## Testing Patterns

### Unit Test Placement

```rust
// In the same file as the code being tested
pub fn my_function() -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_function() {
        assert!(my_function());
    }
}
```

### Integration Test File Structure

```rust
// tests/cli/my_feature.rs
#[test]
fn test_my_feature_with_clean_workspace() {
    let temp = tempfile::TempDir::new().unwrap();
    
    let mut cmd = assert_cmd::Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("my-noun").arg("my-verb")
        .current_dir(temp.path());
    
    cmd.assert().success();
}
```

### Assertion Patterns

```rust
use predicates::prelude::*;

cmd.assert().success();
cmd.assert().failure();
cmd.assert().code(1);

// Check output
cmd.assert()
    .stdout(predicate::str::contains("expected output"));
```

## Related Guides

- [Pull Request Workflow](./02-pull-request-workflow.md) — commit format
- [Adding Features](./03-adding-features.md) — architectural patterns
- [Known Gotchas](./07-known-gotchas.md) — common mistakes
