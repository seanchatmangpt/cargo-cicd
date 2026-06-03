<!-- BEGIN custom:introduction -->
# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces.
<!-- END custom:introduction -->

## Install

```sh
cargo install cargo-cicd
```

## Usage

```sh
cargo cicd <command>
```

<!-- BEGIN ggen:commands -->
<!-- Rendered from ontology/cargo-cicd.ttl. Do not edit by hand. -->

| Command | Description |
|---------|-------------|
| `cargo cicd git close` | Performs the lawful branch-close sequence: ensures tests pass, commits any staged evidence, merges to the trunk branch, and emits a GitCloseEvent as a receipt. |
| `cargo cicd git status` | Surfaces a structured summary of the git working-tree state: branch, ahead/behind counts, staged/unstaged/untracked file counts, and last-commit metadata. |
| `cargo cicd publish run` | Publishes eligible workspace crates to crates.io after verifying all ALIVE release conditions are met. Emits a PublishRunEvent that the wasm4pm oracle may audit post-release. |
| `cargo cicd status show` | Displays the current workspace status: dirty files, pending tests, last-known trybuild result, and publish readiness. Read-only; emits a StatusShowEvent. |
| `cargo cicd target prune` | Removes stale build artefacts from the Cargo target directory according to configurable age/size policy. Emits a TargetPruneEvent recording bytes freed. |
| `cargo cicd target show` | Reports the size and age profile of the local Cargo target directory without modifying it. |
| `cargo cicd test changed` | Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list. |
| `cargo cicd trybuild changed` | Runs trybuild type-law fixtures for changed crates, verifying that compile-fail fixtures fail for the correct named law and compile-pass fixtures succeed. Emits a TrybuildChangedEvent. |
| `cargo cicd workspace doctor` | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, and toolchain mismatch. Emits a WorkspaceDoctorEvent. |

<!-- END ggen:commands -->

<!-- BEGIN custom:cicd-toml -->
## cicd.toml

`cargo-cicd` publishes workspace state to `cicd.toml`.
<!-- END custom:cicd-toml -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
