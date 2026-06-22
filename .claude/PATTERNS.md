# cargo-cicd Architectural Patterns

## 1. Noun-Verb CLI

Trigger: adding any CLI command.

```rust
// src/nouns/my_noun.rs
pub struct MyNoun;
impl NounCommand for MyNoun {
    fn name(&self) -> &str { "mynoun" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> { vec![Box::new(ShowVerb)] }
}

pub struct ShowVerb;
impl VerbCommand for ShowVerb {
    fn name(&self) -> &str { "show" }
    fn about(&self) -> &str { "Show mynoun state" }
    fn run(&self, state: EngineState, _args: Cli) -> Result<()> { Ok(()) }
}
```

- Register in `src/main.rs::inject_default_verbs()` for bare-noun shorthand
- Noun name = singular noun, module in `src/nouns/`
- Authoritative noun list: `src/nouns/` directory (not this doc)

Current nouns: `evidence` · `pipeline` · `status` · `target` · `test` · `trybuild` · `git` · `publish` · `workspace` · `lsp` · `analyze` · `autoarch` · `certification` · `sbom` · `ui` · `affidavit` (feature-gated)

---

## 2. Evidence Emission (CRITICAL)

**OCEL 2.0 is the only format. XES is legacy — do not extend.**

### Cargo.toml dependency
```toml
wasm4pm-compat = { path = "/Users/sac/wasm4pm-compat", features = ["formats", "strict"] }
```

### Required imports
```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCELAttributeValue};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence, AdmittedOcelEvidence};
use wasm4pm_compat::state::{Raw, Admitted};
use wasm4pm_compat::witness::Ocel20;
use wasm4pm_compat::receipt::Receipt;
use wasm4pm_compat::conformance::ConformanceResult;
```

### Emission pattern (every verb that does work)
```rust
// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

### OCEL 2.0 JSON shape
```json
{ "eventTypes": [...], "objectTypes": [...], "events": [...], "objects": [...] }
```
- `OCELEvent.relationships`: `Vec<OCELRelationship { objectId, qualifier }>`
- `OCELObject.relationships`: `Vec<OCELObjectRelationship { objectId, qualifier }>`

### Domain object types
`Workspace` · `Crate` · `TestRun` · `GitCommit` · `Release` · `Receipt` · `EvidenceFile` · `Policy` · `Toolchain`

### Invariants
| # | Rule |
|---|------|
| E1 | cargo-cicd never adjudicates itself; only `wpm` issues verdicts |
| E2 | OCEL file must exist before `audit_ocel()` is called |
| E3 | Oracle unavailable + non-Blocked expectation = panic |
| E4 | Tests assert wpm verdict only, never internal state |
| E7 | `Blocked` is a first-class expectation, not an error |

### FORBIDDEN
- Hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs
- Calling `wpm` on `.xes` files in new code
- Adjudicating inside cargo-cicd (E1)
- Extending `evidence_xes_v2.rs`
- Using `src/ocel.rs` — delete it; replace with wasm4pm-compat imports

### Skip evidence only for
- Pure read-only queries with zero side effects
- Help text / introspection

### wpm binary
```sh
which wpm   # path resolution
wpm audit <file.ocel.json>   # Accept | Refuse | Blocked
wpm receipt doctor --format json --strict <receipt.json>
```
Shell-out only. Never link.

---

## 3. Adapter Pattern

```rust
pub struct MyAdapter;
impl MyAdapter {
    pub fn read(workspace: &Path) -> anyhow::Result<MyState> {
        // translate external format → internal type
        // bail!("context") on failure
    }
}
```

| Rule | Detail |
|------|--------|
| One external source per adapter | git / cargo / fs — never mix |
| Unidirectional | external → EngineState only |
| No business logic | policies and nouns implement logic |
| Graceful degradation | return `anyhow::Result`; failure ≠ crash |

Location: `src/adapters/`

Existing adapters: `GitStatusAdapter` · `TargetScannerAdapter` · `ChangedFileDetector` · `ToolchainDetector` · `CargoMetadataAdapter` · `CicdTomlWriter`

---

## 4. EngineState (Aggregate Root)

```rust
let mut engine_state = EngineState::new(workspace_root);
engine_state.workspace_state = WorkspaceAdapter::read(&workspace_root)?;
engine_state.git_phase_state = GitStatusAdapter::read(&workspace_root)?;
// ... all adapters in sequence, silently handling failures
```

| Field | Contents |
|-------|----------|
| `workspace_state` | members, root, features |
| `toolchain_state` | rust version, compiler flags, editions |
| `target_state` | bin/lib targets, platform specs |
| `changed_files_state` | files changed since last commit |
| `test_plan_state` | test matrix, disabled tests |
| `trybuild_state` | trybuild compilation tests |
| `git_phase_state` | status, branch, merge state |
| `process_events_state` | emitted ProcessEvents for audit |
| `artifacts_state` | build artifacts, test results |
| `policies_state` | policy evaluation results |
| `projection_profile` | feature flag projection |

Access rule: nouns **read**; adapters **write** during init; policies **read**.

---

## 5. Policy Evaluation

```rust
pub struct MyPolicy;
impl CicdPolicy for MyPolicy {
    fn name(&self) -> &str { "my_policy" }
    fn evaluate(&self, engine_state: &EngineState) -> PolicyResult {
        // never panic — always return PolicyResult
        PolicyResult { verdict: Verdict::Pass, .. }
    }
}
```

| Verdict | Meaning |
|---------|---------|
| `Pass` | All checks passed |
| `Warn` | Anomaly detected |
| `Suggest` | Non-blocking recommendation |

- All policies run in `suggest` mode (read-only, never destructive)
- Register in `policies::run_all_policies()`
- Location: `src/policies/`

---

## 6. LSP Analyzer

```rust
pub trait CicdAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> anyhow::Result<Vec<CicdFinding>>;
}
```

- Diagnostic only — report findings, never fix
- Location: `crates/cargo-cicd-lsp/src/analyzers/`
- Finding severities: `Error` · `Warning` · `Information`

Existing: `changed_tests` · `git_phase` · `target_hygiene`

---

## 7. Feature Flags

| Flag | Implies | Enables |
|------|---------|--------|
| `process-data` | — | EngineState, adapters, basic analytics |
| `autonomic` | `process-data` | Policy engine, suggest mode |
| `wasm4pm` | `process-data` | Evidence adjudication, wpm oracle |
| `affidavit` | `process-data` | Receipt engine, affidavit noun |
| `advanced` | — | parallel_scan, blake3, tracing, miette, moka, bitcode, petgraph, jiff, hdrhistogram, aho-corasick |

```rust
// CORRECT: gate internal adapter
#[cfg(feature = "process-data")]
mod git_status_adapter;

