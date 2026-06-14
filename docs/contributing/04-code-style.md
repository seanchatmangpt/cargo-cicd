# Code Style & Patterns

This guide covers Rust conventions, naming patterns, comment guidelines, and module organization used in cargo-cicd.

## Rust Conventions

cargo-cicd follows standard Rust idioms. Key rules:

### Formatting

- Use `cargo fmt` before committing (non-negotiable)
- Line length: soft limit 100 chars, hard limit 120 chars
- Indentation: 4 spaces
- No trailing whitespace

```bash
cargo fmt
```

### Linting

- Use `cargo clippy -- -D warnings` to catch common mistakes
- Fix all clippy warnings before committing

```bash
cargo clippy -- -D warnings
```

### Type Annotations

- Explicit where clarity matters (public APIs, complex logic)
- Implicit for obvious local bindings

```rust
// Good: public API
pub fn query() -> anyhow::Result<MySourceState> {
    // ...
}

// Good: obvious binding
let count = items.len();  // Could also be `let count: usize = ...`

// Bad: unnecessary annotation
let items: Vec<Item> = vec![];  // Vec::new() is clearer
```

### Error Handling

- Use `anyhow::Result<T>` for fallible operations
- Propagate errors with `?` operator
- Add context with `.context("message")?` for crucial errors
- No panics in library code; only in main() or tests as needed

```rust
use anyhow::Result;

pub fn parse_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("failed to read config file")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

### Ownership & Borrowing

- Prefer owned values (`String`, `Vec<T>`) in structs
- Use references (`&str`, `&[T]`) in function parameters
- Avoid excessive cloning; use references when possible

```rust
// Good: struct owns its data
pub struct Config {
    name: String,
    items: Vec<String>,
}

// Good: function borrows
pub fn process(config: &Config, name: &str) -> Result<()> {
    // ...
}

// Avoid: unnecessary cloning
pub fn bad(config: Config) {
    let name = config.name.clone();  // ← unnecessary
}
```

## Naming Patterns

### Modules

- snake_case for module names
- One public type per adapter module (e.g., `GitStatusAdapter`)
- One main state struct per engine dimension (e.g., `GitPhaseState`)

```
src/
├── adapters/
│   ├── git_status.rs          ← GitStatusAdapter
│   ├── target_scanner.rs       ← TargetScannerAdapter
│   └── changed_file_detector.rs ← ChangedFileDetector
├── engine/
│   ├── git_phase_state.rs      ← GitPhaseState
│   ├── target_state.rs         ← TargetState
│   └── ...
└── nouns/
    ├── status.rs               ← StatusNoun (with verbs)
    └── target.rs               ← TargetNoun (with verbs)
```

### Types

- PascalCase for types, traits, and enums
- Nouns for types (`Status`, `Adapter`, `State`)
- Adjectives for traits (`Queryable`, `Serializable`, `Default`)

```rust
pub struct GitStatusAdapter;      // Good
pub struct GIT_STATUS;             // Bad (screaming)
pub struct get_git_status;         // Bad (function-like)

pub trait Queryable { }            // Good
pub trait Adapter { }              // Good
```

### Functions & Methods

- snake_case for function and method names
- Verbs for actions (`parse`, `query`, `emit`)
- Nouns for getters (`name`, `version`, `count`)

```rust
// Good: verb
pub fn query() -> Result<State> { }

// Good: getter
pub fn version(&self) -> &str { }

// Bad: screaming constant function
pub fn GET_VERSION() { }

// Bad: adjective as method
pub fn empty() -> Self { }  // Use `fn new()` instead
```

### Constants

- SCREAMING_SNAKE_CASE for constants
- Descriptive names

```rust
const DEFAULT_TARGET_LIMIT_GB: f64 = 10.0;
const FORBIDDEN_TERMS: &[&str] = &[
    "ALIVE",
    "Nehemiah",
    "CONSTRUCT8",
];
```

### Variable Names

- Descriptive and clear
- Avoid single letters except in loops/temporary bindings

```rust
// Good
let workspace_root = "/path/to/workspace";
let test_count = 42;

// Okay: loop variable
for item in items { }

// Bad: unclear
let ws = "/path/to/workspace";
let tc = 42;
```

## Comments & Documentation

### Doc Comments (Public APIs)

Use `///` for public items. Include examples and invariants.

```rust
/// Query the git repository state.
///
/// Returns a snapshot of the current branch, dirty status, and ahead/behind counts.
/// The state is immutable and reflects `git status --porcelain` at query time.
///
/// # Example
///
/// ```rust
/// let state = GitStatusAdapter::query()?;
/// println!("Branch: {}", state.branch);
/// ```
///
/// # Errors
///
/// Returns an error if git is not available or the cwd is not a git repository.
pub fn query() -> Result<GitPhaseState> {
    // ...
}
```

### Inline Comments

Use `//` for internal logic. Only comment non-obvious behavior.

