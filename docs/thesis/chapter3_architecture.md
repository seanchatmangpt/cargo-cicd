# Chapter 3: System Architecture and Design

## 3.1 Introduction

This chapter presents a rigorous treatment of the architectural decisions and structural organisation of cargo-cicd, a Level 5 process-data engine for Rust workspace CI/CD automation. The system occupies a distinctive position in the build-tooling landscape: rather than being a configuration-driven pipeline runner in the tradition of Makefile or shell-based CI systems, cargo-cicd is a grammar-manufacturing engine whose public command surface is derived from a formal ontology and whose runtime behaviour is recorded as structured process evidence for external adjudication.

The chapter is organised into seven primary sections. Section 3.2 examines the ontology-driven grammar manufacturing pipeline, tracing the transformation from RDF/Turtle concept definitions through SPARQL inference to the compiled Rust binary. Section 3.3 characterises the noun-verb CLI grammar in terms of its structural invariants and dispatch mechanics. Section 3.4 presents the `EngineState` aggregate root, its eleven state dimensions, and the sequential initialisation protocol. Section 3.5 analyses the adapter layer through the lens of the classical Adapter pattern, focussing on the statelessness, silent-failure, and isolation guarantees that make it compositionally safe. Section 3.6 describes `cicd.toml` as a persistent state carrier with a well-defined schema and write semantics. Section 3.7 examines the feature flag architecture, including the lattice structure of feature dependencies. Section 3.8 addresses the cross-cutting design principles — partial data preference, opt-in engine activation, and public boundary enforcement — that give the system its resilience and security properties.

---

## 3.2 The Grammar Manufacturing Pipeline

### 3.2.1 Rationale for Manufactured Grammar

A central design choice in cargo-cicd is that its command-line grammar is **manufactured, not handwritten**. This decision reflects a fundamental principle drawn from model-driven engineering: when an artefact can be mechanically derived from a formal specification, that derivation should be automated, because any manual transcription introduces the possibility of drift between the specification and the implementation.

In practice, this means that adding a new command to cargo-cicd requires, as the primary act, editing the ontology — a formal RDF/Turtle knowledge graph — and then invoking the `ggen` code-generation tool. The Rust handler code, test scaffolding, README reference sections, and command documentation are all derived outputs. This inversion of the usual development flow (write code, update docs) into (update specification, derive code) has significant consequences for consistency and auditability.

### 3.2.2 Ontology Layer: RDF/Turtle Capability Taxonomy

The source of truth for cargo-cicd's command surface is located at `ontology/public/cargo-cicd-capabilities.ttl`. This file is an OWL ontology expressed in Turtle (Terse RDF Triple Language) notation. Each public sub-command is modelled as a `skos:Concept` grounded in PROV-O, DCTERMS, and SKOS vocabularies.

A representative concept definition for the `status show` command reads:

```turtle
@prefix skos:    <http://www.w3.org/2004/02/skos/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix cc:      <https://cargo-cicd.rs/ontology/> .

cc:StatusShow
    a skos:Concept , prov:SoftwareAgent ;
    skos:inScheme cc:CapabilityScheme ;
    skos:prefLabel "status show"@en ;
    dcterms:description
        "Displays the current workspace status: dirty files, pending tests,
         last-known trybuild result, and publish readiness. Read-only;
         emits a StatusShowEvent."@en ;
    cc:cliCommand "cargo cicd status show" ;
    cc:noun "status" ;
    cc:verb "show" ;
    cc:defaultVerb "show" ;
    cc:requiresFeature cc:defaultFeature ;
    cc:emitsEvent cc:StatusShowEvent ;
    cc:publicBoundaryClean true .
```

The ontology thus encodes, for each command: the noun, the verb, the CLI invocation string, the natural language description, the required feature flag, the emitted event type, and an explicit assertion (`cc:publicBoundaryClean`) that the concept's public surface does not expose internal vocabulary. Feature flag concepts are modelled using `skos:broader` relationships that encode the implication lattice described in Section 3.7.

### 3.2.3 The ggen.toml Configuration and SPARQL Inference

The `ggen` code-generation tool is driven by `ggen.toml`, which declares the ontology source, namespace prefix bindings, inference rules, and generation rules. The core inference step is a SPARQL 1.1 CONSTRUCT query that materialises all `cc:Capability` instances from `skos:Concept` triples:

```sparql
PREFIX cc:      <https://cargo-cicd.rs/ontology/>
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX skos:    <http://www.w3.org/2004/02/skos/core#>
CONSTRUCT {
  ?cap a cc:Capability ;
       cc:cliCommand ?cli_command ;
       cc:noun ?noun ;
       cc:verb ?verb ;
       dcterms:description ?description .
}
WHERE {
  ?cap a skos:Concept ;
       cc:cliCommand ?cli_command ;
       cc:noun ?noun ;
       cc:verb ?verb ;
       dcterms:description ?description .
}
ORDER BY ?noun ?verb
```

This CONSTRUCT query projects the ontology's capability surface into a structured result set, ordered by noun and then verb. The projection result drives downstream generation rules, each of which pairs a SPARQL SELECT query with a Tera template and an output destination. The generation rules declared in `ggen.toml` produce: `README.md` (full command reference), per-noun reference documentation in `docs/reference/commands/`, and test scaffolding in `tests/cli/`.

