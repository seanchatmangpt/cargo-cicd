# Test Coverage Guide for cargo-cicd

This document defines coverage philosophy, measurement strategy, and module-level targets for cargo-cicd. It ensures new contributions maintain quality while acknowledging that 100% coverage has diminishing returns.

## 1. Coverage Philosophy

### Goals
- **High coverage, not perfection**: We target 80%+ for new code, not 100%
- **Branch coverage emphasis**: Test both happy paths AND error paths
- **Honest reporting**: Skipping untestable code (e.g., `panic!()`, unreachable edge cases) is acceptable
- **Improve or maintain**: New code must meet targets; existing code must not regress

### Rationale
- 100% coverage is costly and often false security (passing lines != correct behavior)
- Branch coverage catches missing error handling and conditional logic
- Focus on critical modules first (engine, adapters, policies)
- CLI modules (nouns) are lower priority — tested primarily via integration tests

### What We Skip
- Unreachable code branches (ensured by type system or guarantees)
- `panic!()` and `unreachable!()` paths (intentionally unrecoverable)
- Logging instrumentation (calls to `tracing::*` macros)
- Generated code stubs (e.g., ggen-produced noun boilerplate)
- Feature-gated wasm4pm oracles (external integration point, not testable in isolation)

## 2. How to Measure Coverage

### Option A: tarpaulin (Recommended for Quick Local Checks)

**Installation:**
```bash
cargo install cargo-tarpaulin
```

**Basic Run:**
```bash
# Generate HTML coverage report
cargo tarpaulin --features advanced --out Html --output-dir coverage

# Run with timeout (tarpaulin uses LLVM, can be slow)
cargo tarpaulin --features advanced --out Html --timeout 300

# Run coverage for a specific package
cargo tarpaulin --features advanced -p cargo-cicd --out Html
```

**Interpreting Output:**
- **Line Coverage**: % of source lines executed (easiest to game — not the main metric)
- **Branch Coverage**: % of conditional branches taken (more meaningful — prefer this)
- **Function Coverage**: % of functions called (coarse-grained)
- The HTML report shows uncovered lines in red and partially-covered branches in yellow

**Example Report Reading:**
```
src/adapters/git_status.rs: 85% (branch)
  Lines 23–30: 100% (git branch detection)
  Lines 42–51: 60% (git status parsing — missing some XY patterns)
  Lines 57–63: 0% (is_dirty method — not directly tested, only indirectly)
```

Action: Add tests for lines 42–51 (error paths) and consider adding explicit tests for `is_dirty()`.

### Option B: llvm-cov (More Accurate, More Verbose)

**Installation:**
```bash
cargo install cargo-llvm-cov
```

**Run with Region Coverage:**
```bash
# Generate HTML report with region coverage (most accurate)
cargo llvm-cov --features advanced --html

# Generate text summary
cargo llvm-cov --features advanced

# Generate JSON for CI integration
cargo llvm-cov --features advanced --json --output-path coverage.json
```

**Advantages:**
- More accurate branch coverage (region-based)
- Better feature interaction detection
- JSON output suitable for CI/CD tooling

**Disadvantages:**
- Slower than tarpaulin
- Requires Rust nightly for some features (check `cargo llvm-cov --help`)

### Continuous Integration

**Add to CI pipeline:**
```bash
# In your CI config (GitHub Actions, GitLab CI, etc.):
cargo llvm-cov --features advanced --json --output-path coverage.json

# Assert minimum threshold
MIN_COVERAGE=78
ACTUAL=$(jq '.data[0].summary.regions.percent' coverage.json)
if (( $(echo "$ACTUAL < $MIN_COVERAGE" | bc -l) )); then
  echo "Coverage $ACTUAL% below minimum $MIN_COVERAGE%"
  exit 1
fi
```

