# Chapter 3: Architecture and Design

## 3.1 System Architecture Overview

cargo-cicd is designed as a *Level 5 process-data engine* exposed through a conventional Rust CI/CD command-line interface. The distinction between these two planes of identity — the public CI/CD helper and the private process-data engine — is not cosmetic. It reflects a deliberate architectural layering: the public surface (nouns, verbs, help text, exit codes) obeys the expectations of any Rust workspace practitioner, while the internal substrate accumulates, structures, and emits structured process evidence for external adjudication. This separation of concerns is the central design principle from which all architectural decisions follow.

The overall system can be understood as four cooperating layers, arranged from outermost to innermost:

1. **CLI Grammar Layer** — The user-visible command surface, implemented via the `clap-noun-verb` framework. This layer owns argument parsing, default verb injection, and help text generation.

2. **Adapter Layer** — A set of single-responsibility translators that convert external representations (git porcelain output, Cargo metadata JSON, filesystem statistics) into typed internal state values. Adapters are strictly read-only and contain no business logic.

3. **Engine State Layer** — The `EngineState` aggregate root, a single struct that aggregates all runtime dimensions of a Rust workspace. This is the authoritative internal representation against which all policy evaluation and output rendering operates.

4. **Evidence and Policy Layer** — The infrastructure for emitting XES-format process evidence and evaluating autonomic policies. Evidence is consumed by the external wasm4pm oracle; policies produce recommendations consumed by the user.

This chapter documents each layer in turn, covering structure, rationale, and the design trade-offs involved.

---

## 3.2 EngineState: The Aggregate Root

The central data structure of the cargo-cicd engine is `EngineState`, defined in `src/engine/mod.rs`. It is the single source of truth for all runtime information about a Rust workspace during any given invocation:

```rust
/// Full Level 5 engine state — all dimensions
#[derive(Debug, Default)]
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

The design follows the *aggregate root* pattern from Domain-Driven Design [Evans, 2003]. No noun or policy module holds its own mutable state; instead, all runtime information flows through `EngineState` after being populated by adapters. This guarantees a single, consistent view of workspace reality for any given invocation, and ensures that policy evaluation and output rendering are always operating on the same data.

The aggregate structure is visualised below:

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

### 3.2.1 Dimension Catalogue

Each field of `EngineState` represents an independent *dimension* of workspace reality. The eleven dimensions and their responsibilities are as follows.

**WorkspaceState** (`src/engine/workspace_state.rs`) records the structural facts about the Cargo workspace: the workspace name, the root path on the filesystem, the list of member crate paths, the active Rust toolchain channel, and the Rust edition declared in the root manifest. This dimension is populated by `CargoMetadataAdapter`.

**ToolchainState** (`src/engine/toolchain_state.rs`) records toolchain-specific properties: the active toolchain identifier as resolved by rustup, the MSRV declared in `Cargo.toml`, and whether the active toolchain satisfies that MSRV. It is populated by `ToolchainDetector`, which reads both `rust-toolchain.toml` and the output of `rustup show active-toolchain`.

**TargetState** (`src/engine/target_state.rs`) captures the current size of the `target/` directory and the configured limits. The `TargetScannerAdapter` walks the directory tree using `walkdir`, accumulating file sizes. A three-level verdict (`pass` / `warn` / `fail`) is computed from the ratio of actual size to configured maximum.

**ChangedFileState** (`src/engine/changed_file_state.rs`) identifies which source files have changed relative to a configured base branch (`origin/main` by default). The `ChangedFileDetector` adapter calls `git diff --name-only` and maps each changed path to its owning crate.

**TestPlanState** (`src/engine/test_plan_state.rs`) derives from `ChangedFileState` the set of crates that should have their tests executed. It encodes the *changed-tests* optimisation: only crates whose source files appear in the diff are scheduled for test runs. This dimension is not populated by an external adapter but is computed internally from `ChangedFileState`.

**TrybuildState** (`src/engine/trybuild_state.rs`) is specific to Rust compiler error test suites managed by the `trybuild` crate. It identifies which `.rs` fixture files under `tests/ui/` have changed since the last run. Only changed fixtures are executed, reducing turnaround time in large trybuild suites. Populated by `TrybuildDetector`.

**GitPhaseState** (`src/engine/git_phase_state.rs`) captures the full git working-tree status: the current branch name, lists of dirty, staged, and untracked files, the number of commits the local branch is ahead of and behind its upstream, and policy flags such as `require_clean_tree` and `phase_closed`. This is the primary input to the `GitPhaseDirtyPolicy`. Populated by `GitStatusAdapter`.

```rust
pub struct GitPhaseState {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
    pub require_clean_tree: bool,
    pub phase_closed: bool,
}
```

**ProcessEventState** (`src/engine/process_event_state.rs`) is the accumulation buffer for `ProcessEvent` values emitted during the current invocation. Events recorded here are later serialised to XES and JSONL for wasm4pm adjudication.

**ArtifactState** (`src/engine/artifact_state.rs`) tracks the state of compiled binary artifacts and release archives. In v26.6.2, this dimension is primarily used by the `publish` noun to verify that the binary has been built before attempting to publish.

**PolicyState** (`src/engine/policy_state.rs`) holds the collected `PolicyResult` values produced by the autonomic policy engine during the current invocation. When displayed to the user, these results are presented as recommendations rather than directives.

**ProjectionProfile** (`src/engine/projection_profile.rs`) controls which fields of `EngineState` are serialised for external presentation. The profile carries a version string, a public level (controlling which private dimensions are suppressed), and a `suppress_private_fields` flag. At v26.6.2, the default profile is `v26_6_2()`, which sets `public_level = 2` and suppresses private fields. This mechanism ensures that the Level 5 internal structure does not leak into public-facing output.

---

## 3.3 The Adapter Pattern

The adapter layer is the sole point of contact between the engine and the external world. Each adapter is a unit-testable, single-responsibility struct that reads one external source and returns a typed result. No adapter contains business logic; no adapter writes to external sources; and no adapter reads from another adapter. This strict discipline keeps the external boundary narrow and testable.

The architecture is summarised below:

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
```