### 3.2.4 Pipeline Topology

The full manufacturing pipeline can be described as a directed acyclic graph of transformation steps:

```
ontology/public/cargo-cicd-capabilities.ttl
        │
        │  [ggen: SPARQL CONSTRUCT — capability projection]
        ▼
    cc:Capability instances (in-memory RDF graph)
        │
        ├── [ggen: SELECT + README.md.tera] ──────────────► README.md
        │
        ├── [ggen: SELECT + reference-command.md.tera] ───► docs/reference/commands/*.md
        │
        └── [ggen: SELECT + test scaffolding rules] ──────► tests/cli/*.rs (scaffolding)
        
Developer action:
        └── implement NounCommand + VerbCommand traits ────► src/nouns/*.rs
```

The critical property of this pipeline is that it is **idempotent**: running `ggen` multiple times on an unchanged ontology produces identical outputs. This idempotency is verified by the `tests/ggen_customization_guard.rs` test suite, which asserts that the generated artefacts match the ontology state. Any divergence — such as a hand-edited `README.md` that disagrees with the ontology — is detected at test time.

---

## 3.3 The Noun-Verb CLI Grammar

### 3.3.1 Structural Description

cargo-cicd uses the `clap-noun-verb` published crate (version 26.6.2) as its parsing substrate. This crate implements a two-level command hierarchy: the first token after the binary name is a **noun** (a topic category), and the second token is a **verb** (an action within that category). This grammar is isomorphic to the subject-predicate structure of the ontology, where `cc:noun` maps to the first level and `cc:verb` maps to the second.

Formally, let N be the set of nouns and V_n be the set of verbs available under noun n. The command grammar G is:

```
G ::= binary noun verb [flags]
noun ∈ {status, target, test, trybuild, git, publish, workspace, evidence, pipeline, lsp}
verb ∈ V_noun
```

As of version 26.6.2, the grammar contains ten nouns with the following verb sets:

| Noun | Verbs | Verb Category |
|---|---|---|
| `status` | `show`, `audit` | Read-only; Read-adjudicate |
| `target` | `show`, `prune` | Read-only; Execution |
| `test` | `changed` | Execution |
| `trybuild` | `changed`, `full` | Execution |
| `git` | `status`, `close`, `phase` | Read-only; Execution |
| `publish` | `run` | Execution |
| `workspace` | `doctor`, `validate`, `sync`, `list` | Read-only; Execution |
| `evidence` | `doctor`, `audit` | Read-only; Adjudication |
| `pipeline` | `run` | Execution |
| `lsp` | `explain` | Read-only |

Verb categories determine whether user confirmation (`--confirm`) is required, whether evidence is emitted, and whether the wasm4pm oracle may be invoked. Read-only verbs — `show`, `status`, `explain`, `doctor` — never modify workspace state. Execution verbs — `run`, `close`, `prune` — may modify state and must carry confirmation flags for destructive variants.

### 3.3.2 Trait-Based Polymorphism

Each noun implements the `NounCommand` trait, and each verb within a noun implements `VerbCommand`. This trait-based dispatch is the mechanism by which the `clap-noun-verb` parsing layer routes parsed arguments to the appropriate handler.

The `StatusNoun` implementation is representative:

```rust
impl NounCommand for StatusNoun {
    fn name(&self) -> &'static str { "status" }
    fn about(&self) -> &'static str { "Show workspace CI/CD status" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(StatusShowVerb), Box::new(StatusAuditVerb)]
    }
}
```

The return type `Vec<Box<dyn VerbCommand>>` is significant: each verb is heap-allocated behind a trait object, enabling the noun registry in `src/main.rs` to hold a homogeneous collection of heterogeneous verb types. This is the Gang-of-Four *Command* pattern applied at the verb level.

### 3.3.3 Default Verb Injection

A usability concern in two-level command grammars is that users must always supply both levels even for common operations. cargo-cicd addresses this through a **default verb injection** mechanism in `src/main.rs`. When the user supplies only a noun and no verb (or supplies flags directly to the noun), the `inject_default_verbs` function inserts the default verb before dispatch:

```rust
fn inject_default_verbs(mut args: Vec<String>) -> Vec<String> {
    let noun = args.get(1).map(String::as_str).unwrap_or("");
    let has_verb = args.get(2).map(|v| !v.starts_with('-')).unwrap_or(false);
    if !has_verb {
        let default_verb = match noun {
            "status"    => Some("show"),
            "publish"   => Some("run"),
            "workspace" => Some("doctor"),
            "evidence"  => Some("doctor"),
            _           => None,
        };
        if let Some(verb) = default_verb {
            args.insert(2, verb.to_string());
        }
    }
    args
}
```

This preserves the full noun-verb grammar in the ontology while exposing a simpler surface to the user. The mapping from bare nouns to default verbs is itself derived from the ontology (via the `cc:defaultVerb` predicate) and could, in principle, be generated automatically.

### 3.3.4 The Cargo External Subcommand Protocol

cargo-cicd follows the Cargo external subcommand convention: when invoked as `cargo cicd <noun> <verb>`, Cargo forwards the invocation as `cargo-cicd cicd <noun> <verb>`. The binary must strip the redundant `cicd` argument before its own parsing. This is handled in `main()`:

