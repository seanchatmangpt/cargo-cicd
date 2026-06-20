<!-- BEGIN custom:full-doc -->
# Getting Started with cargo-cicd v26.6.19

<!-- BEGIN custom:introduction -->
`cargo-cicd` keeps Rust workspaces clean, fast, and push-ready. It is a Cargo
subcommand that provides targeted CI/CD primitives you run locally — before
you push — so you catch problems early without waiting for a remote pipeline.

**Who this tutorial is for:** Rust developers who have a Cargo workspace and
want to add local CI/CD checks to their workflow. You should be comfortable
with `cargo build` and `cargo test`.
<!-- END custom:introduction -->

<!-- BEGIN ggen:quick-commands -->
<!-- Rendered from ontology. Do not edit. -->
| Command | What it does |
|---------|-------------|
| `cargo cicd status show` | Displays the current workspace status: dirty files, pending tests, last-known trybuild result, and publish readiness. Read-only; emits a StatusShowEvent. |
| `cargo cicd target show` | Reports the size and age profile of the local Cargo target directory without modifying it. |
| `cargo cicd target prune` | Removes stale build artefacts from the Cargo target directory according to configurable age/size policy. Emits a TargetPruneEvent recording bytes freed. |
| `cargo cicd test changed` | Runs cargo test restricted to crates whose source files have changed since the last green commit. Emits a TestChangedEvent with pass/fail counts and affected crate list. |
| `cargo cicd trybuild changed` | Runs trybuild type-law fixtures for changed crates, verifying that compile-fail fixtures fail for the correct named law and compile-pass fixtures succeed. Emits a TrybuildChangedEvent. |
| `cargo cicd git status` | Surfaces a structured summary of the git working-tree state: branch, ahead/behind counts, staged/unstaged/untracked file counts, and last-commit metadata. |
| `cargo cicd git close` | Performs the lawful branch-close sequence: ensures tests pass, commits any staged evidence, merges to the trunk branch, and emits a GitCloseEvent as a receipt. |
| `cargo cicd publish run` | Publishes eligible workspace crates to crates.io after verifying all release readiness conditions are met. Emits a PublishRunEvent that the wasm4pm oracle may audit post-release. |
| `cargo cicd workspace doctor` | Diagnoses the Cargo workspace for structural health: duplicate dependencies, missing feature declarations, version skew, and toolchain mismatch. Emits a WorkspaceDoctorEvent. |
<!-- END ggen:quick-commands -->

<!-- BEGIN custom:first-run -->
## Step 1 — Install cargo-cicd

```sh
cargo install cargo-cicd --version 26.6.19
```

Verify the install:

```sh
cargo cicd --version
# cargo-cicd 26.6.19
```

If the binary is not found, ensure `~/.cargo/bin` is on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## Step 2 — Run the workspace doctor

Navigate to the root of your Cargo workspace — the directory containing the
top-level `Cargo.toml` with a `[workspace]` section — then run:

```sh
cargo cicd workspace doctor
```

The doctor checks for duplicate dependencies, missing feature declarations,
version skew across members, and toolchain mismatch. A passing run looks like:

```
workspace: my-project (3 crates)
duplicates:  none
features:    all declared
version skew: none
toolchain:   nightly-2026-05-01 (matches rust-toolchain.toml)
doctor: PASS
```

Fix any reported errors before proceeding. Warnings do not block later steps
but should be addressed before publish.

## Step 3 — Check workspace status

```sh
cargo cicd status show
```

Status reports the state of each workspace member:

```
workspace: my-project (3 crates)
branch: main (clean)
pending tests: none
trybuild: last run passed
publish ready: yes
```

If you have uncommitted changes, the count of dirty files appears here. This
command is read-only and safe to run at any time.

## Step 4 — Publish a crate

Once the workspace is in a publish-ready state:

```sh
cargo cicd publish run
```

cargo-cicd verifies release readiness conditions, then invokes the publish
sequence for eligible crates. It records a `PublishRunEvent` in `cicd.toml`
as a structured receipt.

Successful output:

```
preflight: pass
publishing my-api 26.6.19 ...
published: ok
PublishRunEvent written to cicd.toml
```

### What just happened?

Each command writes a structured record to `cicd.toml` at your workspace root.
This file accumulates the state needed to answer the question: _is this
workspace ready to push?_
<!-- END custom:first-run -->

<!-- BEGIN custom:next-steps -->
## Next steps

- **How-to guides** — learn specific tasks:
  - [Manage the target directory](../how-to/manage-target-directory.md)
  - [Close a git phase](../how-to/close-git-phase.md)
  - [Inspect workspace status](../how-to/inspect-workspace-status.md)
  - [Run changed tests](../how-to/run-changed-tests.md)
  - [Publish to crates.io](../how-to/publish-cicd-toml.md)

- **Explanation** — understand the design:
  - [Evidence emission](../explanation/evidence-emission.md)
  - [Autonomic policies](../explanation/autonomic-policies.md)
<!-- END custom:next-steps -->
<!-- END custom:full-doc -->
