# /git — Git Phase Management

Manage git phase state in the cargo-cicd workspace. This command guides through
checking phase status, understanding git state dimensions, and safely closing phases.

## Quick Start

Run the full git phase workflow:

```bash
cargo cicd git status
cargo cicd git phase
```

## Step 1: Check Current Git Status

```bash
cargo cicd git status
```

This reads `GitPhaseState` from the engine and displays:

| Field | Meaning |
|---|---|
| `branch` | Current branch name |
| `dirty_files` | Unstaged modified files |
| `staged_files` | Files staged for commit |
| `untracked` | New files not yet tracked |
| `ahead` | Commits ahead of upstream |
| `behind` | Commits behind upstream |

The verdict is `PASS` when the workspace is clean (no dirty, staged, or untracked
files). It is `WARN` when any of those lists are non-empty.

## Step 2: Check Current Phase

```bash
cargo cicd git phase
```

Phases reflect the lifecycle position of the branch:

| Phase | Meaning |
|---|---|
| `development` | Active work; dirty or staged files present |
| `staged` | All changes committed; not yet pushed |
| `clean` | Branch is even with upstream, no local changes |
| `published` | Evidence adjudicated and receipt issued by wasm4pm |

## Step 3: Safe Git Close Workflow

**Safety invariant: `git close` NEVER runs on a dirty workspace.**

The `close` verb enforces this unconditionally. Before attempting close, confirm
the workspace is clean:

```bash
# 1. Verify clean state
cargo cicd git status
# Output must show: dirty_files=[], staged_files=[], untracked=[]

# 2. Confirm phase is ready
cargo cicd git phase
# Output must show phase: "clean" or "staged"

# 3. Close the phase
cargo cicd git close
```

If the workspace is not clean, `git close` will refuse and exit non-zero.

## Evidence Emission for Git Operations

Each git verb emits a `ProcessEvent` to `target/cargo-cicd/evidence/`. The pattern:

```
start event  →  work (git adapter queries)  →  complete event  →  [optional wpm audit]
```

Example XES event for `git status`:

```xml
<event>
  <string key="event_id" value="evt-git-status-20260614134507123Z"/>
  <string key="lifecycle_transition" value="complete"/>
  <string key="verdict_claimed" value="WARN"/>
  <string key="trace_class" value="live_workspace"/>
</event>
```

`verdict_claimed` is `PASS` for a clean workspace, `WARN` for dirty/staged/untracked
state. The wasm4pm oracle (`wpm`) adjudicates the final verdict.

## Common Issues

### Merge Conflicts

If `git status` shows conflict markers in `dirty_files`:

```bash
# Identify conflicted files
git status --porcelain | grep "^UU"

# Resolve conflicts, then stage
git add <resolved-file>

# Re-check cargo-cicd state
cargo cicd git status
```

### Diverged Branch (ahead AND behind > 0)

When both `ahead` and `behind` are non-zero, the branch has diverged from upstream:

```bash
# Option A: rebase onto upstream
git fetch origin
git rebase origin/main

# Option B: merge upstream
git merge origin/main

# Verify after
cargo cicd git status
```

The autonomic `branch_behind` policy (when `--features autonomic` is enabled)
will emit a `Warn` recommendation if `behind > 0`.

### Detached HEAD

`cargo cicd git status` will show an empty or SHA-only branch name when HEAD is
detached. Reattach before attempting `git close`:

```bash
# Reattach to a branch
git checkout -b recovery-branch

# Verify branch is set
cargo cicd git status
```

### Close Blocked by Dirty State

If `git close` refuses:

```bash
# See exactly what is dirty
cargo cicd git status

# Commit all staged files
git commit -m "feat(core): ..."

# Stash or discard untracked/dirty files
git stash        # to save
git checkout .   # to discard

# Retry close
cargo cicd git close
```

## EngineState Fields (git_phase_state)

The `GitPhaseState` struct populated by `GitStatusAdapter`:

```rust
pub struct GitPhaseState {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked: Vec<String>,
    pub ahead: usize,
    pub behind: usize,
}
```

`GitStatusAdapter` runs `git status --porcelain` and parses each status line.
It silently returns defaults on failure — partial state is preferred over a crash.

## Related Commands

- `/workspace` — Full workspace health including git phase
- `cargo cicd evidence doctor` — Check evidence from past git operations
- `cargo cicd pipeline run` — Runs all CI/CD activities in sequence (includes git checks)