```rust
if raw.get(1).map(String::as_str) == Some("cicd") {
    let bin = &raw[0];
    let rest = &raw[2..];
    let status = std::process::Command::new(bin).args(rest).status()?;
    std::process::exit(status.code().unwrap_or(1));
}
```

This re-exec pattern (spawn self without the `cicd` argument) ensures that the main parsing path sees clean `argv` regardless of whether the user invoked `cargo cicd status` or `cargo-cicd status`.

---

## 3.4 The EngineState Aggregate Root

### 3.4.1 Domain-Driven Design: Aggregate Root Pattern

In Domain-Driven Design (Evans 2003), an *aggregate root* is the single entry point through which all interactions with a cluster of related objects must flow. It enforces invariants across the cluster and controls the lifecycle of subordinate entities. `EngineState` fulfils this role in cargo-cicd: it is the single struct that holds all runtime knowledge about the workspace, and all business logic in noun handlers operates on a fully populated `EngineState` rather than calling adapters directly.

The complete type definition, extracted from `src/engine/mod.rs`, is:

```rust
#[derive(Debug, Default)]
pub struct EngineState {
    pub workspace:      WorkspaceState,
    pub toolchain:      ToolchainState,
    pub target:         TargetState,
    pub changed_files:  ChangedFileState,
    pub test_plan:      TestPlanState,
    pub trybuild:       TrybuildState,
    pub git_phase:      GitPhaseState,
    pub process_events: ProcessEventState,
    pub artifacts:      ArtifactState,
    pub policies:       PolicyState,
    pub projection:     ProjectionProfile,
}
```

The `Default` derivation is load-bearing: it establishes the *partial data* baseline. If every adapter fails, `EngineState::default()` yields a struct of safe zero-values — empty strings, empty vectors, zero integers — that noun handlers can interrogate without panicking.

### 3.4.2 State Dimensions

Each field in `EngineState` corresponds to one *state dimension* — a logically cohesive slice of workspace knowledge. The eleven dimensions are:

**WorkspaceState** captures the static identity of the workspace: the project name extracted from `Cargo.toml`, the filesystem root path, the list of member crates, the active Rust toolchain channel, and the declared Rust edition. These values change rarely and are cheap to populate.

```rust
pub struct WorkspaceState {
    pub name:         String,
    pub root_path:    String,
    pub members:      Vec<String>,
    pub toolchain:    String,
    pub rust_edition: String,
}
```

**ToolchainState** captures the dynamic toolchain identity: the channel string (e.g. `"stable"`, `"nightly-2026-05-01"`) and the full `rustc --version` string. This dimension is populated by `ToolchainDetector` and is distinct from `WorkspaceState.toolchain` to separate the *pinned* channel from the *detected* runtime version, enabling the `toolchain_mismatch` autonomic policy to compare them.

**TargetState** holds the path to the Cargo target directory and its total size in bytes. The size measurement is the output of a recursive `walkdir` traversal performed by `TargetScannerAdapter`. This is the slowest dimension to populate (potentially 1–5 seconds on large workspaces) and is a candidate for caching in `cicd.toml`.

**ChangedFileState** describes the set of Rust source files that differ from the base reference (defaulting to `origin/main`) as reported by `git diff --name-only`. Files are partitioned into three subsets: all changed `.rs` files, those matching test file heuristics, and those matching trybuild fixture heuristics. This partitioning drives conservative test selection in the `test changed` and `trybuild changed` verbs.

```rust
pub struct ChangedFileState {
    pub base_ref:                     String,
    pub changed_rs_files:             Vec<String>,
    pub changed_test_files:           Vec<String>,
    pub changed_trybuild_fixtures:    Vec<String>,
    pub total_changed:                usize,
}
```

**TestPlanState** is a derived dimension computed from `ChangedFileState`: it records the estimated test count and a `conservative_mode` flag that is set whenever changed files are detected. Conservative mode prevents verb handlers from running full test suites when only a subset of tests are implicated.

**TrybuildState** holds the full set of trybuild fixtures discovered in `tests/ui/`, the subset of those that are changed, the snapshot mode string, and the `run_all_by_default` flag. The separation of `all_fixtures` from `changed_fixtures` is essential to the `INVARIANT_NO_FULL_TRYBUILD_BY_DEFAULT` invariant tested in `tests/invariants.rs`.

**GitPhaseState** captures the complete git status: current branch name, dirty (unstaged-modified) files, staged files, untracked files, and the ahead/behind commit counts relative to the tracking branch. It also records whether the workspace requires a clean tree before phase closure (`require_clean_tree`) and whether the current phase has been closed (`phase_closed`).

```rust
pub struct GitPhaseState {
    pub branch:             String,
    pub dirty_files:        Vec<String>,
    pub staged_files:       Vec<String>,
    pub untracked_files:    Vec<String>,
    pub ahead:              u32,
    pub behind:             u32,
    pub require_clean_tree: bool,
    pub phase_closed:       bool,
}
```

**ProcessEventState** holds the list of `ProcessEvent` records that have been read back from `cicd.toml` — the historical record of prior command executions. When the `advanced` feature is enabled, this dimension additionally carries a `ProcessTimeline` instance that records high-precision `jiff::Timestamp` values for each event, enabling latency percentile analysis.

**ArtifactState** records the path to the current `cicd.toml` file (if it exists) and the timestamp of the last publish operation. This dimension is intentionally minimal; detailed artefact metadata is carried by `cicd.toml` itself.

