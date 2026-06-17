# cargo-cicd Architecture Deep-Dive

A comprehensive guide to the design principles, component interactions, and extensibility patterns of cargo-cicd v26.6.2.

## Table of Contents

1. [EngineState Design](#enginestate-design)
2. [Adapter Pattern](#adapter-pattern)
3. [Noun-Verb Grammar](#noun-verb-grammar)
4. [ggen Ontology Pipeline](#ggen-ontology-pipeline)
5. [Feature Flag Strategy](#feature-flag-strategy)
6. [Policy System](#policy-system)
7. [cicd.toml Semantics](#cicdtoml-semantics)
8. [Evidence Gate & wasm4pm Integration](#evidence-gate--wasm4pm-integration)

---

## EngineState Design

### Rationale: Aggregate Root Model

EngineState is the **single aggregate root** modeling all runtime dimensions of a Rust workspace CI/CD session. Rather than scattered mutable state across the codebase, EngineState centralizes state as an immutable snapshot, enabling:

- **Single source of truth**: All nouns and verbs read from one consistent view
- **Testability**: State can be constructed in tests without side effects
- **Evidence traceability**: Process events reference state snapshots
- **Feature composition**: Feature flags control which dimensions are populated

### State Dimensions

Located in `src/engine/`, EngineState aggregates 11 dimensions:

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

#### Dimension Descriptions

| Dimension | Purpose | Feature Gate | Example Content |
|-----------|---------|--------------|-----------------|
| **workspace** | Cargo metadata snapshot | (none) | name, root_path, members, toolchain, rust_edition |
| **toolchain** | Active Rust toolchain | (none) | version, components, target info |
| **target** | Compilation target state | (none) | size_gb, verdict (pass/warn/fail), prune candidates |
| **changed_files** | Git-detected changes | (none) | dirty_files, staged_files, untracked |
| **test_plan** | Changed test detection | (none) | changed_tests (paths), coverage delta |
| **trybuild** | Compile-fail test changes | (none) | changed_fixtures, error snapshots |
| **git_phase** | Git repository state | (none) | branch, ahead/behind, phase (pending/unstaged) |
| **process_events** | Emitted evidence | `process-data` | events in XES/JSONL format |
| **artifacts** | Build output snapshots | `process-data` | binaries, rlibs, documented items |
| **policies** | Autonomic policy results | `autonomic` | policy verdicts, recommendations |
| **projection** | Feature flag visibility | (none) | enabled features, capability matrix |

### Invariants

The following invariants **must** hold across all EngineState instances:

1. **I1: Workspace immutability** — WorkspaceState is read-only; it reflects Cargo.toml/workspace at snapshot time. No in-process Cargo.toml mutations.

2. **I2: Git phase alignment** — GitPhaseState's branch and dirty status must align with external `git status --porcelain` output. If the working tree changes between adapter query and consumption, the state is stale but not invalid.

3. **I3: Target size accuracy** — TargetState.size_gb is computed from a full walk of `target/` at snapshot time. It does not account for concurrent builds; for precise tracking, query the adapter again.

4. **I4: No circular dependencies** — NoVerb and VerbCommand implementations must not mutate EngineState. Mutations flow only through adapters → CicdTomlWriter.

5. **I5: Feature flag containment** — If a dimension is unpopulated because its feature gate is off, consuming code must gracefully handle `Default` values or bail early with a clear error.

---

## Adapter Pattern

### Why Adapters Exist

The adapter layer sits **between external representations and the internal state model**. This boundary enforces:

- **Single Responsibility**: Each adapter owns one external source (git, cargo, filesystem)
- **No Leakage**: External tool output (CLI strings, filesystem walks, toml parsing) stays isolated
- **Testability**: Mock adapters can be swapped for real ones in tests
- **Extensibility**: Adding a new external source (e.g., CI platform data) requires only a new adapter, not refactoring nouns

### Adapter Catalog

Located in `src/adapters/`, each adapter exposes **only static methods** and wraps a single external tool or API:

```
adapters/
├── git_status.rs          → `git status --porcelain`, `git rev-parse`
├── target_scanner.rs      → filesystem walk of target/
├── toolchain_detector.rs  → rust-toolchain.toml, rustup
├── cargo_metadata.rs      → cargo metadata JSON
├── changed_file_detector.rs → git diff for test impact analysis
├── trybuild_detector.rs   → trybuild fixture discovery
├── cicd_toml_writer.rs    → TOML serialization to cicd.toml
└── (legacy) cargo_meta.rs, git.rs, target.rs, fs.rs
```

### Translation Pattern: External → Internal

**Example: GitStatusAdapter**

```rust
// External representation (git CLI)
pub struct GitStatusResult {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}

// Internal state model (immutable snapshot)
pub struct GitPhaseState {
    pub branch: String,
    pub phase: GitPhase,  // Pending, Unstaged, Clean
    pub dirty_count: usize,
    pub untracked_count: usize,
}

// Adapter (pure translation)
impl GitStatusAdapter {
    pub fn query() -> Result<GitStatusResult> {
        // Shell out to git, parse output
    }
}

// In a noun verb, consuming the state:
let git_result = GitStatusAdapter::query()?;
let git_state = GitPhaseState {
    branch: git_result.branch.clone(),
    phase: if git_result.dirty_files.is_empty() {
        GitPhase::Clean
    } else {
        GitPhase::Pending
    },
    // ...
};
```

### Extension Points

To add a new external source:

1. **Create a new adapter** in `src/adapters/your_adapter.rs`:
   ```rust
   pub struct YourAdapter;
   impl YourAdapter {
       pub fn query() -> Result<YourResult> { /* */ }
   }
   ```

2. **Add a corresponding state dimension** in `src/engine/your_state.rs`:
   ```rust
   #[derive(Debug, Default, Serialize, Deserialize, Clone)]
   pub struct YourState {
       // Immutable snapshot fields
   }
   ```

3. **Export from `src/adapters/mod.rs` and `src/engine/mod.rs`**:
   ```rust
   pub use your_adapter::YourAdapter;
   pub use your_state::YourState;
   ```

4. **Add the state to EngineState**:
   ```rust
   pub struct EngineState {
       pub your: YourState,
       // ...
   }
   ```

5. **Populate in nouns** as needed (adapters are called on-demand, not eagerly).

---

## Noun-Verb Grammar

### The clap-noun-verb Design

cargo-cicd uses **clap-noun-verb**, a grammatical CLI framework that models each command as `<NOUN> <VERB> [FLAGS]`:

```
cargo cicd status show              # noun=status, verb=show
cargo cicd target prune --confirm   # noun=target, verb=prune
cargo cicd test changed             # noun=test, verb=changed
```

This grammar brings clarity to Rust CI/CD workflows by **modeling CI/CD concepts as domain nouns**:

| Noun | Verbs | Purpose |
|------|-------|---------|
| **status** | show, audit | Report workspace health |
| **target** | show, prune | Manage target/ size |
| **test** | changed | Run only changed tests |
| **trybuild** | changed | Run changed compile-fail fixtures |
| **git** | status, close | Check git state, finalize branches |
| **publish** | run | Emit cicd.toml |
| **workspace** | doctor | Health diagnostics |
| **pipeline** | run | Sequential command runner |
| **evidence** | doctor, audit | Query/verify process logs |
| **lsp** | explain | IDE integration (LSP) |

### Benefits

1. **Discoverability**: `cargo cicd --help` lists nouns; `cargo cicd status --help` lists verbs.
2. **Consistent UX**: All verbs follow the same flags/behavior patterns (e.g., all destructive verbs require `--confirm`).
3. **Composability**: The `pipeline` noun can sequence commands naturally: `pipeline run status.show test.changed git.close`.
4. **Grammar-aware code generation**: The ggen ontology pipeline can generate noun modules and test scaffolding directly from a capability ontology.

### Default Verb Injection

Many users type just the noun (e.g., `cargo cicd status`) without the verb. The **default verb injection** in `src/main.rs` rewrites the command before parsing:

```rust
fn inject_default_verbs(mut args: Vec<String>) -> Vec<String> {
    // [binary, "status"] → [binary, "status", "show"]
    // [binary, "publish"] → [binary, "publish", "run"]
    // etc.
}
```

This allows:
- **Simplified public CLI**: Users never type full `noun verb`; the most common verb is the default.
- **Backward compatibility**: Old scripts using `cargo cicd status` still work.
- **Internal structure**: Internally, nouns and verbs are cleanly separated in code.

The defaults are:
- `status` → `show`
- `publish` → `run`
- `workspace` → `doctor`
- `evidence` → `doctor`

### Adding a New Noun

1. **Create the noun module** at `src/nouns/your_noun.rs`:

```rust
use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};

pub struct YourNoun;

impl YourNoun {
    pub fn new() -> Self { Self }
}

impl NounCommand for YourNoun {
    fn name(&self) -> &'static str { "your_noun" }
    fn about(&self) -> &'static str { "What your noun does" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(YourVerbOne),
            Box::new(YourVerbTwo),
        ]
    }
}

pub struct YourVerbOne;
impl VerbCommand for YourVerbOne {
    fn name(&self) -> &'static str { "verb_one" }
    fn about(&self) -> &'static str { "Does something" }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // Implementation
        Ok(())
    }
}
```

2. **Export from `src/nouns/mod.rs`**:
```rust
pub mod your_noun;
```

3. **Register in `src/main.rs`**:
```rust
let cli = cli.noun(nouns::your_noun::YourNoun::new());
```

4. **Add ontology entry** in `ontology/cargo-cicd.ttl` (for ggen):
```turtle
:YourNoun a cc:Capability ;
    cc:noun "your_noun" ;
    cc:verb "verb_one" ;
    cc:cliCommand "cargo cicd your_noun verb_one" ;
    dcterms:description "Description of what it does" .
```

---

## ggen Ontology Pipeline

### Overview: From RDF to Rust Code

The **ggen manufacturing pipeline** generates cargo-cicd's public surface (CLI docs, test scaffolding, README) from a single semantic source of truth: the ontology. This ensures documentation never drifts from actual capabilities.

```
ontology/cargo-cicd.ttl
    ↓ (SPARQL inference)
Capability graph
    ↓ (SPARQL queries in ggen.toml)
Result sets (Verb descriptions, etc.)
    ↓ (Tera templates)
Rendered markdown, test code, README
```

### Components

**Source**: `ontology/cargo-cicd.ttl` — RDF/Turtle graph of all commands, features, and relationships.

```turtle
@prefix cc:  <https://cargo-cicd.rs/ontology/> .
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .

:StatusNoun a cc:Capability ;
    cc:noun "status" ;
    cc:verb "show" ;
    cc:cliCommand "cargo cicd status show" ;
    dcterms:description "Show workspace CI/CD status" ;
    cc:emoji "📊" .
```

**Inference Rules**: `ggen.toml` [inference.rules] apply SPARQL CONSTRUCT to infer derived facts. Example: capability-projection rule creates clean RDF for querying.

**Queries**: `ggen.toml` [generation.rules] define per-artifact SPARQL SELECT queries. Each query binds variables for template rendering.

```toml
[[generation.rules]]
name = "readme"
query = { inline = """
PREFIX cc:      <https://cargo-cicd.rs/ontology/>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?cli_command ?noun ?verb ?description
WHERE {
  ?cap a cc:Capability ;
       cc:cliCommand ?cli_command ;
       cc:noun ?noun ;
       cc:verb ?verb ;
       dcterms:description ?description .
}
ORDER BY ?noun ?verb
""" }
template = { file = "templates/README.md.tera" }
output_file = "README.md"
```

**Templates**: `templates/*.tera` are Tera (Jinja2-like) templates consuming query results.

```
templates/
├── README.md.tera              # Main CLI reference
├── noun.rs.tera               # Generated noun modules (deferred)
├── cli_test.rs.tera           # Generated integration tests
├── docs/
│   ├── reference-command.md.tera
│   ├── tutorial.md.tera
│   ├── how-to.md.tera
│   └── explanation.md.tera
└── receipts/
    └── prepublish.md.tera     # Release checklist
```

### When to Use ggen

Run `ggen` (the external tool) after:

1. **Adding a new noun** to the ontology:
   ```turtle
   :NewNoun a cc:Capability ;
       cc:noun "new_noun" ;
       cc:verb "default_verb" ;
       cc:cliCommand "cargo cicd new_noun default_verb" ;
       dcterms:description "..." .
   ```

2. **Changing capability descriptions**: Descriptions in the ontology flow to all generated docs via queries.

3. **Adding feature gates or semantic relationships**: E.g., marking a verb as `process-data` only.

### Example: Generating Command Reference

Query: `queries/docs-reference-command.rq`

```sparql
PREFIX cc:      <https://cargo-cicd.rs/ontology/>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?noun ?verb ?cliCommand ?description
WHERE {
  ?cap a cc:Capability ;
       cc:noun ?noun ;
       cc:verb ?verb ;
       cc:cliCommand ?cliCommand ;
       dcterms:description ?description .
  FILTER(?noun = "target" && ?verb = "show")
}
```

Template: `templates/docs/reference-command.md.tera`

```jinja2
# {{ noun | capitalize }} {{ verb | capitalize }}

Command: `{{ cliCommand }}`

{{ description }}

## Flags

- `--help`: Show command help
- [other flags from ontology...]

## Examples

[examples from ontology...]
```

Output: `docs/reference/commands/target-show.md`

### Customization Guard

The test `tests/ggen_customization_guard.rs` enforces that **hand-edited files are never overwritten** by ggen. Files marked `mode = "Preserve"` in `ggen.toml` will not be regenerated if they exist.

---

## Feature Flag Strategy

### Why Feature Gates?

cargo-cicd uses Cargo feature flags to **control capability exposure** while keeping the binary lean. Features are **cumulative** (feature A may imply B) and **statically resolved at compile time**.

```toml
[features]
default = []
process-data = []                    # Level 5 engine state population
autonomic = ["process-data"]         # Policy evaluation (implies process-data)
contrib = ["process-data"]           # Contributor-facing tooling
wasm4pm = ["process-data"]           # Evidence gate integration
```

### Feature Dependency Graph

```
default (empty)
├── process-data
│   ├── autonomic
│   ├── contrib
│   └── wasm4pm
```

### Feature Semantics

| Feature | Scope | Enables | Tests | Default? |
|---------|-------|---------|-------|----------|
| **process-data** | Engine | EngineState population, ProcessEventState, ArtifactState, evidence emission | process-data tests ignore gate | No |
| **autonomic** | Policies | PolicyState, policy evaluation, suggest mode | autonomic_policies | No |
| **contrib** | Development | Contributor commands, ggen tooling, internal diagnostics | none | No |
| **wasm4pm** | Integration | Wasm4pmShell, evidence → XES, oracle adjudication | wasm4pm_* test suite | No |

### Interaction Matrix

| Build | Features | Capabilities | Use Case |
|-------|----------|--------------|----------|
| Release | (none) | status, target, test, git, publish, workspace (no policies, no evidence gates) | Lean binary for users |
| CI Certification | `wasm4pm` | All above + evidence emission + wasm4pm oracle checks | Release validation |
| Local Development | `autonomic` | All above + policy evaluation + suggest mode | Developer workflow |
| Contributor | `contrib` + `autonomic` | All above + internal tools | Maintainer debugging |

### Conditional Compilation in Code

Feature gates use `#[cfg(...)]` to enable/disable code paths:

```rust
// In a noun or adapter
#[cfg(feature = "process-data")]
fn populate_process_events(state: &mut EngineState) {
    state.process_events = ProcessEventState::from_evidence_dir()?;
}

#[cfg(not(feature = "process-data"))]
fn populate_process_events(_state: &mut EngineState) {
    // No-op; process_events remains Default
}

// In tests
#[cfg(feature = "wasm4pm")]
#[test]
fn test_evidence_gate() {
    // Uses wasm4pm shell integration
}

#[cfg(not(feature = "wasm4pm"))]
#[test]
fn test_evidence_gate() {
    // Expects Blocked verdict
}
```

### Testing Implications

**Non-closing tests** (src/tests/invariants.rs, etc.) run on the default feature set. They verify public CLI boundaries and basic functionality.

**Closing tests** (tests/wasm4pm_evidence_gate.rs) require the `wasm4pm` feature and verify process conformance via external oracle. No release can be made without these passing.

### Adding a New Feature

1. **Declare in Cargo.toml**:
   ```toml
   [features]
   your_feature = ["process-data"]  # if it depends on engine state
   ```

2. **Gate code with `#[cfg(feature = "...")]`**:
   ```rust
   #[cfg(feature = "your_feature")]
   pub fn your_capability() { /* */ }
   ```

3. **Create a test module** gated by the same feature:
   ```rust
   #[cfg(feature = "your_feature")]
   mod tests { /* */ }
   ```

4. **Update ggen.toml** if the feature affects capability availability:
   ```turtle
   :YourCapability cc:gatedBy "your_feature" .
   ```

---

## Policy System

### How Autonomic Policies Work

**Autonomic policies** are recommendations that run in **suggest mode by default** — they never take destructive action. They read `PolicyState` and emit verdicts and recommendations to `cicd.toml [autonomic]`.

Located in `src/policies/`, each policy implements `CicdPolicy`:

```rust
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn mode(&self) -> PolicyMode;
    fn evaluate(&self) -> PolicyResult;
}
```

### Available Policies

| Policy | Trigger | Verdict | Recommendation |
|--------|---------|---------|-----------------|
| **git_phase_dirty** | Working tree has uncommitted changes | Warn/Alert | Commit or stash before CI run |
| **toolchain_mismatch** | Active toolchain ≠ rust-toolchain.toml | Warn | Switch toolchain: `rustup default ...` |
| **target_pressure** | target/ size > 70% of max | Warn | Run `cargo cicd target prune` |
| **trybuild_changed** | Trybuild fixtures modified | Alert | Re-run `cargo test` to regenerate snapshots |

### PolicyMode and Verdict

```rust
pub enum PolicyMode {
    Suggest,    // Read-only; emit recommendation
    Apply,      // Would take action (reserved for future)
}

pub enum PolicyVerdict {
    Pass,       // No issues detected
    Warn,       // Non-blocking concern
    Alert,      // Blocking concern
}

pub struct PolicyResult {
    pub name: String,
    pub enabled: bool,
    pub mode: String,
    pub verdict: String,
    pub recommendation: Option<String>,
    pub event_kind: String,
}
```

### Suggest Mode (Default)

Policies run in **suggest mode** (never apply changes). They emit recommendations to the user:

```
policy git_phase_dirty: ALERT
  recommendation: working tree is dirty — commit or stash changes before CI run
```

These recommendations are:
- Stored in `PolicyState` for inspection by higher-level tools
- Printed to stdout for immediate user feedback
- Recorded in `cicd.toml [autonomic] [[policies]]` for audit trails

### PolicyState Persistence

```toml
[autonomic]
mode = "suggest"

[[autonomic.policies]]
name = "git_phase_dirty"
enabled = true
mode = "suggest"
verdict = "alert"
recommendation = "working tree is dirty — commit or stash changes before CI run"

[[autonomic.policies]]
name = "toolchain_mismatch"
enabled = true
mode = "suggest"
verdict = "pass"
```

### Adding a New Policy

1. **Create `src/policies/your_policy.rs`**:

```rust
use super::{CicdPolicy, PolicyMode, PolicyResult, PolicyVerdict};

pub struct YourPolicy;

impl CicdPolicy for YourPolicy {
    fn name(&self) -> &'static str {
        "your_policy"
    }

    fn enabled(&self) -> bool {
        // Check if enabled in cicd.toml [autonomic]
        true
    }

    fn mode(&self) -> PolicyMode {
        PolicyMode::Suggest
    }

    fn evaluate(&self) -> PolicyResult {
        let (verdict, rec) = if /* condition */ {
            ("alert", Some("recommendation text".into()))
        } else {
            ("pass", None)
        };

        PolicyResult {
            name: self.name().into(),
            enabled: true,
            mode: "suggest".into(),
            verdict: verdict.into(),
            recommendation: rec,
            event_kind: "your_policy".into(),
        }
    }
}
```

2. **Export from `src/policies/mod.rs`**:

```rust
pub mod your_policy;
pub use your_policy::YourPolicy;
```

3. **Run in autonomic module** (`src/autonomic/policies.rs`):

```rust
pub fn run_all_policies(state: &EngineState) -> Vec<PolicyResult> {
    vec![
        YourPolicy.evaluate(),
        // ... other policies
    ]
}
```

4. **Add to cicd.toml schema** (`src/cicd_toml.rs`):

```rust
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AutonomicSection {
    pub mode: String,
    pub policies: Vec<PolicyEntry>,
}
```

---

## cicd.toml Semantics

### State Persistence Model

`cicd.toml` is the **carrier file** for EngineState snapshots and event logs. Written to the workspace root by `CicdTomlWriter`, it persists:

1. **Workspace metadata** (name, toolchain, target directory)
2. **State snapshots** (target size, git branch, changed files)
3. **Autonomic policy results** (verdicts, recommendations)
4. **Process events** (evidence from recent runs)

### Schema Structure

```toml
[workspace]
name = "cargo-cicd"
toolchain = "stable"
target_dir = "target"

[state]
dirty = true
target_size_gb = 5.42
changed_files = 3
changed_tests = 1
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
# Future: test-specific settings

[trybuild.changed]
# Future: trybuild-specific settings

[git]
# Future: git-specific settings

[autonomic]
mode = "suggest"

[[autonomic.policies]]
name = "git_phase_dirty"
enabled = true
mode = "suggest"
verdict = "alert"
recommendation = "working tree is dirty — commit or stash changes before CI run"

[[events]]
timestamp = "2026-06-14T13:45:00.000Z"
kind = "status:show"
case_id = "session-abc123"
command = "cargo cicd status show"
duration_ms = 245
verdict = "PASS"
```

### Event Recording

Process events are appended to `[[events]]` as they occur. Each event captures:

```rust
pub struct EventRecord {
    pub timestamp: String,           // ISO-8601
    pub kind: String,                // "status:show", "test:changed", etc.
    pub case_id: Option<String>,     // Session identifier
    pub command: String,             // Full CLI invocation
    pub duration_ms: u64,            // Elapsed time
    pub verdict: String,             // "PASS", "WARN", "FAIL"
}
```

### Workspace Config Inheritance

The `[workspace]` section is auto-detected:

```rust
fn detect_workspace_name() -> String {
    // Read name from Cargo.toml, fallback to cwd name
}

fn detect_toolchain() -> String {
    // Check rust-toolchain.toml, then rust-toolchain, then "stable"
}
```

This allows `cicd.toml` to be checked into git and remain valid across toolchain updates.

### State Mutation & Atomicity

`CicdTomlWriter::write()` is called **after** a successful command execution:

```rust
// In a noun verb:
let mut state = EngineState::from_adapters()?;
// ... perform work ...
CicdTomlWriter::write(&state)?;  // Atomic TOML write
```

**No partial updates**: cicd.toml is either valid or unchanged. Failed writes are logged but do not abort the command.

### Durability Guarantee

The event log `[[events]]` is **append-only**. Once written, events are never modified:

```rust
pub fn append_events(events: &[ProcessEvent], dir: &Path) -> Result<()> {
    let mut toml = CicdToml::load_or_default(dir)?;
    toml.events.extend(events);
    toml.save(dir)?;
}
```

This enables:
- **Audit trails**: Full event history for compliance
- **Replay**: Reconstruct state from event log
- **Diagnostics**: Grep the log for failures

---

## Evidence Gate & wasm4pm Integration

### The Evidence Gate Invariants

The **evidence gate** enforces a critical principle: **cargo-cicd never adjudicates its own conformance**. All process verdicts come from an external wasm4pm oracle.

Seven invariants govern evidence emission and gate testing:

**E1: Cargo-cicd does not self-judge** — All verdicts are issued by wasm4pm, never by internal assertions.

```rust
// WRONG: internal verdict
if test_result.is_ok() {
    println!("PASS");
}

// RIGHT: defer to oracle
emit_evidence()?;
if let Ok(verdict) = wasm4pm_oracle.audit(&evidence_file)? {
    println!("{}", verdict);
}
```

**E2: Evidence before adjudication** — The XES file must exist on disk before `audit_xes()` is called.

```rust
let xes_path = evidence_dir.join("events.xes");
emit_xes_events(&events, &xes_path)?;  // Write to disk first
let verdict = Wasm4pmShell::audit(&xes_path)?;  // Then query
```

**E3: Oracle availability required** — If the oracle is unavailable and the test expects non-Blocked verdict, the test panics.

```rust
let verdict = match Wasm4pmShell::detect() {
    Some(wpm) => wpm.audit(&xes)?,
    None => {
        if expected == ExpectedWpmVerdict::Blocked {
            WpmVerdict::Partial  // Graceful
        } else {
            panic!("wpm required for non-Blocked expected verdict")
        }
    }
};
```

**E4: Assert only oracle verdict** — Test assertions compare against wasm4pm verdicts, not cargo-cicd internal state.

```rust
// WRONG: test asserts internal state
assert_eq!(test_result.passed, 42);

// RIGHT: test asserts oracle verdict
let oracle_verdict = wasm4pm.audit(&evidence_file)?;
assert_eq!(oracle_verdict, WpmVerdict::Pass);
```

**E5: Trace grouping by case_id** — XES emission groups events by `case_id` into separate `<trace>` elements.

```xml
<log xes:version="1.0">
  <trace>
    <string key="concept:name" value="session-abc123"/>
    <event>
      <string key="concept:name" value="status:show"/>
      <date key="time:timestamp" value="2026-06-14T13:45:00Z"/>
    </event>
  </trace>
  <trace>
    <string key="concept:name" value="session-def456"/>
    <!-- other case_id events -->
  </trace>
</log>
```

**E6: XES and JSONL parity** — Both formats emit the same event set; XES is the primary oracle input, JSONL is a companion for tooling.

```rust
pub fn emit_evidence(
    events: &[ProcessEvent],
    format: EvidenceFormat,
) -> Result<()> {
    match format {
        EvidenceFormat::XES => emit_xes(events)?,
        EvidenceFormat::JSONL => emit_jsonl(events)?,
    }
}
```

**E7: Blocked verdict is first-class** — Tests running without wpm must declare `ExpectedWpmVerdict::Blocked`.

```rust
#[test]
fn test_without_wpm() {
    let evidence = collect_evidence()?;
    let verdict = Wasm4pmShell::audit(&evidence)
        .unwrap_or(WpmVerdict::Partial);
    assert_eq!(verdict, WpmVerdict::Partial);
}
```

### wasm4pm Integration Seams

Located in `src/integrations/wasm4pm_shell.rs`, the **SHELL_OUT** adapter invokes the wpm CLI binary:

```rust
pub struct Wasm4pmShell {
    binary: String,
}

impl Wasm4pmShell {
    pub fn detect() -> Option<Self> {
        // Probe PATH for wpm, or use known path
    }

    pub fn audit(&self, xes_path: &Path) -> Result<WpmVerdict> {
        let output = Command::new(&self.binary)
            .args(["audit", xes_path.to_str().unwrap()])
            .output()?;
        // Parse output to verdict
    }

    pub fn receipt_doctor(&self, receipt: &Path) -> Result<WpmVerdict> {
        let output = Command::new(&self.binary)
            .args(["receipt", "doctor", "--format", "json", "--strict", receipt.to_str().unwrap()])
            .output()?;
        // Parse JSON verdict
    }
}
```

**Confirmed working commands**:

| Command | Purpose |
|---------|---------|
| `wpm audit <input.xes>` | XES conformance audit (SIMD replay) |
| `wpm receipt doctor <file>` | Receipt forensic audit |
| `wpm lean` | Lean Six Sigma waste audit |
| `wpm spc status` | Statistical Process Control |
| `wpm doctor` | System health check |
| `wpm telco status` | Telco routing status |
| `wpm autoprocess` | AutoProcess pipeline |

### Test Hierarchy with Evidence Gates

```
tests/
├── invariants.rs                 # Public boundary invariants (non-closing)
├── cicd_toml_truth.rs           # State schema validation (non-closing)
├── cli/                          # CLI parsing tests (non-closing)
├── autonomic_policies.rs         # Policy evaluation (non-closing)
├── feature_projection.rs         # Feature flag matrix (non-closing)
│
├── wasm4pm_evidence_gate.rs      # Evidence gate closure (CLOSING)
├── wasm4pm_evidence_mutation.rs  # Oracle sensitivity (CLOSING)
└── wasm4pm_refusal_cases.rs      # Failure scenarios (CLOSING)
```

**Non-closing tests** (may use internal assertions, no oracle required):
- Verify CLI parsing
- Check schema validity
- Test adapter output
- Assert public boundary invariants

**Closing tests** (must emit evidence, require oracle):
- Verify process conformance
- Submit evidence to wpm
- Assert oracle verdict
- No release without passing

---

## Integration Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     User CLI (main.rs)                       │
│   cargo cicd <noun> <verb> [--flags]                        │
│   └─ Default verb injection (status→show, etc.)             │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────▼────────────┐
         │   CliBuilder::run()     │
         │  (clap-noun-verb)       │
         └────────────┬────────────┘
                      │
        ┌─────────────▼──────────────┐
        │  Noun::new() → VerbCommand  │
        │  (e.g., StatusNoun)         │
        └─────────────┬──────────────┘
                      │
     ┌────────────────┴────────────────┐
     │   Adapter Layer (on-demand)     │
     │                                  │
     │ GitStatusAdapter ────┐          │
     │ TargetScannerAdapter │──┐       │
     │ ToolchainDetector    │  │       │
     │ CargoMetadataAdapter │  │       │
     │ TrybuildDetector     │  │       │
     │ ChangedFileDetector ─┘  │       │
     └────────────────┬────────┘       │
                      │
         ┌────────────▼──────────────┐
         │  EngineState (snapshot)    │
         │                             │
         │ • workspace                │
         │ • toolchain                │
         │ • target                   │
         │ • changed_files            │
         │ • test_plan                │
         │ • trybuild                 │
         │ • git_phase                │
         │ • process_events (if -data) │
         │ • artifacts (if -data)     │
         │ • policies (if autonomic)  │
         │ • projection               │
         └────────────┬───────────────┘
                      │
     ┌────────────────┴─────────────────┐
     │                                   │
     │  Noun/Verb Execution              │
     │  (read-only from EngineState)     │
     │                                   │
     │  [autonomic mode]                 │
     │  └─ PolicyEngine                  │
     │     └─ All policies evaluated     │
     │        → PolicyState              │
     │                                   │
     │  [process-data mode]              │
     │  └─ Evidence emission             │
     │     └─ ProcessEventState          │
     │        → XES / JSONL              │
     └────────────────┬──────────────────┘
                      │
         ┌────────────▼──────────────┐
         │  CicdTomlWriter            │
         │  (persist state to disk)   │
         │                             │
         │  → cicd.toml               │
         │     [workspace]             │
         │     [state]                 │
         │     [autonomic] + policies  │
         │     [[events]]              │
         └────────────┬───────────────┘
                      │
         ┌────────────▼──────────────────────┐
         │  Evidence Gate (if wasm4pm mode)   │
         │                                    │
         │  emit_xes(&events)?                │
         │  → target/cargo-cicd/evidence/     │
         │                                    │
         │  [wasm4pm feature only]            │
         │  Wasm4pmShell::audit(&xes_path)   │
         │  → { Pass | Warn | Fail | Partial}│
         └────────────┬─────────────────────┘
                      │
         ┌────────────▼──────────────┐
         │   User Output             │
         │                           │
         │ stdout: Command results   │
         │ stderr: Warnings/errors   │
         │ exit code: 0 or >0        │
         └───────────────────────────┘
```

---

## Design Patterns & Principles

### 1. Immutability-First State

EngineState is intentionally **read-only** after construction. This enforces:

- **Referential transparency**: Same state input → same output
- **Parallelizability**: Multiple verbs can inspect the same state concurrently
- **Testability**: No setup-teardown; state is a value

### 2. Adapter Pattern for External Integration

Each adapter is a pure translation layer with **zero side effects**. This enables:

- **Swappable mocks**: Tests can replace adapters with deterministic implementations
- **Isolation**: External tool failures are caught early and don't corrupt state
- **Composition**: Adapters can be queried independently or combined

### 3. Noun-Verb Grammar for Clarity

The grammar models **domain concepts** (nouns: status, target, test) and **actions** (verbs: show, prune, changed). This mirrors how users think about CI/CD workflows.

### 4. Feature Gates for Lean Release

Default feature set is empty. Features are opt-in, enabling:

- **Minimal binary size**: Release builds omit ~500KB of policy/evidence code
- **Clear capability contracts**: Feature documentation is explicit in Cargo.toml
- **Safety**: wasm4pm integration is feature-gated so it can't interfere with base commands

### 5. Evidence Gate for External Adjudication

The evidence gate ensures **no self-judging**. All process verdicts come from wasm4pm, enabling:

- **Auditability**: Third-party oracle independent of build system
- **Replaceability**: Evidence format (XES) is tool-agnostic
- **Certification**: Releases are certified by external oracle, not internal tests

---

## Extension Checklist

To add a new major feature:

- [ ] Define a new state dimension in `src/engine/your_state.rs`
- [ ] Create adapters in `src/adapters/your_adapter.rs` to populate it
- [ ] Add a noun module `src/nouns/your_noun.rs` with verbs
- [ ] Implement `NounCommand` and `VerbCommand` traits
- [ ] Register the noun in `src/main.rs`
- [ ] Add an ontology entry in `ontology/cargo-cicd.ttl`
- [ ] Run `ggen` to generate docs and tests
- [ ] Add integration tests in `tests/`
- [ ] If destructive, gate behind `--confirm` flag
- [ ] If gated feature, add feature to Cargo.toml and wrap with `#[cfg(...)]`
- [ ] Update CLAUDE.md with new commands in Build & Test section

---

## Summary Table

| Component | Location | Purpose | Extension |
|-----------|----------|---------|-----------|
| **EngineState** | src/engine/ | Aggregate root; all dimensions | Add state dimension + adapters |
| **Adapters** | src/adapters/ | External tool integration | Implement for new source |
| **Nouns** | src/nouns/ | CLI grammar top-level | Implement NounCommand |
| **Verbs** | src/nouns/{noun}.rs | CLI grammar actions | Implement VerbCommand |
| **Policies** | src/policies/ | Autonomic recommendations | Implement CicdPolicy |
| **Evidence** | src/evidence.rs | Process event capture | Define new event kinds |
| **Integration** | src/integrations/ | External oracle (wasm4pm) | Implement shell-out adapters |
| **cicd.toml** | src/cicd_toml.rs | State persistence | Add schema sections |
| **Ontology** | ontology/ | Semantic source of truth | Add RDF triples |
| **ggen** | ggen.toml | Code generation pipeline | Add query + template |