### 3.3.1 GitStatusAdapter

`GitStatusAdapter` (`src/adapters/git_status.rs`) translates the output of `git status --porcelain` and `git rev-parse --abbrev-ref HEAD` into a `GitStatusResult`. The porcelain output format, which git guarantees to be stable across versions, is parsed line by line. Each line's two-character XY status code is mapped to one of three categories: dirty (unstaged modifications), staged, or untracked. The branch name is retrieved in a separate subprocess call. By using `--porcelain`, the adapter avoids any dependency on git's locale-sensitive prose output.

```rust
impl GitStatusAdapter {
    pub fn query() -> Result<GitStatusResult> {
        // Single git status --porcelain call; parse XY codes per line
        // Separate rev-parse call for branch name
    }

    pub fn is_dirty() -> bool {
        // Fast path: check whether stdout is non-empty
    }
}
```

The `is_dirty()` method provides a fast-path query used by `GitPhaseDirtyPolicy` without constructing the full `GitStatusResult`.

### 3.3.2 TargetScannerAdapter

`TargetScannerAdapter` (`src/adapters/target_scanner.rs`) uses the `walkdir` crate to traverse the `target/` directory tree, accumulating file sizes from filesystem metadata. The total is expressed in gigabytes and compared against a configurable threshold. Crucially, directory metadata is excluded from the sum — only regular file sizes are counted — preventing double-counting of directory block allocation.

The adapter also encodes a three-level verdict function:

- Below 70% of the configured limit: `pass`
- Between 70% and 100%: `warn`
- At or above 100%: `fail`

This graduated response prevents the common failure mode where a CI check passes until the moment it catastrophically fails, by giving the practitioner early warning as the cache approaches its limit.

### 3.3.3 ToolchainDetector and CargoMetadataAdapter

`ToolchainDetector` (`src/adapters/toolchain_detector.rs`) reads `rust-toolchain.toml` or the legacy `rust-toolchain` file to determine the pinned toolchain channel. It does not invoke `rustup` directly, avoiding a mandatory runtime dependency on the toolchain manager.

`CargoMetadataAdapter` (`src/adapters/cargo_metadata.rs`) shells out to `cargo metadata --format-version 1` to discover workspace members, package names, and edition declarations. Using the machine-readable JSON output rather than parsing `Cargo.toml` directly ensures that workspace path resolution, virtual manifest inheritance, and path dependency substitution are all handled by Cargo itself.

### 3.3.4 Other Adapters

`ChangedFileDetector` invokes `git diff --name-only` against a configurable base branch to enumerate changed source paths. `TrybuildDetector` hashes the modification time and content of each `.rs` file under `tests/ui/` to identify which fixtures have changed. `CicdTomlWriter` is the sole adapter that performs writes — it serialises the `CicdToml` struct to disk.

---

## 3.4 The Noun-Verb CLI Grammar

The public command interface follows a *noun-verb* grammar, a deliberate departure from the conventional flat subcommand model. In the flat model, `cargo cicd status` and `cargo cicd status audit` would be two independent commands with no shared namespace. In the noun-verb model, `status` is a noun (a command namespace), and `show` and `audit` are its verbs (the actions within that namespace). This grammar scales cleanly as new verbs are added to existing nouns without polluting the top-level command surface.

