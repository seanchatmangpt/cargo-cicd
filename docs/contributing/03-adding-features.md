# Adding Features

This guide covers common development tasks: adding nouns/verbs, extending EngineState, creating adapters, and defining policies.

## Adding a New Noun (Command Namespace)

A **noun** is a CLI namespace (e.g., `cargo cicd status`, `cargo cicd target`). Each noun contains multiple **verbs** (subcommands).

### Steps

1. **Create the noun module** in `src/nouns/mynoun.rs`:

```rust
use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};

pub struct MyNoun;

impl MyNoun {
    pub fn new() -> Self { Self }
}

impl Default for MyNoun {
    fn default() -> Self { Self::new() }
}

impl NounCommand for MyNoun {
    fn name(&self) -> &'static str { "mynoun" }
    fn about(&self) -> &'static str { "Brief description of what mynoun does" }
    
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(ShowVerb),
            Box::new(ApplyVerb),
        ]
    }
}

pub struct ShowVerb;

impl VerbCommand for ShowVerb {
    fn name(&self) -> &'static str { "show" }
    fn about(&self) -> &'static str { "Show mynoun state" }
    
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // Read EngineState via adapters
        // Perform logic
        // Emit output and events
        println!("mynoun show output");
        Ok(())
    }
}

pub struct ApplyVerb;

impl VerbCommand for ApplyVerb {
    fn name(&self) -> &'static str { "apply" }
    fn about(&self) -> &'static str { "Apply mynoun recommendations" }
    
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // Similar pattern to ShowVerb
        Ok(())
    }
}
```

2. **Register in `src/nouns/mod.rs`:**

```rust
pub mod mynoun;
```

3. **Register in `src/main.rs`:**

Find the `CliBuilder` setup and add:
```rust
let cli = cli.noun(nouns::mynoun::MyNoun::new());
```

4. **Add default verb injection** (optional, for bare noun to work):

In `main.rs::inject_default_verbs()`:
```rust
"mynoun" => Some("show"),
```

And in the `needs_default` check:
```rust
"mynoun" => {
    // Optionally run a default verb
    return Ok(());
}
```

5. **Test:**

```bash
cargo build
cargo cicd mynoun --help       # Should show verbs
cargo cicd mynoun show         # Should run
```

6. **Add integration test** in `tests/cli/`:

```rust
#[test]
fn test_mynoun_show_help() {
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(&["mynoun", "show", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("ALIVE")); // Forbidden term check
}
```

## Adding a New Adapter

An **adapter** translates one external source (git, cargo, filesystem, rustup) into internal `EngineState` dimensions. Adapters have **no business logic**—only translation.

### Steps

1. **Define or reuse State** in `src/engine/<dimension>.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct MySourceState {
    pub field1: String,
    pub field2: u32,
    pub field3: Vec<String>,
}
```

2. **Create adapter** in `src/adapters/my_source.rs`:

```rust
use anyhow::Result;
use crate::engine::MySourceState;
use std::process::Command;

pub struct MySourceAdapter;

impl MySourceAdapter {
    /// Query the external source and return populated state.
    ///
    /// This method has no side effects—it only reads.
    /// Errors are propagated; adapters don't swallow them.
    pub fn query() -> Result<MySourceState> {
        let raw = Self::external_call()?;
        Ok(MySourceState {
            field1: raw.field1,
            field2: raw.field2,
            field3: raw.fields,
        })
    }
}

fn external_call() -> Result<RawData> {
    // Run subprocess, read file, etc.
    let output = Command::new("my-tool")
        .args(&["--json"])
        .output()?;
    
    let raw: RawData = serde_json::from_slice(&output.stdout)?;
    Ok(raw)
}

#[derive(serde::Deserialize)]
struct RawData {
    field1: String,
    field2: u32,
    fields: Vec<String>,
}
```

3. **Register in `src/adapters/mod.rs`:**

```rust
pub mod my_source;
pub use my_source::MySourceAdapter;
```

4. **Call from a noun** in `src/nouns/my_noun.rs`:

```rust
let state = MySourceAdapter::query()?;
if state.field2 > 100 {
    println!("Warning: field2 is high");
}
```

5. **Test with fixture:**

```rust
#[test]
fn test_adapter_on_clean_workspace() {
    let fixture = FixtureWorkspace::clean();
    let state = MySourceAdapter::query()?;
    assert!(!state.field1.is_empty());
}
```

### Adapter Invariants

- **I1: Deterministic** — Same input → same output
- **I2: Idempotent** — Calling twice has no additional side effects
- **I3: No mutation** — Adapters read only
- **I4: Error propagation** — Errors bubble up; adapters don't swallow them
- **I5: Single source** — One adapter per external source

## Extending EngineState

If you need to track new workspace dimensions (e.g., dependency graph, build time metrics):

1. **Create a new state dimension** in `src/engine/new_dimension.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct NewDimensionState {
    pub metric1: f64,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub value: i32,
}
```

2. **Add to EngineState** in `src/engine/mod.rs`:

```rust
pub struct EngineState {
    // ... existing fields ...
    pub new_dimension: NewDimensionState,
}
```

3. **Create an adapter** to populate it (see "Adding a New Adapter" above).

4. **Use in nouns:**