**PolicyState** holds a vector of `PolicyEntry` structs, each describing one autonomic policy's name, enablement status, operating mode, verdict, and recommendation. This dimension is populated either from the default policy configuration or from the full policy evaluation run when the `autonomic` feature is enabled.

**ProjectionProfile** encodes the public API surface contract for the current version. As of v26.6.2, it suppresses private fields at public level 2, ensuring that internal state dimensions are not inadvertently serialised into public-facing outputs.

```rust
pub struct ProjectionProfile {
    pub version:                 String,
    pub public_level:            u8,
    pub suppress_private_fields: bool,
}

impl ProjectionProfile {
    pub fn v26_6_2() -> Self {
        Self {
            version: "26.6.2".into(),
            public_level: 2,
            suppress_private_fields: true,
        }
    }
}
```

### 3.4.3 Initialisation Protocol: EngineState::from_workspace()

The factory method `EngineState::from_workspace()` constructs a fully populated `EngineState` by calling all adapters in a fixed sequence. The sequence is:

1. Start with `Self::default()` (all zero-values, establishing the partial-data baseline).
2. Populate `workspace.name` from `CargoMetadataAdapter::workspace_name()`.
3. Populate `workspace.root_path` by stripping the `/target` suffix from the target directory path.
4. Populate `workspace.members` from `CargoMetadataAdapter::workspace_members()`.
5. Populate `workspace.toolchain` and `workspace.rust_edition` from filesystem heuristics (`rust-toolchain.toml`).
6. Attempt `GitStatusAdapter::query()` under an `if let Ok(git)` guard; populate `git_phase.*` fields only if the query succeeds.
7. Populate `toolchain.active` and `toolchain.rust_version` from `ToolchainDetector`.
8. Populate `target.path` and `target.total_size_bytes` from `TargetScannerAdapter`.
9. Populate the full `changed_files.*` cluster from `ChangedFileDetector`, including the file classification into test and trybuild subsets.
10. Derive `test_plan.*` from `changed_files.total_changed`.
11. Populate `trybuild.*` from `TrybuildDetector` and the already-computed `changed_files.changed_trybuild_fixtures`.
12. Read `cicd.toml` from disk (if it exists) and populate `process_events.events` from the stored event records.
13. Populate `artifacts.cicd_toml_path` if `cicd.toml` exists.
14. Initialise `policies.policies` with the default autonomic policy entry.
15. Set `projection` to `ProjectionProfile::v26_6_2()`.

The critical property of this protocol is the **silent failure contract**: adapter failures are handled individually, not collectively. The `GitStatusAdapter::query()` call, which shells out to `git`, is wrapped in `if let Ok(git) = ...`. If git is not installed, not initialised, or in a corrupted state, the block is simply skipped and the git phase fields remain at their default empty values. The same pattern applies to every other fallible adapter call. This means `from_workspace()` never panics and always returns a usable `EngineState`, even in pathological environments.

This design reflects a deliberate choice to prefer **partial data over no data**: a `status show` command run in a directory that is not a git repository should still display the toolchain version and target size, rather than failing entirely. The trade-off is that noun handlers must be written to handle absent data gracefully — they should not assume, for example, that `git_phase.branch` is non-empty.

---

## 3.5 The Adapter Layer

### 3.5.1 The Adapter Pattern in cargo-cicd

The classical Gang-of-Four *Adapter* pattern converts the interface of one class into an interface that clients expect, allowing classes with incompatible interfaces to work together. In cargo-cicd, the "clients" are the `EngineState` initialisation protocol and individual noun handlers, and the "incompatible interfaces" are the heterogeneous external sources: git porcelain output, TOML manifests, filesystem directory traversals, and sub-process invocations of `rustc`.

Each adapter in `src/adapters/` is a **stateless, pure translator**: it takes no constructor arguments, holds no fields, and exposes only static or `&self` methods that translate from an external representation into a Rust value. The `CargoMetadataAdapter` is the simplest example:

```rust
pub struct CargoMetadataAdapter;

impl CargoMetadataAdapter {
    pub fn workspace_name() -> String {
        std::fs::read_to_string("Cargo.toml")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.trim().starts_with("name = "))
                    .map(|l| l.split('"').nth(1).unwrap_or("workspace").to_string())
            })
            .unwrap_or_else(|| "workspace".into())
    }
    // ...
}
```

Note the use of `ok()` to convert `Result<String, io::Error>` into `Option<String>`, followed by `unwrap_or_else` to provide a safe default. There is no error propagation: the adapter returns `"workspace"` if the file does not exist, cannot be read, or does not contain a `name` field. This is the silent-failure contract in action.

### 3.5.2 Adapter Contracts and Invariants

The four adapter contracts are:

**A1 — Statelessness.** Adapters have no instance fields. All methods are `fn()` or `&self` methods where `self` carries no data. This ensures that two invocations of the same adapter method on the same external state produce identical results (referential transparency), and that adapters can be safely called concurrently without synchronisation.

**A2 — Silent Failure.** Adapters never panic and never propagate errors to callers. When an external source is unavailable or malformed, the adapter returns the type's default value. This is enforced through disciplined use of Rust's `Option` and `Result` combinators — specifically `ok()`, `unwrap_or()`, `unwrap_or_else()`, and `unwrap_or_default()`.

