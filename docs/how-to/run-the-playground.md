# How to Use the Playground

The playground is a self-contained Rust workspace in the `playground/`
directory that exercises every `cargo-cicd` command in a controlled
environment.

## Navigate to the playground

```sh
cd playground
```

## Run a specific command

All `cargo cicd` commands work the same in the playground as in your real
workspace:

```sh
cargo cicd status show
cargo cicd target show
cargo cicd test changed
cargo cicd workspace doctor
```

## Run the full command matrix

A shell script runs all commands in sequence:

```sh
bash scripts/run-matrix.sh
```

This script exercises every command and verifies exit codes. It is used in the
cargo-cicd CI pipeline to confirm all commands work correctly.

## Reset the playground state

If you want to start from a clean slate:

```sh
rm -f cicd.toml
cargo clean
```

Then re-run any commands you want to test.

## Add a new scenario

The playground's `scenarios/` directory contains TOML files describing
preconditions and expected outcomes for each command. To add a new scenario:

1. Create a new file in `playground/scenarios/`.
2. Run `bash scripts/run-matrix.sh` to verify it works.

## Notes

- The playground does not publish to crates.io. The `publish run` scenario
  runs in dry-run mode only.
- Changes to files inside `playground/` do not affect your real projects.
