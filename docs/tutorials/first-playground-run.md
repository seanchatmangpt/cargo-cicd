# Running the cargo-cicd Playground

<!-- BEGIN custom:what-playground-is -->
## What the Playground Is

The playground is a self-contained proof cell for cargo-cicd. It is a small
Rust workspace embedded in the `playground/` directory of this repository that
exercises every command in a controlled environment.

You use the playground to:

- Verify that `cargo-cicd` behaves correctly on your system after install
- Understand what each command does before running it on your real workspace
- Run the complete command matrix as a smoke test

The playground does not affect your real projects. All operations are scoped to
the `playground/` directory.
<!-- END custom:what-playground-is -->

<!-- BEGIN ggen:playground-commands -->
<!-- Rendered from ontology. Do not edit. -->
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
<!-- END ggen:playground-commands -->

<!-- BEGIN custom:walkthrough -->
## Walkthrough

### Prerequisites

- `cargo-cicd` installed (`cargo install cargo-cicd`)
- You are in the `cargo-cicd` repository root

### Step 1: Enter the playground

```sh
cd playground
```

The playground has its own `Cargo.toml` and a small set of crates designed to
trigger each command's code paths.

### Step 2: Run the status check

```sh
cargo cicd status show
```

You should see a report showing a clean workspace ready for operations.

### Step 3: Inspect the target directory

```sh
cargo cicd target show
```

If the playground has not been built yet, the target directory may not exist.
`target show` will report zero bytes.

### Step 4: Build the playground first, then run again

```sh
cargo build
cargo cicd target show
```

Now the target directory has build artefacts and you will see a size report.

### Step 5: Run the test matrix

```sh
cargo cicd test changed
```

Because all files are considered changed on a fresh checkout, this runs the
full playground test suite.

### Step 6: Run workspace doctor

```sh
cargo cicd workspace doctor
```

The playground is designed to pass all health checks. You should see a clean
bill of health.

### Step 7: Review cicd.toml

After running multiple commands, inspect the state file:

```sh
cat cicd.toml
```

You will see each command's last run time and result recorded here.

### What to do if something fails

If any command exits with a non-zero code, the error message includes the
command name and the specific check that failed. Check that:

1. You are inside the `playground/` directory (not the repo root).
2. The Rust toolchain matches the version in `playground/rust-toolchain.toml`.
3. You have run `cargo build` at least once before running test commands.
<!-- END custom:walkthrough -->