## 3. Module Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| `engine/` | **90%+** | State aggregate root; critical for correctness; all dimensions must be tested |
| `adapters/` | **85%+** | Boundary translators; must handle external failures gracefully |
| `policies/` | **80%+** | Autonomic logic; error paths and edge cases essential |
| `advanced/` | **75%+** | Complex, new modules; diminishing returns past 75% |
| `nouns/` | **70%+** | CLI layer; tested primarily via integration tests; 70% sufficient |
| `integrations/` | **75%+** | Seams to external systems (wasm4pm, metrics); focus on contract boundaries |
| `state/` | **85%+** | Internal state tracking; serialization/deserialization must be robust |

### Module Details

#### `src/engine/` (Target: 90%+)
**Critical modules:** `engine.rs`, `workspace_state.rs`, `git_phase_state.rs`, `test_plan_state.rs`

**Must test:**
- All state initialization paths (default, with data, with errors)
- State transitions (empty → populated, valid → invalid)
- Merging/aggregation logic (if applicable)
- Serialization round-trips (if JSON/TOML involved)

**Example:**
```rust
#[test]
fn test_engine_state_aggregate_all_dimensions() {
    let mut state = EngineState::default();
    state.workspace.crates.push("test-crate".to_string());
    state.changed_files.count = 5;
    state.git_phase.is_dirty = true;
    
    // Verify aggregate view
    assert_eq!(state.workspace.crates.len(), 1);
    assert!(state.is_dirty_or_changed()); // if such a method exists
}

#[test]
fn test_git_phase_state_dirty_detection() {
    let mut state = GitPhaseState::default();
    assert!(!state.is_dirty);
    
    state.dirty_files.push("src/main.rs".to_string());
    assert!(state.is_dirty);
}
```

#### `src/adapters/` (Target: 85%+)
**Critical modules:** `git_status.rs`, `cargo_metadata.rs`, `changed_file_detector.rs`, `target_scanner.rs`

**Must test:**
- Success path (data parsed correctly)
- Error paths (missing tools, malformed output, permission denied)
- Empty input (no files, no git history, empty workspace)
- Boundary values (very large counts, long paths, special characters)

**Example:**
```rust
#[test]
fn test_git_status_adapter_parses_porcelain() {
    // Mock: simulate `git status --porcelain` output
    let porcelain = " M src/main.rs\nM  tests/lib.rs\n?? new.txt";
    let result = parse_porcelain(porcelain).unwrap();
    
    assert_eq!(result.dirty_files, vec!["src/main.rs"]);
    assert_eq!(result.staged_files, vec!["tests/lib.rs"]);
    assert_eq!(result.untracked_files, vec!["new.txt"]);
}

#[test]
fn test_git_status_adapter_handles_missing_git() {
    // Test when git is not installed or repo not initialized
    let result = GitStatusAdapter::query();
    assert!(result.is_err());
}

#[test]
fn test_cargo_metadata_adapter_empty_workspace() {
    let workspace_dir = TempDir::new().unwrap();
    // No Cargo.toml present
    let result = CargoMetadataAdapter::scan(workspace_dir.path());
    assert!(result.is_err());
}
```

#### `src/policies/` (Target: 80%+)
**Critical modules:** `git_phase_dirty.rs`, `toolchain_mismatch.rs`, `trybuild_changed.rs`

**Must test:**
- Policy triggers (state conditions that activate policy)
- Suggestion output format
- Policy composition (multiple policies on same state)
- Edge cases (conflicting policies, missing state)

**Example:**
```rust
#[test]
fn test_git_phase_dirty_policy_triggers() {
    let state = PolicyState {
        git_phase_dirty: true,
        ..Default::default()
    };
    let suggestion = evaluate_git_phase_dirty(&state);
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().message.contains("uncommitted"));
}

#[test]
fn test_toolchain_mismatch_policy_no_trigger_on_match() {
    let state = PolicyState {
        toolchain_mismatch: false,
        ..Default::default()
    };
    let suggestion = evaluate_toolchain_mismatch(&state);
    assert!(suggestion.is_none());
}
```

