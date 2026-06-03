<!-- BEGIN custom:introduction -->
# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper that runs before CI, not inside it.
Nine focused commands give you Rust workspace hygiene on your machine — catching
failures at the keyboard, not in a remote pipeline twenty minutes later.

Rust workspaces accumulate problems quietly: the `target/` directory balloons
to gigabytes of stale artefacts, `cargo test` touches every crate when only one
file changed, and git state drifts in ways that only surface when CI finally
sees it. Each of these is a tax on developer time that compounds across a team.

`cargo-cicd` pays that tax down incrementally. You get clean target directories
on a configurable prune policy, focused test and trybuild runs scoped to changed
crates, a phase-aware git workflow that closes branches lawfully, and a
`cicd.toml` snapshot that lets every command skip redundant work and report
accurate readiness without re-running everything from scratch.
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
| `cargo cicd publish run` | Publishes eligible workspace crates to crates.io after verifying all release readiness conditions are met. Emits a PublishRunEvent that the wasm4pm oracle may audit post-release. |
| `cargo cicd status show` | Displays the current workspace status: dirty files, pending tests, last-known trybuild result, and publish readiness. Read-only; emits a StatusShowEvent. |
| `cargo cicd target prune` | Removes stale build artefacts from the Cargo target directory according to configurable age/size policy. Emits a TargetPruneEvent recording bytes freed. |
| `cargo cicd target show` | Reports the size and age profile of the local Cargo target directory without modifying it. |
| `cargo cicd test changed` | Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list. |
| `cargo cicd trybuild changed` | Runs trybuild type-law fixtures for changed crates, verifying that compile-fail fixtures fail for the correct named law and compile-pass fixtures succeed. Emits a TrybuildChangedEvent. |
| `cargo cicd workspace doctor` | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, and toolchain mismatch. Emits a WorkspaceDoctorEvent. |

<!-- END ggen:commands -->

<!-- BEGIN custom:cicd-toml -->
## cicd.toml

After each command runs, `cargo-cicd` writes a `cicd.toml` snapshot to the
workspace root. The file is a structured record of the last-known result for
every command surface: which crates were tested, what the target directory
weighed, whether trybuild fixtures passed, and whether the workspace is
publish-ready. Subsequent commands read this snapshot to skip work that has
not been invalidated rather than re-running from scratch.

```toml
[workspace]
root    = "/home/user/myproject"
toolchain = "nightly-2026-05-01"

[test_changed]
last_run    = "2026-06-02T11:43:00Z"
crates      = ["my-core", "my-cli"]
passed      = 47
failed      = 0

[trybuild_changed]
last_run    = "2026-06-02T11:43:12Z"
crates      = ["my-core"]
pass_count  = 12
fail_count  = 0

[target]
size_bytes          = 2_340_000_000
oldest_artifact_days = 9
last_pruned         = "2026-06-01T08:00:00Z"

[git]
branch       = "feat/my-feature"
ahead        = 2
dirty_files  = 0
publish_ready = true
```

`cicd.toml` is a local machine artefact. Add it to `.gitignore` and do not
commit it to source control.
<!-- END custom:cicd-toml -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
