# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces.

It helps keep repositories clean, target directories under control, test runs focused on what changed, and local state ready before CI runs.

## Slogan

> Clean less. Rebuild less. Test what changed. Push clean.

## Installation

```bash
cargo install cargo-cicd
```

## Commands

### Workspace status

```bash
cargo cicd status
```

Shows toolchain, target directory size, git state, and overall workspace health.

### Target directory management

```bash
cargo cicd target show      # show size, verdict, max configured
cargo cicd target prune     # plan cleanup (safe by default, no accidental deletes)
```

### Changed tests

```bash
cargo cicd test changed     # run tests for changed files only
```

Reads `git diff` against the configured base branch, classifies changed Rust files, and runs only the relevant tests. If exact selection is not possible, emits a conservative plan and says why.

### Changed trybuild fixtures

```bash
cargo cicd trybuild changed  # run trybuild for changed fixtures only
```

Avoids running the entire fixture estate by default. Selects only fixtures that changed.

### Git phase management

```bash
cargo cicd git status       # show branch, dirty files, staged, untracked
cargo cicd git close        # enforce phase closure — refuses dirty trees
```

`git close` enforces that no phase claims closed while the tree is dirty. It does not hide unrelated files.

### Publish workspace state

```bash
cargo cicd publish
```

Emits `cicd.toml` with current workspace state: toolchain, target size, changed files, git state, and event history. Use this as the carrier for CI/CD state.

### Workspace diagnostics

```bash
cargo cicd workspace doctor
```

Checks Cargo.toml, git state, toolchain, and cicd.toml presence.

## cicd.toml

`cargo cicd publish` emits a `cicd.toml` file:

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

## Autonomic CI/CD policies

`cargo-cicd` ships with four suggest-mode policies:

| Policy | Signal | Recommendation |
|--------|--------|----------------|
| `target_pressure` | target/ exceeds configured threshold | run target prune |
| `toolchain_mismatch` | active toolchain doesn't match rust-toolchain.toml | switch toolchain |
| `trybuild_changed` | trybuild fixtures changed | run trybuild changed |
| `git_phase_dirty` | working tree dirty before CI | commit or stash |

Policies are `suggest` mode only. No automatic application by default.

## Feature flags

```toml
[features]
default = []                          # useful local CI/CD helper
process-data = []                     # richer cicd.toml process records
autonomic = ["process-data"]          # suggestion/planning policies
contrib = ["process-data"]            # downstream adapter support
```

## License

MIT