#### `src/nouns/` (Target: 70%+)
**Modules:** `status.rs`, `target.rs`, `test.rs`, `git.rs`, `publish.rs`

**Must test:**
- Noun initialization (default verbs injected)
- Basic verb invocation (show, run, etc.)
- Help text accuracy
- Error handling (missing dependencies, invalid flags)

**Note:** Nouns are tested primarily via integration tests (`tests/cli.rs`). Unit tests here are supplementary.

**Example:**
```rust
#[test]
fn test_status_noun_default_verb_is_show() {
    // cargo-cicd status → cargo-cicd status show
    let cmd = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["status"])
        .output()
        .unwrap();
    // Should execute successfully (default verb injected)
    assert!(cmd.status.success());
}

#[test]
fn test_target_noun_help_mentions_prune() {
    let cmd = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["target", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&cmd.stdout);
    assert!(text.contains("prune") || text.contains("clean"));
}
```

## 4. Required Coverage by Contribution Type

### Feature Contributions
- **New code**: **80%+ branch coverage** minimum
- **Existing code modified**: Must maintain or improve coverage
- **Test plan**: Must include happy path + at least 2 error paths
- **Integration test**: Required for cross-module features

**Example PR checklist:**
```
- [ ] New code branch coverage: 80%+
- [ ] Existing code not regressed (cargo tarpaulin before/after)
- [ ] Error paths tested (network down, file not found, permission denied)
- [ ] Integration test added (tests/*)
- [ ] Documentation updated (help text, README)
```

### Bug Fix Contributions
- **Regression test required**: Test must fail on unpatched code, pass on patched code
- **Coverage target**: 75%+ (may be lower for isolated fixes)
- **Error paths**: Ensure fix handles related edge cases

**Example:**
```rust
#[test]
fn test_regression_git_status_parsing_double_space() {
    // Bug: " M" was parsed as "not dirty" (off-by-one on index)
    let porcelain = " M src/main.rs";  // Note: intentional space prefix
    let result = parse_porcelain(porcelain).unwrap();
    
    // This test fails before the fix, passes after
    assert!(result.dirty_files.contains(&"src/main.rs".to_string()));
}
```

### Refactoring Contributions
- **Coverage maintained or improved**: No coverage loss allowed
- **No new test code required** (unless refactoring simplifies testing)
- **Integration test execution**: Must pass all existing tests

### Test Improvement Contributions
- **Coverage increase**: Adding tests should increase coverage or improve branch coverage
- **No coverage loss**: Never remove tests without replacing them
- **Documentation**: Update this file if adding new test patterns

## 5. How to Write Testable Code

### Principle 1: Inject Dependencies
Don't hardcode external calls; pass them in or use a trait.

**Bad:**
```rust
pub fn scan_workspace() -> Result<WorkspaceState> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1"])
        .output()?;  // Hardcoded dependency on `cargo`
    // ...
}
```

**Good:**
```rust
pub fn scan_workspace<F>(cargo_runner: F) -> Result<WorkspaceState>
where
    F: Fn(&[&str]) -> std::io::Result<std::process::Output>,
{
    let output = cargo_runner(&["metadata", "--format-version=1"])?;
    // ...
}

#[test]
fn test_scan_workspace_with_mock() {
    let mock_output = std::process::Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: br#"{"workspace_members":[]}"#.to_vec(),
        stderr: vec![],
    };
    let result = scan_workspace(|_args| Ok(mock_output.clone()));
    assert!(result.is_ok());
}
```

**Alternative: Adapter pattern (cargo-cicd standard):**
```rust
pub trait CargoRunner {
    fn run(&self, args: &[&str]) -> Result<String>;
}

pub struct RealCargoRunner;
impl CargoRunner for RealCargoRunner {
    fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("cargo").args(args).output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
}

#[test]
fn test_with_mock_runner() {
    struct MockRunner;
    impl CargoRunner for MockRunner {
        fn run(&self, _args: &[&str]) -> Result<String> {
            Ok(r#"{"workspace_members":[]}"#.to_string())
        }
    }
    let result = scan_workspace(&MockRunner);
    assert!(result.is_ok());
}
```