The grammar is implemented by the `clap-noun-verb` crate (version `26.6.2`, published as a companion to this tool). Two traits define the extension points:

- `NounCommand`: implemented by each noun module. Exposes `name()`, `about()`, and `verbs()`, which returns a `Vec<Box<dyn VerbCommand>>`.
- `VerbCommand`: implemented by each verb struct within a noun module. Exposes `name()`, `about()`, and `run()`, which receives a `VerbArgs` reference.

The full set of registered nouns at v26.6.2 is: `evidence`, `pipeline`, `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`, and `lsp`.

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
       │ via Adapters     │
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

### 3.4.1 Default Verb Injection

A usability requirement is that bare noun invocations (e.g., `cargo cicd status` without a verb) should work rather than failing with a usage error. This is satisfied by the `inject_default_verbs()` function in `main.rs`, which inspects `argv` before argument parsing and inserts a default verb when the second argument is a known noun with no following non-flag argument:

```
"status"    → inserts "show"
"publish"   → inserts "run"
"workspace" → inserts "doctor"
"evidence"  → inserts "doctor"
```

For nouns that implement `run_direct()`, the dispatch bypasses the `CliBuilder` entirely and routes directly to the verb implementation, which avoids a parsing round-trip for the common case.

### 3.4.2 Cargo External Subcommand Protocol

Cargo invokes external subcommands by executing the binary `cargo-<name>` with the subcommand name prepended to `argv`. When the user runs `cargo cicd status`, Cargo executes `cargo-cicd cicd status`. The `main()` function detects the `"cicd"` prefix in `argv[1]` and re-executes itself without it, so that the rest of `main()` always sees clean arguments beginning with the noun.

---

## 3.5 cicd.toml as State Carrier

`cicd.toml` serves a dual purpose: workspace configuration and emitted state record. It is written to the workspace root by `CicdTomlWriter` after each significant command invocation and read on subsequent invocations to provide baseline state values without requiring all adapters to re-query their external sources.

The schema, defined in `src/cicd_toml.rs`, is structured into seven sections:

- **`[workspace]`**: Static workspace identity — name, toolchain channel, target directory path.
- **`[state]`**: Dynamic snapshot — dirty flag, `target/` size in GB, changed file count, changed test count, changed trybuild fixture count.
- **`[target]`**: Configuration for the `TargetScannerAdapter` — `max_size_gb` (default 20) and `prune_after_days` (default 14).
- **`[test.changed]`**: Controls the changed-tests optimisation — enabled flag and base branch ref (`origin/main`).
- **`[trybuild.changed]`**: Controls the trybuild changed-fixture optimisation — enabled flag and snapshot mode (`changed-only`).
- **`[git.phase]`**: Git phase enforcement configuration — `require_clean_tree` flag and `commit_after_phase` flag.
- **`[autonomic]`**: Autonomic policy configuration — enabled flag and mode string (`suggest` or `apply`).
- **`[[events]]`**: An append-only array of `EventRecord` values, each carrying a `kind`, `verdict`, and optional `details` and `timestamp`. This is the TOML-level audit trail.

The round-trip property (write → read → equal) is verified by an inline unit test in `src/cicd_toml.rs`. Optional fields on `EventRecord` (specifically `details` and `timestamp`) are decorated with `#[serde(skip_serializing_if = "Option::is_none")]` to keep the TOML file readable when those fields are absent.

---

## 3.6 Feature Flag Architecture

The feature flag design separates the public command surface from the internal engine, making the former available without any overhead from the latter. Four flags are defined in `Cargo.toml`:

```toml
[features]
default        = []
process-data   = []
autonomic      = ["process-data"]
contrib        = ["process-data"]
wasm4pm        = ["process-data"]
```

**`process-data`** is the master gate for all Level 5 engine internals. When disabled (the default), the binary compiles without `EngineState`, adapters, cicd.toml read/write, policy evaluation, or XES emission. When enabled, all internal plumbing becomes available. The separation allows the public CLI to remain lean and auditable without the engine substrate.

**`autonomic`** implies `process-data` and additionally activates the policy evaluation loop. With `autonomic` disabled, policy structs are compiled but never evaluated. With it enabled, all four built-in policies run in `suggest` mode on every invocation that populates `PolicyState`.

