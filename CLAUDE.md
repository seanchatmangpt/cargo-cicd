# CLAUDE.md

This file provides comprehensive guidance to Claude Code (claude.ai/code) when working with code in this repository. It is the single source of truth for cargo-cicd architecture, design patterns, and constraints.

---

## Mission & Identity

**Public Identity:**  
"cargo-cicd keeps Rust workspaces clean, fast, and push-ready."

**Private Identity:**  
cargo-cicd is a Level 5 process-data engine exposed as a CI/CD helper. It manufactures noun-verb command grammar from an ontology and emits process evidence for external adjudication by wasm4pm.

**Key Facts:**
- Manufacturing: `ggen` (ontology) → `clap-noun-verb` (grammar) → `cargo-cicd` (CLI binary)
- State Carrier: `cicd.toml` at workspace root
- Evidence Format: XES (XML Event Stream) + JSONL companions
- Oracle: wasm4pm (`wpm` binary) adjudicates all verdicts
- Gate: Dung Gate (artifact/output manufacture)
- Version: 26.6.2

---

## FORBIDDEN Terms

**Complete list** — MUST NOT appear in any public output (help text, CLI, docs):

| Term | Reason |
|------|--------|
| `ALIVE` | Level 5 engine status marker; internal only |
| `Inspection Gate` | Manufacturing subsystem identity |
| `wall` | Jargon from manufacturing pipeline |
| `Nehemiah` | Code name for manufacturing layer (expose only as ggen) |
| `Field8` | Internal capacity measurement (not user-facing) |
| `Instinct8` | Autonomic reasoning subsystem; not exposed in "suggest" mode |
| `Cargo Court` | Internal adjudication metaphor |
| `AGI` | AI system classification; not disclosed in CLI output |
| `Truex` | Internal truth engine; only XES/evidence models exposed |
| `CONSTRUCT8` | Manufacturing directive system |

**Enforcement:** The `tests/invariants.rs` test `invariant_public_boundary_no_forbidden_terms_in_all_help()` scans all `--help` output. This gate must pass before release.

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
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

# Run a specific test function
cargo test --test invariants test_function_name

# Run with a feature flag
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm

# Run evidence gate (requires wpm oracle)
cargo make test  # includes wasm4pm_evidence_gate
```

---

## Commit Format

```
feat(core|cli|target|test|git|autonomic|docs|receipts): description
```

Choose the scope that best describes the change. Examples:
- `feat(core): add ProcessEvent serialization for XES`
- `fix(cli): ensure status noun injects default show verb`
- `docs(autonomic): clarify policy suggestion lifecycle`
- `test(wasm4pm): add mutation tests for verdict handling`

---

## Architecture Overview

### 1. Manufacturing Pipeline: ggen → clap-noun-verb → cargo-cicd

The CLI grammar is **manufactured, not handwritten**. Any change to noun/verb structure requires regeneration:

```
ontology/cargo-cicd.ttl (RDF/Turtle) 
    ↓ [ggen.toml: SPARQL inference + Tera templates]
    ↓
