# cargo cicd publish

Emit `cicd.toml` with current workspace state.

## Usage

```bash
cargo cicd publish
```

## What it writes

`cargo cicd publish` writes (or overwrites) `cicd.toml` at the workspace root with a snapshot of current state:

```toml
[workspace]
name = "my-workspace"
toolchain = "nightly"
target_dir = "target"

[state]
dirty = false
target_size_gb = 4.2
changed_files = 3
changed_tests = 1
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[autonomic]
enabled = true
mode = "suggest"

[[events]]
kind = "status"
verdict = "pass"
```

## Fields

| Field | Description |
|-------|-------------|
| `workspace.name` | Workspace name from root `Cargo.toml` |
| `workspace.toolchain` | Active Rust toolchain |
| `workspace.target_dir` | Path to target directory |
| `state.dirty` | Whether the working tree has uncommitted changes |
| `state.target_size_gb` | Current target directory size in GB |
| `state.changed_files` | Files changed relative to base branch |
| `state.changed_tests` | Test files among the changed set |
| `state.changed_trybuild_fixtures` | Trybuild fixtures among the changed set |
| `target.max_size_gb` | Configured max target size |
| `target.prune_after_days` | Prune threshold in days |
| `autonomic.enabled` | Whether autonomic policies are enabled |
| `autonomic.mode` | Policy mode (`suggest` only by default) |
| `events` | Event records from this publish run |

## Use in CI

Commit or cache `cicd.toml` to carry workspace state between CI steps. Downstream steps can read it to skip redundant work.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | cicd.toml written successfully |
| 1 | Write failed |
