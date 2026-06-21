# Architectural Patterns in cargo-cicd

This guide documents the recurring architectural patterns used throughout cargo-cicd. New developers should follow these patterns when adding features, adapters, or policies.

---

## 1. Noun-Verb CLI Pattern

**Purpose:** Organize CLI commands hierarchically as noun-verb pairs, making the interface predictable and discoverable.

**Implementation:**
- All commands are noun-verb pairs (e.g., `cargo cicd status show`, `cargo cicd test run`)
- Each noun = a module in `src/nouns/`
- Each noun implements `NounCommand` trait
- Each verb within a noun implements `VerbCommand` trait
- Verbs have three required methods:
  - `name()` — Returns the verb's identifier
  - `about()` — Returns help text
  - `run(state: EngineState, args: Cli) -> Result<()>` — Executes the verb

**Default Verb Injection:**
- `main.rs::inject_default_verbs()` registers a default verb (usually `show`) for bare nouns
- Allows shorthand: `cargo cicd status` → `cargo cicd status show`

**Examples:**
- `src/nouns/git.rs` — implements git noun with verbs: status, phase, closure
- `src/nouns/test.rs` — implements test noun with verbs: run, plan
- `src/nouns/target.rs` — implements target noun with verbs: scan, validate
- `src/nouns/ui.rs` — implements ui noun with verbs: demo, dashboard
- `src/nouns/sbom.rs` — implements sbom noun with verbs: generate, show

> **Note:** The noun list has grown beyond the original set. The authoritative list of all nouns is `src/nouns/` — inspect that directory directly rather than relying on any enumeration in this doc. As of 2026-06-21 the nouns include: evidence, pipeline, status, target, test, trybuild, git, publish, workspace, lsp, analyze, autoarch, certification, sbom, ui (plus affidavit when the `affidavit` feature is enabled).

**When to add a new noun:**
- You have 2+ related commands that operate on a shared domain
- The domain is conceptually distinct from existing nouns
- The noun name is a singular noun (git, test, target, workspace, status)

---

## 2. Evidence Emission Pattern (CRITICAL)

**Purpose:** Every mutation, decision, or work item must emit structured process events so that external auditors (wasm4pm) can verify execution and intent.

**The Rule:**
Every verb that does work **MUST** emit evidence:
1. Call `ProcessEvent::started()` when work begins
2. Perform the work
3. Call `ProcessEvent::completed()` when work ends
4. Both events **MUST** include `case_id` from `read_or_create_session_id()`
5. Use `append_events()`, not `emit_events_jsonl()`, for multi-event traces

**Pattern:**

```rust
let case_id = read_or_create_session_id()?;

let start_event = ProcessEvent::started()
    .case_id(case_id.clone())
    .activity_name("git_phase_closure")
    .timestamp(Utc::now())
    .build();

// ... perform work ...

let end_event = ProcessEvent::completed()
    .case_id(case_id.clone())
    .activity_name("git_phase_closure")
    .timestamp(Utc::now())
    .build();

engine_state.append_events(vec![start_event, end_event])?;
```

**Why this matters:**
- wasm4pm expects XES (XML Event Stream) format with complete traces
- Each case_id links all events in a single execution
- Missing case_id = evidence gap = test failure
- append_events() preserves event order and correlation

**When to skip evidence (rare):**
- Only read-only queries (no side effects)
- Help text or pure introspection
- All mutations, writes, and decisions must emit evidence

**Reference:**
- `src/nouns/git.rs` — Complete example with start/end events
- `src/process_event.rs` — ProcessEvent builder API

---

## 3. Adapter Pattern

**Purpose:** Isolate external system integration (git, cargo, filesystem) from business logic. Each adapter owns translation from external format to internal `EngineState`.

**Principles:**
- **One external source per adapter**
  - GitStatusAdapter for git operations
  - TargetScannerAdapter for cargo metadata
  - ToolchainDetector for rustc/rustup state
  - CargoMetadataAdapter for Cargo.toml parsing
- **Unidirectional translation:** external format → EngineState
- **No business logic:** Adapters only translate; policies and nouns implement logic
- **Graceful degradation:** Adapters return `anyhow::Result`; failure doesn't crash the engine

**Structure:**