**A3 — Isolation.** Adapters never call other adapters. Each adapter is responsible for exactly one external source. This prevents transitive failure propagation: if `GitStatusAdapter` fails because git is not installed, it cannot cause `TargetScannerAdapter` to fail.

**A4 — Deterministic Output Type.** Each adapter method returns a concrete Rust type (`String`, `Vec<String>`, `u64`, `Result<GitStatusResult>`) rather than a trait object or an untyped `Value`. This means the compiler verifies the type contract at every call site, and noun handlers do not need runtime type-checking.

### 3.5.3 Adapter Performance Classification

Adapters are classified by their performance characteristics, which determines how aggressively they should be cached:

*Fast adapters* (sub-millisecond) read local filesystem files without invoking external processes. `CargoMetadataAdapter` (line-by-line `Cargo.toml` scan), `ManifestParser` (TOML crate parsing), and `TrybuildDetector` (filesystem glob) fall into this category.

*Medium adapters* (1–100 ms) shell out to external processes with bounded output. `GitStatusAdapter` (`git status --porcelain`, `git rev-parse`), `ToolchainDetector` (`rustc --version`), and `ChangedFileDetector` (`git diff --name-only`) are in this class.

*Slow adapters* (100 ms to several seconds) perform unbounded work proportional to workspace size. `TargetScannerAdapter` performs a full recursive `walkdir` traversal of the target directory:

```rust
pub fn total_size_bytes(target_dir: &str) -> u64 {
    let path = Path::new(target_dir);
    if !path.exists() { return 0; }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
```

On workspaces with large target directories (100 GB or more, common with incremental compilation caches), this traversal can take several seconds. The mitigation strategy is caching the result in `cicd.toml`, invalidated on modification of `Cargo.toml`.

When the `advanced` feature is enabled, `TargetScannerAdapter` gains a `parallel_scan_if_available` method that delegates to the `ignore` + `rayon` based parallel scanner, which respects `.gitignore` patterns and exploits multi-core parallelism for substantially faster traversal.

### 3.5.4 Advanced Adapter Integrations

The `advanced` feature gate activates four additional adapter modules that extend the base adapter layer with best-of-breed crate capabilities:

`adapters::cached` wraps any adapter result with a `moka` concurrent cache (TTL-aware, bounded capacity), transforming expensive adapter invocations into cheap cache hits on subsequent calls within the same process.

`adapters::fingerprint` computes BLAKE3 cryptographic hashes over artefact byte spans, enabling content-addressed integrity verification of artefacts recorded in `cicd.toml`.

`adapters::state_snapshot` serialises and deserialises the complete `EngineState` to a compact binary format using `bitcode`, enabling inter-process state transfer and warm-up of subsequent invocations.

`adapters::governance_patterns` applies multi-pattern Aho-Corasick automaton matching over workspace paths, enabling policy-driven filtering (e.g., license/copyright detection across large source trees) at rates far exceeding naïve linear scanning.

---

## 3.6 The cicd.toml State Carrier

### 3.6.1 Motivation and Role

A stateless adapter layer provides a clean data-flow model within a single process invocation, but Rust CI tools must commonly communicate across process boundaries — between a `cargo cicd publish run` in one terminal session and a subsequent `cargo cicd evidence audit` in another. cicd.toml is the persistent artefact that bridges these invocations.

The design deliberately resists the temptation to make `cicd.toml` a general configuration file in the tradition of `.cargo/config.toml`. Instead, it is conceived as a **state carrier**: a TOML serialisation of the `EngineState` aggregate root that is written after command execution and read at the start of subsequent commands. As the project documentation states: "`cicd.toml` is a vehicle for state, not a config file."

### 3.6.2 Schema

The `cicd.toml` schema is defined in `src/cicd_toml.rs` as a Serde-serialisable Rust struct hierarchy. The top-level schema is:

```toml
[workspace]
name = "cargo-cicd"
toolchain = "stable"
target_dir = "target"

[state]
dirty = false
target_size_gb = 0.41
changed_files = 3
changed_tests = 1
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

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
```

The `[[events]]` array is the primary state carrier: each entry records the `kind` (command name), the `verdict` (outcome), an optional `details` string, and an optional ISO-8601 timestamp. These records are the human-readable analogue of the XES process events discussed in Section 3.8.

### 3.6.3 Writer Pattern

The `CicdTomlWriter` adapter encapsulates the write path. It provides a single public method:

```rust
pub fn write_current_state(path: &str) -> Result<CicdToml> {
    let cicd = CicdToml::from_current_workspace();
    cicd.write(path)?;
    Ok(cicd)
}
```

`CicdToml::from_current_workspace()` constructs a `CicdToml` from the current working environment by detecting the workspace name and toolchain from local files. The `write` method serialises to TOML using `toml::to_string_pretty` and writes atomically (via `std::fs::write`, which is an O_WRONLY | O_CREAT | O_TRUNC open followed by a write — not truly atomic, but sufficient for the single-writer assumption documented in the FAQ).

### 3.6.4 Lifecycle

The lifecycle of `cicd.toml` follows this state machine:

```
[absent] ──── first publish/workspace doctor ────► [present, initial state]
    │                                                       │
    │                                            subsequent command runs
    │                                                       │
    └──────────────────────────────────────────────────────►[present, updated state]
                                                            │
                                                    manual edit (deprecated;
                                                    overwritten on next run)
```