### Principle 2: Avoid Side Effects in Pure Functions
Separate I/O from computation.

**Bad:**
```rust
pub fn analyze_changes(path: &Path) -> Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)?;  // Side effect
    let lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
    Ok(lines)  // Pure logic mixed with I/O
}
```

**Good:**
```rust
pub fn parse_changes(contents: &str) -> Vec<String> {  // Pure
    contents.lines().map(|l| l.to_string()).collect()
}

pub fn analyze_changes(path: &Path) -> Result<Vec<String>> {  // I/O wrapper
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_changes(&contents))
}

#[test]
fn test_parse_changes_no_io() {
    let contents = "src/main.rs\ntests/lib.rs";
    let result = parse_changes(contents);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_analyze_changes_with_temp_file() {
    use tempfile::NamedTempFile;
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "src/main.rs")?;
    let result = analyze_changes(file.path()).unwrap();
    assert_eq!(result.len(), 1);
}
```

### Principle 3: Keep Functions Small and Focused
One responsibility per function = easier to test.

**Bad:**
```rust
pub fn run_full_pipeline() -> Result<()> {
    let workspace = scan_workspace()?;
    let changed_files = detect_changes()?;
    let git_status = get_git_status()?;
    let test_plan = build_test_plan(&workspace, &changed_files)?;
    execute_tests(&test_plan)?;
    write_cicd_toml(&workspace, &test_plan)?;
    Ok(())
}
// Hard to test; many integration points
```

**Good:**
```rust
pub fn scan_workspace() -> Result<WorkspaceState> { /* testable */ }
pub fn detect_changes() -> Result<ChangedFileState> { /* testable */ }
pub fn get_git_status() -> Result<GitPhaseState> { /* testable */ }
pub fn build_test_plan(ws: &WorkspaceState, changes: &ChangedFileState) -> TestPlanState { /* pure */ }

pub fn run_full_pipeline() -> Result<()> {
    let workspace = scan_workspace()?;
    let changes = detect_changes()?;
    let git = get_git_status()?;
    let plan = build_test_plan(&workspace, &changes);
    execute_tests(&plan)?;
    write_cicd_toml(&workspace, &plan)?;
    Ok(())
}

// Each function is testable independently
#[test]
fn test_build_test_plan_with_no_changes() {
    let ws = WorkspaceState { crates: vec!["lib".to_string()], ..Default::default() };
    let changes = ChangedFileState::default();
    let plan = build_test_plan(&ws, &changes);
    assert!(plan.tests_to_run.is_empty());
}
```

### Principle 4: Use `#[cfg(test)]` Modules
Keep test code separate but colocated.

**Good:**
```rust
pub fn parse_version(s: &str) -> Result<Version> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid version".into());
    }
    Ok(Version {
        major: parts[0].parse()?,
        minor: parts[1].parse()?,
        patch: parts[2].parse()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_valid() {
        let v = parse_version("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_version_invalid_format() {
        assert!(parse_version("1.2").is_err());
        assert!(parse_version("1.2.3.4").is_err());
    }

    #[test]
    fn test_parse_version_invalid_numbers() {
        assert!(parse_version("a.b.c").is_err());
    }
}
```

### Principle 5: Use `tempfile` for Filesystem Tests
Never rely on the real filesystem; use temporary directories.

**Good:**
```rust
#[test]
fn test_scan_workspace_detects_cargo_toml() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    let cargo_toml = temp.path().join("Cargo.toml");
    std::fs::write(&cargo_toml, "[package]\nname=\"test\"\n").unwrap();
    
    let result = scan_workspace(temp.path()).unwrap();
    assert!(result.crates.iter().any(|c| c == "test"));
}

#[test]
fn test_scan_workspace_handles_missing_cargo_toml() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    // No Cargo.toml created
    
    let result = scan_workspace(temp.path());
    assert!(result.is_err());
}
```

