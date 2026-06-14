# Documentation Standards

This guide covers how to document new features, when to update CLAUDE.md, and how to mark visibility levels.

## Documenting New Features

### Public API Documentation

All public types and functions must have doc comments with examples:

```rust
/// Parse and validate a Cargo manifest.
///
/// This adapter reads Cargo.toml and extracts workspace metadata
/// including member crates, edition, and rust-version.
///
/// # Arguments
///
/// * `root` - Path to the workspace root (directory containing Cargo.toml)
///
/// # Returns
///
/// A populated `WorkspaceState` reflecting the current manifest.
///
/// # Example
///
/// ```rust
/// use cargo_cicd::adapters::CargoMetadataAdapter;
///
/// let state = CargoMetadataAdapter::query()?;
/// println!("Workspace: {}", state.name);
/// assert!(state.members.len() > 0);
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Cargo.toml does not exist in the workspace root
/// - Cargo.toml is malformed or unreadable
/// - The workspace contains invalid manifest syntax
///
/// # Panics
///
/// Does not panic. All errors are recoverable.
pub fn query() -> anyhow::Result<WorkspaceState> {
    // ...
}
```

### Feature-Gated Documentation

If your feature is gated behind a flag, document the gate:

```rust
/// Run autonomic policy evaluation.
///
/// This is only available when the `autonomic` feature is enabled.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "autonomic")]
/// # {
/// let state = EngineState::default();
/// let verdict = autonomic::evaluate(&state)?;
/// # }
/// ```
#[cfg(feature = "autonomic")]
pub fn evaluate(state: &EngineState) -> anyhow::Result<PolicyVerdict> {
    // ...
}
```

### Help Text in Nouns/Verbs

Keep help text user-friendly and jargon-free. Avoid internal implementation details.

```rust
impl VerbCommand for StatusShowVerb {
    fn name(&self) -> &'static str { "show" }
    
    fn about(&self) -> &'static str {
        "Display workspace CI/CD readiness: git state, target size, test status"
    }
    
    fn run(&self, args: &VerbArgs) -> Result<()> {
        // Implementation
    }
}
```

Bad help text (too technical, internal jargon):

```rust
"Emit EngineState dimensions and check PolicyState verdicts"  // Bad
```

## Updating CLAUDE.md

CLAUDE.md is the source of truth for internal architecture. Update it when:

1. **Adding a new EngineState dimension**
2. **Adding a new adapter with special patterns**
3. **Changing the noun-verb grammar**
4. **Adding feature flags**
5. **Adding new test categories**
6. **Documenting new architectural decisions**

### Where to Add Documentation

- **New dimensions**: Update the `EngineState` table under "State Dimensions"
- **New adapters**: Add to "Adapter Catalog" with source and responsibility
- **Patterns**: Add to "Common Workflows" if it's reusable
- **Policies**: Update "Policies" section with decision rules
- **Feature gates**: Update "Feature Flags" section
- **Test types**: Add to "Test Hierarchy"

### Example: Documenting a New Dimension

In CLAUDE.md, find the EngineState table and add:

```markdown
| **my_dimension** | Brief purpose | `feature-gate` (or none) | Example: field1, field2 |
```

Then add a section under "EngineState Design":

```markdown
#### my_dimension

Tracks [what it tracks]. Populated by [adapter name].

**Fields:**
- `field1: Type` — Description
- `field2: Type` — Description

**Invariants:**
- [I1: Invariant description]
- [I2: Another invariant]
```

### Example: Documenting a New Adapter

Add to "Adapter Catalog":

```markdown
| MySourceAdapter | `my_source.rs` | Reads X, populates MyDimensionState | `query()` | Single source, immutable output |
```

Then add detailed notes:

```markdown
#### MySourceAdapter

Queries [external source] and translates to MyDimensionState.

**External Source:** [description]

**Responsibility:** Populate fields: field1, field2

