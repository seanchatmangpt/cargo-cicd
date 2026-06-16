# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Claude Code Configuration — cargo-cicd

## Mission
cargo-cicd is a manufactured Level 5 process-data engine exposed as a boring Rust CI/CD helper.

## Key Facts
- Public: "cargo-cicd keeps Rust workspaces clean, fast, and push-ready."
- Private: Level 5 process-data engine with cicd.toml carrier
- Manufacturing: ggen → clap-noun-verb → cargo-cicd
- Gate: Dung Gate (output/artifact manufacture)

## FORBIDDEN in public docs/CLI/help text
ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8

## Commit Format
feat(core|cli|target|test|git|autonomic|docs|receipts): description

---

## Build & Test Commands

```sh
# Build
cargo make build          # preferred — uses cargo-make
cargo build               # fallback if cargo-make unavailable

# Check (lint + type-check without building)
cargo make check

# Run all tests
cargo make test

# Run a single integration test by name
cargo test --test invariants
cargo test --test cli
cargo test --test cicd_toml_truth
cargo test --test autonomic_policies
cargo test --test changed_tests
cargo test --test git_phase_closure
cargo test --test feature_projection

# Run a specific test function
cargo test --test invariants test_function_name

# Run with a feature flag
cargo test --features process-data
cargo test --features autonomic
```

---

## Architecture

### Noun-Verb CLI Grammar
The CLI uses `clap-noun-verb` (local crate at `/Users/sac/clap-noun-verb`). Each noun is a module in `src/nouns/` implementing `NounCommand`. Verbs within each noun implement `VerbCommand`. Default verb injection happens in `main.rs::inject_default_verbs()` so bare nouns work (e.g. `cargo cicd status` → `status show`).

**Nouns:** `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`

### Level 5 Engine State (`src/engine/`)
`EngineState` is the aggregate root — a struct of all runtime dimensions:
- `WorkspaceState`, `ToolchainState`, `TargetState`
- `ChangedFileState`, `TestPlanState`, `TrybuildState`
- `GitPhaseState`, `ProcessEventState`, `ArtifactState`
- `PolicyState`, `ProjectionProfile`

Nouns read from `EngineState`; adapters populate it from external sources.

### Adapters (`src/adapters/`)
Each adapter owns one external source: `GitStatusAdapter`, `TargetScannerAdapter`, `ToolchainDetector`, `CargoMetadataAdapter`, `ChangedFileDetector`, `CicdTomlWriter`, `TrybuildDetector`. Adapters translate external representations into the internal state model — no business logic.

### cicd.toml
`cicd.toml` is the carrier/state file written to the workspace root. It stores workspace config (`[workspace]`, `[state]`, `[target]`, etc.) and emitted `[[events]]`. `CicdToml` in `src/cicd_toml.rs` owns its schema; `CicdTomlWriter` in adapters owns writes.

### ggen / Ontology Pipeline
`ggen.toml` + `ontology/cargo-cicd.ttl` + SPARQL queries in `queries/` + Tera templates in `templates/` are the manufacturing pipeline for generating noun modules and CLI test scaffolding. Run `ggen` to regenerate from ontology changes.

### Feature Flags
- `process-data` — enables Level 5 engine internals
- `autonomic` — implies `process-data`; enables policy/suggest mode
- `wasm4pm` — implies `process-data`; wasm4pm integration seam (richer runtime integration, not the evidence-gate acceptance law)
- `contrib` — implies `process-data`

### wasm4pm Evidence Gate

wasm4pm is not an optional future integration for acceptance testing.

For v26.6.2, cargo-cicd must emit process evidence and the evidence-gate tests must submit that evidence to the discovered current wpm oracle.

Internal smoke tests may pass, but release closure requires:

  cargo-cicd emits → wasm4pm adjudicates → tests assert wasm4pm verdict.

The wasm4pm feature flag gates richer runtime integration, not the evidence-gate acceptance law.

wpm binary: /Users/sac/wasm4pm/target/release/wpm
Primary oracle command: wpm receipt doctor --format json --strict <receipt.json>
Secondary XES health check: wpm audit <file.xes>
Evidence format: XES (XML Event Stream), not JSONL
Evidence dir: target/cargo-cicd/evidence/

