# Known Gotchas

Common pitfalls and how to avoid them.

## Forbidden Terms in Public Output

**The Rule:** The following terms must **never** appear in user-visible output (help text, stdout, error messages):

```
ALIVE
Nehemiah
CONSTRUCT8
Instinct8
Inspection Gate
Cargo Court
AGI
Truex
Field8
wall
```

These are internal implementation details and architectural references. Users should see only: "CI/CD helper", "workspace cleaner", "test runner", etc.

### How to Catch This

The `invariants` test enforces this:

```bash
cargo test --test invariants invariant_public_boundary
```

This test runs every `--help` command and checks for forbidden terms.

### Example: The Bug

```rust
// Bad: Nehemiah is forbidden
println!("Running Nehemiah workspace scan...");

// Good
println!("Scanning workspace...");
```

If you see this test fail, search for the term and remove it:

```bash
grep -r "ALIVE" src/
grep -r "Nehemiah" src/
# ... remove the offending line ...
```

## State Mutation Patterns

**The Rule:** `EngineState` is an immutable snapshot. Mutations happen only through adapters → `CicdTomlWriter`.

### Anti-Pattern 1: Mutating EngineState in a Verb

```rust
// Bad: verbs are read-only consumers
impl VerbCommand for MyVerb {
    fn run(&self, args: &VerbArgs) -> Result<()> {
        let mut state = EngineState::default();
        state.target.size_gb = 100.0;  // ← Do not mutate!
        Ok(())
    }
}
```

**Fix:** Have the adapter populate the state, then read it:

```rust
impl VerbCommand for MyVerb {
    fn run(&self, args: &VerbArgs) -> Result<()> {
        let state = adapters::target_scanner::query()?;  // Adapter populates
        println!("Target size: {} GB", state.size_gb);    // Verb reads
        Ok(())
    }
}
```

### Anti-Pattern 2: Adapter Returns Mutable State

```rust
// Bad: state is modified after construction
pub fn query() -> Result<TargetState> {
    let mut state = TargetState::default();
    state.size_gb = 10.0;
    state.size_gb = 20.0;  // Why mutate twice?
    Ok(state)
}
```

**Fix:** Build immutable values directly:

```rust
pub fn query() -> Result<TargetState> {
    let size = calculate_total_size()?;
    Ok(TargetState {
        size_gb: size,
        // ... other fields ...
    })
}
```

### Anti-Pattern 3: Circular Dependency in State

```rust
// Bad: MyState depends on AnotherState, which is also in EngineState
pub struct MyState {
    pub dependency: AnotherState,  // ← Redundancy!
}
```

**Fix:** Reference by ID or path, never embed state:

```rust
pub struct MyState {
    pub affected_crates: Vec<String>,  // Just names, not full state
}
```

## Test Isolation Failures

**The Rule:** Tests must not depend on external state: previous test runs, git commits, filesystem files, environment variables.

### Anti-Pattern 1: Test Depends on External Git State

```rust
#[test]
fn test_git_phase_detection() {
    // Bad: assumes we're in a real git repo with a main branch
    let state = adapters::git_status::query()?;
    assert_eq!(state.branch, "main");  // ← What if we're on a different branch?
}
```

**Fix:** Use `FixtureWorkspace` to create isolated state:

```rust
#[test]
fn test_git_phase_detection() {
    let fixture = FixtureWorkspace::clean();  // ← Isolated fixture
    // The fixture is a fresh git repo; we control its state
    let state = adapters::git_status::query_in(&fixture.root)?;
    assert_eq!(state.branch, "main");  // Safe assumption in the fixture
}
```

### Anti-Pattern 2: Test Leaves Behind Temp Files

```rust
#[test]
fn test_target_scanning() {
    std::fs::create_dir_all("/tmp/test-workspace/target")?;
    // ← /tmp/test-workspace is left behind after test!
    
    let state = adapters::target_scanner::query()?;
    assert!(state.size_gb > 0.0);
}
```

**Fix:** Use `tempfile::TempDir` which cleans up automatically:

