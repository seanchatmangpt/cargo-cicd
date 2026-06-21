# cargo-cicd Architecture Guide

A visual guide to cargo-cicd's system design, data flows, and component interactions.

---

## 1. System Architecture Diagram

### Top-Level Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    User CLI Command                              │
│            cargo cicd <noun> [<verb>] [flags]                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                ┌────────────▼────────────┐
                │  Clap + clap-noun-verb  │
                │      CLI Parser         │
                └────────────┬────────────┘
                             │
        ┌────────────────────▼──────────────────┐
        │   Noun Router → Verb Dispatcher       │
        │  (status, git, test, workspace, etc.) │
        └────────────────────┬──────────────────┘
                             │
        ┌────────────────────▼──────────────────┐
        │   Verb Implementation                 │
        │  (show, doctor, status, close, etc.)  │
        │   Emits ProcessEvents                 │
        └────────────────────┬──────────────────┘
                             │
    ┌────────────────────────▼─────────────────────┐
    │          Adapters (External Sources)         │
    │  - GitStatusAdapter                          │
    │  - CargoMetadataAdapter                      │
    │  - ToolchainDetector                         │
    │  - ChangedFileDetector                       │
    │  - TargetScannerAdapter                      │
    │  - TrybuildDetector                          │
    │  - CicdTomlWriter                            │
    └────────────────────┬─────────────────────────┘
                         │
    ┌────────────────────▼─────────────────────┐
    │      EngineState (Aggregate Root)        │
    │  Unified workspace state snapshot        │
    │  All dimensions: workspace, toolchain,   │
    │  target, git, test, policy, etc.         │
    └────────────────────┬─────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
    ┌────────┐    ┌──────────────┐  ┌──────────┐
    │Policies│    │LSP Analyzers │  │Evidence  │
    │Engine  │    │              │  │Emission  │
    └────┬───┘    └──────┬───────┘  └────┬─────┘
         │                │               │
         │                ▼               │
         │         ┌──────────────┐      │
         └────────▶│  Findings    │◀─────┘
                   │ + Diagnostics│
                   └──────┬───────┘
                          │
                   ┌──────▼───────┐
                   │ Evidence Dir  │
                   │ (JSON, XES)   │
                   └──────┬───────┘
                          │
                   ┌──────▼──────────┐
                   │ wasm4pm Oracle   │
                   │ (wpm receipt     │
                   │  doctor --strict)│
                   └──────┬──────────┘
                          │
                   ┌──────▼──────────┐
                   │   WpmVerdict     │
                   │  ACCEPT/REFUSE   │
                   └──────────────────┘
```

### Key Design Principles

1. **Single Responsibility**: Each adapter translates one external source into the state model.
2. **Aggregate Root**: `EngineState` owns all runtime dimensions; verbs read from it.
3. **No Business Logic in Adapters**: Adapters are pure translation; logic lives in nouns/policies/analyzers.
4. **Evidence-Driven**: All work emits process evidence for wasm4pm adjudication.
5. **Recommend, Don't Mandate**: Policies run in suggest mode by default; users control action.

---

## 2. Data Flow Diagram

### Workspace → State → Insights

```
External World                        cargo-cicd internals
──────────────────────────────────────────────────────────