### Test Hierarchy

1. Unit/smoke/projection tests (non-closing):
   - May use assert_cmd/tempfile
   - May test CLI parsing, public boundaries, schemas
   - Files: tests/invariants.rs, tests/cli/, tests/feature_projection.rs, etc.

2. wasm4pm evidence-gate tests (closing — release gate):
   - Must emit process evidence as XES
   - Must invoke wpm oracle: `wpm audit <file.xes>`
   - Must invoke receipt doctor: `wpm receipt doctor --format json --strict` on emitted receipts
   - Must assert wasm4pm Accept/Refuse verdict from both oracle and receipt doctor
   - Files: tests/wasm4pm_evidence_gate.rs, tests/wasm4pm_evidence_mutation.rs, tests/wasm4pm_refusal_cases.rs

No release may claim ALIVE solely from cargo-cicd internal tests.

### Policies (`src/policies/`)
Autonomic policies run in `suggest` mode by default (configured in `cicd.toml [autonomic]`). They read `PolicyState` and emit recommendations, never take destructive action.

### Tests
Integration tests in `tests/` use `assert_cmd` + `tempfile` + fixture workspaces under `tests/fixtures/`. The `invariants` test enforces the 7 non-negotiable public boundary invariants. `feature_projection` verifies the feature flag surface contract.

---

## Debugging Guide

### Tracing EngineState
`EngineState` is the single source of truth for all runtime state. When debugging a noun's output or a policy verdict, the path is always:
1. **Identify which dimension is wrong**: check the relevant `*State` field (e.g., `WorkspaceState`, `TargetState`, `GitPhaseState`).
2. **Trace the adapter**: find the adapter that populates that dimension. Adapters are in `src/adapters/`; each implements a single query against one external source.
3. **Inspect the external source**: if the adapter's output is wrong, the fault is either in the adapter's translation logic or the external tool (git, cargo, filesystem).

**Example trace — "git status shows dirty but cargo-cicd shows clean":**
1. Bug is in `GitPhaseState` (contains dirty flag).
2. `GitPhaseState` is populated by `GitStatusAdapter` in `src/adapters/git_status.rs`.
3. Open `git_status.rs`, check how it parses `git status --porcelain`.
4. Run `git status --porcelain` manually in the workspace to compare.

**Key EngineState fields:**
```rust
// src/engine/mod.rs
pub struct EngineState {
    pub workspace: WorkspaceState,      // Cargo.toml discovery, manifest validity
    pub toolchain: ToolchainState,      // rustup, rust-toolchain.toml, MSRV
    pub target: TargetState,            // target/ size, cache pressure
    pub changed_files: ChangedFileState, // git diff, git status
    pub test_plan: TestPlanState,       // which tests to run
    pub trybuild: TrybuildState,        // trybuild ui/ fixture state
    pub git_phase: GitPhaseState,       // branch, dirty, untracked, commit history
    pub process_events: ProcessEventState, // XES evidence emission
    pub artifacts: ArtifactState,       // binary, archive state
    pub policies: PolicyState,          // policy evaluation results
    pub projection: ProjectionProfile,  // feature flag compliance
}
```

### Testing with Fixtures
Fixture workspaces in `tests/fixtures/mod.rs` are the primary testing tool. Each fixture is a `FixtureWorkspace` struct that owns a `TempDir` and exposes a `root` path.

**Available fixtures:**
- `FixtureWorkspace::clean()` — minimal valid workspace, fully committed, no target/, no cicd.toml. Verdict: **pass**.
- `FixtureWorkspace::dirty()` — clean workspace + one untracked file. Verdict: **warn** (git dirty).
- `FixtureWorkspace::missing_manifest()` — empty dir, no Cargo.toml. Verdict: **refuse**.
- `FixtureWorkspace::with_toolchain_mismatch()` — clean + rust-toolchain.toml with ancient channel. Verdict: **warn**.
- `FixtureWorkspace::with_target_over_limit()` — clean + target/debug/ with 1 MB placeholder. Verdict: **warn** (target pressure).
- `FixtureWorkspace::with_corrupted_cicd_toml()` — clean + corrupted cicd.toml. Verdict: **fail/refuse**.
- `FixtureWorkspace::with_stale_cicd_toml()` — clean + stale cicd.toml, then made dirty. Verdict: **warn** (cache mismatch).
- `FixtureWorkspace::with_changed_trybuild_fixture()` — clean + tests/ui/ with 10 unchanged + 1 changed fixture. Verdict: **pass** (only changed is run).