## 6. Edge Cases Checklist

Every module should test these scenarios:

### Empty Input
- [ ] Empty workspace (no crates, no Cargo.toml)
- [ ] Empty file (0 bytes)
- [ ] Empty git repository (no commits)
- [ ] Empty test suite (no tests)
- [ ] Empty changed files (clean working tree)

**Example:**
```rust
#[test]
fn test_detect_changes_empty_workspace() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    let result = detect_changes(temp.path()).unwrap();
    assert!(result.files.is_empty());
}
```

### Large Input
- [ ] Large workspace (100+ crates)
- [ ] Large crate (1000+ source files)
- [ ] Large test suite (1000+ tests)
- [ ] Large file (100+ MB)
- [ ] Deep directory tree (100+ levels)

**Example:**
```rust
#[test]
fn test_scan_workspace_scales_to_many_crates() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    
    for i in 0..100 {
        let crate_dir = temp.path().join(format!("crate{}", i));
        std::fs::create_dir(&crate_dir).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            format!("[package]\nname=\"crate{}\"\n", i),
        ).unwrap();
    }
    
    let start = std::time::Instant::now();
    let result = scan_workspace(temp.path()).unwrap();
    let elapsed = start.elapsed();
    
    assert_eq!(result.crates.len(), 100);
    assert!(elapsed.as_secs() < 10, "Scanning 100 crates took too long");
}
```

### Error Paths
- [ ] Network unavailable (git remote unreachable)
- [ ] Permission denied (read-only file, no execute permission)
- [ ] Corrupted file (malformed JSON, TOML)
- [ ] Missing tool (git not installed, cargo not on PATH)
- [ ] Timeout (slow network, large workspace)

**Example:**
```rust
#[test]
fn test_git_status_adapter_handles_permission_denied() {
    // Simulate: git command fails with permission error
    let result = GitStatusAdapter::query_with_command("/nonexistent/git");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found") 
        || result.unwrap_err().to_string().contains("Permission"));
}

#[test]
fn test_parse_cicd_toml_handles_malformed_json() {
    let bad_toml = "[workspace\nthis is not valid toml";
    let result = parse_cicd_toml(bad_toml);
    assert!(result.is_err());
}
```

### Boundary Conditions
- [ ] Maximum values (u32::MAX, usize::MAX)
- [ ] Minimum values (0, empty string)
- [ ] Off-by-one errors (< vs <=, first vs last)
- [ ] Floating point precision (if applicable)

**Example:**
```rust
#[test]
fn test_count_tests_boundary_zero() {
    let plan = TestPlanState { test_count: 0, ..Default::default() };
    assert!(!plan.has_tests());
}

#[test]
fn test_count_tests_boundary_one() {
    let plan = TestPlanState { test_count: 1, ..Default::default() };
    assert!(plan.has_tests());
}

#[test]
fn test_parse_version_max_values() {
    let v = parse_version("999.999.999").unwrap();
    assert_eq!(v.major, 999);
}
```

### Concurrent Access (if applicable)
- [ ] Multiple threads reading the same state
- [ ] Multiple threads writing different state fields
- [ ] Deadlock scenarios (circular locks)
- [ ] Memory safety (no race conditions, no use-after-free)

**Example:**
```rust
#[test]
fn test_engine_state_thread_safe_reads() {
    use std::sync::Arc;
    use std::thread;
    
    let state = Arc::new(EngineState::default());
    let mut handles = vec![];
    
    for i in 0..10 {
        let state_clone = Arc::clone(&state);
        let handle = thread::spawn(move || {
            let _ = state_clone.workspace.crates.len();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

## 7. Known Gaps

These areas are difficult to test in isolation and require different approaches:

### wasm4pm Integration Testing
- **Issue**: wasm4pm oracle is an external process; only available at `/Users/sac/wasm4pm/target/release/wpm`
- **Approach**: Feature-gate wasm4pm tests (`#[cfg(feature = "wasm4pm")]`)
- **Mitigation**: Mock the `wpm` output in unit tests; real integration tested in dedicated `wasm4pm_evidence_gate.rs`

