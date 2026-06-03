# How to Close a Git Phase

`cargo cicd git close` performs the branch-close sequence: it verifies tests
pass, commits any staged evidence files, and merges to the trunk branch.

## Prerequisites

Before running `git close`:

1. All tests must pass (`cargo cicd test changed` exits 0).
2. The branch must be ahead of trunk (there is something to merge).
3. There must be no merge conflicts with trunk.

## Run the command

```sh
cargo cicd git close
```

## What it does

1. Runs `cargo cicd status show` to verify workspace readiness.
2. Stages any pending evidence files (e.g., updated `cicd.toml`).
3. Creates a commit if there are staged changes.
4. Merges the current branch into the configured trunk branch (default: `main`).
5. Emits a `GitCloseEvent` recorded in `cicd.toml`.

## Example output

```
status: clean (0 dirty files)
staging: cicd.toml
commit: "chore(cicd): update workspace state"
merging feat/my-feature → main... ok

GitCloseEvent written to cicd.toml
```

## Configure trunk branch

In `cicd.toml`:

```toml
[git]
trunk_branch = "main"   # default: "main"
```

## If the close fails

| Failure | Cause | Fix |
|---------|-------|-----|
| Tests failing | `cargo cicd test changed` failed | Fix tests first |
| Dirty files | Uncommitted changes present | Commit or stash |
| Merge conflict | Diverged from trunk | Merge trunk into branch first |

## Notes

- `git close` does not push to a remote. After closing, push manually:
  ```sh
  git push origin main
  ```
- This command is a local operation only and will not affect your remote
  repository unless you push afterward.