**Using fixtures in a test:**
```rust
#[test]
fn test_dirty_workspace_verdict() {
    let fixture = FixtureWorkspace::dirty();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .arg("status")
        .current_dir(&fixture.root)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("dirty"));
}
```

The fixture's `TempDir` is automatically cleaned up when the fixture is dropped.

### Enabling Logging
cargo-cicd does not yet use a standard logging framework (log, tracing, env_logger). For now, instrumentation is via:
1. **`ProcessEvent` emission**: see `src/evidence.rs` and `src/session.rs`. Events are appended to XES files in `target/cargo-cicd/evidence/`.
2. **Direct `println!`/`eprintln!`**: used in verb implementations for user-facing output.
3. **Manual test assertions**: verify output against expected patterns.

**To add structured logging in the future:**
- Add `tracing` and `tracing-subscriber` to `[dependencies]` in `Cargo.toml`.
- Initialize in `main()` with `tracing_subscriber::fmt::init()`.
- Add `tracing::debug!()`, `tracing::info!()` in hot paths (adapters, policy evaluation).
- Callers enable with `RUST_LOG=debug cargo cicd status`.

### Debugging Test Failures
1. **Run the failing test in isolation:** `cargo test --test <test_name> -- --nocapture` to see println output.
2. **Inspect the fixture:** if using `FixtureWorkspace`, print `fixture.root` and manually inspect the directory.
3. **Check git state:** in a fixture workspace, run `git status --porcelain` to verify the adapter's view matches reality.
4. **Check cicd.toml:** if present in the fixture, parse it and print its state.
5. **Trace adapter calls:** add temporary `eprintln!` in the adapter to see what it's reading.

**Example — debugging why trybuild fixture selection is wrong:**
```rust
#[test]
fn debug_trybuild_fixture_selection() {
    let fixture = FixtureWorkspace::with_changed_trybuild_fixture();
    eprintln!("Fixture root: {}", fixture.root.display());
    let ui_dir = fixture.root.join("tests/ui");
    for entry in std::fs::read_dir(&ui_dir).unwrap() {
        let path = entry.unwrap().path();
        eprintln!("Fixture file: {}", path.display());
    }
    // Now run the adapter and inspect its output
    let detector = TrybuildDetector::scan(&fixture.root);
    eprintln!("Detected changed fixtures: {:?}", detector.changed);
}
```

---

## Architecture Diagrams

### EngineState Aggregate
```
         ┌─────────────────────────────────────────┐
         │        EngineState (Aggregate Root)      │
         └─────────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼─────┐    ┌────▼─────┐    ┌────▼──────┐
    │Workspace │    │Toolchain │    │  Target   │
    │ State    │    │  State   │    │  State    │
    └──────────┘    └──────────┘    └───────────┘
         │                 │                 │
    (cwd, manifest)  (rustup, MSRV)  (target/, .gb)
         │
    ┌────┴─────┬──────────┬──────────┬──────────┐
    │           │          │          │          │
 ┌──▼──┐  ┌───▼──┐  ┌───▼──┐  ┌───▼──┐  ┌───▼───┐
 │Chg'd│  │Test  │  │Tryb'd│  │GitPhase  │Artifact│
 │Files│  │ Plan │  │ State│  │  State   │ State  │
 └─────┘  └──────┘  └──────┘  └─────────┘  └────────┘
 (git)   (tests/)  (tests/ui/) (git status) (bins)
         │                              │
    ┌────┴──────┬──────────┐          ┌┴────────┐
    │            │          │          │         │
 ┌──▼─┐  ┌─────▼──┐  ┌───▼──┐  ┌───▼──┐  ┌───▼──┐
 │Proc│  │ Policy │  │Project│  │Events│  │...   │
 │Evt │  │ State  │  │Profile│  │State │  │      │
 └────┘  └────────┘  └───────┘  └──────┘  └──────┘
```

