# cargo-cicd

cargo-cicd is a local-first CI/CD helper for Rust workspaces.

It helps keep repositories clean, target directories under control, test runs focused
on what changed, and local state ready before CI runs.

> Clean less. Rebuild less. Test what changed. Push clean.

## Install

```bash
cargo install cargo-cicd
```

## Usage

### Workspace status

```bash
cargo cicd status
```

Shows toolchain, target directory size, git state, and overall workspace health.

### Target directory

```bash
cargo cicd target show      # show current size, max, and verdict
cargo cicd target prune     # plan cleanup (prints plan only; use --apply to act)
```

`prune` without `--apply` is always safe — it only prints what would be removed.

### Changed tests

```bash
cargo cicd test changed
```

Reads `git diff` against the configured base branch, classifies changed Rust files,
and runs only the tests relevant to what changed. If exact selection is not possible,
emits a conservative plan and explains why.

### Trybuild fixtures

```bash
cargo cicd trybuild changed
```

Selects and runs only the trybuild fixtures that changed, instead of the full fixture
estate. Useful for compile-fail / compile-pass fixture suites where full runs are slow.

### Git phase closure

```bash
cargo cicd git status       # show branch, dirty files, staged, untracked
cargo cicd git close        # refuse closure if working tree is dirty
```

`git close` exits non-zero if the tree is dirty. It does not commit, stash, or hide
files — it only checks and reports. Use it as a phase gate before CI runs.

### Publish workspace state

```bash
cargo cicd publish
```

Emits `cicd.toml` at the workspace root with current state: toolchain, target size,
changed file counts, git phase, and an event record. Use `cicd.toml` to carry state
between CI steps or to audit workspace condition at a point in time.

### Workspace diagnostics

```bash
cargo cicd workspace doctor
```

Checks Cargo.toml, workspace members, git state, toolchain, rust-toolchain.toml
match, and cicd.toml presence. Exits 0 if all checks pass; 1 if any check fails.
Warnings (missing cicd.toml, missing rust-toolchain.toml) do not cause failure.

## cicd.toml

`cargo cicd publish` writes `cicd.toml` at the workspace root:

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

[test.changed]
enabled = true
base = "origin/main"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
mode = "suggest"

[[events]]
kind = "status"
verdict = "pass"
```

### Schema

| Key | Description |
|-----|-------------|
| `workspace.name` | Workspace name from root `Cargo.toml` |
| `workspace.toolchain` | Active Rust toolchain |
| `workspace.target_dir` | Path to the target directory |
| `state.dirty` | Whether the working tree has uncommitted changes |
| `state.target_size_gb` | Current target directory size in GB |
| `state.changed_files` | Files changed relative to the base branch |
| `state.changed_tests` | Test files among the changed set |
| `state.changed_trybuild_fixtures` | Trybuild fixtures among the changed set |
| `target.max_size_gb` | Configured max target size before warn/fail |
| `target.prune_after_days` | Age threshold for prune candidates (days) |
| `test.changed.base` | Git ref used as the diff base for test selection |
| `trybuild.changed.snapshot_mode` | `changed-only` or `all` |
| `git.phase.require_clean_tree` | Whether `git close` enforces a clean tree |
| `autonomic.mode` | Policy mode (`suggest` is the only public mode) |
| `events` | Array of event records from this publish run |

## Autonomic CI/CD policies

When the `autonomic` feature is enabled, cargo-cicd evaluates suggest-mode policies
against the current workspace state. Policies never take automatic action — they emit
human-readable recommendations.

| Policy | Signal | Recommendation |
|--------|--------|----------------|
| `target_pressure` | `target/` exceeds `max_size_gb` | run `cargo cicd target prune` |
| `toolchain_mismatch` | active toolchain differs from `rust-toolchain.toml` | switch toolchain |
| `trybuild_changed` | trybuild fixtures changed in diff | run `cargo cicd trybuild changed` |
| `git_phase_dirty` | working tree dirty before CI | commit or stash before pushing |

Suggestions appear in the output of `cargo cicd status` and `cargo cicd publish` when
the `autonomic` feature is enabled and `autonomic.mode = "suggest"`.

## Feature flags

| Feature | Default | Description |
|---------|:-------:|-------------|
| `default` | on | Core CI/CD helpers (status, target, test, trybuild, git, publish, workspace) with no extra dependencies |
| `process-data` | off | Richer event records in `cicd.toml`; enables structured process data in publish output |
| `autonomic` | off | Suggest-mode CI/CD policies; requires `process-data` |
| `contrib` | off | Downstream adapter support for extending cargo-cicd with project-local policies; requires `process-data` |

Enable features at install time:

```bash
cargo install cargo-cicd --features autonomic
```

## Documentation

Per-command reference in `docs/commands/`:

- [status.md](docs/commands/status.md)
- [target.md](docs/commands/target.md)
- [test.md](docs/commands/test.md)
- [trybuild.md](docs/commands/trybuild.md)
- [git.md](docs/commands/git.md)
- [publish.md](docs/commands/publish.md)
- [workspace.md](docs/commands/workspace.md)

## License

MIT OR Apache-2.0