```rust
#[test]
fn test_target_scanning() {
    let temp = tempfile::TempDir::new()?;
    let workspace_root = temp.path();  // Created automatically
    
    std::fs::create_dir_all(workspace_root.join("target"))?;
    // temp is dropped here, and workspace_root is deleted
    
    // Or use FixtureWorkspace which does this internally
    let fixture = FixtureWorkspace::with_target_over_limit();
    let state = adapters::target_scanner::query_in(&fixture.root)?;
    assert!(state.size_gb > 0.0);
}
```

### Anti-Pattern 3: Test Assumes Environment Variable

```rust
#[test]
fn test_uses_environment_config() {
    // Bad: CI or other developers might not have MY_CONFIG set
    let config = std::env::var("MY_CONFIG").expect("MY_CONFIG required");
    assert!(!config.is_empty());
}
```

**Fix:** Either make the test not depend on environment, or mock it:

```rust
#[test]
fn test_reads_config_from_file() {
    let fixture = FixtureWorkspace::clean();
    let config_path = fixture.root.join("my.config");
    std::fs::write(&config_path, "test_data")?;
    
    let config = read_config(&config_path)?;
    assert!(!config.is_empty());
}
```

## Feature Flag Gating Mistakes

**The Rule:** Use `#[cfg(feature = "...")]` at compile time, not runtime checks.

### Anti-Pattern 1: Runtime Feature Check

```rust
// Bad: feature is checked at runtime
pub fn use_engine() -> Result<()> {
    if cfg!(feature = "process-data") {
        let state = EngineState::default();
        println!("{:?}", state);
    } else {
        println!("Feature disabled");
    }
    Ok(())
}
```

**Fix:** Compile out the code entirely:

```rust
#[cfg(feature = "process-data")]
pub fn use_engine() -> Result<()> {
    let state = EngineState::default();
    println!("{:?}", state);
    Ok(())
}

#[cfg(not(feature = "process-data"))]
pub fn use_engine() -> Result<()> {
    println!("Feature disabled");
    Ok(())
}
```

### Anti-Pattern 2: Feature-Gated Code Without Stub

```rust
// Bad: code doesn't compile without the feature
pub fn query() -> Result<State> {
    #[cfg(feature = "process-data")]
    {
        // Only this exists
        // If feature is off, what does query() return?
    }
}
```

**Fix:** Provide a stub implementation:

```rust
#[cfg(feature = "process-data")]
pub fn query() -> Result<State> {
    // Real implementation
    Ok(State::default())
}

#[cfg(not(feature = "process-data"))]
pub fn query() -> Result<State> {
    // Stub: return default or error
    Err(anyhow::anyhow!("process-data feature required"))
}
```

## Adapter Query Mistakes

**The Rule:** Adapters query external sources once and return immutable results. No caching, no side effects.

### Anti-Pattern 1: Adapter Modifies External State

```rust
// Bad: adapter calls git commands that modify the repo
pub fn query() -> Result<State> {
    // This silently resets uncommitted changes!
    Command::new("git").args(&["reset", "--hard"]).output()?;
    
    let output = Command::new("git").args(&["status"]).output()?;
    Ok(parse_status(&output.stdout)?)
}
```

**Fix:** Adapters only read, never write:

```rust
pub fn query() -> Result<State> {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    Ok(parse_status(&output.stdout)?)
}
```

### Anti-Pattern 2: Adapter Caches Incorrectly

```rust
// Bad: static cache can be stale
lazy_static::lazy_static! {
    static ref CACHE: State = expensive_query().unwrap();
}

pub fn query() -> Result<State> {
    Ok(CACHE.clone())  // ← Always returns old data!
}
```

**Fix:** If caching is needed, validate freshness:

```rust
pub fn query() -> Result<State> {
    // Query fresh data every time
    // If optimization is needed, add a cicd.toml cache layer at a higher level
    expensive_query()
}
```

## Common cicd.toml Mistakes

**The Rule:** cicd.toml is a state carrier. It should be written by adapters only, read by nouns.

### Anti-Pattern 1: Manually Editing cicd.toml

```bash
# Bad: editing by hand introduces inconsistencies
$ cat >> cicd.toml << EOF
[state]
target_size_gb = 99.0  # Hand-edited