### Adapter Pipeline
```
External World              ┌────────────────────────┐              Internal State
                            │    EngineState         │
┌─────────────────────┐     │  (aggregate root)      │
│  Git Repository     ├────►│  git_phase             │
└─────────────────────┘     │  changed_files         │◄────┐
                            │                        │     │
┌─────────────────────┐     │  workspace             │     │
│  Cargo.toml         ├────►│  toolchain             │     │
│  Cargo.lock         │     │  test_plan             │     │
│  rust-toolchain.toml├────►│  artifacts             │     │
└─────────────────────┘     └────────────────────────┘     │
                                                            │
┌─────────────────────┐     ┌────────────────────────┐     │
│  target/ dir        ├────►│  TargetScannerAdapter  ├─────┘
└─────────────────────┘     └────────────────────────┘
                                                      
┌─────────────────────┐     ┌────────────────────────┐     ┌────────────────┐
│  tests/ui/*.rs      ├────►│  TrybuildDetector      ├────►│  Trybuild      │
│  tests/*.rs         │     └────────────────────────┘     │  State         │
└─────────────────────┘                                    └────────────────┘

Each adapter:
- Reads ONE external source (git, cargo, filesystem, rustup)
- Translates to internal State representation
- No business logic — pure translation
- No side effects — read-only
```

### Noun-Verb CLI Flow
```
User Input: cargo cicd status [verb] [opts]
        │
        ▼
┌──────────────────────────────┐
│  main() + inject_default_    │
│  verbs()                     │
│  (cargo status → status show)│
└──────────┬───────────────────┘
           │
           ▼
    ┌──────────────────┐
    │  CliBuilder      │
    │  .noun(...)      │
    │  .run()          │
    └────────┬─────────┘
             │
             ▼
┌────────────────────────┐
│  StatusNoun::new()     │
│  .verbs() →            │
│  [StatusShowVerb,      │
│   StatusAuditVerb]     │
└───────────┬────────────┘
            │
            ▼
    ┌───────────────────┐
    │ StatusShowVerb    │
    │ .run()            │
    │ .execute()        │
    └──────────┬────────┘
               │
               ▼
       ┌──────────────────┐
       │ Read EngineState │
       │ via Adapters:    │
       │ - Toolchain      │
       │ - TargetScanner  │
       │ - GitStatus      │
       └──────────┬───────┘
                  │
                  ▼
          ┌──────────────────┐
          │ Render output    │
          │ (println)        │
          └──────────┬───────┘
                     │
                     ▼
          ┌──────────────────┐
          │ Emit ProcessEvent│
          │ to evidence/     │
          └──────────────────┘
```

### Policy Evaluation Loop
```
┌────────────────────────────────────┐
│  EngineState (fully populated)      │
│  - workspace, toolchain, target,    │
│  - changed_files, git_phase, etc.   │
└──────────────┬─────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
    ▼                     ▼
┌──────────────────┐ ┌────────────────────┐
│Autonomic Policy  │ │ Policy Decision    │
│Engine reads      │ │ (suggest/apply)    │
│PolicyState +     │ │ configured in      │
│other dimensions  │ │ cicd.toml [autonomic]
└────────┬─────────┘ └────────────────────┘
         │
    ┌────┴────────────────────────┐
    │                             │
    ▼                             ▼
┌──────────────────────┐  ┌──────────────────────┐
│ GitPhaseDirtyPolicy  │  │ TargetPressurePolicy │
│ (checks git state)   │  │ (checks target size) │
└────────┬─────────────┘  └──────────┬───────────┘
         │                            │
         ▼                            ▼
    ┌──────────────┐            ┌──────────────┐
    │ PolicyResult │            │PolicyResult  │
    │ .verdict()   │            │ .verdict()   │
    │ .recommend() │            │ .recommend() │
    └──────────────┘            └──────────────┘
         │                            │
         └──────────────┬─────────────┘
                        │
                        ▼
              ┌──────────────────────┐
              │ Collect results in   │
              │ PolicyState          │
              │ (shown to user)      │
              └──────────────────────┘

Policies run in **suggest** mode by default — never destructive.
User must pass --apply to activate recommendations.
```

---

## Common Workflows

### Adding a New Noun
A noun is a command namespace (e.g., `cargo cicd status`, `cargo cicd test`). Each noun owns a set of verbs (subcommands).