**Invariants:**
- [Specific invariants for this adapter]
```

## Visibility Levels

### Public API Surface

These are visible to users and must be stable:

- **Noun names** (e.g., `cargo cicd status`)
- **Verb names** (e.g., `show`, `apply`)
- **Help text** (output of `--help`)
- **Exit codes** (0 for success, nonzero for failure)
- **Output format** (what gets printed to stdout)
- **Configuration format** (cicd.toml schema)

### Internal API Surface

These are implementation details and can change:

- **EngineState structure** (internal snapshot)
- **Adapter implementations** (how we get data)
- **Policy logic** (decision rules, internal thresholds)
- **Event formats** (XES/JSON structure)

### Private/Forbidden Visibility

These should **never** appear in public output:

- ALIVE, Nehemiah, CONSTRUCT8, Instinct8
- Inspection Gate, Cargo Court, AGI, Truex, Field8
- "wall" or other internal terminology
- Process-data engine details (unless explicitly documented as public)

### Checking Visibility

Before committing, ensure no forbidden terms appear:

```bash
# Check all public help text
cargo cicd --help
cargo cicd status --help
cargo cicd target --help
# ... etc ...

# Verify with the invariants test
cargo test --test invariants invariant_public_boundary
```

## Writing Clear Examples

### Good Example (Runnable, Self-Contained)

```rust
/// # Example
///
/// ```rust
/// use cargo_cicd::adapters::GitStatusAdapter;
///
/// let state = GitStatusAdapter::query()?;
/// if state.is_dirty {
///     eprintln!("Warning: workspace is dirty");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
```

### Bad Examples

```rust
// Bad: unclear what the return value is
/// let state = GitStatusAdapter::query();

// Bad: depends on external setup
/// let state = GitStatusAdapter::query(&workspace_path)?;
/// assert_eq!(state.branch, "main");  // Assumes we're on main

// Bad: too much boilerplate
/// # use std::path::PathBuf;
/// # let workspace_path = PathBuf::from("/tmp/test");
/// # std::fs::create_dir_all(&workspace_path)?;
/// # std::process::Command::new("git").args(&["init"]).current_dir(&workspace_path).output()?;
/// let state = GitStatusAdapter::query()?;
```

## Documentation Checklist

When adding a feature:

- [ ] **Public functions have doc comments** with examples, errors, and invariants
- [ ] **Help text is user-friendly** (no internal jargon)
- [ ] **No forbidden terms in public output** (test with `cargo test --test invariants`)
- [ ] **CLAUDE.md updated** if architectural changes
- [ ] **New dimensions documented** in CLAUDE.md's EngineState table
- [ ] **Examples are runnable** (use `#[doc = include_str!(...)]` if non-trivial)
- [ ] **Feature-gated code documented** with `#[cfg(...)]` notation
- [ ] **Visibility clearly marked** (public API vs. internal implementation)
- [ ] **README.md updated** if user-facing behavior changes

## Example: Complete Documentation

Here's how a new feature should be documented end-to-end:

### In Code (src/adapters/my_new.rs)

```rust
//! My New Adapter
//!
//! This adapter queries [external source] and populates MyNewState.

use anyhow::Result;
use crate::engine::MyNewState;

/// Query [external source] and return populated state.
///
/// # Returns
///
/// A snapshot of [what's tracked], immutable and deterministic.
///
/// # Example
///
/// ```rust
/// use cargo_cicd::adapters::MyNewAdapter;
///
/// let state = MyNewAdapter::query()?;
/// println!("Metric: {}", state.metric);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns `Err` if [external source] is unavailable.
pub struct MyNewAdapter;

impl MyNewAdapter {
    pub fn query() -> Result<MyNewState> {
        // ...
    }
}
```

### In CLAUDE.md (EngineState section)

Add to the table:

```markdown
| **my_new_dimension** | Tracks XYZ | (none) | metric: f64, items: Vec<Item> |
```

And add a section:

```markdown
#### my_new_dimension

Populated by `MyNewAdapter::query()`. Immutable snapshot of [source].

**Invariants:**
- I1: Metric is always >= 0
- I2: Items are deduplicated
```

### In Help Text (src/nouns/my_new.rs)

```rust
impl VerbCommand for MyNewShowVerb {
    fn name(&self) -> &'static str { "show" }
    fn about(&self) -> &'static str { "Display [what the user cares about]" }
    
    fn run(&self, args: &VerbArgs) -> Result<()> {
        // ...
    }
}
```

### In a Test (tests/my_new.rs)

```rust
#[test]
fn test_show_verb_displays_metric() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(&["my-new", "show"])
        .output()
        .unwrap();
    
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Metric:"));
}
```

## Further Reading

- [CLAUDE.md](../../CLAUDE.md) — where to document architectural decisions
- [04-code-style.md](./04-code-style.md) — doc comment format specifics
- [03-adding-features.md](./03-adding-features.md) — patterns for common additions
