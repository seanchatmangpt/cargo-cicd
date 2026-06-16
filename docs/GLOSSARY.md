# cargo-cicd Glossary

This is the canonical reference for all terms used in cargo-cicd. Terms are grouped by domain. Any term listed under **Forbidden Terms** must never appear in public-facing output (help text, CLI output, documentation).

---

## CLI Grammar Terms

### noun
A top-level command category that groups related actions. Nouns form the first positional argument in the `cargo cicd <noun> <verb>` grammar. All noun names are lowercase ASCII.

Available nouns:
- `evidence` — Process evidence emission and adjudication
- `git` — Git phase tracking and closure
- `lsp` — Language server for IDE integration
- `pipeline` — Sequential execution of all CI/CD activities
- `publish` — Artifact publishing gate
- `status` — Workspace health snapshot
- `target` — Target directory analysis and cleanup
- `test` — Selective test execution by changed files
- `trybuild` — Compiler error snapshot tests
- `workspace` — Workspace-wide diagnostics

### verb
An action within a noun. Verbs form the second positional argument. Each noun exposes one or more verbs.

Common verbs by category:
- **Read-only:** `show`, `status`, `explain`, `doctor`
- **Dry-run:** `prune --dry-run` (planning, never destructive)
- **Execution:** `run`, `close` (may be destructive)
- **Special:** `audit` (adjudication only), `changed` (scope-limited execution)

### default verb
The verb automatically injected when a bare noun is issued without an explicit verb. Default verbs simplify the most common usage of each noun.

| Bare noun | Resolved command |
|-----------|-----------------|
| `cargo cicd status` | `cargo cicd status show` |
| `cargo cicd publish` | `cargo cicd publish run` |
| `cargo cicd workspace` | `cargo cicd workspace doctor` |
| `cargo cicd evidence` | `cargo cicd evidence doctor` |

### verb injection
The `main.rs` logic (`inject_default_verbs()`) that inspects the argument list and inserts a default verb when a bare noun is detected. This runs before clap dispatch so the rest of the application sees a fully-qualified `<noun> <verb>` pair.

### NounCommand
The trait that every noun module implements. Provides the noun name, description, and the list of verbs it exposes.

### VerbCommand
The trait that every verb struct implements. Contains the `run()` method that performs work, emits evidence, and returns a result.

---

## State Terms

### EngineState
The aggregate root struct (`src/engine/mod.rs`) containing all 11 runtime dimensions. Every noun reads from `EngineState`; adapters populate it. Business logic flows through this struct.

```rust
pub struct EngineState {
    pub workspace:       WorkspaceState,
    pub toolchain:       ToolchainState,
    pub target:          TargetState,
    pub changed_files:   ChangedFileState,
    pub test_plan:       TestPlanState,
    pub trybuild:        TrybuildState,
    pub git_phase:       GitPhaseState,
    pub process_events:  ProcessEventState,
    pub artifacts:       ArtifactState,
    pub policies:        PolicyState,
    pub projection:      ProjectionProfile,
}
```

Initialized via `EngineState::from_workspace()`, which calls all adapters in sequence and silently handles failures (partial data is preferred over crashes).

### WorkspaceState
Dimension tracking workspace name, root path, member crates, active toolchain, and Rust edition. Populated by `CargoMetadataAdapter` and `ManifestParser`.

### ToolchainState
Dimension tracking the active Rust toolchain identifier and `rustc` version string. Populated by `ToolchainDetector`.

### TargetState
Dimension tracking the target directory path and its total size in bytes. Populated by `TargetScannerAdapter` (slow — recursive `walkdir`).

### ChangedFileState
Dimension tracking the base ref, list of changed `.rs` files, changed test files, and changed trybuild fixtures relative to `origin/main`. Populated by `ChangedFileDetector`.

### TestPlanState
Dimension tracking estimated test count and whether conservative mode is active. Used by the `test changed` verb to scope execution.

### TrybuildState
Dimension tracking fixture sets, changed fixtures, and the active projection profile for compiler error snapshot tests.

### GitPhaseState
Dimension tracking the current branch, dirty/staged/untracked file lists, and ahead/behind commit counts relative to the tracking branch.