**`wasm4pm`** implies `process-data` and activates the integration seams in `src/integrations/` for richer wasm4pm interaction, including direct evidence submission. Crucially, the feature flag gates the *integration depth*, not the evidence-gate law itself. Even without `wasm4pm` enabled, cargo-cicd emits XES evidence to `target/cargo-cicd/evidence/`, and the evidence-gate tests invoke the wasm4pm oracle. The release closure requirement — that wasm4pm must issue an Accept verdict before a release is certified — is unconditional.

**`contrib`** implies `process-data` and is reserved for contributor-only utilities and debugging aids. It is not part of the public API surface contract.

The `ProjectionProfile` struct enforces the public/private boundary at serialisation time, ensuring that internal state fields gated behind `process-data` are not exposed in public output regardless of which features happen to be enabled at compile time.

---

## 3.7 The Autonomic Policy Engine

The autonomic policy engine is the mechanism by which cargo-cicd reasons about workspace health and communicates recommendations to the practitioner without taking destructive action. The design is explicitly aligned with the *MAPE-K* (Monitor, Analyse, Plan, Execute — Knowledge) autonomic computing reference model [Kephart & Chess, 2003], with the key constraint that the Execute phase is permanently limited to `suggest` mode in the v26.6.2 release.

```
┌────────────────────────────────────┐
│  EngineState (fully populated)      │
│  - workspace, toolchain, target,    │
│  - changed_files, git_phase, etc.   │
└──────────────┬─────────────────────┘
               │ Monitor
    ┌──────────▼──────────┐
    │  Adapter outputs    │
    │  (raw measurements) │
    └──────────┬──────────┘
               │ Analyse
    ┌──────────▼──────────┐
    │  Policy::evaluate() │
    │  per-policy result  │
    └──────────┬──────────┘
               │ Plan
    ┌──────────▼──────────┐
    │  PolicyState:        │
    │  collected results  │
    │  + recommendations  │
    └──────────┬──────────┘
               │ Execute (suggest only)
    ┌──────────▼──────────┐
    │  User-visible output │
    │  (no side effects)  │
    └─────────────────────┘
```

### 3.7.1 Policy Interface

All policies implement the `CicdPolicy` trait:

```rust
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn mode(&self) -> PolicyMode;
    fn evaluate(&self) -> PolicyResult;
}
```

`PolicyMode` has two variants: `Suggest` and `Apply`. All current policies return `PolicyMode::Suggest`. The `Apply` variant is recognised at the type level to permit future work, but is not yet connected to any destructive action.

`PolicyResult` carries the policy name, enabled flag, mode string, verdict string (`"pass"` / `"warn"` / `"alert"`), an optional recommendation string, and an `event_kind` for XES emission.

### 3.7.2 Built-in Policies

Four policies are implemented at v26.6.2:

**GitPhaseDirtyPolicy** (`src/policies/git_phase_dirty.rs`) queries `GitStatusAdapter::is_dirty()`. If the working tree is dirty, it emits an `"alert"` verdict with the recommendation to commit or stash before running CI. A clean tree yields `"pass"` with no recommendation.

**TargetPressurePolicy** (`src/policies/target_pressure.rs`) reads the `target/` directory size via `TargetScannerAdapter::total_size_gb()` and compares it against a configurable `max_gb` threshold (default 20 GB). The graduated response (warn at 70%, alert at 100%) mirrors the adapter's own verdict function, ensuring consistent messaging across the system.

**ToolchainMismatchPolicy** (`src/policies/toolchain_mismatch.rs`) compares the active toolchain channel from `ToolchainDetector` against the MSRV declared in `Cargo.toml`. A mismatch — for example, an `nightly` channel used in a workspace that declares a stable MSRV — yields a `"warn"` verdict.

**TrybuildChangedPolicy** (`src/policies/trybuild_changed.rs`) reports whether changed trybuild fixtures were detected. It is informational rather than advisory: the verdict is always `"pass"`, but the recommendation string reports the count of changed fixtures to guide the practitioner's attention.

### 3.7.3 MAPE-K Alignment

The Monitor phase is embodied by the adapters, which run unconditionally on every invocation and populate their respective `EngineState` dimensions. The Analyse phase is embodied by the `evaluate()` implementations, each of which reads its relevant dimension(s) and computes a typed result. The Plan phase corresponds to the collection of results into `PolicyState`, from which a unified recommendation surface is constructed. The Execute phase is presently limited to emitting those recommendations as user-visible output and XES evidence events — no filesystem, git, or network operations are performed without explicit user confirmation.

This constraint is intentional. Autonomic systems that take destructive action without confirmation violate the practitioner's expectation of transparency. The `suggest`-only default is the correct posture for a tool operating in a shared workspace where other processes may be running concurrently.

---

## 3.8 Design Decisions and Trade-offs

### 3.8.1 Single Aggregate Root versus Per-Noun State