```rust
impl MyAdapter {
    pub fn query() -> Result<State> {
        // Group files by crate before processing.
        // This avoids redundant manifest lookups.
        let mut by_crate: HashMap<String, Vec<Path>> = HashMap::new();
        
        for file in changed_files {
            let crate_name = manifest_for(&file)?;
            by_crate.entry(crate_name).or_insert_with(Vec::new).push(file);
        }
        
        // ... process by_crate ...
    }
}
```

### Avoid Comments

- Don't repeat what the code says
- Don't comment type annotations (let the type speak)
- Don't comment obvious loops/conditionals

```rust
// Bad: comment repeats the code
let count = items.len();  // Get the length of items

// Bad: obvious
if x > 0 {  // Check if x is positive
    // ...
}

// Bad: type is clear
let mut state: State = State::default();  // Initialize state

// Good: explains why, not what
let limit = 20_000_000_000;  // 20 GB; limit chosen empirically
```

### TODO Comments

Acceptable in limited cases. Must include context.

```rust
// TODO(issue #42): Implement parallel test execution
// Currently gated by cicd.toml race conditions.
pub fn run_tests_parallel() -> Result<()> {
    // ... serial implementation ...
}
```

## Module Organization

### Public API Layout

```rust
// Good: public API first, then private implementation
pub struct MyAdapter;

impl MyAdapter {
    pub fn query() -> Result<State> { }
    pub fn query_in(path: &Path) -> Result<State> { }
    // ... public methods ...
}

// Private implementation
impl MyAdapter {
    fn internal_parse() -> Result<()> { }
    fn validate() -> Result<()> { }
}

// Free functions (private)
fn external_call() -> Result<RawData> { }
```

### Imports

- Group standard library, external crates, then internal modules
- Alphabetize within groups
- Avoid glob imports (`use *`)

```rust
// Good: organized imports
use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde_json;
use walkdir::WalkDir;

use crate::engine::GitPhaseState;
use crate::adapters;
```

### Visibility

- Mark public APIs with `pub`
- Keep everything else private by default
- Use `pub(crate)` for internal APIs shared across modules

```rust
// Public, part of the library surface
pub struct MyAdapter;
pub fn query() -> Result<State> { }

// Private to this module
fn internal_helper() { }

// Visible to other crate modules
pub(crate) fn shared_utility() { }
```

## Feature-Gated Code

Use `#[cfg(...)]` attributes, not runtime checks, for feature gates.

```rust
// Good: compile-time gating
#[cfg(feature = "process-data")]
pub fn use_engine() -> Result<()> {
    let state = EngineState::default();
    Ok(())
}

// Bad: runtime check in always-compiled code
pub fn use_engine() -> Result<()> {
    if cfg!(feature = "process-data") {
        let state = EngineState::default();
    }
    Ok(())
}
```

## Testing Code Style

### Test Organization

```rust
#[test]
fn test_adapter_on_clean_workspace() {
    // Arrange
    let fixture = FixtureWorkspace::clean();
    
    // Act
    let state = MyAdapter::query().unwrap();
    
    // Assert
    assert!(!state.is_empty());
}
```

### Test Names

- Descriptive, verb-first: `test_<subject>_<condition>_<expected>`
- Example: `test_adapter_on_dirty_workspace_returns_warn`

```rust
#[test]
fn test_policy_on_large_target_returns_warn() { }

#[test]
fn test_noun_with_missing_manifest_exits_nonzero() { }
```

### Assertions

- Use descriptive assertion messages

```rust
assert_eq!(state.branch, "main", "expected main branch, not {}", state.branch);
assert!(output.contains("expected text"), "output missing 'expected text': {}", output);
```

## Example Module

Here's a complete, well-organized module:

```rust
//! Git repository state detection.
//!
//! This adapter queries `git status --porcelain` and `git rev-parse`
//! to populate GitPhaseState.

use anyhow::Result;
use std::process::Command;
use crate::engine::GitPhaseState;

/// Query the git repository state.
///
/// # Errors
/// Returns an error if git is not available or not in a git repository.
pub struct GitStatusAdapter;

impl GitStatusAdapter {
    pub fn query() -> Result<GitPhaseState> {
        let branch = Self::get_branch()?;
        let is_dirty = Self::check_dirty()?;
        
        Ok(GitPhaseState {
            branch,
            is_dirty,
            // ...
        })
    }
}

// Private implementation
impl GitStatusAdapter {
    fn get_branch() -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    }
    
    fn check_dirty() -> Result<bool> {
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .output()?;
        
        Ok(!output.stdout.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_on_clean_repo() {
        let fixture = FixtureWorkspace::clean();
        let state = GitStatusAdapter::query().unwrap();
        assert!(!state.is_dirty);
    }
}
```

## Checklist

Before committing:

- [ ] `cargo fmt` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No single-letter variables (except loop counters)
- [ ] Public APIs have doc comments with examples
- [ ] Comments explain *why*, not *what*
- [ ] No `TODO`s without issue numbers
- [ ] Tests follow `test_<subject>_<condition>_<expected>` naming
- [ ] Imports are organized and alphabetized
- [ ] Feature gates use `#[cfg(...)]`, not runtime checks
