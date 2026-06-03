<!-- BEGIN custom:full-doc -->
# How to Manage the Target Directory

Use `target show` to inspect the Cargo target directory and `target prune` to
remove stale build artefacts.

## Inspect target directory size

```sh
cargo cicd target show
```

Reports the total size, oldest artefact age, and a breakdown by crate. This
command is read-only and safe to run at any time. It does not modify any files.

**Example output:**

```
target/
  total size:       3.1 GB
  oldest artefact:  21 days (debug/my-crate-abc123)
  newest artefact:  today
  crate count:      12
```

Run `target show` before pruning to understand exactly what will be removed.

## Prune stale artefacts

```sh
cargo cicd target prune
```

Removes artefacts older than the configured age threshold (default: 14 days).
After pruning, the command prints the bytes freed and records a
`TargetPruneEvent` in `cicd.toml`.

**Example output:**

```
pruned 847 MB (artefacts older than 14 days)
remaining: 2.3 GB
TargetPruneEvent written to cicd.toml
```

## Configure the prune policy

In `cicd.toml`, set the age threshold and optional size ceiling:

```toml
[target]
prune_older_than_days = 7   # default: 14
max_size_gb = 5.0           # prune when total exceeds this ceiling
```

When `max_size_gb` is set, `target prune` removes the oldest artefacts first
until the total falls below the ceiling, even if those artefacts are newer than
`prune_older_than_days`.

## Scope and safety

- `target prune` only removes files inside Cargo's `target/` directory.
- It never touches source files, `Cargo.lock`, `cicd.toml`, or any other
  workspace files.
- Pruning before a large rebuild can free disk space and speed up incremental
  builds by removing stale index entries that Cargo would otherwise scan.

## Resolving common issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| `target show` reports size as 0 | `target/` directory does not exist yet | Run `cargo build` first |
| `target prune` removes nothing | All artefacts are newer than the threshold | Lower `prune_older_than_days` or set `max_size_gb` |
| Disk still full after prune | Large artefacts within threshold | Set `max_size_gb` to enforce an absolute ceiling |
<!-- END custom:full-doc -->