The choice to centralise all runtime state into `EngineState` rather than allowing each noun to own its own state carries both benefits and costs. The primary benefit is that policy evaluation always operates on a coherent, simultaneous snapshot of all workspace dimensions. A policy that needs both git state and target size — for example, a hypothetical `BlockPublishPolicy` that refuses to publish from a dirty, oversized workspace — can read both from the same `EngineState` without coordinating between separate state owners. The cost is that populating `EngineState` requires running all adapters, even those whose output is not needed for the current noun. This is mitigated in practice by the speed of the individual adapters (all of which are single subprocess calls or filesystem traversals) and by the `cicd.toml` state cache, which allows adapters to skip expensive re-queries when workspace state has not changed since the last invocation.

### 3.8.2 External Adjudication via wasm4pm

A defining architectural invariant is that cargo-cicd never adjudicates its own process conformance (Evidence Invariant E1). The XES emission infrastructure exists to produce evidence; the verdict on whether that evidence demonstrates a conformant process is issued exclusively by the external wasm4pm oracle. This separation reflects a fundamental principle: a tool that certifies its own correctness provides no stronger guarantee than an unchecked tool. By routing evidence through an independent oracle, the certification chain has a boundary that cargo-cicd's own test suite cannot corrupt.

The practical consequence is that the release gate is not satisfied by passing internal tests alone. The evidence-gate tests (`tests/wasm4pm_evidence_gate.rs`, `tests/wasm4pm_evidence_mutation.rs`, `tests/wasm4pm_refusal_cases.rs`) must invoke the wasm4pm oracle and assert an Accept verdict. This is enforced structurally: the `WpmEvidenceOracle` panics if the oracle is unavailable and the expected verdict is not `Blocked` (Evidence Invariant E3).

### 3.8.3 No Parallel Test Execution

The `--jobs` flag is recognised but not yet functional. Tests are executed serially. This is a deliberate limitation: because `cicd.toml` is a file in the workspace root and all adapters read from and write to the same `target/` directory, concurrent test invocations would risk race conditions on shared state. The correct solution — partitioning state by session identifier — is deferred to a future release. Premature parallelism in a tool that manages workspace state would introduce non-determinism that is difficult to observe and harder to reproduce in CI.

### 3.8.4 Feature-Flag Isolation of Engine Internals

Compiling the Level 5 engine internals behind the `process-data` feature flag preserves the option of shipping a minimal public binary that has no compile-time dependency on the engine infrastructure. This is valuable for two reasons. First, it enforces the layering discipline at the type level: code that should not reference `EngineState` will fail to compile if it attempts to do so while `process-data` is disabled. Second, it simplifies audit of the public binary surface: reviewers need only inspect the feature-flag-free compilation path to verify that no internal state leaks into public output.

### 3.8.5 XES as the Evidence Format

The choice of XES (XML Event Stream, as defined by the IEEE XES standard for process mining) as the evidence emission format is driven by the requirements of the wasm4pm oracle, which implements conformance checking via token-replay fitness against a process model derived from the declared activity set. JSON or JSONL alone would satisfy a logging requirement, but would not satisfy the conformance checking requirement without additional transformation. The dual emission (XES for oracle adjudication, JSONL for downstream tooling) provides both the oracle-compatible format and a machine-readable companion for programmatic consumers.

The production XES writer (`emit_xes_filtered`) applies three quality constraints that improve token-replay fitness: (1) only `"complete"` lifecycle events are included, since start events duplicate activity names in the derived Petri net; (2) only the ten declared model activities are included, filtering out noise events such as `"git:status"` that would introduce unmodelled transitions; and (3) events are sorted by timestamp within each trace to ensure the directly-follows graph reflects the actual execution order. These constraints are validated by the mutation and refusal test suites.

---

## 3.9 Summary

This chapter has presented the architecture of cargo-cicd as a four-layer system in which a conventional CLI grammar exposes a Level 5 process-data engine. The `EngineState` aggregate root provides a coherent, simultaneous snapshot of all workspace dimensions; the adapter layer translates external sources into typed state without business logic or side effects; the noun-verb CLI grammar scales cleanly across a growing command surface; and `cicd.toml` serves as both configuration carrier and append-only event log. The autonomic policy engine implements the MAPE-K loop in a non-destructive `suggest`-only mode, while the XES evidence infrastructure supports external adjudication by the wasm4pm oracle. The principal design decisions — centralised state, external adjudication, serial test execution, and feature-flag isolation — are conservative choices that prioritise correctness and transparency over performance and flexibility, consistent with the requirements of a tool operating at the boundary of Rust workspace automation and process conformance certification.