**Example:**
```rust
#[cfg(feature = "wasm4pm")]
#[test]
fn test_emit_xes_evidence_receipt_valid() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    
    let evidence = emit_xes_evidence(&EngineState::default(), temp.path()).unwrap();
    
    // Verify XES structure (hard to test without running wpm)
    assert!(evidence.contains("<?xml"));
    assert!(evidence.contains("event"));
}
```

### Real-World Workspace Testing
- **Issue**: Full testing requires large, complex workspaces (expensive in CI)
- **Approach**: Use fixture workspaces under `tests/fixtures/`; CI runs subset
- **Mitigation**: Document fixture design in `FIXTURES.md`; periodically test against real repos

**Example:**
```rust
#[ignore]  // Run manually with `cargo test -- --ignored`
#[test]
fn test_scan_large_real_workspace() {
    // Requires: cargo-cicd cloned repo at ~/cargo-cicd-large
    let workspace = std::env::var("CARGO_CICD_LARGE_REPO")
        .unwrap_or_else(|_| "../cargo-cicd-large".to_string());
    
    if !Path::new(&workspace).exists() {
        println!("Skipping: large workspace not found");
        return;
    }
    
    let result = scan_workspace(Path::new(&workspace)).unwrap();
    assert!(result.crates.len() > 50);
}
```

### Network Failure Scenarios
- **Issue**: Hard to reliably simulate network failures in tests
- **Approach**: Use mocking frameworks (e.g., `mockito`, `httptest`)
- **Mitigation**: Test retry logic with fast mocks; document real-world failure modes

**Example:**
```rust
#[test]
fn test_fetch_git_remote_with_mock_failure() {
    // Requires: mockito or similar; not currently in dev-dependencies
    // struct MockGitRunner {
    //     should_fail: bool,
    // }
    // impl GitRunner for MockGitRunner { ... }
    
    // For now: manually test with `unplug network card`
    // Document in TESTING.md for CI operators
}
```

## 8. Running Coverage Locally

### Quick Check (tarpaulin, ~30 seconds)
```bash
cargo tarpaulin --features advanced --out Html --timeout 300
# Open: tarpaulin-report.html
```

### Detailed Check (llvm-cov, ~60 seconds)
```bash
cargo llvm-cov --features advanced --html
# Open: target/llvm-cov/html/index.html
```

### Per-Module Coverage
```bash
# Adapters only
cargo tarpaulin -p cargo-cicd src/adapters --features advanced --out Html

# Engine + state
cargo tarpaulin -p cargo-cicd src/engine src/state --features advanced --out Html
```

### Coverage Before/After Refactor
```bash
# Before
cargo llvm-cov --features advanced --json --output-path coverage-before.json

# Refactor...

# After
cargo llvm-cov --features advanced --json --output-path coverage-after.json

# Compare
diff <(jq '.data[0].summary.regions.percent' coverage-before.json) \
     <(jq '.data[0].summary.regions.percent' coverage-after.json)
```

## 9. CI/CD Integration

### GitHub Actions Example
```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Install llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --features advanced --json --output-path coverage.json
      - name: Check minimum coverage
        run: |
          MIN_COVERAGE=78
          ACTUAL=$(jq '.data[0].summary.regions.percent' coverage.json)
          echo "Coverage: ${ACTUAL}% (minimum: ${MIN_COVERAGE}%)"
          if (( $(echo "$ACTUAL < $MIN_COVERAGE" | bc -l) )); then
            exit 1
          fi
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: coverage.json
```

## 10. References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [proptest for Property-Based Testing](https://docs.rs/proptest/)
- [tempfile for Filesystem Tests](https://docs.rs/tempfile/)
- [assert_cmd for CLI Testing](https://docs.rs/assert_cmd/)

---

**Last Updated:** 2026-06-14  
**Maintained by:** cargo-cicd contributors
