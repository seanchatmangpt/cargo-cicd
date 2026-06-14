# Adding Features

How to implement and structure new capabilities in cargo-cicd.

## Feature Categories

### 1. Public Commands (Nouns)

Add a new `cargo cicd <noun>` command.

**Files involved:**
- `src/nouns/your_noun.rs` — the noun implementation
- `src/nouns/mod.rs` — register the module
- `src/main.rs` — register in CliBuilder
- Tests in `tests/cli/` — command parsing tests
- `ontology/cargo-cicd.ttl` — semantic definition (for ggen)

**Example: Adding a `cargo cicd lint` noun**

```rust
// src/nouns/lint.rs
use clap_noun_verb::{NounCommand, VerbCommand};

pub struct LintNoun;

impl NounCommand for LintNoun {
    fn name() -> &'static str { "lint" }
    fn about() -> &'static str { "Run workspace linting checks" }
}

impl LintNoun {
    pub fn new() -> Self { Self }
    pub fn run_direct() -> anyhow::Result<()> {
        // Default verb behavior: lint show
        Self::show()
    }
    
    fn show() -> anyhow::Result<()> {
        // Implementation
        Ok(())
    }
}
```

Register in `src/nouns/mod.rs`:
```rust
pub mod lint;
```

Register in `src/main.rs`:
```rust
let cli = cli
    .noun(nouns::lint::LintNoun::new())
    // ... other nouns
```

**When to gate behind feature flags:**
- If the noun adds internal state inspection: gate behind `process-data`
- If the noun suggests actions: gate behind `autonomic`
- If the noun is experimental: gate behind `contrib`

### 2. State Extensions (EngineState)

Add new data to `EngineState` to enable nouns to access it.

**Files involved:**
- `src/engine/` — EngineState structure
- `src/state/` — state type definitions
- Corresponding `Adapter` — to populate the new state

**Architecture:**

```
External Source → Adapter → State Type → EngineState → Noun (read-only)
    (git)      →(adapter)→ (GitState)  → (engine) → (status shows it)
```

**Example: Adding workspace linting state**

```rust
// src/state/lint_state.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintState {
    pub issues: Vec<LintIssue>,
    pub total_issues: usize,
    pub high_priority: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub severity: LintSeverity,
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}
```

```rust
// src/engine/mod.rs
pub struct EngineState {
    pub workspace_state: WorkspaceState,
    pub lint_state: LintState,  // NEW
    // ... other state fields
}
```

```rust
// src/adapters/lint_scanner.rs
pub struct LintScannerAdapter;

impl LintScannerAdapter {
    pub fn scan(workspace_root: &Path) -> anyhow::Result<LintState> {
        // Scan workspace for lint issues
        // Return LintState
        Ok(LintState {
            issues: vec![],
            total_issues: 0,
            high_priority: 0,
        })
    }
}
```

### 3. Feature Flags

Guard new functionality behind feature flags.

**Flags in use:**
- `process-data` — enables Level 5 engine and internal state structures
- `autonomic` — implies `process-data`; enables policy suggestions
- `wasm4pm` — implies `process-data`; wasm4pm integration
- `contrib` — implies `process-data`; experimental features

**Add to `Cargo.toml`:**

```toml
[features]
my-new-feature = ["process-data"]
```

**Use in code:**

```rust
#[cfg(feature = "my-new-feature")]
pub fn my_new_function() {
    // Only compiled when feature is enabled
}
```

**Test with feature:**

```bash
cargo test --features my-new-feature
```

## Workflow: Adding a Complete Feature

### Example: Implement `cargo cicd lint show`

#### Phase 1: Design (No Code)

1. Update `CLAUDE.md` with the concept
2. Sketch the state structure (what data does lint need?)
3. Sketch the adapter (where does data come from?)
4. Sketch the noun (how is it displayed?)

#### Phase 2: Core Types

```rust
// src/state/lint_state.rs
#[derive(Debug, Clone, Default)]
pub struct LintState {
    pub issues: Vec<LintIssue>,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub rule: String,
    pub severity: String,
}
```

Register in `src/state/mod.rs`:
```rust
pub mod lint_state;
pub use lint_state::{LintState, LintIssue};
```

#### Phase 3: Adapter

```rust
// src/adapters/lint_scanner.rs
pub struct LintScannerAdapter;

impl LintScannerAdapter {
    pub fn scan(root: &Path) -> anyhow::Result<LintState> {
        // Read Cargo.toml, scan for issues
        Ok(LintState { issues: vec![] })
    }
}
```

Register in `src/adapters/mod.rs`:
```rust
pub mod lint_scanner;
```

#### Phase 4: EngineState Integration

```rust
// src/engine/mod.rs
pub struct EngineState {
    pub lint_state: LintState,
    // ... other fields
}

impl EngineState {
    pub fn new(root: &Path) -> anyhow::Result<Self> {
        let lint_state = LintScannerAdapter::scan(root)?;
        Ok(Self {
            lint_state,
            // ... init other fields
        })
    }
}
```

