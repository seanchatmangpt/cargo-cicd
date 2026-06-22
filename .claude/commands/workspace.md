# /workspace — Workspace Diagnostics

Trigger: user asks about workspace health, adapter failures, cicd.toml, or policy verdicts.
Action: run `cargo cicd workspace doctor`, interpret output, remediate WARNs.

## Canonical Pattern

```bash
cargo cicd workspace doctor
```

## Dimensions Checked

| Dimension | Checks |
|---|---|
| `workspace` | name, root path, members, Rust edition |
| `toolchain` | active toolchain, rustc version |
| `target` | directory path, total size bytes |
| `changed_files` | base ref, changed .rs files, test files, trybuild fixtures |
| `test_plan` | estimated test count, conservative mode flag |
| `trybuild` | fixture sets, changed fixtures, projection profile |
| `git_phase` | branch, dirty/staged/untracked, ahead/behind counts |
| `process_events` | emitted ProcessEvent structs |
| `artifacts` | manifests, registry metadata |
| `policies` | policy verdicts (requires `--features autonomic`) |

Verdicts: `PASS` · `WARN` · `FAIL`

## cicd.toml Freshness

```bash
ls -la cicd.toml
```

Required sections: `[workspace]` · `[state]` · `[target]` · `[[events]]`

If absent or stale (no recent `[[events]]`):
```bash
cargo cicd workspace doctor   # triggers CicdTomlWriter
```

Read-only verbs (`status show`) do not write cicd.toml.

## Autonomic Policies

```bash
cargo build --features autonomic && cargo cicd workspace doctor
```

| Policy | Trigger | Verdict |
|---|---|---|
| `target_pressure` | target dir exceeds size threshold | Warn |
| `toolchain_mismatch` | rustc version differs from lockfile expectation | Warn |
| `trybuild_changed` | trybuild fixtures changed but not run | Warn |
| `branch_behind` | local branch behind upstream by N commits | Warn |
| `evidence_stale` | last evidence emission exceeds age threshold | Warn |
| `publish_not_adjudicated` | publish ran but no wpm verdict recorded | Warn |
| `git_phase_dirty` | dirty or staged files detected | Warn |

All policies are suggest-mode only — never destructive.

## Warn Remediation

```bash
# target_pressure
cargo cicd target prune --dry-run
cargo cicd target prune --confirm

# toolchain_mismatch
rustup update stable && rustup override set stable

# branch_behind
git fetch origin && git merge origin/main

# evidence_stale
cargo cicd status show && ls -la target/cargo-cicd/evidence/

# git_phase_dirty
git add -A && git commit -m "fix(core): ..."
```

## Adapter Failure Diagnosis

Adapters fail silently — partial state is preferred over crashes.

```bash
RUST_LOG=debug cargo cicd workspace doctor 2>&1 | grep -i adapter
```

| Adapter | Silent Symptom | Root Cause | Verify With |
|---|---|---|---|
| `CargoMetadataAdapter` | `workspace.members` empty | Malformed `[workspace]` in Cargo.toml | `grep -A5 '\[workspace\]' Cargo.toml` |
| `GitStatusAdapter` | `git_phase.*` empty | Not in git repo or `git` not on PATH | `git status --porcelain` |
| `ToolchainDetector` | `toolchain.rust_version` empty | `rustc` not on PATH | `rustc --version` |
| `TargetScannerAdapter` | `target.total_size_bytes = 0` | Target dir missing (`cargo build` not run) | `du -sh target/` |
| `ChangedFileDetector` | `changed_files` empty | `origin/main` ref absent | `git diff origin/main --name-only` |
| `TrybuildDetector` | `trybuild.fixture_sets` empty | `tests/ui/` does not exist | `ls tests/ui/` |

## EngineState Fields

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
    pub policies: PolicyState,           // autonomic only
    pub projection: ProjectionProfile,
}
```

`EngineState::from_workspace()` runs all adapters in sequence; each failure is swallowed.

## wasm4pm Evidence (when emitting workspace evidence)

FORBIDDEN: hand-rolling `OcelLog`, `OcelEvent`, `OcelObject` structs.
FORBIDDEN: adjudicating inside cargo-cicd (invariant E1).

```rust
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship};
use wasm4pm_compat::evidence::{Evidence, RawOcelEvidence};
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

// 1. Build OCEL
let log = OCEL { event_types, object_types, events, objects };
// 2. Wrap
let evidence = Evidence::<OCEL, Raw, Ocel20>::raw(log);
// 3. Serialize
serde_json::to_writer(file, &evidence.inner())?;
// 4. Adjudicate (shell-out only)
// wpm audit <file.ocel.json>  → Accept | Refuse | Blocked
```

Object types in domain: `Workspace` · `Crate` · `TestRun` · `GitCommit` · `Release` · `Receipt` · `EvidenceFile` · `Policy` · `Toolchain`

## Related

- `cargo cicd git status` — git phase
- `cargo cicd evidence doctor` — emitted process evidence
- `cargo cicd status show` — quick snapshot
- `cargo cicd pipeline run` — full CI/CD execution
