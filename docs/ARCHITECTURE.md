# cargo-cicd Architecture Deep Dive

**Version**: 26.6.2  
**Date**: June 2026  
**Audience**: Contributors, maintainers, and advanced users seeking deep understanding of cargo-cicd's design.

See also: [SOLUTION_ARCHITECTURE.md](SOLUTION_ARCHITECTURE.md) for the canonical law-based architecture.

---

## Table of Contents

1. [EngineState Design](#enginestate-design)
2. [Adapter Pattern](#adapter-pattern)
3. [Noun-Verb Grammar](#noun-verb-grammar)
4. [ggen Ontology Pipeline](#ggen-ontology-pipeline)
5. [Feature Flag Strategy](#feature-flag-strategy)
6. [Policy System](#policy-system)
7. [cicd.toml Semantics](#cicdtoml-semantics)
8. [Evidence Gate Architecture](#evidence-gate-architecture)
9. [Extension and Customization](#extension-and-customization)

---

## EngineState Design

### Rationale: Aggregate Root Model

`EngineState` is the **aggregate root** of the cargo-cicd system—a single struct containing all runtime dimensions. This design choice enforces several critical properties:

1. **Single Source of Truth**: Nouns read from one `EngineState` instance, ensuring consistent snapshots across a single invocation.
2. **Testability**: Test code can construct and assert on a complete `EngineState` without mocking external systems.
3. **Serialization**: All state is serializable (via serde), enabling snapshot-based testing and audit trails.
4. **Clear Invariants**: The aggregate root makes invariants explicit (e.g., "if `git_phase.phase_closed`, then `git_phase.dirty_files.is_empty()`").

### State Dimensions

Located in `src/engine/`, each dimension captures one domain:

```rust
pub struct EngineState {
    pub workspace: WorkspaceState,       // workspace members, toolchain, edition
    pub toolchain: ToolchainState,       // active rustup, pinned rust-toolchain.toml
    pub target: TargetState,             // target/ size, verdict, prune history
    pub changed_files: ChangedFileState, // git diff output, file paths
    pub test_plan: TestPlanState,        // selected tests, conservative mode flags
    pub trybuild: TrybuildState,         // fixture changes, snapshot mode
    pub git_phase: GitPhaseState,        // branch, dirty/staged/untracked files, closure state
    pub process_events: ProcessEventState, // all events emitted this session
    pub artifacts: ArtifactState,        // build outputs, binary paths, metadata
    pub policies: PolicyState,           // policy verdicts and recommendations
    pub projection: ProjectionProfile,   // feature flag profile, public/private filtering
}
```

Each state dimension is serializable and has clear ownership:

| Dimension | Populated By | Invariants |
|-----------|------|-----------|
| `workspace` | `CargoMetadataAdapter` | name, toolchain, members must be valid |
| `toolchain` | `ToolchainDetector` | active must match rustup; pinned optional |
| `target` | `TargetScannerAdapter` | verdict derived from size vs. max_gb |
| `changed_files` | `ChangedFileDetector` | all paths relative to workspace root |
| `test_plan` | Test analyzer | conservative_mode flags when needed |
| `trybuild` | `TrybuildDetector` | fixture paths only; no build artifacts |
| `git_phase` | `GitStatusAdapter` | phase_closed ⟹ dirty_files.is_empty() |
| `process_events` | Nouns + Evidence module | all events timestamped and ordered |
| `artifacts` | Build system integration | paths must exist on completion |
| `policies` | `PolicyEngine` (when autonomic enabled) | verdicts from policy evaluation |
| `projection` | Projection profile at construction | immutable public/private filter |

### Invariants and Consistency

The following invariants are enforced by adapters and nouns:

| Invariant | Enforcement | Validation |
|-----------|------------|-----------|
| `git_phase.phase_closed ⟹ dirty_files.is_empty()` | Adapter (GitStatusAdapter) | `tests/git_phase_closure.rs` |
| `target.total_size_bytes < target.max_size_bytes` or verdict is Warn/Fail | TargetScannerAdapter | `tests/invariants.rs` (INVARIANT 4) |
| `projection.version == "26.6.2"` always | On EngineState::new() | `tests/feature_projection.rs` |
| All events are timestamped | ProcessEvent constructor | `src/evidence.rs` (E2) |
| No forbidden terms in public output | CLI boundary | `tests/invariants.rs` (INVARIANT 1) |

### Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        EngineState (Aggregate Root)                 │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ workspace: WorkspaceState                                    │  │
│  │   ├─ name, root_path, members, toolchain, rust_edition      │  │
│  │   └─ Populated by: CargoMetadataAdapter                     │  │
│  │                                                              │  │
│  │ toolchain: ToolchainState                                   │  │
│  │   ├─ active_toolchain, pinned_toolchain, mismatch_detected  │  │
│  │   └─ Populated by: ToolchainDetector                        │  │
│  │                                                              │  │
│  │ target: TargetState                                         │  │
│  │   ├─ path, total_size_bytes, max_size_bytes, verdict        │  │
│  │   └─ Populated by: TargetScannerAdapter                     │  │
│  │                                                              │  │
│  │ git_phase: GitPhaseState                                    │  │
│  │   ├─ branch, dirty_files, staged_files, phase_closed        │  │
│  │   └─ Populated by: GitStatusAdapter                         │  │
│  │                                                              │  │
│  │ process_events: ProcessEventState                           │  │
│  │   ├─ events[*]: kind, verdict, timestamp, details           │  │
│  │   └─ Populated by: StatusNoun, Evidence module              │  │
│  │                                                              │  │
│  │ policies: PolicyState (when autonomic enabled)              │  │
│  │   ├─ policies[*]: name, enabled, mode, verdict, rec.        │  │
│  │   └─ Populated by: PolicyEngine                             │  │
│  │                                                              │  │
│  │ [... more dimensions ...]                                   │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘

Each noun reads from EngineState; adapters populate it.
Nouns NEVER modify EngineState directly (nouns are read-only consumers).
```

---

## Adapter Pattern

### Design Rationale

Adapters solve the **impedance mismatch** between external representations and the internal `EngineState` model:

- **Git**: Outputs `git status --porcelain` (text with status codes)
- **Cargo**: Emits JSON via `cargo metadata` (nested structures)
- **Filesystem**: Directories and file metadata (WalkDir traversal)
- **Rustup**: Writes `~/.rustup/settings.toml`, environment variables

Adapters **translate** these external inputs into flat, serializable, testable state dimensions.

### Key Properties

1. **No Business Logic**: Adapters are dumb translators. A `GitStatusAdapter` queries git and populates `GitPhaseState`; it never decides *what to do* with dirty files.
2. **Single Responsibility**: Each adapter owns one external source.
3. **Testability**: Adapters are mocked in tests; `EngineState` can be constructed with fake data.
4. **Composability**: Multiple adapters run independently and feed the same `EngineState`.

### Adapter Registry

Located in `src/adapters/`:

| Adapter | External Source | Output State | Key Methods |
|---------|-----------------|--------------|------------|
| `GitStatusAdapter` | `git status`, `git rev-parse` | `GitPhaseState` | `query()`, `is_dirty()` |
| `TargetScannerAdapter` | `WalkDir` on `target/` | `TargetState` | `total_size_bytes()`, `total_size_gb()`, `verdict()` |
| `ToolchainDetector` | `rustup show`, `rust-toolchain.toml` | `ToolchainState` | `active_toolchain()`, `pinned_toolchain()` |
| `CargoMetadataAdapter` | `cargo metadata --format-version 1` | `WorkspaceState` | `detect_workspace()`, `detect_members()` |
| `ChangedFileDetector` | `git diff --name-only` | `ChangedFileState` | `changed_rs_files()`, `is_test_file()`, `is_trybuild_fixture()` |
| `TrybuildDetector` | Filesystem scan + `git diff` | `TrybuildState` | `detect_changed_fixtures()` |
| `CicdTomlWriter` | Serialization target | `cicd.toml` on disk | `write_current_state()`, `write_cicd_toml()` |

---

## Noun-Verb Grammar

### Design Rationale

cargo-cicd uses **clap-noun-verb**, a custom DSL built on clap that models CLI commands as *nouns* (subjects) with *verbs* (actions):

```
cargo cicd <noun> [verb] [options]
```

Examples:
```
cargo cicd status                    → status show (default verb)
cargo cicd status audit              → status audit (explicit verb)
cargo cicd target show               → target show
cargo cicd target prune --confirm    → target prune (with flag)
cargo cicd test changed              → test changed
```

### Benefits

1. **Discoverability**: `cargo cicd help` lists nouns; `cargo cicd status help` lists verbs within status. No deep command hierarchy.
2. **Consistency**: All nouns follow the same pattern (name, about, verbs).
3. **Natural Language**: CLI reads like English: "status show", "target prune", "git close".
4. **Default Verb Injection**: Bare nouns work without explicit verb (simplifies UX).

### Nouns in cargo-cicd

Located in `src/nouns/`:

| Noun | Verbs | Purpose |
|------|-------|---------|
| `status` | show, audit | Display/verify workspace status |
| `target` | show, prune | Manage target directory |
| `test` | changed | Run only changed tests |
| `trybuild` | changed | Run only changed trybuild fixtures |
| `git` | status, close | Git state and phase closure |
| `publish` | run | Publish crate and update cicd.toml |
| `workspace` | doctor | Workspace health diagnostics |
| `pipeline` | run | Execute full pipeline (all noun verbs) |
| `evidence` | doctor, audit | Evidence-gate analysis (wasm4pm oracle) |
| `lsp` | server | LSP server for IDE integration |

### Default Verb Injection

In `src/main.rs`:

```rust
fn inject_default_verbs(mut args: Vec<String>) -> Vec<String> {
    let noun = args.get(1).map(String::as_str).unwrap_or("");
    let has_verb = args.get(2).map(|v| !v.starts_with('-')).unwrap_or(false);
    
    if !has_verb {
        let default_verb = match noun {
            "status" => Some("show"),
            "publish" => Some("run"),
            "workspace" => Some("doctor"),
            "evidence" => Some("doctor"),
            _ => None,
        };
        if let Some(verb) = default_verb {
            args.insert(2, verb.to_string());
        }
    }
    args
}
```

### How to Add a New Noun

1. Create `src/nouns/mynoun.rs` implementing `NounCommand`
2. Add to `src/nouns/mod.rs`
3. Register in `src/main.rs` with `.noun(nouns::mynoun::MyNoun::new())`
4. Write tests in `tests/cli/` or update `tests/cli.rs`

---

## ggen Ontology Pipeline

### Architecture Overview

`ggen` is a **code generation framework** that uses RDF/OWL ontologies and Tera templates to generate and maintain consistency across the codebase:

```
ontology/public/cargo-cicd-capabilities.ttl
    ↓ (SPARQL queries)
Selected capabilities
    ↓ (Tera templates)
Generated artifacts (README.md, docs/reference/*, etc.)
```

### Configuration

Located at `ggen.toml`:

```toml
[project]
name = "cargo-cicd"
version = "26.6.2"

[ontology]
source = "ontology/public/cargo-cicd-capabilities.ttl"
base_iri = "https://cargo-cicd.rs/ontology/"

[generation]
output_dir = "."

[[generation.rules]]
name = "readme"
query = { inline = "SELECT ... FROM capabilities ..." }
template = { file = "templates/README.md.tera" }
output_file = "README.md"
mode = "Overwrite"
```

### When to Use ggen

1. **New noun or verb added**: Update TTL, run `ggen`
2. **Description changed**: Edit TTL, run `ggen`
3. **Documentation templates updated**: Run `ggen` to re-render all docs

### Running ggen

```bash
ggen
# Reads ggen.toml, loads ontology, runs SPARQL queries, renders templates, writes outputs.
```

---

## Feature Flag Strategy

### Overview

cargo-cicd uses feature flags to gate advanced functionality, control testing scope, and ensure internal systems don't leak into public binaries.

### Feature Flags

Defined in `Cargo.toml`:

```toml
[features]
default = []
process-data = []
autonomic = ["process-data"]      # implies process-data
contrib = ["process-data"]        # implies process-data
wasm4pm = ["process-data"]        # implies process-data
```

### Feature Dependency Graph

```
default (no features)
├─ No process-data internals exposed
├─ CLI is public-facing only
├─ Autonomic policies disabled
└─ wasm4pm integration disabled

process-data (opt-in)
├─ Enables internal EngineState, PolicyState, ProcessEventState
├─ Enables evidence emission (XES/JSONL)
└─ Used by autonomic and wasm4pm

autonomic (implies process-data)
├─ Enables PolicyState and policy evaluation
├─ PolicyMode::Suggest (recommend, never enforce)
└─ Default mode in v26.6.2

wasm4pm (implies process-data)
├─ Enables XES evidence format
├─ Enables integration with wpm oracle
└─ Release blocker if wpm unavailable
```

### Interaction Matrix

| Scenario | Feature | Behavior |
|----------|---------|----------|
| `cargo build` (no flags) | — | CLI works, no internal state exposed |
| `cargo build --features autonomic` | autonomic | Policies run in suggest mode |
| `cargo build --features wasm4pm` | wasm4pm | XES emission enabled |

### Testing Feature Interactions

- **Smoke Tests**: Run without feature flags. Cover public CLI surface only.
- **Policy Tests**: Run with `--features autonomic`. Cover policy logic.
- **Evidence-Gate Tests**: Run with `--features wasm4pm`. Emit XES, invoke oracle.
- **Feature Projection Tests**: Always run. Verify feature gates are correct.

---

## Policy System

### Design Rationale

Policies are **autonomous recommendations** that evaluate workspace state and suggest corrective actions:

1. **Non-Invasive**: Policies run in `Suggest` mode by default—they recommend but never enforce.
2. **Stateless**: Each policy evaluation reads from `PolicyState` and `EngineState`.
3. **Serializable**: Policy results are emitted as events and can be audited.
4. **Extensible**: New policies are added by implementing the `CicdPolicy` trait.

### Policy Trait

Located in `src/policies/mod.rs`:

```rust
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn mode(&self) -> PolicyMode;
    fn evaluate(&self) -> PolicyResult;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyMode {
    Suggest,  // Default; recommend only
    Apply,    // Reserved; not enabled in v26.6.2
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyVerdict {
    Pass,   // No action needed
    Warn,   // Approaching threshold
    Alert,  // Action recommended
}
```

### Built-in Policies

Located in `src/policies/`:

1. **TargetPressurePolicy**: target/ size exceeds threshold → suggest prune
2. **ToolchainMismatchPolicy**: active toolchain ≠ pinned → suggest switch
3. **GitPhaseDirtyPolicy**: working tree dirty → suggest commit/stash
4. **TrybuildChangedPolicy**: fixtures changed → suggest snapshot

### Suggest Mode vs. Apply Mode

**Suggest Mode** (default, v26.6.2):
- Policy evaluation runs; verdicts and recommendations are emitted.
- No destructive actions are taken.

**Apply Mode** (reserved, not implemented):
- Would automatically execute recommended actions.
- Requires explicit user opt-in.

---

## cicd.toml Semantics

### Purpose

`cicd.toml` is the **carrier file**—a persistent store of workspace CI/CD state, configuration, and event history. It lives at the workspace root and is committed to version control.

### Schema Overview

```toml
[workspace]
name = "cargo-cicd"
toolchain = "stable"
target_dir = "target"

[state]
dirty = false
target_size_gb = 5.2
changed_files = 3
changed_tests = 1

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
mode = "suggest"

[[events]]
kind = "status"
verdict = "pass"
timestamp = "2026-06-14T13:45:07.123Z"
details = "workspace clean"
```

### Sections

| Section | Purpose | Populated By |
|---------|---------|------|
| `[workspace]` | Workspace metadata | `CargoMetadataAdapter` |
| `[state]` | Snapshot of state at last run | Adapters |
| `[target]` | Target directory config | User config |
| `[test.changed]` | Test runner config | User config |
| `[trybuild.changed]` | Trybuild config | User config |
| `[git.phase]` | Git phase closure config | User config |
| `[autonomic]` | Policy engine config | User config |
| `[[events]]` | Audit trail of all operations | Nouns + Evidence module |

### State Persistence Model

**Workflow**:
1. Command invocation (e.g., `cargo cicd status`)
2. Adapters query external systems and populate `EngineState`
3. Noun reads `EngineState` and emits output + event
4. `CicdTomlWriter` serializes current state to cicd.toml

### Event Recording

Events are the **audit trail**. Every significant operation emits an event:

```toml
[[events]]
kind = "status"
verdict = "pass"
timestamp = "2026-06-14T10:00:00.000Z"
details = "workspace clean"
```

---

## Evidence Gate Architecture

### Overview

The **evidence gate** is the release decision point. cargo-cicd emits process evidence (in XES format); an external oracle (`wpm` from wasm4pm) adjudicates conformance.

### Key Invariants

| Invariant | Meaning |
|-----------|---------|
| **E1** | cargo-cicd NEVER adjudicates its own conformance. All verdicts come from wpm. |
| **E2** | Evidence is emitted before adjudication. XES must exist on disk before audit is called. |
| **E3** | If wpm is unavailable and expected verdict is not Blocked, tests panic. |
| **E4** | Tests assert only wasm4pm verdict, never internal cargo-cicd state. |

### ProcessEvent Type

Located in `src/evidence.rs`:

```rust
pub struct ProcessEvent {
    pub event_id: String,                  // Unique ID
    pub timestamp_iso: String,             // ISO-8601
    pub case_id: Option<String>,           // Session grouping
    pub lifecycle_transition: String,      // "start" | "complete"
    pub command: String,                   // "status:show", "target:prune", etc.
    pub verdict_claimed: String,           // "PASS", "WARN", "FAIL"
    pub verdict_adjudicated: Option<String>, // "Accept", "Refuse" (from wpm)
    pub trace_class: String,               // "pipeline_run" | "live_workspace"
}
```

### XES Format (Evidence Emission)

XES (XML Event Stream) is a standard process-mining format:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <event>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="start"/>
      <date key="time:timestamp" value="2026-06-14T13:45:07.123Z"/>
    </event>
    <event>
      <string key="concept:name" value="status:show"/>
      <string key="lifecycle:transition" value="complete"/>
      <int key="duration:ms" value="927"/>
      <string key="verdict:claimed" value="PASS"/>
    </event>
  </trace>
</log>
```

---

## Extension and Customization

### Adding a New Noun

1. Create `src/nouns/mynoun.rs` implementing `NounCommand`
2. Implement `VerbCommand` for each verb
3. Add to `src/nouns/mod.rs`
4. Register in `src/main.rs`
5. Write tests in `tests/cli/`

### Adding a New Adapter

1. Create `src/adapters/myadapter.rs`
2. Implement a public struct with query methods
3. Add to `src/adapters/mod.rs`
4. Use in a noun's verb

### Adding a New Policy

1. Create `src/policies/mynewpolicy.rs` implementing `CicdPolicy`
2. Add to `src/policies/mod.rs`
3. Register in `PolicyEngine::new()`
4. Write tests with `#[cfg(feature = "autonomic")]`

### Regenerating Docs with ggen

1. Update `ontology/public/cargo-cicd-capabilities.ttl` with new capability
2. Add generation rule to `ggen.toml`
3. Run `ggen`
4. Commit updated docs

---

## Summary: Key Design Principles

| Principle | Implementation | Benefit |
|-----------|---|---|
| **Aggregate Root** | `EngineState` holds all runtime state | Single source of truth; consistent snapshots |
| **Adapter Pattern** | Each external source has one adapter | Testability; clear boundaries; easy mocking |
| **Noun-Verb Grammar** | CLI modeled as subjects + actions | Discoverability; consistency; natural UX |
| **ggen Pipeline** | Ontology + SPARQL + Tera → docs | Single source of truth for public surface |
| **Feature Flags** | process-data, autonomic, wasm4pm layer functionality | Control internal exposure; test scope management |
| **Suggest-Only Policies** | Policies recommend; never enforce | Safe, non-invasive guidance |
| **cicd.toml Carrier** | Persistent config + event log | Audit trail; shared team state; evidence foundation |
| **Evidence Gate** | cargo-cicd emits; wpm oracle adjudicates | No self-judgment; external trust anchor |

---

## References

- **Main Source**: `src/`
- **Test Hierarchy**: `tests/`
- **Configuration**: `Cargo.toml`, `ggen.toml`
- **Ontology**: `ontology/public/cargo-cicd-capabilities.ttl`
- **Templates**: `templates/`
- **SOLUTION_ARCHITECTURE.md**: Law-based architecture reference

---

**Document Version**: 26.6.2  
**Last Updated**: June 2026  
**Maintainers**: cargo-cicd contributors
