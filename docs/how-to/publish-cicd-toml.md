# How to Publish Crates to crates.io

`cargo cicd publish run` publishes eligible workspace crates to crates.io
after verifying all release readiness conditions are met.

## Prerequisites

Before running `publish run`:

1. You must be logged in to crates.io (`cargo login`).
2. All tests must pass.
3. The workspace must be clean (no dirty files).
4. Crate versions must be bumped appropriately.

## Run the command

```sh
cargo cicd publish run
```

## What it does

1. Checks workspace readiness via `status show`.
2. Identifies crates with version bumps not yet published to crates.io.
3. Publishes each eligible crate in dependency order.
4. Emits a `PublishRunEvent` recorded in `cicd.toml`.

## Example output

```
readiness check: pass
eligible crates: my-core@1.2.0, my-cli@2.0.0
publishing my-core@1.2.0... ok
publishing my-cli@2.0.0... ok

PublishRunEvent written to cicd.toml
```

## Dry run first

To see what would be published without actually publishing:

```sh
cargo cicd publish run --dry-run
```

## If publish fails

| Failure | Cause | Fix |
|---------|-------|-----|
| Not logged in | Missing crates.io token | Run `cargo login` |
| Version already exists | Version not bumped | Bump version in `Cargo.toml` |
| Failed dependency check | Dependency not yet published | Publish dependencies first |
| Dirty workspace | Uncommitted changes | Commit changes first |

## Notes

- Crates are published in topological order (dependencies before dependents).
- If any crate fails to publish, the command stops and reports the error.
  Already-published crates in the same run are not rolled back.
- The `cicd.toml` `publish_ready` field is set to `false` after a failed
  publish until the next successful run.