Read-only verbs (`show`, `doctor`, `status`, `explain`) do not write `cicd.toml`. Execution verbs (`run`, `close`) and state-updating verbs (`workspace doctor`) write it. The write policy ensures that `cicd.toml` always reflects the state *after* the most recent mutating operation, enabling downstream tools (including the wasm4pm oracle) to inspect the last-known workspace state without re-running the command.

---

## 3.7 Feature Flag Architecture

### 3.7.1 The Default-Off Engine

A fundamental principle of cargo-cicd's design is that the Level 5 engine (the full `EngineState` machinery, adapter layer, and evidence emission pipeline) is **opt-in, not opt-out**. The default binary, compiled with `cargo build`, activates no feature flags and produces a lean executable that exercises only the noun/verb dispatch layer. This keeps the binary small, the startup time fast, and the dependency surface minimal for users who only need basic workspace CI/CD helpers.

The feature flags are declared in `Cargo.toml`:

```toml
[features]
default = []
process-data = []
autonomic    = ["process-data"]
contrib      = ["process-data"]
wasm4pm      = ["process-data"]
advanced     = [
  "process-data",
  "dep:ignore", "dep:rayon", "dep:blake3",
  "dep:tracing", "dep:tracing-subscriber",
  "dep:miette", "dep:thiserror",
  "dep:moka", "dep:bitcode", "dep:petgraph",
  "dep:jiff", "dep:hdrhistogram", "dep:aho-corasick",
]
```

### 3.7.2 Feature Lattice

The feature flags form a dependency lattice. Let F = {process-data, autonomic, contrib, wasm4pm, advanced} and let → denote the implication relation "enabling this feature also enables". The lattice is:

```
process-data
    │
    ├──────────────────┬─────────────────┬──────────────────────────┐
    │                  │                 │                          │
  autonomic          contrib           wasm4pm                  advanced
```

All non-default features imply `process-data`. No non-default feature implies any other non-default feature (the lattice is a tree, not a diamond). This property ensures that feature combinations are predictable: enabling `autonomic` does not accidentally enable `wasm4pm`, and there are no hidden coupling effects between the non-default features.

The `advanced` feature is special: in addition to implying `process-data`, it directly enables thirteen optional crate dependencies via `dep:` syntax. This follows Cargo's *weak dependency* feature mechanism, ensuring that crates like `rayon` and `moka` are not compiled unless `advanced` is explicitly requested.

### 3.7.3 Conditional Compilation Protocol

Feature-gated code uses standard Rust `#[cfg(feature = "...")]` attributes. The conventions in cargo-cicd are:

1. **Module-level gates** are placed on `mod` declarations in `mod.rs` files, e.g., `#[cfg(feature = "advanced")] pub mod analyze;` in `src/nouns/mod.rs`.
2. **Function-level gates** are placed on individual functions, e.g., `#[cfg(feature = "autonomic")] pub fn eval(state: &EngineState) -> PolicyEntry { ... }`.
3. **Block-level gates** inside functions use `#[cfg(feature = "advanced")]` on blocks: `{ let _stage = PipelineStage::enter("git_status"); }`.
4. **Struct field gates** use `#[cfg(feature = "advanced")]` on individual fields, as in `ProcessEventState.timeline`.

The compiler verifies that all conditional compilation paths are type-correct. This means that a `cargo check` without feature flags detects errors in the non-gated code paths, while `cargo check --features advanced` additionally checks the gated paths. The test suite is run under multiple feature combinations to ensure correctness across the entire product surface.

### 3.7.4 Feature Semantics

**process-data** activates the Level 5 engine. When enabled, `EngineState::from_workspace()` performs all adapter calls; when disabled, noun handlers operate with minimal local data. The `ProcessEvent` type and XES emission machinery are always available (they are unconditionally compiled), but the full engine state is only populated when `process-data` is active.

**autonomic** activates the policy suggestion layer in `src/autonomic/`. When enabled, `status show` calls `policy_engine::run_suggestions(&engine)` and appends the resulting recommendation strings to its output. All policies run in `suggest` mode: they never modify workspace state, only emit human-readable recommendations. An `apply` mode exists in the `policy_engine` module for the one eligible policy (`evidence_stale`), but it is only activated when explicitly requested.

**wasm4pm** activates the `Wasm4pmShell` integration in `src/integrations/`. When enabled, evidence gate tests call the external `wpm` binary for XES adjudication. The `wasm4pm` feature is noted in `Cargo.toml` as deferred to v26.6.3 for full library coupling; in v26.6.2, the shell-out adapter (`Wasm4pmShell`) is the sole integration path.

**advanced** activates the ten best-of-breed crate integrations. Each integration is exposed through a dedicated module in `src/advanced/`: `parallel_scan`, `fingerprint`, `observability`, `diagnostics`, `cache`, `snapshot`, `dep_graph`, `timeline`, `histogram`, and `pattern`. Adapters in `src/adapters/` gain feature-gated extensions (e.g., `TargetScannerAdapter::parallel_scan_if_available`) that delegate to the corresponding `advanced` module when the feature is active.

---

## 3.8 Cross-Cutting Design Principles

### 3.8.1 Partial Data Preference

The most pervasive design principle in cargo-cicd is the preference for **partial data over process failure**. This principle manifests at every level of the system:

At the adapter level, silent failure contracts (Section 3.5.2) ensure that individual adapter failures do not propagate. At the engine level, the sequential initialisation protocol in `from_workspace()` continues to the next adapter even when a prior one has failed. At the verb handler level, noun commands check whether state dimensions are populated before using them, and gracefully omit sections of their output when data is absent.

This approach contrasts with the alternative *fail-fast* philosophy, in which any missing data causes immediate process termination with an error message. The fail-fast approach is appropriate when the user has explicitly requested an operation that requires specific data (for example, `git close` when there is no git repository). But for ambient health-checking commands like `status show`, the preference is to show as much as is available: displaying toolchain and target size even when git status is unavailable is more useful than showing nothing.

The principle is enforced at test time by `invariant_status_exits_zero` in `tests/invariants.rs`, which asserts that `cargo cicd status show` exits 0 in any environment, including environments without git.

### 3.8.2 Public Boundary Enforcement

cargo-cicd maintains a strict separation between its internal vocabulary — terms used in the manufacturing pipeline, internal subsystem identities, and engineering classifications — and its public vocabulary, which is everything that appears in `--help` text, stdout, or stderr.

The list of forbidden public terms is codified in `tests/invariants.rs` and mirrors the table in `CLAUDE.md`. As of v26.6.2, ten terms are forbidden — internal identifiers covering the manufacturing pipeline, autonomic reasoning subsystem, internal adjudication metaphor, AI classification, truth engine, capacity measurement, grammar generation code name, directive system, and jargon from the manufacturing pipeline. Each term refers to an internal concept that has a public-safe alternative: for example, the internal engine status marker's public analogue is a simple process state report.

The boundary is enforced by the `invariant_public_boundary_no_forbidden_terms_in_all_help` test, which exercises every noun's `--help` output and asserts the absence of all forbidden terms:

```rust
#[test]
fn invariant_public_boundary_no_forbidden_terms_in_all_help() {
    let forbidden = ["ALIVE", /* internal-only terms redacted */, /* ... */];
    let noun_verbs = [
        vec!["--help"], vec!["status", "--help"],
        vec!["target", "--help"], /* ... */
    ];
    for args in &noun_verbs {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap().args(args.iter()).output().unwrap();
        let text = String::from_utf8_lossy(&output.stdout).to_string()
                 + &String::from_utf8_lossy(&output.stderr);
        for term in &forbidden {
            assert!(!text.contains(term), /* ... */);
        }
    }
}
```

This test must pass before any release. It is a type of *information-hiding gate* — it mechanically enforces at the test layer what the ontology encodes at the specification layer via `cc:publicBoundaryClean true`.

### 3.8.3 Process Evidence and External Adjudication

A distinctive architectural feature of cargo-cicd is its relationship with the wasm4pm oracle. The system follows an evidence emission pattern inspired by process mining (van der Aalst 2016): every verb execution emits a `ProcessEvent` that is serialised to XES (XML Event Stream) format and accumulated in a persistent evidence log.

The central invariant (E1 in `src/evidence.rs`) is that **cargo-cicd never adjudicates its own process conformance**: it claims a verdict (`PASS`, `WARN`, `FAIL`) based on the observed state, but the binding verdict is issued by the external wasm4pm oracle after token-replay fitness analysis against the declared process model.

This separation of concerns maps to the CQRS (Command Query Responsibility Segregation) pattern at the process level: cargo-cicd is the *command* side (it executes operations and emits events), while wasm4pm is the *query* side (it reads the event log and issues conformance judgements). The XES event log is the durable boundary between the two.

The `ProcessEvent` structure carries all fields needed for both emission and adjudication:

```rust
pub struct ProcessEvent {
    pub event_id:             String,
    pub timestamp_iso:        String,
    pub case_id:              Option<String>,
    pub lifecycle_transition: String,  // "start" or "complete"
    pub workspace_id:         String,
    pub repo_path:            String,
    pub command:              String,
    pub verdict_claimed:      String,  // "PASS", "WARN", or "FAIL"
    pub duration_ms:          Option<u64>,
    pub verdict_adjudicated:  Option<String>,
    pub adjudicated_at:       Option<String>,
    pub oracle_command:       Option<String>,
    pub trace_class:          String,  // "live_workspace" or "pipeline_run"
}
```

The `verdict_adjudicated` field is populated only after the oracle has been called; until then, it is `None`. Tests assert on `verdict_adjudicated`, never on `verdict_claimed`, honouring invariant E4.

The XES emission pipeline implements three quality filters for token-replay fitness:

1. **Lifecycle filter**: only `"complete"` events are included; `"start"` events duplicate activity names in the DFG-derived Petri net and corrupt token counts.
2. **Activity filter**: only the ten declared model activities (e.g., `"status:show"`, `"publish:run"`) pass through; noise events like `"git:status"` are excluded.
3. **Timestamp sort**: events are sorted by ISO-8601 timestamp ascending within each trace, ensuring the DFG reflects the true execution order.

These filters are applied in `emit_xes_filtered`, the production XES writer called by `append_events`, the canonical evidence emission entry point used by all verb handlers.

### 3.8.4 Conservative Test Selection

The `INVARIANT_NO_FULL_TRYBUILD_BY_DEFAULT` invariant codifies a safety property: the `trybuild changed` verb must never run the full fixture set by default, regardless of how many fixtures exist. This prevents accidental multi-minute test runs in developer workflows where only one or two trybuild fixtures have changed.

