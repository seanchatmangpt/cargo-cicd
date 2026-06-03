# How to Run Tests for Changed Files Only

`cargo cicd test changed` runs `cargo test` scoped to crates whose source
files have changed since the last green commit. This is faster than
`cargo test --workspace` in large workspaces.

## Run the command

```sh
cargo cicd test changed
```

## How "changed" is determined

A crate is considered changed if any of its source files (`src/**/*.rs`) have
been modified, added, or deleted since the last commit recorded in `cicd.toml`
as a passing test run.

On a fresh checkout with no `cicd.toml`, all crates are considered changed and
the full workspace test suite runs.

## Example output

```
changed crates: my-parser, my-cli (2 of 5)
running tests for my-parser... ok (47 tests)
running tests for my-cli... ok (12 tests)
skipped: my-core, my-types, my-utils (unchanged)

TestChangedEvent written to cicd.toml
```

## Run with extra cargo flags

Pass additional flags after `--`:

```sh
cargo cicd test changed -- --nocapture
```

## Notes

- If tests fail, `cicd.toml` is updated to record the failure and the
  `publish ready` status is set to false.
- To force a full test run regardless of change detection, use:
  ```sh
  cargo test --workspace
  ```
- For trybuild (compile-error fixtures), see
  [run-changed-trybuild](run-changed-trybuild.md).
