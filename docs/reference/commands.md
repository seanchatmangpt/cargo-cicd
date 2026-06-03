# Command Reference

<!-- BEGIN ggen:commands -->
<!-- Rendered from ontology. Do not edit. -->
| Command | Noun | Verb | Description |
|---------|------|------|-------------|
| `cargo cicd git close` | git | close | Performs the lawful branch-close sequence: ensures tests pass, commits any staged evidence, merges to the trunk branch, and emits a GitCloseEvent as a receipt. |
| `cargo cicd git status` | git | status | Surfaces a structured summary of the git working-tree state: branch, ahead/behind counts, staged/unstaged/untracked file counts, and last-commit metadata. |
| `cargo cicd publish run` | publish | run | Publishes eligible workspace crates to crates.io after verifying all release readiness conditions are met. Emits a PublishRunEvent that the wasm4pm oracle may audit post-release. |
| `cargo cicd status show` | status | show | Displays the current workspace status: dirty files, pending tests, last-known trybuild result, and publish readiness. Read-only; emits a StatusShowEvent. |
| `cargo cicd target prune` | target | prune | Removes stale build artefacts from the Cargo target directory according to configurable age/size policy. Emits a TargetPruneEvent recording bytes freed. |
| `cargo cicd target show` | target | show | Reports the size and age profile of the local Cargo target directory without modifying it. |
| `cargo cicd test changed` | test | changed | Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list. |
| `cargo cicd trybuild changed` | trybuild | changed | Runs trybuild type-law fixtures for changed crates, verifying that compile-fail fixtures fail for the correct named law and compile-pass fixtures succeed. Emits a TrybuildChangedEvent. |
| `cargo cicd workspace doctor` | workspace | doctor | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, and toolchain mismatch. Emits a WorkspaceDoctorEvent. |
<!-- END ggen:commands -->

## Command structure

All commands follow the `cargo cicd <noun> <verb>` pattern. When a noun has
only one verb, the verb may be omitted (e.g., `cargo cicd status` is equivalent
to `cargo cicd status show`).

## Global flags

| Flag | Description |
|------|-------------|
| `--help` | Print help for a command |
| `--version` | Print cargo-cicd version |
| `--cicd-toml <path>` | Use a different `cicd.toml` path (default: workspace root) |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Command failed (check stderr for details) |
| 2 | Workspace not found or invalid |
| 3 | Readiness check failed (workspace not ready for requested operation) |
