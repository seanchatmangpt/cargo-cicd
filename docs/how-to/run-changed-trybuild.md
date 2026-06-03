# How to Run Trybuild Fixtures for Changed Crates

`cargo cicd trybuild changed` runs trybuild type-law fixtures scoped to
crates whose source files have changed. It verifies that compile-fail fixtures
produce the correct error and compile-pass fixtures succeed.

## What is trybuild?

[trybuild](https://crates.io/crates/trybuild) is a testing tool for crates
that export compile-time type laws. Each fixture is a small Rust file that
should either compile successfully or fail with a specific error message.

## Run the command

```sh
cargo cicd trybuild changed
```

## How changed detection works

Same as `test changed`: a crate is considered changed if any source file has
been modified since the last passing trybuild recorded in `cicd.toml`.

## Example output

```
changed crates: my-derive (1 of 3 trybuild crates)
running trybuild for my-derive...
  compile-pass: 4 fixtures ok
  compile-fail: 6 fixtures ok (errors matched)
  compile-fail: 0 fixtures failed

TrybuildChangedEvent written to cicd.toml
```

## When a fixture fails

If a compile-fail fixture does not produce the expected error, trybuild prints
a diff:

```
FAILED: tests/ui/wrong_type.rs
expected: error[E0308]: mismatched types
     got: error[E0277]: the trait bound ...
```

Update the `.stderr` file next to the fixture to match the new expected output.

## Notes

- Trybuild fixtures live in `tests/ui/` by convention.
- If a crate has no trybuild fixtures, it is silently skipped.
- `trybuild changed` only runs for crates that have a `tests/ui/` directory
  or a `build.rs` that invokes trybuild.