```rust
pub struct MyAdapter;

impl MyAdapter {
    pub fn read(workspace: &Path) -> anyhow::Result<MyState> {
        // Read from external system
        // Translate to internal type
        // Return Ok(state) or bail!("context")
    }
}
```

**Examples:**
- `src/adapters/git_status_adapter.rs` — Reads git status, populates `GitPhaseState`
- `src/adapters/target_scanner_adapter.rs` — Scans Cargo.toml and workspace members
- `src/adapters/changed_file_detector.rs` — Detects changed files from git diff
- `src/adapters/toolchain_detector.rs` — Detects rustc version and features

**Location:** All adapters live in `src/adapters/`

**When to add an adapter:**
- You need to integrate a new external system
- The integration is read-only or safe to externalize
- The data should be stored in `EngineState` and reused by multiple nouns

---

## 4. EngineState as Aggregate Root

**Purpose:** Single source of truth for all runtime state. Adapters populate it; nouns and policies read from it. Ensures consistency and testability.

**The 11 Sub-States:**
1. `workspace_state` — Workspace members, root, features
2. `toolchain_state` — Rust version, compiler flags, editions
3. `target_state` — Targets (bin/lib), platform specs
4. `changed_files_state` — Files changed since last commit
5. `test_plan_state` — Test matrix, disabled tests
6. `trybuild_state` — Trybuild compilation tests
7. `git_phase_state` — Git status, branch, merge state
8. `process_events_state` — Emitted ProcessEvents for audit
9. `artifacts_state` — Build artifacts, test results
10. `policies_state` — Policy evaluation results
11. `projection_profile` — Feature flag projection

**Initialization:**
```rust
let mut engine_state = EngineState::new(workspace_root);

// Adapters populate sub-states
engine_state.workspace_state = WorkspaceAdapter::read(&workspace_root)?;
engine_state.git_phase_state = GitStatusAdapter::read(&workspace_root)?;
// ... etc for all adapters
```

**Access Pattern:**
- Nouns read from `engine_state.some_sub_state`
- Adapters write during initialization
- Policies read from `engine_state` to evaluate rules

**Why this matters:**
- No hidden state or side effects
- Tests can construct EngineState with fixtures
- State is serializable to cicd.toml for persistence

---

## 5. Policy Evaluation Pattern

**Purpose:** Autonomic policies read state and emit recommendations (never actions). Each policy is independently evaluable, enabling composition and extensibility.

**The Policy Trait:**

```rust
pub trait CicdPolicy {
    fn name(&self) -> &str;
    fn evaluate(&self, engine_state: &EngineState) -> PolicyResult;
}
```

**PolicyResult Structure:**
```rust
pub struct PolicyResult {
    pub name: String,
    pub verdict: Verdict,  // Pass, Warn, or Suggest
    pub recommendation: String,
    pub event: ProcessEvent,
    pub mode: PolicyMode,  // Suggest (default) or Apply (reserved)
}
```

**Verdict Types:**
- `Pass` — All checks passed, no action needed
- `Warn` — Anomaly detected, user should investigate
- `Suggest` — Recommendation for improvement (default, non-blocking)

**Evaluation Pattern:**

```rust
pub struct MyPolicy;

impl CicdPolicy for MyPolicy {
    fn name(&self) -> &str { "my_policy" }
    
    fn evaluate(&self, engine_state: &EngineState) -> PolicyResult {
        if engine_state.some_condition {
            PolicyResult {
                verdict: Verdict::Suggest,
                recommendation: "Do X to improve Y".to_string(),
                ..
            }
        } else {
            PolicyResult {
                verdict: Verdict::Pass,
                ..
            }
        }
    }
}
```

**Policy Execution:**
- All policies are evaluated in `suggest` mode by default (configured in `cicd.toml [autonomic]`)
- Policies are read-only; they never take destructive action
- Results are collected and reported to the user

**Location:** `src/policies/`

**When to add a policy:**
- You have a rule that should be checked automatically
- The rule can be evaluated from `EngineState` alone
- The recommendation doesn't require destructive action

---

## 6. LSP Analyzer Pattern

**Purpose:** Language server analyzers perform code-level analysis (changed tests, git phase, target hygiene) and emit diagnostic findings without executing repairs.

**The Analyzer Trait:**