**Steps:**

1. **Create noun module** in `src/nouns/<noun_name>.rs`:
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
       fn about(&self) -> &'static str { "Do something useful" }
       fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
           vec![Box::new(MyVerbOne), Box::new(MyVerbTwo)]
       }
   }
   
   pub struct MyVerbOne;
   impl VerbCommand for MyVerbOne {
       fn name(&self) -> &'static str { "action" }
       fn about(&self) -> &'static str { "The action to take" }
       fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
           // Read EngineState via adapters, do work, emit events
           Ok(())
       }
   }
   ```

2. **Register noun in `src/nouns/mod.rs`:**
   ```rust
   pub mod mynoun;
   ```

3. **Register in `src/main.rs`:**
   ```rust
   let cli = cli.noun(nouns::mynoun::MyNoun::new());
   ```

4. **Add to default-verb injection** (if the noun should work bare, e.g., `cargo cicd mynoun` → `mynoun action`):
   ```rust
   // In inject_default_verbs()
   "mynoun" => Some("action"),
   
   // In main(), add to needs_default check:
   "mynoun" => return nouns::mynoun::MyNoun::run_direct(),
   ```

5. **Test:**
   ```sh
   cargo build
   cargo cicd mynoun --help       # Should show verbs
   cargo cicd mynoun action       # Should run the default verb
   ```

### Adding a New Adapter
An adapter reads one external source (git, cargo, filesystem, rustup) and populates one or more `EngineState` dimensions. Adapters have **no business logic** — they translate, nothing more.

**Steps:**

1. **Create adapter** in `src/adapters/<source>_adapter.rs`:
   ```rust
   use anyhow::Result;
   use crate::engine::*;
   
   pub struct MySourceAdapter;
   impl MySourceAdapter {
       /// Query the external source and return a populated State.
       pub fn query() -> Result<MySourceState> {
           // Read from external source (subprocess, filesystem, etc.)
           let raw_data = self::external_call()?;
           
           // Translate to internal State — no filtering, no interpretation
           Ok(MySourceState {
               field1: raw_data.field1,
               field2: raw_data.field2,
               // ...
           })
       }
   }
   
   fn external_call() -> Result<RawData> {
       // Run git command, read file, call rustup, etc.
       // Errors are propagated — adapters don't swallow errors.
   }
   ```

2. **Define or reuse State** in `src/engine/<dimension>_state.rs`:
   ```rust
   #[derive(Debug, Clone, Default)]
   pub struct MySourceState {
       pub field1: String,
       pub field2: u32,
   }
   ```

3. **Register in `src/adapters/mod.rs`:**
   ```rust
   pub mod my_source_adapter;
   pub use my_source_adapter::MySourceAdapter;
   ```

4. **Call from noun** in `src/nouns/my_noun.rs`:
   ```rust
   let state = MySourceAdapter::query()?;
   // Use state to decide output or policy verdict
   ```

5. **Test with fixture:**
   ```rust
   #[test]
   fn test_adapter_on_clean_workspace() {
       let fixture = FixtureWorkspace::clean();
       let state = MySourceAdapter::query_in(&fixture.root)?;
       assert_eq!(state.field1, "expected_value");
   }
   ```

### Debugging Test Failures

**Scenario 1: An invariant test fails**

Example: `invariant_public_boundary_no_forbidden_terms_in_all_help` fails because "ALIVE" appears in `cargo cicd target --help`.

1. Run the test with output:
   ```sh
   cargo test --test invariants invariant_public_boundary \
     -- --nocapture --exact
   ```

2. Capture the failing help text:
   ```sh
   cargo cicd target --help > /tmp/target_help.txt
   grep -i "ALIVE" /tmp/target_help.txt
   ```

3. Trace the help text back to the source:
   - Search `src/nouns/target.rs` for the help string.
   - Fix the help text to remove the forbidden term.

4. Re-run:
   ```sh
   cargo test --test invariants invariant_public_boundary
   ```

**Scenario 2: A fixture-based test fails**

Example: `test_dirty_workspace_shows_warn_verdict` fails — the test expects "warn" but sees "pass".

1. Run with output:
   ```sh
   cargo test test_dirty_workspace_shows_warn_verdict -- --nocapture
   ```

2. Add debug output:
   ```rust
   let fixture = FixtureWorkspace::dirty();
   eprintln!("Fixture root: {}", fixture.root.display());
   eprintln!("Untracked files:");
   for entry in std::fs::read_dir(&fixture.root).unwrap() {
       let path = entry.unwrap().path();
       eprintln!("  {}", path.display());
   }
   ```

3. Manually inspect the fixture:
   ```sh
   cd /tmp/test_<random>  # From the eprintln output
   git status --porcelain
   ls -la
   ```

4. Check if the adapter is reading what you expect:
   ```rust
   let git = GitStatusAdapter::query().unwrap();
   eprintln!("Git state: {:?}", git);
   ```

5. Fix the adapter or the fixture setup, then re-run.

**Scenario 3: A policy verdict is wrong**

Example: `TargetPressurePolicy` should warn when target/ is 25 GB, but it doesn't.

1. Trace to the policy:
   ```bash
   grep -r "TargetPressurePolicy" src/
   ```

2. Read `src/policies/target_pressure.rs`:
   ```rust
   pub fn evaluate(&self) -> PolicyResult {
       let target_size = /* read from TargetState */;
       let verdict = if target_size > 20.0 {
           PolicyVerdict::Warn
       } else {
           PolicyVerdict::Pass
       };
   }
   ```

3. Check if the threshold is right. If not, update it. If the threshold is right, check if `TargetState` is being populated correctly:
   ```rust
   let state = TargetScannerAdapter::query()?;
   eprintln!("Target size: {}", state.size_gb);
   ```

4. Fix the adapter or the policy threshold.

---

## Performance Tips

### Caching Patterns
cargo-cicd is designed to run fast even on large workspaces. Key caching strategies:

1. **cicd.toml state cache:**
   - `cicd.toml` in the workspace root caches the last-known state (dirty flag, target size, test results).
   - On subsequent runs, adapters can skip expensive operations (e.g., `git status` is always cheap, but parsing Cargo.lock for dependency tree is not).
   - Pattern: adapter reads cicd.toml first, then re-queries external source. If unchanged, use cached value.
   - Files: `src/adapters/cicd_toml_writer.rs`, `src/cicd_toml.rs`.

2. **Workspace scanning optimization:**
   - `TargetScannerAdapter` walks target/ once and caches the total size (not per-file).
   - Use `walkdir` (already imported) with `.max_depth()` to avoid deep traversals.
   - Pattern in `src/adapters/target_scanner.rs`:
     ```rust
     pub fn total_size_gb(root: &str) -> f64 {
         walkdir::WalkDir::new(root)
             .max_depth(3)  // Limit traversal depth
             .into_iter()
             .filter_map(|entry| entry.ok())
             .map(|entry| entry.metadata().ok().map(|m| m.len()).unwrap_or(0))
             .sum::<u64>() as f64 / 1_000_000_000.0
     }
     ```

3. **Git queries batched:**
   - Always use `git status --porcelain` (single invocation), not multiple `git ls-files` calls.
   - Pattern in `src/adapters/git_status.rs`:
     ```rust
     let output = Command::new("git")
         .args(&["status", "--porcelain"])
         .output()?;
     // Parse once, cache results
     ```

4. **Test plan deduplication:**
   - `TestPlanState` tracks which tests to run; avoid re-running already-passed tests.
   - Cached in cicd.toml `[[events]]` section.
   - Pattern: `changed_tests` integration test checks that only changed tests are selected.

### Workspace Scanning Optimization
For large monorepos (100+ crates):

1. **Use `cargo metadata --format-version 1`** to get manifest metadata once, not per-crate:
   ```rust
   // Good: single metadata call
   let metadata = cargo_metadata::MetadataCommand::new().exec()?;
   for package in metadata.packages {
       // Process packages
   }
   
   // Bad: spawning cargo for each crate
   for manifest_path in manifests {
       Command::new("cargo")
           .args(&["metadata", "--manifest-path", manifest_path])
           .output()?;
   }
   ```

2. **Limit test discovery to changed files:**
   - `ChangedFileDetector` identifies which crate each changed file belongs to.
   - Only run tests in changed crates (via `TestPlanState`).
   - Files: `src/adapters/changed_file_detector.rs`, `src/engine/test_plan_state.rs`.

3. **Cache target/ size across invocations:**
   - Persists in cicd.toml `[state]` section.
   - Only re-scan if git has changed since last run (quick check via HEAD commit hash).

---

## Known Limitations

### Feature Flag Gating

#### `process-data` (disabled by default)
When **not** enabled, the following are **not available:**
- Level 5 engine internals (`EngineState`, all adapters)
- cicd.toml reading/writing
- Policy evaluation
- Process event emission to XES
- Autonomic mode

When **enabled**, all of the above become available. This is intentional: the public surface (public nouns/verbs) works without the engine; internal plumbing is opt-in via the feature flag.

**Check if enabled in code:**
```rust
#[cfg(feature = "process-data")]
fn use_engine_state() {
    let state = EngineState::default();
    // ...
}
```

#### `autonomic` (disabled by default; implies `process-data`)
When **not** enabled:
- Policies are defined but not evaluated (`src/policies/`)
- Policy-based suggestions are not shown to the user
- `cicd.toml [autonomic]` is parsed but not applied

When **enabled:**
- All policies run in `suggest` mode by default
- User sees recommendations (never destructive)
- User can pass `--apply` to activate (future work)

**Reason:** Policies are advanced, and the default behavior is safe and non-invasive.

#### `wasm4pm` (disabled by default; implies `process-data`)
When **not** enabled:
- wasm4pm integration seams exist but are stubbed
- Process evidence is still emitted to XES (per evidence gate)
- Evidence-gate tests may skip wasm4pm oracle validation

When **enabled:**
- `src/integrations/wasm4pm_*.rs` activates
- Tests invoke `wpm receipt doctor` and `wpm audit`
- Release closure requires wasm4pm verdict (Accept)

**Important:** wasm4pm is **not optional** for v26.6.2 releases. The feature flag gates richer integration (e.g., direct evidence submission), not the evidence-gate law itself.

**Release-blocking check:**
```sh
# Feature disabled (default)
cargo test --test wasm4pm_evidence_gate
# Evidence is emitted, but oracle validation is skipped

