# /git — Git Phase Management

Trigger: user requests git phase status, phase transition, or safe close.

## Canonical sequence

```bash
cargo cicd git status
cargo cicd git phase
cargo cicd git close  # only after status shows clean
```

## `git status` output fields

| Field | Meaning |
|---|---|
| `branch` | Current branch name |
| `dirty_files` | Unstaged modified files |
| `staged_files` | Files staged for commit |
| `untracked` | Untracked files |
| `ahead` | Commits ahead of upstream |
| `behind` | Commits behind upstream |

Verdict `PASS` = all lists empty. Verdict `WARN` = any list non-empty.

## Phases

| Phase | Condition |
|---|---|
| `development` | dirty or staged files present |
| `staged` | all committed, not pushed |
| `clean` | even with upstream, no local changes |
| `published` | evidence adjudicated by wasm4pm |

## Safe close invariant

`git close` NEVER runs on a dirty workspace. Preconditions:
- `dirty_files=[]`, `staged_files=[]`, `untracked=[]`
- phase is `clean` or `staged`

On failure: exits non-zero with refusal message.

## Evidence emission pattern

Each git verb emits to `target/cargo-cicd/evidence/`:
```
start event → work (git adapter queries) → complete event → [optional wpm audit]
```
`verdict_claimed`: `PASS` (clean) or `WARN` (dirty/staged/untracked).

## Failure modes

| Symptom | Fix |
|---------|-----|
| Merge conflicts in `dirty_files` | `git status --porcelain \| grep "^UU"` → resolve → `git add` → re-check |
| `ahead>0` AND `behind>0` | `git fetch origin && git rebase origin/main` |
| Empty/SHA-only branch name (detached HEAD) | `git checkout -b recovery-branch` before close |
| Close blocked by dirty state | `git commit` staged, then `git stash` or `git checkout .` for dirty |

## GitPhaseState struct

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

`GitStatusAdapter` runs `git status --porcelain`, parses each line, silently returns defaults on failure.

Autonomic `branch_behind` policy (feature `autonomic`): emits `Warn` when `behind > 0`.