### ProcessEventState
Dimension holding the list of `ProcessEvent` structs emitted during the current session. Serialized to XES and JSONL evidence files.

### ArtifactState
Dimension tracking artifact manifests and registry metadata for the `publish` noun.

### PolicyState
Dimension holding `PolicyEntry` structs produced by the autonomic policy layer. Each entry records name, verdict, recommendation, and emission timestamp.

### ProjectionProfile
Feature flag surface contract that controls which capabilities are active. Populated from compiled feature flags (`process-data`, `autonomic`, `wasm4pm`).

### cicd.toml
The persistent state carrier file written to the workspace root after each major verb execution. Serialized by `CicdTomlWriter`. Not a user-editable config file — contents are overwritten on each run.

Structure:
```toml
[workspace]
name      = "cargo-cicd"
root_path = "/home/user/cargo-cicd"
members   = [".", "crates/cargo-cicd-core", "crates/cargo-cicd-lsp"]

[state]
git_phase        = "clean"
target_size_bytes = 524288000

[target]
total_size_bytes = 524288000
pruned_bytes     = 0

[[events]]
event_id           = "evt-status-show-20260614134507123Z"
timestamp          = "2026-06-14T13:45:07.123Z"
command            = "status show"
verdict_claimed    = "PASS"
verdict_adjudicated = "Accept"
```

### adapter
A stateless, pure translator from one external source into `EngineState`. Adapters have no internal state; all methods are `&self` or free functions. Adapters silently fail — they return defaults rather than panicking. Adapters never call other adapters.

| Adapter | Source | Speed |
|---------|--------|-------|
| `CargoMetadataAdapter` | Line-by-line `Cargo.toml` scan | Fast |
| `ManifestParser` | TOML parsing via `toml` crate | Fast |
| `GitStatusAdapter` | `git status --porcelain` | Medium |
| `ToolchainDetector` | `rustc --version` | Medium |
| `TargetScannerAdapter` | Recursive `walkdir` over target dir | Slow |
| `ChangedFileDetector` | `git diff origin/main --name-only` | Medium |
| `TrybuildDetector` | Filesystem scan of `tests/ui/` | Fast |
| `CicdTomlWriter` | Serialize `EngineState` → `cicd.toml` | Fast |

---

## Evidence Terms

### ProcessEvent
A structured event emitted at verb start and again at verb completion. Each event records identity, timing, outcome, and optional oracle verdict.

```rust
pub struct ProcessEvent {
    pub event_id:             String,         // "evt-status-show-20260614134507123Z"
    pub timestamp_iso:        String,         // "2026-06-14T13:45:07.123Z"
    pub case_id:              Option<String>, // Groups start+complete into one trace
    pub lifecycle_transition: String,         // "start" or "complete"
    pub workspace_id:         String,
    pub repo_path:            String,
    pub command:              String,         // "status show"
    pub verdict_claimed:      String,         // "PASS", "WARN", or "FAIL"
    pub duration_ms:          Option<u64>,    // None for "start" events
    pub verdict_adjudicated:  Option<String>, // Set after oracle adjudication
    pub adjudicated_at:       Option<String>,
    pub oracle_command:       Option<String>,
    pub trace_class:          String,         // "live_workspace" or "pipeline_run"
}
```

### lifecycle_transition
The phase field on a `ProcessEvent`. Always either `"start"` (emitted when a verb begins work) or `"complete"` (emitted when a verb finishes, with verdict and duration).

### case_id
An opaque string identifier that groups the `start` and `complete` `ProcessEvent` pair for a single verb execution into one XES `<trace>` element. Enables the oracle to correlate the full lifecycle of a command.

### verdict_claimed
The outcome self-reported by cargo-cicd after completing a verb. One of:
- `PASS` — All checks satisfied
- `WARN` — Completed with warnings; work continues
- `FAIL` — Blocking error; work halts
- `WARN:dry_run` — Planning run only (no side effects)
- `WARN:oracle_unavailable` — wpm binary not found

cargo-cicd never adjudicates its own verdict claims. Only the wasm4pm oracle issues final verdicts.

