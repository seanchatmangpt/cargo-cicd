# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces.

It helps keep repositories clean, target directories under control, test runs focused on what changed, and local state ready before CI runs.

> Clean less. Rebuild less. Test what changed. Push clean.

## Install

```sh
cargo install cargo-cicd
```

## Usage

```sh
cargo cicd status
cargo cicd target show
cargo cicd target prune
cargo cicd test changed
cargo cicd trybuild changed
cargo cicd git status
cargo cicd git close
cargo cicd publish
cargo cicd workspace doctor
```

## Commands

| Command | What it does |
|---|---|
| `status` | Show workspace state (toolchain, target size, dirty files, changed file count) |
| `target show` | Show target directory size and artifact breakdown |
| `target prune` | Prune stale target artifacts safely (dry-run by default) |
| `test changed` | Run only tests for changed source files |
| `trybuild changed` | Run only trybuild fixtures for changed test files |
| `git status` | Show git phase state |
| `git close` | Enforce clean phase closure — refuses to hide dirty files |
| `publish` | Emit `cicd.toml` with current workspace state |
| `workspace doctor` | Diagnose workspace health and suggest autonomic actions |

## cicd.toml

`cargo cicd publish` records workspace state into `cicd.toml`:

```toml
[workspace]
name = "my-project"
toolchain = "nightly-2026-04-15"

[state]
target_size_gb = 3.15
dirty = false
changed_files = 0
```

## Features

| Feature | Default | Purpose |
|---|:---:|---|
| `process-data` | no | Enables process data emission |
| `autonomic` | no | Adds autonomic policy suggestions |
| `wasm4pm` | no | Enables wasm4pm integration (requires wpm binary) |

## License

MIT OR Apache-2.0
