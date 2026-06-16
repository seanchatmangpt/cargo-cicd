# /workspace — Workspace Diagnostics

Run a full workspace health assessment for the cargo-cicd workspace. This command
guides through interpreting `workspace doctor` output, validating structure,
checking cicd.toml freshness, and diagnosing adapter failures.

## Quick Start

```bash
cargo cicd workspace doctor
```

This is the primary health check. It populates all `EngineState` dimensions and
reports on each one.

## Step 1: Run Workspace Doctor

```bash
cargo cicd workspace doctor
```

`workspace doctor` queries every adapter in sequence and emits a summary. Expect
output grouped by dimension:

| Dimension | What it checks |
|---|---|
| `workspace` | Name, root path, members list, Rust edition |
| `toolchain` | Active toolchain, `rustc` version |
| `target` | Target directory path, total size in bytes |
| `changed_files` | Base ref, changed `.rs` files, test files, trybuild fixtures |
| `test_plan` | Estimated test count, conservative mode flag |
| `trybuild` | Fixture sets, changed fixtures, projection profile |
| `git_phase` | Branch, dirty/staged/untracked files, ahead/behind counts |
| `process_events` | Previously emitted `ProcessEvent` structs |
| `artifacts` | Artifact manifests, registry metadata |
| `policies` | Policy verdicts (requires `--features autonomic`) |

Verdicts: `PASS` (all healthy), `WARN` (attention needed), `FAIL` (blocking issue).

## Step 2: Check Cargo.toml Workspace Structure

For a workspace root the `[workspace]` section must be present:

```toml
[workspace]
members = [".", "crates/cargo-cicd-core", "crates/cargo-cicd-lsp"]
resolver = "2"
```

Validate manually:

```bash
grep -A 5 '\[workspace\]' Cargo.toml
```

Each member listed must have its own `Cargo.toml`. `CargoMetadataAdapter` scans
member paths by reading `Cargo.toml` line-by-line (no `cargo metadata` invocation).
Missing members cause silent adapter failure — the member list will be shorter than
expected, not an error.

## Step 3: Validate Each Workspace Member

Each member crate should have:

```
crates/<name>/
├── Cargo.toml        # [package] with name, version, edition
└── src/
    └── lib.rs        # or main.rs
```

Check a member:

```bash
cat crates/cargo-cicd-core/Cargo.toml
ls crates/cargo-cicd-core/src/
```

`ManifestParser` reads each member's `Cargo.toml` for `name`, `version`, and
`edition`. Missing fields are silently defaulted to empty strings.

## Step 4: Check cicd.toml Freshness

`cicd.toml` is the persistent state carrier. It should be updated after each
major verb run:

```bash
# Check it exists and when it was last written
ls -la cicd.toml

# Inspect contents
cat cicd.toml
```

Expected sections:

```toml
[workspace]
name = "cargo-cicd"
root_path = "/home/user/cargo-cicd"
members = [".", "crates/cargo-cicd-core", "crates/cargo-cicd-lsp"]

[state]
git_phase = "clean"
target_size_bytes = 524288000

[target]
total_size_bytes = 524288000
pruned_bytes = 0

[[events]]
event_id = "evt-status-show-20260614134507123Z"
command = "status show"
verdict_claimed = "PASS"
verdict_adjudicated = "Accept"
```

If `cicd.toml` is absent or stale (no recent `[[events]]`), re-run any execution
verb to refresh it:

```bash
cargo cicd workspace doctor
ls -la cicd.toml
```

`CicdTomlWriter` serializes the full `EngineState` to TOML after each major
operation. Read-only verbs like `status show` may not write it.

## Step 5: Autonomic Policy Verdicts

When built with `--features autonomic`, `workspace doctor` also runs all policies
and reports their verdicts:

```bash
cargo build --features autonomic
cargo cicd workspace doctor
```

Policies and their triggers:

