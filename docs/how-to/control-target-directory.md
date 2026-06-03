# How to Control the Target Directory

`cargo-cicd` provides two commands for managing the Cargo target directory:
`target show` to inspect it and `target prune` to clean it up.

## Inspect target directory size

```sh
cargo cicd target show
```

Reports the total size, oldest artefact age, and a breakdown by crate. This
is read-only and safe to run at any time.

**Example output:**

```
target/
  total size:       3.1 GB
  oldest artefact:  21 days (debug/my-crate-abc123)
  newest artefact:  today
  crate count:      12
```

## Prune stale artefacts

```sh
cargo cicd target prune
```

Removes artefacts older than the configured threshold (default: 14 days).
The command prints how many bytes were freed and records a `TargetPruneEvent`
in `cicd.toml`.

**Example output:**

```
pruned 847 MB (artefacts older than 14 days)
remaining: 2.3 GB
```

## Configure prune policy

In `cicd.toml`, set the prune threshold:

```toml
[target]
prune_older_than_days = 7   # default: 14
max_size_gb = 5.0           # prune when total exceeds this
```

## Notes

- `target prune` only removes artefacts from Cargo's `target/` directory.
  It does not touch source files, `Cargo.lock`, or any other workspace files.
- Running `target prune` before a large build can free disk space and speed
  up incremental builds by removing stale index entries.
- Use `target show` first to understand what will be removed before pruning.