noun modules (src/nouns/*.rs)
CLI test scaffolding (tests/cli/*)
README.md reference sections
docs/reference/commands/*.md
```

**How it works:**
1. Edit `ontology/cargo-cicd-capabilities.ttl` to define nouns, verbs, and descriptions
2. Run `ggen` to regenerate all outputs
3. Implement noun + verb handlers in `src/nouns/`
4. Each noun implements `NounCommand` trait; verbs implement `VerbCommand` trait

**Key config:** `/home/user/cargo-cicd/ggen.toml` defines:
- SPARQL capability projection rules
- Tera template paths for code generation
- Output destinations (README, reference docs, test scaffolding)

**Default Verbs (noun-level shortcuts):**  
Bare nouns automatically inject default verbs to simplify the user experience:
- `cargo cicd status` → `status show`
- `cargo cicd publish` → `publish run`
- `cargo cicd workspace` → `workspace doctor`
- `cargo cicd evidence` → `evidence doctor`

This is implemented in `src/main.rs::inject_default_verbs()` for local reference and in the clap builder for actual dispatch.

---

### 2. Noun-Verb CLI Grammar (`src/nouns/`)

The CLI uses `clap-noun-verb` (published crate). Each noun is a module implementing `NounCommand`; verbs within implement `VerbCommand`.

**Available Nouns:**
1. **evidence** — Process evidence emission and adjudication (`doctor`, `audit`)
2. **pipeline** — Sequential execution of all declared CI/CD activities (`run`)
3. **status** — Workspace health snapshot (`show`)
4. **target** — Target directory analysis and cleanup (`show`, `prune`)
5. **test** — Selective test execution by changed files (`changed`)
6. **trybuild** — Compiler error snapshot tests (`changed`, `full`)
7. **git** — Git phase tracking and closure (`status`, `close`, `phase`)
8. **publish** — Artifact publishing gate (`run`)
9. **workspace** — Workspace-wide diagnostics (`doctor`)
10. **lsp** — Language server for IDE integration (`explain`)

**Verb Categories:**
- **Read-only:** `show`, `status`, `explain`, `doctor`
- **Dry-run:** `prune --dry-run` (planning, not destructive)
- **Execution:** `run`, `close` (may be destructive)
- **Special:** `audit` (adjudication only)

---

### 3. Level 5 Engine State (`src/engine/`)

`EngineState` is the aggregate root — a struct of all runtime dimensions:

```rust
pub struct EngineState {
    pub workspace: WorkspaceState,
    pub toolchain: ToolchainState,
    pub target: TargetState,
    pub changed_files: ChangedFileState,
    pub test_plan: TestPlanState,
    pub trybuild: TrybuildState,
    pub git_phase: GitPhaseState,
    pub process_events: ProcessEventState,
    pub artifacts: ArtifactState,
    pub policies: PolicyState,
    pub projection: ProjectionProfile,
}
```

**Key Invariant:** Nouns **read** from `EngineState`; adapters **populate** it from external sources. All business logic flows through this state.

**Initialization:** `EngineState::from_workspace()` queries all adapters in sequence, silently handling failures (partial data is better than crashes).

**State Modules (`src/engine/*.rs`):**
- `workspace_state` — Workspace name, root path, members, toolchain, Rust edition
- `toolchain_state` — Active toolchain, Rust version
- `target_state` — Target directory path and total size
- `changed_file_state` — Base ref, changed .rs files, test files, trybuild fixtures
- `test_plan_state` — Estimated test count, conservative mode flag
- `trybuild_state` — Fixture sets, changed fixtures, projection profile
- `git_phase_state` — Branch, dirty/staged/untracked files, ahead/behind counts
- `process_event_state` — List of emitted ProcessEvent structs
- `artifact_state` — Artifact manifests, registry metadata
- `policy_state` — PolicyEntry structs for each policy evaluation
- `projection_profile` — Feature flag surface contract

---

### 4. Adapters (`src/adapters/`)

Adapters are **stateless, pure translators** from external representations to `EngineState`. Each owns one external source:

| Adapter | Responsibility | Performance |
|---------|---|---|
| `CargoMetadataAdapter` | Cargo workspace name, target dir, members (line-by-line Cargo.toml scan) | Fast (no cargo invocation) |
| `ManifestParser` | Cargo.toml TOML parsing for package names, workspace.package metadata | Fast (toml crate) |
| `GitStatusAdapter` | `git status --porcelain`, branch tracking, ahead/behind counts | Medium (git invocation) |
| `ToolchainDetector` | `rustc --version`, active toolchain | Medium (rustc invocation) |
| `TargetScannerAdapter` | Recursive target dir size calculation | Slow (walkdir traversal) |
| `ChangedFileDetector` | `git diff origin/main --name-only` to classify .rs files | Medium (git invocation) |
| `TrybuildDetector` | Scan `tests/ui/` for fixture files, match against changed | Fast (local filesystem scan) |
| `CicdTomlWriter` | Serialize EngineState → cicd.toml on disk | Fast (toml::to_string) |

**Adapter Pattern:**
1. Adapters have **no state** — all methods are `&self` or `fn()`
2. Adapters **silently fail** — return defaults, never panic
3. Adapters **never call other adapters** — each is independent
4. EngineState **calls all adapters in sequence** — failures don't block later adapters

**Example Usage in EngineState:**
```rust
// In src/engine/mod.rs
state.workspace.name = CargoMetadataAdapter::workspace_name();
state.target.total_size_bytes = TargetScannerAdapter::total_size_bytes(&target_dir);
```

---

### 5. Evidence Emission Pattern

**All verbs follow the same evidence pattern:**  
`start` → [work] → `complete` → [optional adjudication]

**Key Invariants (E1–E7 in `src/evidence.rs`):**

| Invariant | Rule |
|-----------|------|
| **E1** | cargo-cicd never adjudicates itself; only wasm4pm issues verdicts |
| **E2** | XES file must exist on disk before `audit_xes()` is called |
| **E3** | If oracle unavailable and expected verdict isn't `Blocked`, panic (certification requires oracle) |
| **E4** | Tests assert only wasm4pm verdict, never internal cargo-cicd state |
| **E5** | XES groups events by `case_id` into `<trace>` elements |
| **E6** | JSONL emission mirrors XES (same event set, machine-readable) |
| **E7** | `Blocked` is a first-class expectation, not an error (for offline tests) |

**ProcessEvent Structure:**
```rust
pub struct ProcessEvent {
    pub event_id: String,              // "evt-status-show-20260614134507123Z"
    pub timestamp_iso: String,         // "2026-06-14T13:45:07.123Z"
    pub case_id: Option<String>,       // Groups into <trace> in XES
    pub lifecycle_transition: String,  // "start" or "complete"
    pub workspace_id: String,
    pub repo_path: String,
    pub command: String,               // "status show"
    pub verdict_claimed: String,       // "PASS", "WARN", or "FAIL"
    pub duration_ms: Option<u64>,      // None for "start"
    pub verdict_adjudicated: Option<String>,  // Set after wpm oracle
    pub adjudicated_at: Option<String>,
    pub oracle_command: Option<String>,
    pub trace_class: String,           // "live_workspace" or "pipeline_run"
}
```

**Verdict Categories:**
- `PASS` — Normal completion, all checks satisfied
- `WARN` — Completion with warnings; work continues
- `FAIL` — Blocking error; work halts
- Special: `WARN:dry_run` (planning), `WARN:oracle_unavailable` (wpm not found)

**How Verbs Emit:**
1. Create a `ProcessEvent::new("command name", "PASS")` at entry
2. Perform work (may call adapters, read EngineState)
3. Set `verdict_claimed` based on work outcome
4. Serialize to XES (via `src/evidence.rs`) to `target/cargo-cicd/evidence/`
5. If `#[cfg(feature = "wasm4pm")]`, call `Wasm4pmShell::audit_xes()` for external verdict
6. Tests assert the wasm4pm verdict, never internal state

**Example in `src/nouns/status.rs`:**
```rust
let event = ProcessEvent::new("status show", "PASS");
let dirty = state.git_phase.dirty_files.len() > 0;
let verdict = if dirty { "WARN" } else { "PASS" };
// ... emit to XES ...
// In test: assert wpm_verdict == Expected::Accept
```

---

### 6. cicd.toml — The State Carrier

`cicd.toml` (at workspace root) is the persistent carrier for configuration and emitted events.

**Structure:**
```toml
[workspace]
name = "cargo-cicd"
root_path = "/home/user/cargo-cicd"
members = [".","crates/cargo-cicd-core", "crates/cargo-cicd-lsp"]

[state]
git_phase = "clean"
target_size_bytes = 524288000

[target]
total_size_bytes = 524288000
pruned_bytes = 0

[[events]]
event_id = "evt-status-show-20260614134507123Z"
timestamp = "2026-06-14T13:45:07.123Z"
command = "status show"
verdict_claimed = "PASS"
verdict_adjudicated = "Accept"
```

**Who Writes:** `CicdTomlWriter` (adapter) serializes EngineState → TOML on disk  
**Who Reads:** Nouns and policies read `cicd.toml` via the state model  
**When:** After each major operation (status, test, publish)

---

### 7. Feature Flags Deep Dive

```toml
# In Cargo.toml
[features]
default = []
process-data = []
autonomic = ["process-data"]
contrib = ["process-data"]
wasm4pm = ["process-data"]
```

| Flag | Purpose | Enables | Gate | Runtime Effect |
|------|---------|---------|------|---|
| **process-data** | Enable Level 5 engine internals | EngineState, adapters, cicd.toml | Optional (defaults off) | Populates all state dimensions |
| **autonomic** | Policy suggestions in suggest mode | `policies::run_all_policies()` | Optional (implies process-data) | Reads PolicyState; emits recommendations, never takes action |
| **contrib** | Community contribution tooling | Extra diagnostics for maintainers | Optional (implies process-data) | Enables verbose logging, debug output |
| **wasm4pm** | wasm4pm oracle integration | Wasm4pmShell, verdict adjudication | Optional (implies process-data) | Calls `wpm audit <xes>` and `wpm receipt doctor` |

**Feature Coupling:** All non-default flags imply `process-data`. The Level 5 engine is **opt-in**, not forced on users.

**Test With Features:**
```sh
cargo test --features process-data
cargo test --features autonomic
cargo test --features wasm4pm
```

---

### 8. Autonomic Policies (`src/autonomic/`)

Autonomic layer runs only when `autonomic` feature is enabled. **All policies run in `suggest` mode** (read-only):

| Policy | Trigger | Recommendation |
|--------|---------|---|
| `target_pressure` | Target dir > threshold | "Run `cargo cicd target prune`" |
| `toolchain_mismatch` | rustc version != lockfile | "Update toolchain or lock" |
| `trybuild_changed` | trybuild fixtures changed | "Run `cargo cicd trybuild changed`" |
| `branch_behind` | Local behind main by N commits | "Pull or rebase" |
| `evidence_stale` | Last evidence > age threshold | "Re-run evidence gate" |
| `publish_not_adjudicated` | Publish happened but no wpm verdict | "Require wpm oracle for releases" |
| `git_phase_dirty` | Dirty/staged files present | "Commit or stash changes" |

**PolicyState:**
```rust
pub struct PolicyState {
    pub entries: Vec<PolicyEntry>,
}

pub struct PolicyEntry {
    pub policy_name: String,
    pub verdict: PolicyVerdict,
    pub recommendation: String,
    pub emitted_at: String,
}
```

**Verdict Categories:**
- `Pass` — All conditions met
- `Warn` — Condition detected; recommendation issued
- `Skip` — Policy inapplicable (e.g., single-crate workspace)

**How to Add a Policy:**
1. Create `src/policies/your_policy.rs` with `fn eval() -> PolicyEntry`
2. Add to `policies::run_all_policies()` dispatch
3. Emit via `workspace doctor` noun
4. Test in `tests/autonomic_policies.rs`

---

### 9. Test Hierarchy

Tests are **stratified by gate type** and **expected oracle involvement**:

#### Tier 1: Unit & Smoke Tests (Non-Closing)
**Purpose:** Validate internal logic and public boundaries  
**Tools:** `assert_cmd`, `tempfile`, local fixtures  
**Files:**
- `tests/invariants.rs` — 7 non-negotiable public boundary invariants
- `tests/cli/` — Noun/verb CLI parsing
- `tests/feature_projection.rs` — Feature flag surface contract
- `tests/cicd_toml_truth.rs` — Serialization/deserialization
- `tests/autonomic_policies.rs` — Policy evaluation logic
- `tests/changed_tests.rs` — File classification accuracy
- `tests/git_phase_closure.rs` — Git state detection

**Invariants Enforced:**
1. No forbidden terms in help output
2. No destructive action without `--confirm`
3. No full trybuild run by default (conservative mode)
4. Noun names are lowercase ASCII
5. Binary name is `cargo-cicd`
6. Status command exits 0 (baseline health)
7. Git close has safety warnings (no false close)

#### Tier 2: Evidence Gate Tests (Closing — Release Gate)
**Purpose:** Verify process conformance via external oracle  
**Oracle:** wasm4pm (`wpm` binary)  
**Tools:** XES emission, receipt doctor  
**Files:**
- `tests/wasm4pm_evidence_gate.rs` — Happy path evidence → adjudication
- `tests/wasm4pm_evidence_mutation.rs` — Corrupt evidence, verify rejection
- `tests/wasm4pm_refusal_cases.rs` — Edge cases (oracle unavailable, etc.)

**Assertion Pattern:**
```rust
// Do NOT assert on cargo-cicd internal state
assert_eq!(state.target.size, expected_size);  // ❌ WRONG

// DO assert on wasm4pm verdict
assert_eq!(wpm_verdict, WpmVerdict::Accept);   // ✅ CORRECT
```

**No Release Without Evidence Gate:** Tests must call the wpm oracle. If wpm is unavailable, tests must declare `ExpectedWpmVerdict::Blocked` and skip oracle verification.

---

### 10. wasm4pm Integration — Evidence Adjudication

wasm4pm is **not optional** for release v26.6.2. The oracle adjudicates all process evidence.

**Evidence Flow:**
```
cargo-cicd executes
    ↓
ProcessEvent emitted (verdict_claimed = "PASS"|"WARN"|"FAIL")
    ↓
Serialize to XES (target/cargo-cicd/evidence/evt-*.xes)
    ↓
Call wpm oracle: wpm audit <evt-*.xes>
    ↓
Oracle returns Accept/Refuse/Blocked
    ↓
Tests assert wpm verdict (not cargo-cicd state)
```

**wpm Commands:**

| Command | Purpose | Input | Output |
|---------|---------|-------|--------|
| `wpm audit <file.xes>` | Adjudicate process evidence | XES file | `Accept`/`Refuse`/`Blocked` |
| `wpm receipt doctor --format json --strict <receipt.json>` | Validate receipt integrity | Receipt JSON | `Accept`/`Refuse` |

**XES Format (XML Event Stream):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <string key="case_id" value="status_show_phase"/>
    <event>
      <string key="event_id" value="evt-status-show-20260614134507123Z"/>
      <string key="timestamp" value="2026-06-14T13:45:07.123Z"/>
      <string key="lifecycle_transition" value="complete"/>
      <string key="verdict_claimed" value="PASS"/>
      <string key="trace_class" value="live_workspace"/>
    </event>
  </trace>
</log>
```

**JSONL Format (companion to XES):**
```jsonl
{"event_id":"evt-status-show-20260614134507123Z","timestamp":"2026-06-14T13:45:07.123Z","command":"status show","verdict_claimed":"PASS"}
```

**Expected Verdict Modes:**
- `ExpectedWpmVerdict::Accept` — Normal operation
- `ExpectedWpmVerdict::Refuse` — Expected failure (test data, corrupt evidence)
- `ExpectedWpmVerdict::Blocked` — Oracle unavailable (for CI without wpm installed)

---

## Common Workflows

### Workflow 1: Add a New Feature Command

**Scenario:** Add a new verb to an existing noun (e.g., `target repair`)

1. **Edit ontology:**
   ```turtle
   # In ontology/cargo-cicd-capabilities.ttl
   cc:target-repair a skos:Concept ;
       cc:noun "target" ;
       cc:verb "repair" ;
       cc:cliCommand "cargo cicd target repair" ;
       dcterms:description "Repair target directory issues (e.g., stale locks)" .
   ```

2. **Regenerate from ontology:**
   ```sh
   ggen
   ```
   This updates `README.md`, test scaffolding, and reference docs.

3. **Implement verb handler:**
   ```rust
   // In src/nouns/target.rs
   pub struct RepairVerb;
   
   impl VerbCommand for RepairVerb {
       fn run() -> Result<()> {
           let state = EngineState::from_workspace();
           // ... implement repair logic ...
           Ok(())
       }
   }
   ```

4. **Register verb in noun:**
   ```rust
   // In TargetNoun::new()
   self.add_verb(RepairVerb)
   ```

5. **Write tests:**
   ```rust
   // In tests/cli/test_target.rs
   #[test]
   fn test_target_repair_dry_run() { ... }
   #[test]
   fn test_target_repair_confirm() { ... }
   ```

6. **Emit evidence:**
   In the verb handler, follow the ProcessEvent pattern:
   ```rust
   let mut event = ProcessEvent::new("target repair", "PASS");
   // ... work ...
   let xes_path = emit_xes_event(&event);
   // Optionally: audit via wasm4pm
   ```

---

### Workflow 2: Fix a Bug

**Example:** Status noun shows wrong git phase

1. **Locate the bug:**
   ```sh
   cargo test --test invariants test_invariant_status_exits_zero
   # If failing, debug with:
   cargo run -- status show
   ```

2. **Understand the state model:**
   - Read `src/engine/git_phase_state.rs` to see what's tracked
   - Check `GitStatusAdapter` to see how it's populated
   - Verify `src/nouns/status.rs` reads the correct field

3. **Fix the bug:**
   ```rust
   // Example: status.rs was reading the wrong field
   - let dirty_count = state.git_phase.ahead;  // WRONG
   + let dirty_count = state.git_phase.dirty_files.len();  // CORRECT
   ```

4. **Add regression test:**
   ```rust
   #[test]
   fn test_status_show_detects_dirty_files() {
       let mut dir = TempDir::new().unwrap();
       // Create dirty state
       std::fs::write(dir.path().join("test.txt"), b"dirty").unwrap();
       let output = Command::cargo_bin("cargo-cicd")
           .unwrap()
           .current_dir(dir.path())
           .args(["status", "show"])
           .output()
           .unwrap();
       let text = String::from_utf8_lossy(&output.stdout);
       assert!(text.contains("dirty") || text.contains("WARN"));
   }
   ```

5. **Run evidence gate** (if the bug affected verdict):
   ```sh
   cargo test --test wasm4pm_evidence_gate -- --nocapture
   ```

6. **Commit:**
   ```sh
   git add -A
   git commit -m "fix(cli): status show now reads dirty_files instead of ahead count"
   ```

---

### Workflow 3: Add a New Policy

**Scenario:** Add a policy to detect stale Cargo.lock files

1. **Create policy module:**
   ```rust
   // src/policies/cargo_lock_age.rs
   pub fn eval(state: &EngineState) -> PolicyEntry {
       let lock_path = format!("{}/Cargo.lock", state.workspace.root_path);
       let age = /* calculate age via filesystem metadata */;
       let verdict = if age > MAX_AGE { PolicyVerdict::Warn } else { PolicyVerdict::Pass };
       PolicyEntry {
           policy_name: "cargo_lock_age".to_string(),
           verdict,
           recommendation: "Run `cargo update` to refresh lockfile".to_string(),
           emitted_at: crate::evidence::now_iso8601(),
       }
   }
   ```

2. **Register in policy engine:**
   ```rust
   // In src/autonomic/policies.rs
   let mut entries = vec![];
   entries.push(cargo_lock_age::eval(state));
   // ... other policies ...
   ```

3. **Test in autonomic suite:**
   ```rust
   // tests/autonomic_policies.rs
   #[test]
   fn test_cargo_lock_age_policy_detects_stale_lock() {
       let state = EngineState { /* stale lock setup */ };
       let policies = run_all_policies(&state);
       let lock_policy = policies.iter().find(|p| p.policy_name == "cargo_lock_age").unwrap();
       assert_eq!(lock_policy.verdict, PolicyVerdict::Warn);
   }
   ```

4. **Ensure autonomic feature:**
   ```rust
   #[cfg(feature = "autonomic")]
   pub fn eval(state: &EngineState) -> PolicyEntry { ... }
   ```

---

### Workflow 4: Write an Integration Test

**Example:** Test the `publish run` noun

1. **Create fixture workspace:**
   ```rust
   // tests/fixtures/publish_ready/Cargo.toml
   [package]
   name = "publish_ready"
   version = "0.1.0"
   description = "A crate ready for publishing"
   license = "MIT"
   readme = "README.md"
   ```

2. **Write test:**
   ```rust
   // tests/cli/test_publish.rs
   use tempfile::TempDir;
   use assert_cmd::Command;

   #[test]
   fn test_publish_run_with_complete_metadata() {
       let dir = TempDir::new().unwrap();
       // Copy fixture into temp dir
       copy_fixture("publish_ready", dir.path()).unwrap();
       
       let output = Command::cargo_bin("cargo-cicd")
           .unwrap()
           .current_dir(dir.path())
           .args(["publish", "run"])
           .output()
           .unwrap();
       
       assert!(output.status.success());
       let stdout = String::from_utf8_lossy(&output.stdout);
       assert!(stdout.contains("ready") || stdout.contains("PASS"));
   }

   #[test]
   fn test_publish_run_missing_license_warns() {
       let dir = TempDir::new().unwrap();
       // Create Cargo.toml WITHOUT license
       let cargo_toml = r#"
       [package]
       name = "no_license"
       version = "0.1.0"
       description = "Missing license"
       "#;
       std::fs::write(dir.path().join("Cargo.toml"), cargo_toml).unwrap();
       
       let output = Command::cargo_bin("cargo-cicd")
           .unwrap()
           .current_dir(dir.path())
           .args(["publish", "run"])
           .output()
           .unwrap();
       
       let stdout = String::from_utf8_lossy(&output.stdout);
       assert!(stdout.contains("WARN") || stdout.contains("license"));
   }
   ```

3. **Run and validate:**
   ```sh
   cargo test --test cli test_publish_run_with_complete_metadata
   ```

---

### Workflow 5: Run Evidence Gate (Release Checklist)

**Before any release:**

1. **Ensure wpm binary is available:**
   ```sh
   wpm --version
   # If not found: install wasm4pm and add /path/to/wpm to PATH
   ```

2. **Run evidence gate tests:**
   ```sh
   cargo test --test wasm4pm_evidence_gate -- --nocapture
   cargo test --test wasm4pm_evidence_mutation
   cargo test --test wasm4pm_refusal_cases
   ```

3. **Check evidence was emitted:**
   ```sh
   ls -la target/cargo-cicd/evidence/
   # Should contain *.xes and *.jsonl files
   ```

4. **Manually verify an XES file:**
   ```sh
   wpm audit target/cargo-cicd/evidence/evt-*.xes
   # Output: Accept/Refuse/Blocked
   ```

5. **Check for receipts:**
   ```sh
   ls -la receipts/
   wpm receipt doctor --format json --strict receipts/*.json
   ```

6. **All gates must pass:**
   ```sh
   cargo make test
   # All test suites must exit 0
   ```

7. **Commit and tag:**
   ```sh
   git add CHANGELOG.md
   git commit -m "chore(release): v26.6.2 evidence gate pass"
   git tag -a v26.6.2 -m "Release v26.6.2 — evidence adjudicated by wasm4pm"
   git push origin main --tags
   ```

---

## Performance Notes

### Fast Operations
- **ManifestParser** — Direct TOML parsing of Cargo.toml (no cargo invocation)
- **CargoMetadataAdapter** — Line-by-line Cargo.toml scan (no external process)
- **TrybuildDetector** — Local filesystem glob scan
- **ProcessEvent construction** — JSON serialization to XES
- **Noun help text** — Compiled into binary, instant

### Medium Operations
- **GitStatusAdapter** — `git status --porcelain` invocation
- **ToolchainDetector** — `rustc --version` invocation
- **ChangedFileDetector** — `git diff origin/main --name-only` (depends on tree size)

### Slow Operations
- **TargetScannerAdapter** — Recursive `walkdir` over entire target directory
  - Mitigation: Cache total size in cicd.toml, only recalculate if stale
  - Can be 1-5+ seconds on large workspaces (100GB+ targets)
- **CargoMetadataAdapter (future)** — If switched to `cargo metadata` output parsing
  - Current: simple line-by-line scan (fast)
  - Future: lazy-load only when workspace members needed

**Optimization Strategies:**
1. **Lazy loading:** Only call adapters when nouns need specific state
2. **Caching:** Store adapter results in `cicd.toml`, invalidate on Cargo.toml change
3. **Partial data:** Adapters silently fail; partial state is better than blocking
4. **Feature gating:** Only populate Level 5 engine when `process-data` feature enabled

---

## Troubleshooting

### Issue: Test Fails with "Forbidden term 'ALIVE' found in output"

**Cause:** A forbidden term leaked into help text.

**Fix:**
1. Identify which noun/verb leaked the term:
   ```sh
   cargo run -- <noun> <verb> --help | grep ALIVE
   ```

2. Search the source for the term:
   ```sh
   rg "ALIVE" src/
   ```

3. Replace with public alternative:
   ```rust
   // WRONG:
   println!("ALIVE status: {}",  state.is_alive);
   
   // CORRECT:
   println!("Process state: {}",  state.is_complete);
   ```

4. Re-run invariant test:
   ```sh
   cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
   ```

---

### Issue: Status Command Exits Non-Zero

**Cause:** EngineState population failed; partial data couldn't satisfy a noun.

**Debug:**
1. Check individual adapters:
   ```sh
   cargo run -- status show 2>&1 | head -20
   ```

2. Check git state:
   ```sh
   git status --porcelain
   git diff origin/main --name-only
   ```

3. Check Cargo.toml:
   ```sh
   cat Cargo.toml | grep -A 5 '\[workspace\]'
   ```

4. If adapters are failing silently, enable debug logging:
   ```sh
   RUST_LOG=debug cargo run -- status show
   ```

5. Check for missing files:
   ```sh
   ls -la Cargo.toml Cargo.lock target/
   ```

---

### Issue: Evidence Gate Fails with "Oracle Unavailable"

**Cause:** wpm binary not found on PATH.

**Fix:**
1. Verify wpm is installed:
   ```sh
   which wpm
   wpm --version
   ```

2. If not installed, build wasm4pm:
   ```sh
   cd /path/to/wasm4pm
   cargo build --release
   export PATH="/path/to/wasm4pm/target/release:$PATH"
   ```

3. Verify it works:
   ```sh
   echo '<?xml version="1.0"?><log></log>' > /tmp/test.xes
   wpm audit /tmp/test.xes
   # Should output: Accept/Refuse/Blocked
   ```

4. If tests still fail with `Blocked` verdict:
   - This is expected if wpm is unavailable
   - Tests must declare `ExpectedWpmVerdict::Blocked`
   - Full release gate requires oracle to be available

---

### Issue: "No such file or directory: Cargo.toml"

**Cause:** Running cargo-cicd outside a workspace root.

**Fix:**
1. Verify you're in the workspace root:
   ```sh
   pwd
   ls -la Cargo.toml
   ```

2. If in a sub-crate, navigate to workspace root:
   ```sh
   cd ../..  # or wherever Cargo.toml is at workspace level
   ```

3. If this is a single-crate project, that's OK:
   ```sh
   # Single-crate root has Cargo.toml
   cargo cicd status show
   ```

---

### Issue: Trybuild Test Runs All Fixtures, Not Just Changed

**Cause:** Conservative mode not triggered; `INVARIANT_NO_FULL_TRYBUILD_BY_DEFAULT` violated.

**Debug:**
1. Check if files are marked as changed:
   ```sh
   git diff origin/main --name-only | grep -i trybuild
   ```

2. If no changed files, the "changed" logic will report 0 and exit safely:
   ```sh
   cargo run -- trybuild changed
   # Output: "No changed trybuild fixtures"
   ```

3. If changed files exist but full set still runs, check `ChangedFileDetector::is_trybuild_fixture()`:
   ```sh
   rg "is_trybuild_fixture" src/adapters/
   ```

4. Verify fixture paths match expected pattern (`tests/ui/compile_fail/*.rs`):
   ```sh
   find . -name "*ui*" -type d
   ls -la tests/ui/compile_fail/ | head -10
   ```

---

### Issue: cicd.toml Not Written After Command

**Cause:** CicdTomlWriter not called; changes not persisted.

**Fix:**
1. Check if the noun is supposed to write cicd.toml:
   - Some read-only verbs (e.g., `status show`) may not write
   - Execution verbs (e.g., `target prune --confirm`) should write

2. Manually trigger a write:
   ```sh
   cargo run -- workspace doctor
   # This should write cicd.toml
   ls -la cicd.toml
   ```

3. If cicd.toml still doesn't exist, check for permissions:
   ```sh
   touch cicd.toml && chmod 644 cicd.toml
   # Retry command
   ```

4. Verify serialization in tests:
   ```sh
   cargo test --test cicd_toml_truth
   ```

---

### Issue: Feature Flag Not Compiling

**Cause:** Code gated by feature flag is not compiled due to missing dependency.

**Debug:**
1. Check which feature is failing:
   ```sh
   cargo build --features autonomic 2>&1 | grep -i error
   ```

2. Verify feature is declared in Cargo.toml:
   ```toml
   [features]
   autonomic = ["process-data"]
   ```

3. Check for missing conditional compilation:
   ```rust
   #[cfg(feature = "autonomic")]
   pub fn my_fn() { ... }
   ```

4. If a dependency is missing, add it:
   ```toml
   # Cargo.toml
   [dependencies]
   some_crate = { version = "1.0", optional = true }
   
   [features]
   autonomic = ["process-data", "some_crate"]
   ```

---

## Project Layout

```
/home/user/cargo-cicd/
├── Cargo.toml                  # Root workspace manifest
├── Cargo.lock                  # Locked dependency versions
├── CLAUDE.md                   # This file
├── README.md                   # Public user guide (generated)
├── src/
│   ├── main.rs                 # Entry point; verb injection
│   ├── lib.rs                  # Public API
│   ├── evidence.rs             # ProcessEvent, XES emission
│   ├── session.rs              # Session lifecycle
│   ├── cicd_toml.rs            # cicd.toml schema
│   ├── nouns/                  # Noun modules (CLI grammar)
│   │   ├── status.rs           # Workspace snapshot
│   │   ├── target.rs           # Target directory
│   │   ├── test.rs             # Changed test selection
│   │   ├── trybuild.rs         # Compiler error tests
│   │   ├── git.rs              # Git phase tracking
│   │   ├── publish.rs          # Publishing gate
│   │   ├── workspace.rs        # Workspace diagnostics
│   │   ├── evidence.rs         # Evidence management
│   │   ├── pipeline.rs         # Sequential CI/CD runs
│   │   ├── lsp.rs              # IDE integration
│   │   └── mod.rs              # Noun registry
│   ├── engine/                 # Level 5 state aggregate
│   │   ├── mod.rs              # EngineState struct
│   │   ├── workspace_state.rs
│   │   ├── toolchain_state.rs
│   │   ├── target_state.rs
│   │   ├── changed_file_state.rs
│   │   ├── test_plan_state.rs
│   │   ├── trybuild_state.rs
│   │   ├── git_phase_state.rs
│   │   ├── process_event_state.rs
│   │   ├── artifact_state.rs
│   │   ├── policy_state.rs
│   │   └── projection_profile.rs
│   ├── adapters/               # External source adapters
│   │   ├── cargo_metadata.rs   # Cargo.toml name/members
│   │   ├── manifest_parser.rs  # Direct TOML parsing
│   │   ├── git_status.rs       # `git status` output
│   │   ├── toolchain_detector.rs  # `rustc --version`
│   │   ├── target_scanner.rs   # Target dir size
│   │   ├── changed_file_detector.rs  # `git diff` classification
│   │   ├── trybuild_detector.rs  # tests/ui/ fixture scan
│   │   ├── cicd_toml_writer.rs  # Serialize to cicd.toml
│   │   └── mod.rs              # Adapter registry
│   ├── autonomic/              # Policy suggestion layer
│   │   ├── mod.rs              # Feature gate
│   │   ├── policies.rs         # Policy runner
│   │   ├── policy_engine.rs    # Policy evaluation
│   │   └── signals.rs          # Internal signals
│   ├── policies/               # Individual policy implementations
│   │   ├── target_pressure.rs
│   │   ├── toolchain_mismatch.rs
│   │   ├── trybuild_changed.rs
│   │   ├── branch_behind.rs
│   │   ├── evidence_stale.rs
│   │   ├── publish_not_adjudicated.rs
│   │   ├── git_phase_dirty.rs
│   │   └── mod.rs              # Policy registry
│   ├── integrations/           # External service integration
│   │   ├── mod.rs
│   │   ├── wasm4pm_shell.rs    # wpm oracle invocation
│   │   └── wasm4pm_current.rs  # Current oracle state
│   └── state/                  # Duplicate state module (legacy)
├── tests/
│   ├── invariants.rs           # 7 non-negotiable public boundary rules
│   ├── feature_projection.rs   # Feature flag contract
│   ├── cicd_toml_truth.rs      # Serialization round-trip
│   ├── autonomic_policies.rs   # Policy evaluation
│   ├── changed_tests.rs        # File classification
│   ├── git_phase_closure.rs    # Git state detection
│   ├── cli/                    # Noun/verb CLI tests
│   │   ├── test_status.rs
│   │   ├── test_target.rs
│   │   ├── test_publish.rs
│   │   ├── test_git.rs
│   │   ├── test_workspace.rs
│   │   ├── test_evidence.rs
│   │   ├── command_projection.rs
│   │   ├── verb_registry.rs
│   │   └── mod.rs
│   ├── wasm4pm_evidence_gate.rs  # Happy path evidence → verdict
│   ├── wasm4pm_evidence_mutation.rs  # Corrupt evidence rejection
│   ├── wasm4pm_refusal_cases.rs  # Edge cases
│   ├── wasm4pm_harness.rs       # Test harness
│   ├── wpm_verdict_key_contract.rs  # Verdict structure contract
│   ├── fixtures/                # Test fixture workspaces
│   │   ├── trybuild_changed_only/
│   │   ├── trybuild_huge_set/
│   │   └── mod.rs
│   ├── fixture_workspaces.rs   # Fixture builders
│   ├── ggen_customization_guard.rs  # Generation idempotency
│   ├── refusal_calibration.rs  # Oracle refusal cases
│   ├── lsp_explain.rs          # LSP explain endpoint
│   ├── interactions.rs         # User interaction tests
│   ├── policies.rs             # Policy contract tests
│   └── publish_gate.rs         # Publishing gate behavior
├── ontology/
│   └── cargo-cicd-capabilities.ttl  # RDF/Turtle capability definitions
├── queries/
│   └── *.sparql                # SPARQL inference rules
├── templates/
│   ├── README.md.tera          # README template
│   └── docs/
│       └── reference-command.md.tera
├── crates/
│   ├── cargo-cicd-core/        # Shared core utilities
│   └── cargo-cicd-lsp/         # Language server protocol server
├── docs/
│   ├── reference/
│   │   └── commands/           # Generated command reference
│   ├── architecture/           # Design docs
│   ├── wasm4pm/                # Evidence adjudication docs
│   └── testing/                # Test strategy
├── receipts/                   # wpm receipt artifacts
└── target/                     # Build output
    └── cargo-cicd/
        └── evidence/           # Emitted XES/JSONL evidence
```

---

## FAQ

**Q: Can I use cargo-cicd without enabling process-data?**  
A: Yes. The default build has no Level 5 engine. Nouns work with minimal state. To enable autonomic policies, pass `--features autonomic`.

**Q: Do I need wpm installed to run cargo-cicd?**  
A: No. The binary works without wpm. Evidence gate tests require wpm for release, but local development can run with `ExpectedWpmVerdict::Blocked`.

**Q: Can I edit cicd.toml manually?**  
A: You can, but changes will be overwritten by the next verb run. cicd.toml is vehicle for state, not a config file.

**Q: How do I add a new workspace state dimension?**  
A: Add a new field to `EngineState`, create `src/engine/new_dimension_state.rs`, implement an adapter to populate it, and update `EngineState::from_workspace()`.

**Q: What happens if two verbs run in parallel?**  
A: cicd.toml writes are not atomic. Use `--confirm` flags or external locking if concurrent execution is needed.

**Q: Can policies take destructive action?**  
A: No. Policies run in **suggest mode only**. They emit recommendations; actual remediation is user-initiated or verb-invoked.

**Q: Where are slow commands documented?**  
A: See **Performance Notes** section above. TargetScannerAdapter is the slowest; consider caching in cicd.toml.

---

## Release Checklist

Before tagging a release:

- [ ] All tests pass: `cargo make test`
- [ ] No forbidden terms: `cargo test --test invariants`
- [ ] Feature flags compile: `cargo build --features autonomic,wasm4pm`
- [ ] Evidence gate passes: `cargo test --test wasm4pm_evidence_gate`
- [ ] wpm receipts are valid: `wpm receipt doctor --format json --strict receipts/*.json`
- [ ] README is up-to-date: `ggen` has been run
- [ ] CHANGELOG.md updated with new features/fixes
- [ ] Version bumped in Cargo.toml and main.rs
- [ ] Git is clean: `git status`

Then:
```sh
git add -A
git commit -m "chore(release): v<VERSION> ready for release"
git tag -a v<VERSION> -m "Release v<VERSION>"
git push origin main --tags
```

---

## Contact & Contribution

This document is the source of truth for cargo-cicd contributors using Claude Code. For questions about implementation, architecture decisions, or release procedure, refer back to the relevant section above.

**Last Updated:** 2026-06-14

---

## Claude Code Ecosystem

The `.claude/` directory wires up slash commands, subagents, skills, hooks, and settings so Claude Code can assist with cargo-cicd development tasks out of the box. A companion plugin bundle lives under `plugins/cargo-cicd-kit/`.

### Slash Commands (`.claude/commands/`)

| Command | Purpose |
|---|---|
| `release` | Run the full release checklist: bump version, run `cargo make check`, run all test suites (including wasm4pm evidence-gate), tag, and summarize what's left. |
| `audit-evidence` | Invoke `wpm audit` and `wpm receipt doctor --format json --strict` against every XES file in `target/cargo-cicd/evidence/`, then report Accept/Refuse verdicts. |
| `check-invariants` | Run `cargo test --test invariants` and surface any failing public-boundary contracts with file and line context. |
| `new-noun` | Scaffold a new noun module in `src/nouns/` following the clap-noun-verb grammar: creates the module file, registers it in `src/nouns/mod.rs`, and adds a default-verb entry in `main.rs::inject_default_verbs()`. |
| `ui-demo` | Run `cargo cicd ui demo` and capture/display terminal output so UI component changes can be previewed quickly. |
| `clean-target` | Remove stale build artifacts and prune `target/cargo-cicd/evidence/` of old XES files while preserving the latest receipt per command. |
| `phase-close` | Invoke `cargo cicd git close` to advance the git phase, then confirm cicd.toml `[state]` reflects the new phase. |

### Subagents (`.claude/agents/`)

| Agent | Purpose |
|---|---|
| `rust-reviewer` | Reviews Rust source changes in `src/` for correctness, safety, and adherence to the adapter/engine/noun architecture boundaries. Triggered when reviewing PRs or asking "is this safe?" |
| `invariant-guardian` | Specialises in the 7 public-boundary invariants. Runs `cargo test --test invariants` and maps failures back to the specific noun/verb output contract that broke. |
| `test-author` | Writes new integration tests under `tests/` using `assert_cmd` + `tempfile` patterns already established in the repo. |
| `ggen-regenerator` | Runs `ggen` after ontology or template changes and verifies the regenerated noun modules compile and pass `cargo make check`. |
| `evidence-gate-runner` | Orchestrates the wasm4pm evidence-gate: runs `cargo test --features wasm4pm`, collects XES from `target/cargo-cicd/evidence/`, invokes `wpm audit` and `wpm receipt doctor`, and reports verdicts. |
| `ui-polisher` | Reviews and improves terminal UI output in `src/ui/` — checks colour/glyph contracts, plain-mode cleanliness, and consistency with the design system. |
| `release-captain` | Drives the end-to-end release flow: version bump, full test run, evidence-gate closure, changelog entry, tag. Refuses to proceed if any wasm4pm verdict is Refuse. |

### Skills (`.claude/skills/`)

| Skill | Purpose |
|---|---|
| `release-checklist` | Step-by-step release procedure specific to cargo-cicd: version bump in `Cargo.toml`, `cargo make check`, invariants, evidence-gate, tag format `v<semver>`. |
| `evidence-audit` | How to collect XES evidence, run `wpm audit <file.xes>`, run `wpm receipt doctor --format json --strict <receipt.json>`, and interpret Accept/Refuse. |
| `noun-scaffold` | Concrete steps to add a new noun: create `src/nouns/<noun>.rs`, implement `NounCommand` + at least one `VerbCommand`, register in `src/nouns/mod.rs`, wire default verb in `main.rs`. |
| `ui-component` | How to add a new component to `src/ui/`: implement in a dedicated module, re-export from `src/ui/mod.rs`, ensure `Style::paint` for colour and `symbols::*` for glyphs, add a demo entry to `cargo cicd ui demo`. |
| `invariant-audit` | How to read `tests/invariants.rs`, map each invariant to its noun/verb output contract, and fix a failing assertion without breaking the public boundary. |

### Hooks (`.claude/hooks/`)

Hooks fire on Claude Code lifecycle events. The project registers hooks in `.claude/settings.json`. A `SessionStart` hook prints a workspace-readiness summary and verifies the toolchain. A `public-boundary-guard.sh` helper scans edited public files for forbidden terms.

### Plugin Bundle (`plugins/cargo-cicd-kit/`)

`plugins/cargo-cicd-kit/` is a self-contained Claude Code plugin that bundles the commands, agents, skills, and hooks above into a distributable unit. `plugins/cargo-cicd-kit/.claude-plugin/plugin.json` declares the bundle metadata. A root `.claude-plugin/marketplace.json` points to it. Installing the plugin in another workspace gives that workspace the full cargo-cicd assistant toolkit.

---

## Terminal UI Design System (`src/ui/`)

`src/ui/` is a zero-dependency terminal UI toolkit used by all noun output. It has no external crate dependencies — only `std`. Modules:

| Module | Role |
|---|---|
| `caps` | Detect terminal capabilities: colour support, Unicode support, TTY vs pipe. |
| `style` | `Style::paint(text, style)` — the single entry point for all coloured output. Auto-disables when stdout is not a TTY, so piped output is always plain. |
| `symbols` | Named glyph constants (`CHECK`, `CROSS`, `ARROW`, `BULLET`, etc.) with ASCII fallbacks selected at runtime via `caps`. All glyphs in noun output must come from this module. |
| `text` | String helpers including `display_width` (Unicode-aware column width for table alignment) and `truncate`. |
| `table` | Columnar table renderer. Column widths use `text::display_width`. |
| `panel` | Bordered content panel with an optional title. |
| `badge` | Inline status badge (e.g. `[PASS]`, `[FAIL]`, `[SKIP]`). |
| `progress` | Single-line progress indicator / spinner for long-running noun verbs. |
| `chart` | Horizontal bar chart for target-size or test-count summaries. |
| `tree` | Hierarchical tree renderer for workspace/package structures. |
| `theme` | Named colour palette (`Theme::default()`, `Theme::plain()`). Plain theme is selected automatically off-TTY. |
| `layout` | Composes panels/tables/trees into a full-screen layout for dashboard use. |
| `diagnostics` | Structured diagnostic message renderer (error/warn/info/hint with source location). |
| `dashboard` | Full-workspace status dashboard combining multiple components; invoked by `cargo cicd ui dashboard`. |

**Rules that must never be broken:**

1. All colour goes through `Style::paint` — never write raw ANSI escape codes directly in noun modules.
2. All glyphs go through `symbols::*` — never hard-code Unicode characters or box-drawing glyphs inline.
3. Column widths for tables/panels must use `text::display_width`, not `.len()`, to handle multi-byte characters correctly.
4. When stdout is not a TTY, output must be plain text with no escape codes and ASCII-only glyphs — this is enforced by `caps` and verified by tests that capture non-TTY output.

Run `cargo cicd ui demo` to render every component in isolation. Run `cargo cicd ui dashboard` for the full workspace overview. Both commands exercise the complete design-system surface.

---

## Advanced Capabilities (Feature: advanced)

10 opt-in best-of-breed crates extend cargo-cicd with hyper-fast scanning, observability, caching, metrics, and dependency analysis. All are gated behind the `advanced` feature flag, keeping the default binary lean and fast.

### Quick Reference

| Module | Crate(s) | Use Case |
|--------|----------|----------|
| `parallel_scan` | `ignore` + `rayon` | Gitignore-aware, multi-threaded workspace scanning |
| `fingerprint` | `blake3` | Content-addressed Merkle fingerprinting of artifacts |
| `observability` | `tracing` + `tracing-subscriber` | Structured span instrumentation & JSON traces |
| `diagnostics` | `miette` + `thiserror` | Rich, rendered diagnostic error messages |
| `cache` | `moka` | Concurrent, TTL-aware engine result caching |
| `snapshot` | `bitcode` | Compact binary serialization of engine state |
| `dep_graph` | `petgraph` | Workspace dependency graphs & build order |
| `timeline` | `jiff` | High-precision, zoned process timestamps |
| `histogram` | `hdrhistogram` | Latency percentiles for pipeline stages |
| `pattern` | `aho-corasick` | Multi-pattern governance & path scanning |

### Advanced Feature Examples

#### Using `parallel_scan` in an Adapter

```rust
use cargo_cicd::advanced::parallel_scan::scan_workspace;
use std::path::Path;

// In your adapter:
let report = scan_workspace(Path::new("."))?;
println!("Total files: {}", report.total_files);
println!("Total bytes: {}", report.total_bytes);
println!("Reclaimable (target/): {} bytes", report.reclaimable_bytes());

// Per-extension breakdown is deterministic (BTreeMap):
for (ext, stats) in report.per_extension.iter() {
    println!("{}: {} files, {} bytes", ext, stats.count, stats.bytes);
}
```

#### Instrumenting a Pipeline Stage with `observability`

```rust
use cargo_cicd::advanced::observability::{init_tracing, PipelineStage, record_event};

// Once per process:
init_tracing();

// Around a unit of work:
{
    let _stage = PipelineStage::enter("my_adapter_scan");
    // ... populate engine state ...
    record_event("my_adapter_scan", true);
} // Drops here; emits elapsed_ms + structured JSON trace
```

#### Caching Adapter Results with `cache`

```rust
use cargo_cicd::advanced::cache::{EngineCache, CachedEntry};
use std::time::Duration;

let cache = EngineCache::new(100, Duration::from_secs(300));

// Store a serialized result:
let entry = CachedEntry::with_label(serialized_bytes, "CargoMetadata");
cache.insert("workspace_metadata".to_string(), entry);

// Retrieve cheaply (Arc clone):
if let Some(hit) = cache.get("workspace_metadata") {
    let bytes = hit.bytes.clone();
    // deserialize bytes ...
}

// Force eviction/expiry:
cache.run_pending_tasks();
```

#### Accessing Timeline Events from `ProcessEventState`

```rust
use cargo_cicd::advanced::timeline::ProcessTimeline;
use jiff::Timestamp;

let mut timeline = ProcessTimeline::new();

// Record an event at current time:
timeline.record("workspace_scan");

// Or at a fixed time (for testing):
timeline.record_at("workspace_scan", Timestamp::now());

// Iterate in order:
for event in timeline.iter() {
    println!("{}: {}", event.label, event.at);
}

// Measure span between events:
let elapsed = timeline.span(0, 1); // jiff::Span
println!("Duration: {}", elapsed);
```

### Testing Advanced Features

```sh
# Run all tests with advanced capabilities enabled
cargo test --features advanced

# Quick syntax check (lib + advanced)
cargo check --lib --features advanced

# Unit tests only
cargo test --lib --features advanced

# Test feature combinations (advanced + autonomic)
cargo test --features advanced,autonomic

# Run a specific advanced test
cargo test --test feature_projection --features advanced
```

### Advanced Adapter Integrations

| Adapter | What It Does | Key Methods | When to Use |
|---------|--------------|-------------|------------|
| `cached` | Wraps any adapter result with moka cache hits/misses | `EngineCache::new()`, `insert()`, `get()` | When adapter recomputation is expensive (metadata, toolchain probes) |
| `fingerprint` | Computes BLAKE3 hashes over artifact byte spans | `fingerprint_bytes()`, `verify_checksum()` | For artifact content-addressing or integrity checks in cicd.toml |
| `state_snapshot` | Serializes/deserializes `EngineState` to compact binaries | `snapshot_state()`, `restore_state()` | For inter-process checkpointing or distributed cache warm-up |
| `governance_patterns` | Scans paths/files against multi-pattern rules via aho-corasick | `PatternScanner::new()`, `scan_path()` | For policy-driven path filtering or license/copyright detection |

See `src/advanced/` for full API docs and `src/adapters/` for integration patterns.