Workspace Artifacts              Adapters                State
┌─────────────────┐         ┌──────────────┐        ┌──────────────┐
│ .git/HEAD       │────────▶│ GitStatus    │───────▶│ GitPhaseState│
│ .git/refs/*     │         │ Adapter      │        │  branch      │
│ .git/index      │         └──────────────┘        │  dirty_files │
└─────────────────┘                                 │  ahead/behind│
                                                    └──────────────┘
┌─────────────────┐         ┌──────────────┐        ┌──────────────┐
│ Cargo.toml      │────────▶│ Cargo        │───────▶│Workspace     │
│ Cargo.lock      │         │ Metadata     │        │State         │
│ rust-toolchain  │         │ Adapter      │        │  name        │
│ .rustfmt.toml   │         └──────────────┘        │  root_path   │
└─────────────────┘                                 │  members     │
                                                    └──────────────┘
┌─────────────────┐         ┌──────────────┐        ┌──────────────┐
│ target/         │────────▶│ Target       │───────▶│TargetState   │
│                 │         │ Scanner      │        │  path        │
└─────────────────┘         │ Adapter      │        │  size_bytes  │
                            └──────────────┘        └──────────────┘
┌─────────────────┐         ┌──────────────┐        ┌──────────────┐
│ src/            │────────▶│ Changed      │───────▶│ChangedFile   │
│ tests/          │         │ File         │        │State         │
│ .git/objects    │         │ Detector     │        │  total_changed
└─────────────────┘         └──────────────┘        │  changed_rs_ │
                                                    │  files       │
                                                    └──────────────┘
┌─────────────────┐         ┌──────────────┐        ┌──────────────┐
│ tests/trybuild/ │────────▶│ Trybuild     │───────▶│TrybuildState │
│ tests/ui/       │         │ Detector     │        │  changed_    │
└─────────────────┘         └──────────────┘        │  fixtures    │
                                                    └──────────────┘


EngineState (Complete Snapshot)
┌──────────────────────────────────────────────┐
│ workspace: WorkspaceState                    │
│ toolchain: ToolchainState                    │
│ target: TargetState                          │
│ changed_files: ChangedFileState              │
│ test_plan: TestPlanState                     │
│ trybuild: TrybuildState                      │
│ git_phase: GitPhaseState                     │
│ process_events: ProcessEventState            │
│ artifacts: ArtifactState                     │
│ policies: PolicyState                        │
│ projection: ProjectionProfile                │
└────────────┬───────────────────────────────┘
             │
   ┌─────────┼─────────┬──────────┐
   │         │         │          │
   ▼         ▼         ▼          ▼
Policy   Findings  Evidence   UI Display
Engine   + LSP     Emission
         Analyzers
```

### Policy Evaluation Loop

```
EngineState (read-only)
       │
       ▼
┌─────────────────────────────────┐
│ run_all_policies()              │
│ - PolicyEngine evaluates each   │
│   policy in parallel            │
└─────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│ Policy Registry (7 policies)    │
│ ┌─────────────────────────────┐ │
│ │ GitPhaseDirtyPolicy         │ │
│ │ TargetPressurePolicy        │ │
│ │ ToolchainMismatchPolicy     │ │
│ │ TrybuildChangedPolicy       │ │
│ │ BranchBehindPolicy          │ │
│ │ EvidenceStalePoliciy        │ │
│ │ PublishNotAdjudicatedPolicy │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
       │
       ▼
   [PARALLEL]
       │
   ┌───┴───────────────────┬──────────────────┐
   │                       │                  │
   ▼                       ▼                  ▼
[Policy 1]            [Policy 2]         [Policy 3]
   Signals               Signals            Signals
   + Verdict             + Verdict          + Verdict
   + Recommendation      + Recommendation   + Recommendation
   │                       │                  │
   └───────────┬───────────┴──────────────────┘
               │
               ▼
       Vec<PolicyState>
               │
               ▼
       ┌────────────────┐
       │ User sees:     │
       │ [PASS] policy1 │
       │ [WARN] policy2 │
       │  suggestion    │
       └────────────────┘
```

### Evidence Emission & wasm4pm Adjudication

```
Verb executes
   │
   ▼
ProcessEvent::started("verb:action")
   │ (emitted to ProcessEventState)
   │
   ▼
[Do Work]
   │
   ▼
Determine outcome:
  PASS / FAIL / WARN / SKIP
   │
   ▼
ProcessEvent::completed(
  "verb:action",
  start_time,
  end_time,
  verdict
)
   │ (appended to ProcessEventState)
   │
   ▼
[Events collected]
   │
   ▼
evidence_dir/events.jsonl
evidence_dir/events.xes (XES format)
   │
   ▼
User: cargo cicd evidence doctor
   │
   ▼
wpm receipt doctor --format json --strict
   │
   ▼
┌──────────────────┐
│ WpmVerdict {     │
│  overall_fitness │
│  precision       │
│  verdict         │
│ }                │
└──────────────────┘
   │
   ▼
ACCEPT or REFUSE
```

---

## 3. Noun-Verb Structure

The CLI exposes a **noun-verb grammar** via `clap-noun-verb`. Each noun is a module in `src/nouns/`; verbs are subcommands within.

```
┌──────────────────────────────────────────────────────────────────┐
│                    Nouns & Verbs Registry                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  status (module: src/nouns/status.rs)                           │
│    ├─ show (default) — display workspace status                 │
│    └─ [implicit via default verb injection]                     │
│       usage: cargo cicd status                                  │
│              cargo cicd status show                             │
│                                                                  │
│  git (module: src/nouns/git.rs)                                 │
│    ├─ status — git phase state                                  │
│    ├─ close — finalize git transaction                          │
│    ├─ diff — show diff against origin/main                      │
│    ├─ stage — git add changed files                             │
│    ├─ commit — git commit staged                                │
│    └─ fetch — git fetch origin                                  │
│                                                                  │
│  test (module: src/nouns/test.rs)                               │
│    ├─ changed — run tests for changed files only                │
│    ├─ all — run all tests in workspace                          │
│    └─ by-name — run tests matching pattern                      │
│                                                                  │
│  workspace (module: src/nouns/workspace.rs)                     │
│    ├─ doctor (default) — check workspace health                 │
│    ├─ members — list workspace members                          │
│    └─ graph — show member dependency graph                      │
│       usage: cargo cicd workspace                               │
│              cargo cicd workspace doctor                        │
│                                                                  │
│  target (module: src/nouns/target.rs)                           │
│    ├─ clean — remove target/ artifacts                          │
│    ├─ inspect — analyze target/ size                            │
│    └─ estimate — predict clean impact                           │
│                                                                  │
│  publish (module: src/nouns/publish.rs)                         │
│    ├─ run (default) — publish to registry                       │
│    ├─ dry-run — test publication                               │
│    └─ yank — remove published version                           │
│       usage: cargo cicd publish                                 │
│              cargo cicd publish run                             │
│                                                                  │
│  trybuild (module: src/nouns/trybuild.rs)                       │
│    ├─ test — run trybuild UI/macro tests                        │
│    └─ update — regenerate .stderr files                         │
│                                                                  │
│  evidence (module: src/nouns/evidence.rs)                       │
│    ├─ doctor (default) — audit process evidence                 │
│    ├─ audit — check evidence integrity                          │
│    └─ export — export evidence to wasm4pm                       │
│       usage: cargo cicd evidence                                │
│              cargo cicd evidence doctor                         │
│                                                                  │
│  lsp (module: src/nouns/lsp.rs)                                 │
│    ├─ run — start LSP diagnostics server                        │
│    └─ check — run all LSP analyzers once                        │
│                                                                  │
│  pipeline (module: src/nouns/pipeline.rs)                       │
│    ├─ run — execute full CI/CD pipeline                         │
│    └─ dry-run — show pipeline steps without executing           │
│                                                                  │
│  analyze (module: src/nouns/analyze.rs)                         │
│    ├─ deps — analyze dependency graph                           │
│    ├─ complexity — code complexity metrics                      │
│    └─ coverage — test coverage summary                          │
│                                                                  │
│  autoarch (module: src/nouns/autoarch.rs)                       │
│    ├─ show — display auto-architecture suggestions              │
│    └─ apply — apply architecture recommendations                │
│                                                                  │
│  certification (module: src/nouns/certification.rs)             │
│    ├─ check — verify certification requirements                 │
│    ├─ sign — sign a release artifact                            │
│    └─ verify — verify a signed artifact                         │
│                                                                  │
│  sbom (module: src/nouns/sbom.rs)                               │
│    ├─ generate — produce a Software Bill of Materials           │
│    └─ verify — validate SBOM against workspace                  │
│                                                                  │
│  ui (module: src/nouns/ui.rs)                                   │
│    ├─ demo — showcase terminal UI components                    │
│    └─ dashboard — display workspace dashboard                   │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

Default Verb Injection (main.rs::inject_default_verbs)
────────────────────────────────────────────────────────

  status  →  status show
  publish →  publish run
  workspace → workspace doctor
  evidence → evidence doctor

This preserves internal noun-verb grammar while exposing a simpler
public surface: users can type bare nouns for common operations.
```

---

## 4. EngineState Composition

`EngineState` is the **aggregate root** — a single struct containing all workspace state dimensions.

```
┌────────────────────────────────────────────────────────────────┐
│                       EngineState                              │
│          (src/engine/mod.rs — all runtime dimensions)         │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ pub struct EngineState {                                        │
│                                                                │
│   ┌─ WorkspaceState                                            │
│   │  ├─ name: String              // e.g., "cargo-cicd"      │
│   │  ├─ root_path: String         // workspace root          │
│   │  ├─ members: Vec<String>      // crate members           │
│   │  ├─ toolchain: String         // rustup toolchain        │
│   │  └─ rust_edition: String      // e.g., "2021"            │
│   │                                                            │
│   ├─ ToolchainState                                            │
│   │  ├─ active: String            // active toolchain name   │
│   │  └─ rust_version: String      // rustc version output    │
│   │                                                            │
│   ├─ TargetState                                               │
│   │  ├─ path: String              // target/ directory path  │
│   │  └─ total_size_bytes: u64     // du -s result            │
│   │                                                            │
│   ├─ ChangedFileState                                          │
│   │  ├─ base_ref: String          // base branch (origin/main)
│   │  ├─ total_changed: usize      // count of changed files  │
│   │  ├─ changed_rs_files: Vec<String>    // .rs files        │
│   │  ├─ changed_test_files: Vec<String>  // test files       │
│   │  └─ changed_trybuild_fixtures: Vec<String>               │
│   │                                                            │
│   ├─ TestPlanState                                             │
│   │  ├─ estimated_count: usize    // tests to run            │
│   │  └─ conservative_mode: bool   // run more if changed     │
│   │                                                            │
│   ├─ TrybuildState                                             │
│   │  ├─ fixture_count: usize      // total fixtures          │
│   │  └─ changed_fixtures: Vec<String>  // changed ones       │
│   │                                                            │
│   ├─ GitPhaseState                                             │
│   │  ├─ branch: String            // current branch name     │
│   │  ├─ dirty_files: usize        // unstaged changes        │
│   │  ├─ staged_files: usize       // staging area            │
│   │  ├─ untracked_files: usize    // new files              │
│   │  ├─ ahead: usize              // commits ahead of base   │
│   │  └─ behind: usize             // commits behind base     │
│   │                                                            │
│   ├─ ProcessEventState                                         │
│   │  └─ events: Vec<ProcessEvent>  // [started, completed]   │
│   │                                                            │
│   ├─ ArtifactState                                             │
│   │  ├─ binaries: Vec<String>     // built binaries          │
│   │  └─ publish_ready: bool       // can publish now?        │
│   │                                                            │
│   ├─ PolicyState                                               │
│   │  └─ entries: Vec<PolicyEntry> // policy verdicts         │
│   │                                                            │
│   └─ ProjectionProfile                                         │
│      └─ version: String           // compatibility marker    │
│                                                                │
│ }                                                              │
└────────────────────────────────────────────────────────────────┘

Construction: EngineState::from_workspace()
───────────────────────────────────────────

Called once per command invocation. Queries all adapters in parallel.
Failures are silenced — partial data is better than no data.

  1. CargoMetadataAdapter::workspace_name()        → workspace.name
  2. CargoMetadataAdapter::target_dir()            → workspace.root_path
  3. CargoMetadataAdapter::workspace_members()     → workspace.members
  4. detect_toolchain()                            → workspace.toolchain
  5. detect_rust_edition()                         → workspace.rust_edition
  6. GitStatusAdapter::query()                     → git_phase.*
  7. ToolchainDetector::*                          → toolchain.*
  8. TargetScannerAdapter::*                       → target.*
  9. ChangedFileDetector::changed_rs_files()       → changed_files.*
 10. TrybuildDetector::*                           → trybuild.*
 11. (ProcessEvents, Artifacts, Policies populated by verbs at runtime)
```

---

## 5. Adapter Composition

Each **adapter** is a pure translator from external sources into `EngineState`.

```
External Sources         Adapters              EngineState
──────────────────────────────────────────────────────────

┌─────────────────┐   ┌──────────────────┐
│ .git/HEAD       │   │                  │
│ .git/refs/*     │──▶│ GitStatusAdapter │──▶ git_phase:
│ .git/index      │   │                  │    - branch
└─────────────────┘   └──────────────────┘    - dirty_files
                                              - ahead/behind


┌─────────────────┐   ┌──────────────────┐
│ Cargo.toml      │   │ CargoMetadata    │
│ Cargo.lock      │──▶│ Adapter          │──▶ workspace:
│ Cargo.lock.lock │   │                  │    - name
└─────────────────┘   └──────────────────┘    - root_path
                                              - members


┌─────────────────┐   ┌──────────────────┐
│ rust-toolchain  │   │ Toolchain        │
│ rustc --version │──▶│ Detector         │──▶ toolchain:
│                 │   │                  │    - active
└─────────────────┘   └──────────────────┘    - rust_version


┌─────────────────┐   ┌──────────────────┐
│ target/         │   │ TargetScanner    │
│ du -sh target/  │──▶│ Adapter          │──▶ target:
│                 │   │                  │    - path
└─────────────────┘   └──────────────────┘    - total_size_bytes


┌─────────────────┐   ┌──────────────────┐
│ src/, tests/    │   │ ChangedFile      │
│ git diff        │──▶│ Detector         │──▶ changed_files:
│                 │   │                  │    - total_changed
└─────────────────┘   └──────────────────┘    - changed_rs_files


┌─────────────────┐   ┌──────────────────┐
│ tests/trybuild/ │   │ Trybuild         │
│ tests/ui/       │──▶│ Detector         │──▶ trybuild:
│                 │   │                  │    - fixture_count
└─────────────────┘   └──────────────────┘    - changed_fixtures


┌─────────────────┐   ┌──────────────────┐
│ cicd.toml       │   │ CicdToml         │
│ (in workspace)  │──▶│ Writer           │──▶ cicd_toml state
│                 │   │  [persistence]   │    (written back)
└─────────────────┘   └──────────────────┘


Design Principles:
─────────────────
✓ One adapter = one external source
✓ Adapters read, never write (except CicdTomlWriter)
✓ No business logic — pure translation
✓ Silent failures — partial data OK
✓ All adapters queried in parallel (via EngineState::from_workspace)
```

---

## 6. LSP Analyzer Stack

LSP (Language Server Protocol) analyzers run **static checks** on the workspace without executing code.

```
WorkspaceSnapshot (root path, readonly)
       │
       ▼
    run_all()
    ├─────────────────────────────────────────┤
    │      [10+ Parallel Analyzers]           │
    │                                          │
    ├─ WorkspaceStructureAnalyzer             │
    │  • Check member manifest correctness     │
    │  • Verify Cargo.toml schema              │
    │  • Detect malformed workspace            │
    │                                          │
    ├─ PipelineCheckAnalyzer                  │
    │  • Verify CI pipeline .yml syntax        │
    │  • Check GitHub Actions workflows        │
    │                                          │
    ├─ RemoteTrackingAnalyzer                 │
    │  • Verify remote branches are tracked    │
    │  • Check origin/main exists              │
    │                                          │
    ├─ ChangedTestsAnalyzer                   │
    │  • Detect tests changed since base       │
    │  • Flag modified test fixtures           │
    │                                          │
    ├─ GitPhaseAnalyzer                       │
    │  • Check for uncommitted changes         │
    │  • Warn if ahead/behind threshold        │
    │  • Detect detached HEAD state            │
    │                                          │
    ├─ TargetHygieneAnalyzer                  │
    │  • Warn if target/ oversized             │
    │  • Suggest cleanup before publish        │
    │                                          │
    ├─ PublicBoundaryAnalyzer                 │
    │  • Enforce forbidden term ban            │
    │  • Verify public API surface             │
    │  • Check documentation completeness      │
    │                                          │
    ├─ PublishAnalyzer                        │
    │  • Check crate metadata (name, version)  │
    │  • Verify publish permissions            │
    │  • Warn if version already published     │
    │                                          │
    ├─ GgenCustomizationAnalyzer              │
    │  • Detect custom noun/verb implementations
    │  • Flag out-of-date ggen artifacts       │
    │                                          │
    ├─ RenderedSurfaceAnalyzer                │
    │  • Verify all generated CLI code         │
    │  • Check against ggen.toml               │
    │                                          │
    └─ CloseReadinessAnalyzer                 │
       • Final release gate checks             │
       • Evidence emitted & adjudicated        │
       • All tests passing                     │
    │                                          │
    └──────────────────────────────────────────┘
       │
       ▼
    Vec<CicdFinding>
    ┌──────────────────────────────────┐
    │ struct CicdFinding {              │
    │   file: String,                  │
    │   line: usize,                   │
    │   level: Severity,               │
    │   message: String,               │
    │   suggestion: Option<String>,    │
    │ }                                │
    └──────────────────────────────────┘
       │
       ▼
    User sees findings as:
    • LSP Diagnostics (in IDE)
    • CLI output (cargo cicd lsp check)
    • Evidence entries (for wasm4pm)
```

---

## 7. Policy Evaluation Loop

**Policies** are autonomic agents that read workspace state and emit recommendations.

```
┌───────────────────────────────────────────────────────────────┐
│          Policy Evaluation Loop (src/autonomic/)              │
└───────────────────────────────────────────────────────────────┘

Input: EngineState (read-only), PolicyMode, AutonomicMode

AutonomicMode:
  • Suggest (default) — print recommendations only
  • Apply             — execute safe remediation automatically


Policy Registry (7 policies in parallel):
────────────────────────────────────────

┌──────────────────────────┐
│ GitPhaseDirtyPolicy      │
│ Signals: [dirty_files]   │
│ Checks: unstaged changes │
│ Action: Suggest commit   │
└──────────────────────────┘

┌──────────────────────────┐
│ TargetPressurePolicy     │
│ Signals: [size > 5GB]    │
│ Checks: target/ size     │
│ Action: Suggest clean    │
└──────────────────────────┘

┌──────────────────────────┐
│ ToolchainMismatchPolicy  │
│ Signals: [mismatch]      │
│ Checks: rust-toolchain   │
│ Action: Suggest update   │
└──────────────────────────┘

┌──────────────────────────┐
│ TrybuildChangedPolicy    │
│ Signals: [fixtures]      │
│ Checks: changed ui tests │
│ Action: Suggest re-run   │
└──────────────────────────┘

┌──────────────────────────┐
│ BranchBehindPolicy       │
│ Signals: [behind > 10]   │
│ Checks: ahead/behind     │
│ Action: Suggest pull     │
│ (Warning only)           │
└──────────────────────────┘

┌──────────────────────────┐
│ EvidenceStalePoliciy     │
│ Signals: [stale events]  │
│ Checks: process events   │
│ Action: Run changed tests│
│ (Auto-remediate in       │
│  Apply mode)             │
└──────────────────────────┘

┌──────────────────────────┐
│ PublishNotAdjudicated    │
│ Signals: [no verdict]    │
│ Checks: wasm4pm status   │
│ Action: Suggest audit    │
└──────────────────────────┘


Output per policy:
────────────────

struct PolicyState {
  name: String,
  mode: PolicyMode,            // Disabled | Suggest | Apply
  signals: Vec<String>,        // what triggered this
  recommendation: String,      // human-readable action
  verdict: PolicyVerdict,      // Pass | Warn | Fail
}


User Output:
────────────

[PASS] git_phase_dirty
  ✓ No uncommitted changes


[WARN] target_pressure
  ⚠ target/ is 6.2GB
  → run: cargo cicd target clean


[PASS] toolchain_mismatch
  ✓ Toolchain matches rust-toolchain


etc.
```

---

## 8. Evidence Emission Lifecycle

All cargo-cicd work emits process evidence as it runs.

```
┌─────────────────────────────────────────────────────────────┐
│         Evidence Emission Lifecycle                         │
│     (src/evidence.rs + ProcessEventState)                  │
└─────────────────────────────────────────────────────────────┘

1. Verb starts
   ─────────────
   User: cargo cicd test changed
            │
            ▼
   VerbImpl::run()
            │
            ▼
   ProcessEvent::started("test:changed")
            │
            ├─ timestamp: SystemTime::now()
            ├─ action: "test:changed"
            ├─ status: "started"
            └─ → ProcessEventState.events.push()


2. Do work
   ──────────
   Run tests, collect results
            │
            ▼
   Observe outcomes:
   ├─ Tests passed     → PASS
   ├─ Tests failed     → FAIL
   ├─ Tests skipped    → SKIP
   └─ Test timeout     → FAIL (with reason)


3. Complete & emit verdict
   ────────────────────────
   ProcessEvent::completed(
     "test:changed",
     start_time,
     end_time,
     verdict: PASS | FAIL | WARN | SKIP
   )
            │
            ├─ duration: end_time - start_time
            ├─ status: "completed"
            └─ → ProcessEventState.events.push()


4. Persist evidence
   ─────────────────
   In target/cargo-cicd/evidence/:
            │
            ├─ events.jsonl
            │  Line 1: {"action":"test:changed","status":"started",...}
            │  Line 2: {"action":"test:changed","status":"completed","verdict":"pass",...}
            │
            └─ events.xes
               XES (XML Event Stream) format for wasm4pm
               <event>
                 <string key="action" value="test:changed"/>
                 <string key="status" value="completed"/>
                 <string key="verdict" value="pass"/>
               </event>


5. User audits evidence
   ──────────────────────
   cargo cicd evidence doctor
            │
            ▼
   Reads target/cargo-cicd/evidence/events.xes
            │
            ▼
   Invokes wasm4pm oracle:
   $ wpm receipt doctor --format json --strict \
     target/cargo-cicd/evidence/latest.json
            │
            ▼
   WpmVerdict {
     overall_fitness: f64,    // 0.0–1.0
     precision: f64,          // evidence quality
     verdict: "ACCEPT" | "REFUSE"
   }
            │
            ▼
   User sees:
   ✓ Evidence adjudicated: ACCEPT
   or
   ✗ Evidence adjudicated: REFUSE (check wasm4pm logs)


6. Release gate
   ──────────────
   No release may claim completion without:
   ├─ Internal tests passing (cargo test --workspace)
   ├─ Evidence gate ACCEPT (wasm4pm verdict)
   ├─ Workspace doctor PASS (all policies clean)
   └─ cicd.toml up-to-date
```

---

## 9. Release Gate Checklist

Before pushing a release:

```
┌────────────────────────────────────────────────────────┐
│       Release Gate Checklist (v26.6.2)                 │
├────────────────────────────────────────────────────────┤
│                                                         │
│  [ ] Unit & integration tests pass                     │
│      $ cargo make test                                 │
│      $ cargo test --test invariants                    │
│      $ cargo test --test feature_projection            │
│                                                         │
│  [ ] Evidence gate ACCEPT                              │
│      $ cargo cicd evidence doctor                      │
│      → wasm4pm verdict: ACCEPT                         │
│                                                         │
│  [ ] Workspace health PASS                             │
│      $ cargo cicd workspace doctor                     │
│      → All policies: [PASS]                            │
│                                                         │
│  [ ] cicd.toml up-to-date                              │
│      $ git status cicd.toml                            │
│      → Should be committed or absent                   │
│                                                         │
│  [ ] No forbidden terms in public APIs                 │
│      ✗ ALIVE, Inspection Gate, wall, Nehemiah,        │
│        Field8, Instinct8, Cargo Court, AGI, Truex,    │
│        CONSTRUCT8                                      │
│      $ cargo cicd lsp check                            │
│      → PublicBoundaryAnalyzer: clean                   │
│                                                         │
│  [ ] Dependency updates applied                        │
│      $ cargo update                                    │
│      $ cargo outdated                                  │
│      → Review and commit Cargo.lock                    │
│                                                         │
│  [ ] Commit message format                             │
│      feat(core|cli|target|test|git|autonomic|docs|receipts): ...
│                                                         │
│  [ ] Git state ready                                   │
│      $ cargo cicd git status                           │
│      → No dirty files, no untracked                    │
│      → Not ahead/behind origin/main                    │
│                                                         │
└────────────────────────────────────────────────────────┘
```

---

## 10. Feature Flags & Configuration

### Feature Flags

```
process-data
  Enables:
    • EngineState internals
    • Adapters
    • ProcessEventState
    • Target-internal LSP analyzers
  Default: disabled (internal use only)

autonomic
  Implies: process-data
  Enables:
    • PolicyEngine
    • Autonomic policy suggestions
    • Safe auto-remediation in Apply mode
  Default: disabled (needs explicit --features)

wasm4pm
  Implies: process-data
  Enables:
    • WasmPM oracle integration seam
    • Evidence serialization to XES
    • Runtime integration for adjudication
  Note: NOT optional for v26.6.2+ releases

contrib
  Implies: process-data
  Enables:
    • Contrib noun commands
    • Extended diagnostics for contributors
  Default: disabled (for development only)


Build examples:
───────────────
$ cargo build
  → Minimal binary, no engine internals

$ cargo build --features process-data
  → Full engine, adapters, LSP, no policies

$ cargo build --features autonomic
  → Full engine + policies + auto-suggest

$ cargo build --features wasm4pm
  → Full engine + evidence gate + wasm4pm oracle integration

$ cargo build --features autonomic,wasm4pm
  → Complete release build
```

### cicd.toml Configuration

```toml
[workspace]
name = "cargo-cicd"
root = "/path/to/repo"
members = ["src/", "tests/", ...]

[state]
last_updated = "2026-06-14T10:30:00Z"
git_branch = "main"
git_ahead = 0
git_behind = 0

[target]
path = "target/"
size_gb = 1.2

[autonomic]
mode = "suggest"  # or "apply"
enabled = true

[[events]]
action = "test:changed"
status = "started"
timestamp = "2026-06-14T10:30:00Z"

[[events]]
action = "test:changed"
status = "completed"
verdict = "pass"
duration_ms = 5432
timestamp = "2026-06-14T10:35:32Z"
```

---

## 11. File Structure Overview

```
/home/user/cargo-cicd/
├── src/
│   ├── main.rs                    # CLI entry point, default verb injection
│   ├── lib.rs
│   │
│   ├── adapters/                  # External source → EngineState
│   │   ├── mod.rs
│   │   ├── git_status.rs          # git status query
│   │   ├── cargo_metadata.rs      # cargo metadata
│   │   ├── toolchain_detector.rs  # rustup queries
│   │   ├── target_scanner.rs      # du target/
│   │   ├── changed_file_detector.rs # git diff
│   │   ├── trybuild_detector.rs   # tests/ scan
│   │   └── cicd_toml_writer.rs    # cicd.toml persistence
│   │
│   ├── engine/                    # EngineState aggregate root
│   │   ├── mod.rs                 # EngineState struct
│   │   ├── workspace_state.rs
│   │   ├── toolchain_state.rs
│   │   ├── target_state.rs
│   │   ├── git_phase_state.rs
│   │   ├── changed_file_state.rs
│   │   ├── test_plan_state.rs
│   │   ├── trybuild_state.rs
│   │   ├── process_event_state.rs # ProcessEvent, evidence
│   │   ├── artifact_state.rs
│   │   ├── policy_state.rs
│   │   └── projection_profile.rs
│   │
│   ├── nouns/                     # Noun implementations (CLI commands)
│   │   ├── mod.rs
│   │   ├── status.rs              # status noun
│   │   ├── git.rs                 # git noun
│   │   ├── test.rs                # test noun
│   │   ├── workspace.rs           # workspace noun
│   │   ├── target.rs              # target noun
│   │   ├── publish.rs             # publish noun
│   │   ├── trybuild.rs            # trybuild noun
│   │   ├── evidence.rs            # evidence noun
│   │   ├── lsp.rs                 # lsp noun
│   │   ├── pipeline.rs            # pipeline noun
│   │   ├── analyze.rs             # analyze noun
│   │   ├── autoarch.rs            # autoarch noun
│   │   ├── certification.rs       # certification noun
│   │   ├── sbom.rs                # sbom noun
│   │   └── ui.rs                  # ui noun
│   │
│   ├── policies/                  # Autonomic policy implementations
│   │   ├── mod.rs
│   │   ├── git_phase_dirty.rs
│   │   ├── target_pressure.rs
│   │   ├── toolchain_mismatch.rs
│   │   ├── trybuild_changed.rs
│   │   ├── branch_behind.rs
│   │   ├── evidence_stale.rs
│   │   └── publish_not_adjudicated.rs
│   │
│   ├── autonomic/                 # Policy engine & evaluation
│   │   ├── mod.rs
│   │   ├── policy_engine.rs       # run_suggestions(), run_with_mode()
│   │   ├── signals.rs             # signal extraction from state
│   │   └── policies.rs            # trait CicdPolicy definition
│   │
│   ├── integrations/              # External system integration
│   │   ├── mod.rs
│   │   ├── wasm4pm_shell.rs       # shell invocation of wpm
│   │   └── wasm4pm_current.rs     # XES serialization
│   │
│   ├── evidence.rs                # ProcessEvent, evidence emission API
│   ├── cicd_toml.rs               # cicd.toml schema, parsing
│   ├── session.rs                 # Session state for long-running ops
│   └── state/                     # Legacy state schema (deprecated)
│       ├── mod.rs
│       ├── event.rs
│       ├── policy.rs
│       ├── git_phase.rs
│       ├── workspace.rs
│       ├── target.rs
│       ├── test_plan.rs
│       ├── projection.rs
│       ├── toolchain.rs
│       └── changed.rs
│
├── tests/
│   ├── invariants.rs              # 7 public boundary invariants
│   ├── cli/                       # CLI parsing tests
│   ├── cicd_toml_truth.rs         # cicd.toml schema tests
│   ├── autonomic_policies.rs      # Policy evaluation tests
│   ├── changed_tests.rs           # ChangedFileDetector tests
│   ├── git_phase_closure.rs       # GitPhase state tests
│   ├── feature_projection.rs      # Feature flag surface contract
│   ├── wasm4pm_evidence_gate.rs   # Evidence → wasm4pm tests
│   ├── wasm4pm_evidence_mutation.rs
│   ├── wasm4pm_refusal_cases.rs
│   └── fixtures/                  # Test workspace fixtures
│       ├── simple/
│       ├── multi-member/
│       ├── with-trybuild/
│       └── dirty-git/
│
├── Cargo.toml                     # Package manifest
├── Cargo.lock                     # Lockfile (committed)
├── Makefile.toml                  # cargo-make tasks
├── cicd.toml                      # Current workspace state (generated)
│
├── CLAUDE.md                      # Project instructions (this file)
├── .claude/
│   └── ARCHITECTURE.md            # This file
│
├── ggen.toml                      # Code generation config
├── ontology/
│   └── cargo-cicd.ttl             # RDF ontology (manufacturing spec)
├── templates/                     # Tera code generation templates
├── queries/                       # SPARQL queries for ggen
│
└── .gitignore
    target/
    cicd.toml
    .cargo/
```

---

## 12. Example: End-to-End Flow

### Scenario: User runs `cargo cicd test changed`

```
1. User types command
   ────────────────────
   $ cargo cicd test changed
   
   ↓ Cargo protocol adds prefix
   $ cargo-cicd cicd test changed

   ↓ main.rs strips "cicd" prefix
   argv: ["cargo-cicd", "test", "changed"]


2. CLI parsing
   ────────────
   CliBuilder::new()
     .name("cargo-cicd")
     .version("26.6.2")
   
   Clap parses:
     noun = "test"
     verb = "changed"
   
   Calls: nouns::test::TestNoun::changed()


3. Noun/Verb implementation
   ───────────────────────────
   TestNoun::changed() {
     // Create snapshot of workspace state
     let state = EngineState::from_workspace();
     
     // Queries all adapters in parallel:
     //   - CargoMetadataAdapter → workspace
     //   - GitStatusAdapter → git_phase
     //   - ToolchainDetector → toolchain
     //   - TargetScannerAdapter → target
     //   - ChangedFileDetector → changed_files
     //   - etc.
     
     ↓ state.changed_files.changed_rs_files = ["src/main.rs", "tests/cli.rs"]
     
     // Determine which tests to run
     let mut cmd = Command::new("cargo");
     cmd.args(&["test", "--changed-only"]);
     
     // Emit start event
     ProcessEvent::started("test:changed")
     
     ↓ Appended to ProcessEventState
   }


4. Subprocess execution
   ──────────────────────
   $ cargo test --lib --test cli  (only changed test files)
   
   stdout:
     running 42 tests
     
     test test_cli_help ... ok
     test test_cli_status ... ok
     ...
     test result: ok. 42 passed


5. Verdict & completion
   ──────────────────────
   Observe exit code: 0 (success)
   
   Determine verdict: PASS
   
   ProcessEvent::completed(
     action: "test:changed",
     start_time: 10:30:00,
     end_time: 10:35:32,
     verdict: PASS,
     duration_ms: 5432
   )
   
   ↓ Appended to ProcessEventState


6. Evidence emission
   ──────────────────
   Create target/cargo-cicd/evidence/:
   
   events.jsonl:
   {"action":"test:changed","status":"started","timestamp":"2026-06-14T10:30:00Z"}
   {"action":"test:changed","status":"completed","verdict":"pass","duration_ms":5432,"timestamp":"2026-06-14T10:35:32Z"}
   
   events.xes:
   <log>
     <event>
       <string key="action" value="test:changed"/>
       <string key="status" value="started"/>
     </event>
     <event>
       <string key="action" value="test:changed"/>
       <string key="status" value="completed"/>
       <string key="verdict" value="pass"/>
     </event>
   </log>


7. User output
   ────────────
   ✓ Tests passed (42/42)
   ✓ Evidence saved to target/cargo-cicd/evidence/
   
   User can then audit:
   $ cargo cicd evidence doctor
   
   ↓ wasm4pm oracle invoked
   ↓ XES file validated
   ↓ Receipt doctor runs strict checks
   
   ✓ Evidence adjudicated: ACCEPT


8. Policy suggestions (autonomic mode)
   ─────────────────────────────────────
   If --autonomic flag or [autonomic] section in cicd.toml:
   
   run_suggestions(&state)
   
   [PASS] git_phase_dirty
   [PASS] target_pressure
   [WARN] branch_behind: "3 commits behind origin/main"
     → run: git fetch && git pull
   
   User can inspect/act on suggestions.
```

---

## Summary

**cargo-cicd** is a **manufactured Level 5 process-data engine** that:

1. **Parses CLI** via `clap-noun-verb` noun-verb grammar
2. **Snapshots workspace state** via parallel adapters into `EngineState`
3. **Executes work** (test, publish, clean) via noun/verb implementations
4. **Emits evidence** as process events (JSONL + XES)
5. **Evaluates policies** asynchronously (7 autonomic agents)
6. **Runs LSP analyzers** for static diagnostics
7. **Submits evidence** to wasm4pm oracle for adjudication
8. **Reports verdicts** to user (ACCEPT/REFUSE/WARN/PASS)

All work is **evidence-driven**: internal tests pass, but **release closure requires wasm4pm ACCEPT**.

---

**File location**: `/home/user/cargo-cicd/.claude/ARCHITECTURE.md`

**Generated**: 2026-06-14

**Version**: cargo-cicd 26.6.2
