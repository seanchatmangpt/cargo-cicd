# cargo cicd target show / prune

Show and safely manage the `target/` directory.

## Commands

### show

```bash
cargo cicd target show
```

Reports:

- Current size of `target/` in GB
- Configured maximum (`max_size_gb` from `cicd.toml`)
- Verdict: `pass` / `warn` / `fail`
- Number of incremental build artifacts older than `prune_after_days`

### prune

```bash
cargo cicd target prune
```

Plans a cleanup of `target/`. In plan mode (default), prints what would be removed without deleting anything.

To apply the plan:

```bash
cargo cicd target prune --apply
```

Prune removes build artifacts older than `prune_after_days` and respects the `max_size_gb` threshold. It does not delete the full `target/` directory.

## Configuration

In `cicd.toml`:

```toml
[target]
max_size_gb = 20
prune_after_days = 14
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Target within configured limits |
| 1 | Target exceeds max_size_gb (fail verdict) |

## Notes

- `prune` without `--apply` is always safe — it only prints a plan.
- For a full clean, use `cargo clean` directly. `prune` is for incremental maintenance.
- The `target_pressure` autonomic policy will suggest `target prune` when the threshold is exceeded.