// WRONG: never gate public verb/trait
#[cfg(feature = "process-data")]
pub struct GitVerb;  // breaks public API
```

---

## 8. Error Handling

| Context | Type | Rule |
|---------|------|------|
| Adapters | `anyhow::Result` | Graceful degradation |
| Policies | `PolicyResult` | Never panic |
| CLI verbs | `clap_noun_verb::error::Result` | User-friendly messages |
| Core logic | `anyhow::Result` | Use `bail!()` with context |

```rust
// propagate with context
result.context("failed to scan targets")?;

// never silence
result.ok();  // FORBIDDEN
result.unwrap_or(default);  // only for truly infallible cases
```

---

## 9. Testing

| Tier | Location | Tools | Gate |
|------|----------|-------|------|
| Smoke/Unit | `tests/` | `assert_cmd`, `TempDir` | Always |
| Integration | `tests/` | `assert_cmd`, fixtures | Always |
| Evidence-gate | `tests/wasm4pm_*` | `assert_cmd`, `wpm` | Release |
| Feature projection | `tests/feature_projection.rs` | feature combinations | Always |

```rust
// Smoke
#[test]
fn test_git_status_shows_branch() {
    Command::cargo_bin("cargo-cicd").unwrap()
        .args(["git", "status"])
        .current_dir(TempDir::new().unwrap().path())
        .assert().success().stdout(predicate::str::contains("branch"));
}

// Evidence-gate (assert wpm verdict, never internal state)
#[test]
fn test_evidence_gate() {
    // run verb → emit OCEL evidence → shell wpm → assert Accept|Blocked
    let verdict = Command::new("wpm")
        .args(["audit", evidence_path])
        .output().expect("wpm failed");
    // Accept or Blocked are both valid; Refuse is a defect
}
```

Fixtures: `tests/fixtures/`

---

## Pattern → Location Map

| Pattern | Location | Trigger |
|---------|----------|---------|
| Noun-Verb | `src/nouns/` | New CLI command |
| Evidence Emission | Any verb doing work | Any mutation/decision |
| Adapter | `src/adapters/` | New external system |
| EngineState | `src/engine/` | Shared runtime state |
| Policy | `src/policies/` | New autonomic rule |
| LSP Analyzer | `crates/cargo-cicd-lsp/src/analyzers/` | Diagnostic findings |
| Lifecycle | `crates/cargo-cicd-lsp/src/lifecycle/` | Finding tracking |
| Feature Flags | `Cargo.toml` | Conditional compilation |
| Error Handling | All modules | Every fallible operation |
| Testing | `tests/` | Every feature, verb, adapter |
