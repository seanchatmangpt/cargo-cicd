---
description: Run git status then git close; enforce clean-tree requirement; refuse to proceed over dirty files.
allowed-tools: Bash, Read
---

Trigger: user says "close phase", "phase close", or runs `/phase-close`.

## 1 — Inspect git state

```bash
cargo cicd git status
```

Capture full output. Required fields: branch, HEAD SHA, staged/modified/untracked counts, clean flag.

## 2 — Pre-flight: dirty tree blocks closure

**Tree clean** → proceed to step 3.

**Tree dirty** → STOP. Do not run `git close`.

Dirty-tree resolution:

| Dirty files belong to this phase? | Action |
|-----------------------------------|--------|
| Yes | `git add <files> && git commit -m "feat(...): ..."` then re-run `/phase-close` |
| No | `git stash` unrelated changes, then re-run `/phase-close` |

Closing over unrelated dirty files corrupts the audit trail (wrong-phase attribution in `cicd.toml`).

## 3 — Close the phase

```bash
cargo cicd git close
```

Writes to: `cicd.toml [[events]]`, `target/cargo-cicd/evidence/` (XES ProcessEvent).
Prints: HEAD SHA + timestamp on success.

## 4 — Verify

```bash
cargo cicd git status
```

Confirm:
- Phase shown as closed.
- New `.xes` file in `target/cargo-cicd/evidence/`.
- `cicd.toml` contains new `[[events]]` entry.

## 5 — Commit reminder

`git close` does not auto-commit. If `cicd.toml` is dirty after closure:

```
feat(git): close phase <short-description>
```