#### Phase 5: Noun (CLI)

```rust
// src/nouns/lint.rs
use crate::engine::EngineState;

pub struct LintNoun;

impl NounCommand for LintNoun {
    fn name() -> &'static str { "lint" }
    fn about() -> &'static str { "Lint workspace" }
}

impl LintNoun {
    pub fn new() -> Self { Self }
    
    pub fn run_direct() -> anyhow::Result<()> {
        Self::show()
    }
    
    fn show() -> anyhow::Result<()> {
        let root = std::env::current_dir()?;
        let engine = EngineState::new(&root)?;
        
        println!("Lint Issues: {}", engine.lint_state.issues.len());
        for issue in &engine.lint_state.issues {
            println!("  - {} ({})", issue.rule, issue.severity);
        }
        
        Ok(())
    }
}
```

Register in `src/main.rs`:
```rust
let cli = cli.noun(nouns::lint::LintNoun::new());
```

#### Phase 6: Tests

```rust
// tests/cli/lint_command.rs
#[test]
fn test_lint_show_command() {
    // Create temp workspace with fixtures
    let temp = tempfile::TempDir::new().unwrap();
    
    // Run the command
    let mut cmd = assert_cmd::Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("lint").arg("show").current_dir(temp.path());
    
    // Assert output
    cmd.assert().success();
}
```

#### Phase 7: Documentation

Update relevant docs:
- Add example in README.md
- Add to CLAUDE.md if architecture changed
- Update CONTRIBUTING.md if new patterns introduced
- Add code comments for non-obvious logic

## Extending Adapters

When adding a new adapter:

1. **One responsibility per adapter** — GitStatusAdapter only touches git, not cargo
2. **No business logic** — adapters translate, not transform
3. **Return internal types** — adapters populate EngineState types, not display types
4. **Handle errors gracefully** — return `anyhow::Result<T>`, use context

```rust
// GOOD: One responsibility
pub struct LintScannerAdapter;
impl LintScannerAdapter {
    pub fn scan(root: &Path) -> anyhow::Result<LintState> { }
}

// BAD: Multiple responsibilities (scanning + formatting + displaying)
pub struct LintAdapter;
impl LintAdapter {
    pub fn scan_and_display() { } // Violates single responsibility
}
```

## Extending EngineState

When adding a new field to `EngineState`:

1. **Define the state type** — create a new type in `src/state/`
2. **Create an adapter** — to populate it (even if it's a simple initialization)
3. **Make the field public** — so nouns can read it
4. **Initialize in `EngineState::new()`** — call the adapter
5. **Document the field** — comment what it represents

```rust
pub struct EngineState {
    /// Workspace lint issues discovered by LintScannerAdapter
    pub lint_state: LintState,
}
```

## Feature Flag Decisions

| Feature | When to Use |
|---------|------------|
| `process-data` | New state types, adapters, or engine functionality |
| `autonomic` | Suggestions, policies, or "recommend" verbs |
| `wasm4pm` | Evidence emission, oracle invocation, receipt validation |
| `contrib` | Experimental features not yet stabilized |

**Example: Gating a suggestion**

```rust
#[cfg(feature = "autonomic")]
fn suggest_fixes() -> Vec<String> {
    // Only compiled with autonomic feature
}
```

**Example: Gating a noun**

```rust
pub mod lint; // Always available

#[cfg(feature = "autonomic")]
pub mod linter; // Linter suggestions only with autonomic
```

## Testing New Features

### Unit Tests
```bash
cargo test --test invariants         # Boundary invariants
cargo test --lib                     # Library unit tests
```

### Integration Tests
```bash
cargo test --test cli                # CLI parsing
cargo test --test your_feature_name  # Feature-specific tests
```

### With Feature Flags
```bash
cargo test --features autonomic
cargo test --all-features
```

### Against Fixtures
Use workspace fixtures in `tests/fixtures/`:
```rust
#[test]
fn test_with_fixture() {
    let fixture = "tests/fixtures/clean_workspace";
    std::env::set_current_dir(fixture).unwrap();
    // Your test
}
```

## No Breaking Changes

cargo-cicd follows semantic versioning. When adding features:

- **Patch version** — backwards compatible fixes
- **Minor version** — backwards compatible features
- **Major version** — breaking changes (rare)

New features should:
- Not change existing command signatures
- Not break existing cicd.toml files
- Support graceful degradation if a feature is missing

## Related Guides

- [Code Style & Patterns](./04-code-style.md) — naming conventions
- [Documentation Standards](./05-documentation-standards.md) — what to document
- [Known Gotchas](./07-known-gotchas.md) — common pitfalls
- [CLAUDE.md](../../CLAUDE.md) — architecture reference
