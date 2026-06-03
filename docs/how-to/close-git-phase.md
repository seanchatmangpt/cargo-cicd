<!-- BEGIN custom:full-doc -->
# How to Close a Git Phase

`cargo cicd git close` performs the branch-close sequence: it verifies tests
pass, commits any staged evidence files, and merges to the trunk branch.

## Prerequisites

Before running `git close`, the following conditions must all hold:

1. All changed tests pass — `cargo cicd test changed` exits 0.
2. The current branch is ahead of trunk — there is at least one commit to merge.
3. No merge conflicts exist between the current branch and trunk.
4. No uncommitted source changes are present in the working tree.

Run `cargo cicd status show` to confirm readiness before proceeding.

## Run the command

```sh
cargo cicd git close
```

## What it enforces

`git close` is not a convenience wrapper around `git merge`. It enforces a
structured sequence and refuses to proceed if any precondition is unmet:

1. Runs a status check to verify workspace readiness.
2. Stages any pending evidence files (for example, an updated `cicd.toml`).
3. Creates a commit if there are staged changes.
4. Merges the current branch into the configured trunk branch (default: `main`).
5. Emits a `GitCloseEvent` recorded in `cicd.toml` as a structured receipt.

**Example output:**

```
status: clean (0 dirty files)
staging: cicd.toml
commit: "chore(cicd): update workspace state"
merging feat/my-feature → main... ok

GitCloseEvent written to cicd.toml
```

## Configure the trunk branch

In `cicd.toml`:

```toml
[git]
trunk_branch = "main"   # default: "main"
```

## Resolving refusals

`git close` emits a structured refusal message when a precondition is unmet.
The message names the specific condition that blocked the close.

| Refusal message | Cause | Resolution |
|----------------|-------|------------|
| `tests failing` | `cargo cicd test changed` exited non-zero | Fix the failing tests, then retry |
| `dirty files: N` | Uncommitted source changes in working tree | Commit or stash changes, then retry |
| `merge conflict with trunk` | Current branch has diverged from trunk | Merge trunk into your branch first: `git merge main` |
| `branch not ahead of trunk` | No commits to merge | Ensure you have at least one commit on the branch |
| `evidence staging failed` | `cicd.toml` could not be written | Check file permissions on `cicd.toml` |

Each refusal exits with a non-zero code, making it safe to use in scripts.

## After closing

`git close` is a local operation. It does not push to a remote. After closing,
push manually:

```sh
git push origin main
```

To verify the close was recorded:

```sh
cargo cicd status show
```

The status output will reflect the merged branch and the updated `cicd.toml`
state.
<!-- END custom:full-doc -->