| Policy | Trigger | Verdict |
|---|---|---|
| `target_pressure` | Target dir exceeds size threshold | `Warn` |
| `toolchain_mismatch` | `rustc` version differs from lockfile expectation | `Warn` |
| `trybuild_changed` | Trybuild fixtures changed but not run | `Warn` |
| `branch_behind` | Local branch behind upstream by N commits | `Warn` |
| `evidence_stale` | Last evidence emission exceeds age threshold | `Warn` |
| `publish_not_adjudicated` | Publish ran but no wpm verdict recorded | `Warn` |
| `git_phase_dirty` | Dirty or staged files detected | `Warn` |

All policies run in **suggest mode** — they emit recommendations only, never
take action.

## Step 6: Remediate Warn Verdicts

### target_pressure

```bash
# See current target size
cargo cicd target show

# Dry-run prune to see what would be removed
cargo cicd target prune --dry-run

# Execute prune
cargo cicd target prune --confirm
```

### toolchain_mismatch

```bash
# Check active toolchain
rustc --version
rustup show

# Update if needed
rustup update stable
rustup override set stable
```

### branch_behind

```bash
git fetch origin
git rebase origin/main
cargo cicd git status
```

### evidence_stale

```bash
# Re-run any verb to emit fresh evidence
cargo cicd status show
ls -la target/cargo-cicd/evidence/
```

### git_phase_dirty

```bash
cargo cicd git status
git add -A && git commit -m "feat(core): ..."
# or
git stash
```

## Diagnosing Adapter Failures

Adapters fail silently — partial state is preferred over crashes. To surface
failures, enable debug logging:

```bash
RUST_LOG=debug cargo cicd workspace doctor 2>&1 | grep -i adapter
```

Common silent failures:

| Adapter | Silent failure symptom | Root cause |
|---|---|---|
| `CargoMetadataAdapter` | `workspace.members` is empty | Malformed `[workspace]` in Cargo.toml |
| `GitStatusAdapter` | `git_phase.*` fields are empty | Not inside a git repo, or `git` not on PATH |
| `ToolchainDetector` | `toolchain.rust_version` is empty | `rustc` not on PATH |
| `TargetScannerAdapter` | `target.total_size_bytes = 0` | Target dir does not exist yet (`cargo build` not run) |
| `ChangedFileDetector` | `changed_files` list is empty | `origin/main` ref not present (fresh clone, no remote) |
| `TrybuildDetector` | `trybuild.fixture_sets` is empty | `tests/ui/` directory does not exist |

To confirm an adapter is the issue, run its underlying command directly:

```bash
# GitStatusAdapter
git status --porcelain

# ToolchainDetector
rustc --version

# ChangedFileDetector
git diff origin/main --name-only

# TargetScannerAdapter
du -sh target/
```

## EngineState Dimensions Reference

```rust
pub struct EngineState {
    pub workspace: WorkspaceState,       // name, root_path, members, edition
    pub toolchain: ToolchainState,       // active toolchain, rust_version
    pub target: TargetState,             // path, total_size_bytes
    pub changed_files: ChangedFileState, // base_ref, changed .rs files
    pub test_plan: TestPlanState,        // estimated test count, conservative
    pub trybuild: TrybuildState,         // fixture sets, changed fixtures
    pub git_phase: GitPhaseState,        // branch, dirty/staged/untracked, ahead/behind
    pub process_events: ProcessEventState, // emitted ProcessEvent list
    pub artifacts: ArtifactState,        // manifests, registry metadata
    pub policies: PolicyState,           // PolicyEntry vec (autonomic only)
    pub projection: ProjectionProfile,   // feature flag surface contract
}
```

`EngineState::from_workspace()` calls all adapters in sequence. Each failure is
swallowed; the next adapter still runs. The resulting state may be partial.

## Related Commands

- `/git` — Git phase status and close workflow
- `cargo cicd evidence doctor` — Inspect emitted process evidence
- `cargo cicd status show` — Quick workspace snapshot
- `cargo cicd pipeline run` — Sequential execution of all CI/CD activities
