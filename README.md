# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces.

It helps keep repositories clean, target directories under control, test runs focused on what changed, and local state ready before CI runs.

## Install

```sh
cargo install cargo-cicd
```

## Usage

`cargo-cicd` exposes a Cargo external subcommand. After install, use it as:

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

### `cargo cicd status`

Show workspace CI/CD status: toolchain, target directory size, git branch, dirty file count.

### `cargo cicd target show`

Show target directory size and state versus configured limits.

### `cargo cicd target prune`

Remove stale artifacts from the target directory. Use `--dry-run` to preview.

### `cargo cicd test changed`

Identify and run tests for files changed relative to a base branch.

### `cargo cicd trybuild changed`

Run only changed trybuild fixtures rather than the full suite.

### `cargo cicd git status`

Show git branch, dirty files, and untracked files.

### `cargo cicd git close`

Stage, commit, and clean up the current git phase.

### `cargo cicd publish`

Emit `cicd.toml` with current workspace state.

### `cargo cicd workspace doctor`

Run workspace health checks and emit autonomic suggestions.

## cicd.toml

`cargo-cicd` can publish local CI/CD process data into `cicd.toml`.

This records workspace state, target size, changed files, selected checks, git phase state, and command events.

Example:

```toml
[workspace]
name = "my-crate"
toolchain = "stable-aarch64-apple-darwin"
target_dir = "target"

[state]
dirty = false
target_size_gb = 1.24

[target]
max_size_gb = 20
prune_after_days = 14
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