```rust
pub trait CicdAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> anyhow::Result<Vec<CicdFinding>>;
}
```

**Finding Structure:**
```rust
pub struct CicdFinding {
    pub code: String,           // e.g., "changed-test-001"
    pub title: String,
    pub description: String,
    pub severity: DiagnosticSeverity,  // Error, Warning, Information
    pub location: FileLocation,
    pub related_info: Vec<RelatedInfo>,
}
```

**Examples:**
- `crates/cargo-cicd-lsp/src/analyzers/changed_tests.rs` — Detects test files changed without corresponding test changes
- `crates/cargo-cicd-lsp/src/analyzers/git_phase.rs` — Analyzes git reachability and merge state
- `crates/cargo-cicd-lsp/src/analyzers/target_hygiene.rs` — Checks for orphaned or misconfigured targets

**Key Principle:**
Analyzers are **diagnostic only**. They report findings; they don't fix them. Repairs are separate operations.

**Location:** `crates/cargo-cicd-lsp/src/analyzers/`

**When to add an analyzer:**
- You want to report problems to the IDE/user without fixing them
- The analysis is per-file or per-target
- The analysis benefits from LSP integration (hover, inline fixes)

---

## 7. Lifecycle Management in LSP

**Purpose:** Track the lifecycle of findings from discovery to resolution: raised → routed → [pending repair | preserved | cleared].

**Lifecycle States:**
```rust
pub enum DiagnosticLifecycle {
    Raised,               // Newly discovered
    Routed,               // Assigned to developer
    PendingRepair,        // User acknowledged, awaiting fix
    ResidualPreserved,    // Intentionally preserved (wont fix)
    Cleared,              // Fixed or dismissed
}
```

**Lifecycle Operations:**

```rust
// Raise: Insert a new finding
engine_state.raise_finding(finding)?;

// Clear: Remove findings by code
engine_state.clear_by_code("changed-test-001")?;

// Transition: Move to pending repair
engine_state.route_finding(finding_id)?;
```

**Location:** `crates/cargo-cicd-lsp/src/lifecycle/`

**When to use lifecycle management:**
- You need to track findings across multiple executions
- Users should be able to dismiss or preserve findings
- You want audit trail of what was fixed and when

---

## 8. Feature Flag Guards

**Purpose:** Gate internal machinery without breaking public APIs. Feature flags control which sub-engines are compiled in.

**The Three Main Flags:**

| Flag | Implies | Enables |
|------|---------|---------|
| `process-data` | (none) | EngineState, adapters, basic analytics |
| `autonomic` | `process-data` | Policy engine, suggest mode |
| `wasm4pm` | `process-data` | Evidence adjudication, wpm oracle calls |
| `contrib` | `process-data` | Development tooling, contrib features |

**Rules:**
- **Never gate public APIs** — All noun verbs, CLIs, and public traits must compile without flags
- **Only gate internal machinery** — Adapters, policies, sub-engines, and internal types can be gated
- **Implication order matters** — Check feature hierarchy in `Cargo.toml`

**Pattern:**

```rust
// OK: Gate internal adapter
#[cfg(feature = "process-data")]
mod git_status_adapter;

// WRONG: Don't gate public verb
#[cfg(feature = "process-data")]
pub struct GitVerb;  // BREAKS public API

// OK: Gate internal policy
#[cfg(feature = "autonomic")]
impl CicdPolicy for MyPolicy { .. }
```

**Location:** `Cargo.toml` features section

---

## 9. Error Handling

**Purpose:** Consistent error handling across CLI, adapters, and policies. Errors are informative and actionable.

**Error Types by Context:**

| Context | Type | Usage |
|---------|------|-------|
| Adapters | `anyhow::Result` | Graceful degradation on failure |
| Policies | No panic, always return `PolicyResult` | Safe evaluation |
| CLI verbs | `clap_noun_verb::error::Result` | User-friendly error messages |
| Core logic | `anyhow::Result` | Rich context with `bail!()` |

**Pattern:**