### verdict_adjudicated
The outcome issued by the wasm4pm oracle after inspecting the XES evidence. One of `Accept`, `Refuse`, or `Blocked`. Set on the `ProcessEvent` after `wpm audit` returns.

### XES
**XML Event Stream.** The wire format used to serialize `ProcessEvent` structs for external adjudication. Written to `target/cargo-cicd/evidence/evt-*.xes`.

Structure:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<log>
  <trace>
    <string key="case_id" value="status_show_phase"/>
    <event>
      <string key="event_id"             value="evt-status-show-20260614134507123Z"/>
      <string key="timestamp"            value="2026-06-14T13:45:07.123Z"/>
      <string key="lifecycle_transition" value="complete"/>
      <string key="verdict_claimed"      value="PASS"/>
      <string key="trace_class"          value="live_workspace"/>
    </event>
  </trace>
</log>
```

### JSONL
**JSON Lines.** A machine-readable companion format to XES, written alongside each `.xes` file. Each line is a self-contained JSON object representing one `ProcessEvent`. Useful for streaming log ingestion and grep-friendly inspection.

```jsonl
{"event_id":"evt-status-show-20260614134507123Z","timestamp":"2026-06-14T13:45:07.123Z","command":"status show","verdict_claimed":"PASS"}
```

### evidence invariants (E1–E7)
Seven rules enforced in `src/evidence.rs` that govern correct evidence emission:

| Code | Rule |
|------|------|
| E1 | cargo-cicd never adjudicates itself; only wasm4pm issues verdicts |
| E2 | XES file must exist on disk before `audit_xes()` is called |
| E3 | If oracle unavailable and expected verdict is not `Blocked`, panic |
| E4 | Tests assert only the wasm4pm verdict, never internal cargo-cicd state |
| E5 | XES groups events by `case_id` into `<trace>` elements |
| E6 | JSONL emission mirrors XES (same event set, machine-readable) |
| E7 | `Blocked` is a first-class expectation, not an error |

---

## Oracle Terms

### wasm4pm
The external process conformance oracle. cargo-cicd emits XES evidence; wasm4pm adjudicates it. The oracle is the sole authority on whether a command execution conforms to process expectations. cargo-cicd never issues its own final verdicts.

### wpm
The wasm4pm binary. Invoked by `Wasm4pmShell` when the `wasm4pm` feature flag is enabled.

Primary commands:

| Command | Input | Output |
|---------|-------|--------|
| `wpm audit <file.xes>` | XES evidence file | `Accept` / `Refuse` / `Blocked` |
| `wpm receipt doctor --format json --strict <receipt.json>` | Receipt artifact | `Accept` / `Refuse` |

### ExpectedWpmVerdict
An enum used in tests to declare the anticipated oracle outcome. Prevents tests from silently passing when the oracle is unavailable.

- `Accept` — Oracle should accept the evidence (normal operation)
- `Refuse` — Oracle should reject the evidence (expected failure)
- `Blocked` — Oracle is unavailable; skip oracle assertion

### receipt
An evidence artifact produced after successful oracle adjudication. Stored in `receipts/`. Validated by `wpm receipt doctor --format json --strict`. Required for release.

### Blocked
The oracle verdict returned when the `wpm` binary is not found on `PATH`. In tests, `ExpectedWpmVerdict::Blocked` is a first-class expectation that skips oracle assertions rather than failing the test. The full release gate requires the oracle to be available and return `Accept`.

### audit_xes()
The `Wasm4pmShell` method that shells out to `wpm audit <xes_path>` and returns the parsed oracle verdict. Only compiled when the `wasm4pm` feature flag is active.

---

## Policy Terms

### autonomic policy
A read-only workspace rule implemented in `src/policies/`. Each policy inspects `EngineState` and emits a `PolicyEntry` with a recommendation. Policies never take destructive action.

Available policies:
- `target_pressure` — Target directory exceeds size threshold
- `toolchain_mismatch` — Active `rustc` version differs from lockfile expectation
- `trybuild_changed` — Trybuild fixtures changed but not re-run
- `branch_behind` — Local branch is behind `main` by N commits
- `evidence_stale` — Last evidence emission exceeds age threshold
- `publish_not_adjudicated` — Publish occurred without oracle verdict
- `git_phase_dirty` — Dirty or staged files are present

### suggest mode
The default and only permitted mode for autonomic policies. In suggest mode a policy reads state, evaluates a condition, and emits a human-readable recommendation. It never modifies files, runs commands, or takes any other side-effecting action.

### apply mode
A future policy mode that would auto-remediate a detected condition. Currently not implemented. Any policy that takes action without explicit user confirmation violates the autonomic contract.

### PolicyVerdict
The outcome of a single policy evaluation. One of:

| Verdict | Meaning |
|---------|---------|
| `Pass` | Condition not triggered; no recommendation needed |
| `Warn` | Condition detected; recommendation emitted |
| `Suggest` | Condition partially met; soft recommendation emitted |
| `Skip` | Policy inapplicable to this workspace (e.g., single-crate workspace) |

### PolicyEntry
The struct returned by each policy's `eval()` function:

```rust
pub struct PolicyEntry {
    pub policy_name:    String,
    pub verdict:        PolicyVerdict,
    pub recommendation: String,
    pub emitted_at:     String,
}
```

### run_all_policies()
The dispatch function in `src/autonomic/policies.rs` that calls every registered policy's `eval()` function against the current `EngineState` and collects results into `PolicyState`. Only compiled when the `autonomic` feature flag is active.

---

## Manufacturing Terms

### ggen
The code generator that manufactures noun/verb scaffolding from the RDF ontology. Running `ggen` regenerates noun modules, CLI test scaffolding, README sections, and reference docs. The CLI grammar is manufactured, not handwritten — any structural change to nouns or verbs requires a `ggen` run.

### ontology
The RDF/Turtle file (`ontology/cargo-cicd-capabilities.ttl`) that declares all noun/verb capabilities, their CLI command strings, and human-readable descriptions. The single source of truth for the CLI grammar.

### SPARQL
The query language used by `ggen` to extract capability definitions from the ontology. SPARQL rules are stored in `queries/*.sparql` and referenced by `ggen.toml`.

### Tera
The template engine used by `ggen` to render extracted capabilities into Rust source files, Markdown docs, and test scaffolding. Templates live in `templates/`.

### ggen.toml
The configuration file that wires together the ontology, SPARQL queries, and Tera templates. Defines output destinations for each generated artifact.

### clap-noun-verb
The published crate that provides the `NounCommand` and `VerbCommand` traits and wires the noun/verb grammar into a clap-compatible CLI structure.

---

## Feature Flag Terms

### process-data
The base feature flag enabling the Level 5 engine internals: `EngineState`, all adapters, and `cicd.toml` writes. Off by default. Required by all other non-default flags.

### autonomic
Feature flag enabling the policy suggestion layer. Implies `process-data`. Activates `run_all_policies()` and surfaces recommendations in `workspace doctor` output.

### wasm4pm
Feature flag enabling oracle integration. Implies `process-data`. Activates `Wasm4pmShell::audit_xes()` and verdict adjudication in evidence tests.

### contrib
Feature flag for community maintainer tooling. Implies `process-data`. Enables verbose logging and debug output not intended for end users.

---

## Forbidden Terms (Internal Only)

The following terms are reserved for internal use and **must never appear** in any public-facing output: help text, CLI stdout/stderr, documentation, or user-visible error messages. The invariant test `invariant_public_boundary_no_forbidden_terms_in_all_help()` scans all `--help` output and will fail the release gate if any of these terms are found.

| Term | Internal meaning |
|------|-----------------|
| `ALIVE` | Level 5 engine status marker |
| `Inspection Gate` | Manufacturing subsystem identity |
| `wall` | Jargon from manufacturing pipeline |
| `Nehemiah` | Code name for manufacturing layer (exposed only as `ggen`) |
| `Field8` | Internal capacity measurement |
| `Instinct8` | Autonomic reasoning subsystem |
| `Cargo Court` | Internal adjudication metaphor |
| `AGI` | AI system classification |
| `Truex` | Internal truth engine |
| `CONSTRUCT8` | Manufacturing directive system |