```rust
let state = EngineState::default();
if state.new_dimension.metric1 > 0.5 {
    println!("High metric value");
}
```

5. **Document in CLAUDE.md** under "EngineState Design":

```
| **new_dimension** | Description | (none) | metric1, items |
```

6. **Test the state flow:**

```rust
#[test]
fn test_new_dimension_populated() {
    let state = EngineState::default();
    assert_eq!(state.new_dimension.metric1, 0.0); // Default
}
```

## Adding a Feature Flag

If you're gating new code behind a feature:

1. **Add to Cargo.toml** `[features]`:

```toml
[features]
my-feature = ["process-data"]  # If it depends on the engine
```

2. **Gate code with `#[cfg(...)]`:**

```rust
#[cfg(feature = "my-feature")]
pub fn my_feature_function() {
    // Only compiled when feature is enabled
}

#[cfg(not(feature = "my-feature"))]
pub fn my_feature_function() {
    // Stub implementation for when feature is disabled
}
```

3. **Test both paths:**

```bash
# Test without feature
cargo test

# Test with feature
cargo test --features my-feature
```

4. **Document in CLAUDE.md**:

```markdown
#### `my-feature` (disabled by default; depends on `process-data`)
When **not** enabled:
- ...

When **enabled:**
- ...
```

5. **Update feature projection test** in `tests/feature_projection.rs` to verify the feature contract.

## Adding an Autonomic Policy

Policies are smart recommendations that analyze `PolicyState` and return verdicts (pass/warn/fail). They never take destructive action by default (suggest mode).

### Steps

1. **Create policy** in `src/policies/my_policy.rs`:

```rust
use crate::engine::PolicyState;

pub struct MyPolicy;

#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub verdict: PolicyVerdict,
    pub message: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Fail,
}

impl MyPolicy {
    pub fn evaluate(state: &PolicyState) -> PolicyResult {
        // Read relevant dimensions from state
        // Apply decision logic
        // Return verdict + message + optional recommendation
        
        PolicyResult {
            verdict: PolicyVerdict::Pass,
            message: "All checks passed".into(),
            recommendation: None,
        }
    }
}
```

2. **Register in `src/policies/mod.rs`:**

```rust
pub mod my_policy;
pub use my_policy::{MyPolicy, PolicyResult, PolicyVerdict};
```

3. **Integrate in autonomic mode** (feature-gated):

```rust
#[cfg(feature = "autonomic")]
let result = policies::my_policy::MyPolicy::evaluate(&state.policies);
```

4. **Test the policy:**

```rust
#[test]
fn test_my_policy_pass() {
    let state = PolicyState::default();
    let result = MyPolicy::evaluate(&state);
    assert_eq!(result.verdict, PolicyVerdict::Pass);
}

#[test]
fn test_my_policy_warn() {
    let mut state = PolicyState::default();
    state.some_field = some_problematic_value();
    let result = MyPolicy::evaluate(&state);
    assert_eq!(result.verdict, PolicyVerdict::Warn);
    assert!(result.recommendation.is_some());
}
```

5. **Document in CLAUDE.md** under "Policies":

Describe the policy's decision rules, thresholds, and what recommendations it makes.

## Common Patterns

### Reading EngineState in a Verb

```rust
impl VerbCommand for MyVerb {
    fn run(&self, args: &VerbArgs) -> Result<()> {
        // Populate state via adapters
        let workspace = adapters::workspace_adapter::query()?;
        let git_phase = adapters::git_status::query()?;
        
        // Use state to decide behavior
        if git_phase.is_dirty {
            println!("Workspace is dirty; skipping operation");
            return Ok(());
        }
        
        println!("Workspace: {}", workspace.name);
        Ok(())
    }
}
```

### Emitting Evidence Events

If your code should emit XES evidence:

```rust
#[cfg(feature = "process-data")]
use crate::evidence::ProcessEvent;

// ...

#[cfg(feature = "process-data")]
let event = ProcessEvent {
    timestamp: std::time::SystemTime::now(),
    event_type: "noun_verb_executed",
    details: serde_json::json!({
        "noun": "mynoun",
        "verb": "myverb",
        "status": "success",
    }),
};
ProcessEvent::emit(&event)?;
```

### Using Fixtures in Tests

```rust
use tempfile::TempDir;

fn test_noun_on_clean_workspace() {
    let fixture = FixtureWorkspace::clean();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .arg("mynoun")
        .current_dir(&fixture.root)
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("expected output"));
}
```

## Checklist for Feature Completion

- [ ] **Code written** — noun, adapter, or policy implementation
- [ ] **Tests added** — unit and/or integration tests
- [ ] **Fixtures used** — tests isolated from external state
- [ ] **Feature-gated** — new engine code behind `process-data` / `autonomic` / etc.
- [ ] **Forbidden terms checked** — no ALIVE, Nehemiah, etc. in public output
- [ ] **Commit message** — follows `type(scope): description` format
- [ ] **CLAUDE.md updated** — if architectural changes
- [ ] **Invariants pass** — `cargo test --test invariants`
- [ ] **All tests pass** — `cargo test`
- [ ] **Code formatted** — `cargo fmt`
- [ ] **Lints pass** — `cargo clippy -- -D warnings`