```rust
// Adapters: Use anyhow::Result
fn read(workspace: &Path) -> anyhow::Result<State> {
    let content = std::fs::read_to_string(path)
        .context("failed to read config")?;
    Ok(state)
}

// Verbs: Return clap_noun_verb::error::Result
fn run(&self, _state: EngineState, _args: Cli) -> clap_noun_verb::error::Result<()> {
    do_work().context("operation failed")?;
    Ok(())
}

// Policies: Never panic
fn evaluate(&self, state: &EngineState) -> PolicyResult {
    // Always return a verdict, even on internal errors
    if let Err(e) = evaluate_logic(state) {
        return PolicyResult {
            verdict: Verdict::Warn,
            recommendation: format!("evaluation failed: {}", e),
            ..
        };
    }
    // ...
}
```

**Use `bail!()` for context:**
```rust
if config_is_invalid {
    bail!("invalid config: expected [workspace], found [other]");
}
```

**No swallowing errors:**
```rust
// Good: propagate with context
result.context("failed to scan targets")?;

// Bad: silence the error
result.ok();
```

---

## 10. Testing Pattern

**Purpose:** Comprehensive testing at smoke, integration, and acceptance levels. Every feature has tests; evidence-gate tests use wasm4pm.

**Test Hierarchy:**

| Level | Location | Tools | Purpose |
|-------|----------|-------|---------|
| Smoke/Unit | `tests/` | `assert_cmd`, `TempDir` | Fast feedback, API contracts |
| Integration | `tests/` | `assert_cmd`, realistic fixtures | End-to-end scenarios |
| Evidence-gate | `tests/wasm4pm_*` | `assert_cmd`, `wpm` oracle | Release closure |
| Feature projection | `tests/feature_projection.rs` | Feature flag combinations | API stability |

**Smoke Test Pattern:**

```rust
#[test]
fn test_git_status_shows_branch() {
    let temp = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    
    cmd.arg("git")
       .arg("status")
       .current_dir(temp.path());
    
    cmd.assert_success()
       .stdout(predicate::str::contains("branch"));
}
```

**Integration Test Pattern:**

```rust
#[test]
fn test_cicd_toml_truth() {
    let temp = copy_fixture("realistic_workspace");
    
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("status").arg("show").current_dir(temp.path());
    
    cmd.assert_success();
    
    let cicd_toml = temp.path().join("cicd.toml");
    assert!(cicd_toml.exists());
}
```

**Evidence-Gate Pattern (Release Gating):**

```rust
#[test]
fn test_wasm4pm_evidence_gate() {
    let temp = copy_fixture("realistic_workspace");
    
    // Run verb, which emits evidence
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    cmd.arg("test").arg("run").current_dir(temp.path());
    cmd.assert_success();
    
    // Invoke wpm oracle
    let evidence = temp.path().join("target/cargo-cicd/evidence/");
    let wpm_output = Command::new("wpm")
        .args(&["audit", evidence.to_str().unwrap()])
        .output()
        .expect("wpm failed");
    
    assert_eq!(wpm_output.status.code(), Some(0), "wpm rejected evidence");
}
```

**Feature Projection Pattern:**

```rust
#[test]
fn test_feature_process_data_alone() {
    // Verify cargo-cicd compiles and runs with only process-data
}

#[test]
fn test_feature_autonomic_implies_process_data() {
    // Verify autonomic implies process-data
}
```

**Fixture Location:** `tests/fixtures/`

**When to add a test:**
- Every noun verb should have at least one smoke test
- Integration tests for new adapters or policies
- Evidence-gate tests for any work that mutates state
- Feature projection tests for new feature flags

---

## Summary

| Pattern | Location | Trigger |
|---------|----------|---------|
| Noun-Verb | `src/nouns/` | New CLI command |
| Evidence Emission | Any verb doing work | Any mutation/decision |
| Adapter | `src/adapters/` | New external system |
| EngineState | `src/engine/` | Any state that's shared |
| Policy | `src/policies/` | New rule to evaluate |
| LSP Analyzer | `crates/cargo-cicd-lsp/src/analyzers/` | Diagnostic findings |
| Lifecycle | `crates/cargo-cicd-lsp/src/lifecycle/` | Finding tracking |
| Feature Flags | `Cargo.toml` | Conditional compilation |
| Error Handling | All modules | Every fallible operation |
| Testing | `tests/` | Every feature, verb, adapter |

When in doubt, examine similar code in the repository—these patterns are demonstrated throughout.