# Feature enabled
cargo test --test wasm4pm_evidence_gate --features wasm4pm
# Oracle validation is enforced; test fails if wpm verdict is Refuse
```

#### `contrib` (disabled by default; implies `process-data`)
Reserved for contributor-only utilities and debugging aids. Not part of the public surface.

### Intentionally Not Implemented

1. **Destructive operations by default:**
   - `cargo cicd target prune` requires explicit `--confirm` flag.
   - `cargo cicd git close` emits warnings and requires review.
   - **Why:** No silent data loss; always make the user think.

2. **Policy apply mode:**
   - Policies are **suggest only** in the current version.
   - `--apply` flag is recognized but not yet functional.
   - **Why:** Policies need more testing and user feedback before automation.

3. **Parallel test execution:**
   - Tests are run serially (one at a time).
   - `--jobs` flag is recognized but ignored.
   - **Why:** Workspace state is global; parallel runs risk race conditions (e.g., two tests modifying cicd.toml).

4. **Custom policy definitions:**
   - Users cannot define or register their own policies.
   - Only built-in policies are available.
   - **Why:** Policy semantics are baked into the engine; no plugin system yet.

5. **Remote state synchronization:**
   - cicd.toml is local-only; no push/pull from remote.
   - `git push` does not sync cicd.toml state.
   - **Why:** Remote state is out of scope for v26.6.2; defer to post-release.

6. **Workspace federation (monorepo multiple workspaces):**
   - Only one workspace root is recognized (CWD or explicit `--root`).
   - Cross-workspace test dependencies are not modeled.
   - **Why:** Federation semantics are complex; monoworkspace is the target for v26.6.2.

### Other Limitations

- **Git requirement:** All state-tracking features require git. Non-git workspaces are unsupported (tests will skip).
- **No MSRV guarantee below 1.86:** Some dependencies require Rust 1.86 or later. Older toolchains will fail to compile.
- **No Windows cross-compile support:** Tested on Linux/macOS only. Windows users may encounter path issues.
- **No Bazel/Buck/other build systems:** Cargo is the only supported build system. Other systems must use cargo-install shim.