The invariant is enforced through the interaction between `ChangedFileState.changed_trybuild_fixtures` (populated by `ChangedFileDetector.is_trybuild_fixture()`) and `TrybuildState.run_all_by_default` (initialised to `false` in `EngineState::from_workspace()`). The `trybuild changed` verb reads `trybuild.changed_fixtures`; if the list is empty (because no trybuild fixtures have changed relative to `origin/main`), it exits without running any fixtures.

### 3.8.5 Destructive Action Safety

The principle that **no destructive action is taken without explicit confirmation** is enforced both architecturally and by test. The `target prune` verb, which deletes files from the target directory, requires a `--confirm` flag. Without it, the verb runs in dry-run mode, printing what it *would* delete without actually deleting anything. This is verified by `invariant_no_destructive_default_target_prune_is_safe` in `tests/invariants.rs`:

```rust
#[test]
fn invariant_no_destructive_default_target_prune_is_safe() {
    // ... set up fake target directory with a binary ...
    let output = Command::cargo_bin("cargo-cicd")
        .current_dir(dir.path())
        .args(["target", "prune"])
        .output().unwrap();
    assert!(
        fake_target.join("binary").exists(),
        "target prune without --confirm must not delete files"
    );
}
```

The same principle applies to `git close`: it requires explicit confirmation and its `--help` text contains safety-related language (one of `dry`, `safe`, `confirm`, or `check`), which is verified by `invariant_no_false_close_git_close_help_mentions_safety`.

---

## 3.9 Workspace Structure and Crate Organisation

cargo-cicd is a Cargo workspace with three members: the root crate (`cargo-cicd`), `crates/cargo-cicd-core`, and `crates/cargo-cicd-lsp`. This organisation reflects a separation of concerns between the CLI driver, shared core utilities, and the Language Server Protocol integration.

The root crate (`src/`) is the binary driver: it owns `main.rs`, the noun modules, the adapter layer, the engine, the evidence module, the autonomic and policy layers, the wasm4pm integration, and the advanced capabilities. Shared utilities that need to be accessible to the LSP server (e.g., workspace scanning) are placed in `cargo-cicd-core`. The LSP server itself (`cargo-cicd-lsp`) implements the `explain` verb endpoint.

The workspace `Cargo.toml` declares shared dependency versions at the workspace level using the `[workspace.dependencies]` table, enabling consistent version pinning across all three crates without duplication. The minimum supported Rust version (`rust-version = "1.85"`) is declared at the root package level, ensuring that the codebase compiles on stable Rust without nightly features.

---

## 3.10 Summary and Architectural Synthesis

cargo-cicd's architecture can be understood through the lens of three orthogonal design axes:

**Vertical axis — abstraction layers.** From bottom to top: external environment (git, filesystem, Cargo) → adapters (stateless translators) → `EngineState` (aggregate root) → noun handlers (business logic) → `clap-noun-verb` (grammar dispatch) → CLI surface (user-facing). Each layer has a well-defined interface and communication is always downward-only: noun handlers call adapters through `EngineState`, never directly.

**Horizontal axis — feature surface.** From narrow to wide: default binary (no feature flags, minimal dependencies, noun/verb dispatch only) → `process-data` (full engine) → `autonomic` (policy suggestions) → `wasm4pm` (oracle integration) → `advanced` (best-of-breed performance and observability). The horizontal axis is controlled by Cargo feature flags, enabling users and downstream integrators to select exactly the capability surface they require.

**Temporal axis — process evidence.** From instantaneous to persistent: `ProcessEvent` (in-memory, per verb execution) → XES file (on-disk, accumulated per session) → `cicd.toml` (on-disk, last-known workspace state) → wasm4pm receipt (on-disk, external adjudication record). The temporal axis enables audit traceability across process boundaries and provides the evidence substrate for the wasm4pm conformance gate.

The integration of an RDF/Turtle ontology as the source of truth for the command grammar, the manufacturing pipeline from SPARQL inference through Tera templates to Rust code, and the process evidence/external adjudication architecture collectively give cargo-cicd a distinctive character among Rust build tools: it is as much a process-data engineering artefact as it is a developer utility. The architectural choices described in this chapter — manufactured grammar, aggregate root state, silent-failure adapters, opt-in engine, public boundary enforcement, and external adjudication — form a coherent design philosophy whose coherence can be traced back to a single commitment: the workspace CI/CD tool should itself be provably conformant to the process model it enforces.

---

## References

- Evans, E. (2003). *Domain-Driven Design: Tackling Complexity in the Heart of Software*. Addison-Wesley.
- Gamma, E., Helm, R., Johnson, R., and Vlissides, J. (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley.
- van der Aalst, W. M. P. (2016). *Process Mining: Data Science in Action* (2nd ed.). Springer.
- IEEE Std 1849-2016 — *XES Standard for Process Mining Event Logs*.
- W3C (2014). *SKOS Simple Knowledge Organization System Reference*. W3C Recommendation.
- W3C (2013). *PROV-O: The PROV Ontology*. W3C Recommendation.
- SPARQL 1.1 Query Language. W3C Recommendation, 2013. https://www.w3.org/TR/sparql11-query/
- Young, G. (2010). *CQRS Documents*. https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf
