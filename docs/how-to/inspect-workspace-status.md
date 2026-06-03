# How to Inspect Workspace Status

Use `cargo cicd status show` to get a structured summary of your workspace's
current state without running any tests or modifying any files.

## When to use this

- Before starting work on a new feature, to confirm the workspace is clean
- After a git pull, to see what changed
- Before running other commands, to understand the baseline state

## Run the command

```sh
cargo cicd status show
```

## What it reports

| Field | Meaning |
|-------|---------|
| workspace | Workspace name and crate count |
| branch | Current git branch |
| dirty files | Files with uncommitted changes |
| pending tests | Crates not yet tested against current source |
| last trybuild | Pass/fail status of the last trybuild run |
| publish ready | Whether all crates satisfy publish conditions |

## Example output

```
workspace: my-project (5 crates)
branch: feat/new-parser (2 commits ahead of main)
dirty files: 3
pending tests: src/parser.rs (modified)
last trybuild: passed (2 hours ago)
publish ready: no (dirty files present)
```

## Notes

- `status show` is read-only. It does not run tests or modify `cicd.toml`.
- "Publish ready" is false whenever there are dirty files or pending tests,
  even if the last run of each command passed.
- For a complete health check that diagnoses structural issues, see
  [workspace doctor](run-changed-tests.md).
