# cargo cicd target show / prune

Show and safely manage the `target/` directory.

## Commands

### show

```bash
cargo cicd target show
```

Reports:

- Path to the target directory
- Total size in GB
- Configured maximum (default 20 GB)
- Verdict: `pass` / `warn` / `fail`
- Recommendation to prune if the verdict is not `pass`

### prune

```bash
cargo cicd target prune
```

Plans a cleanup of `target/`. The default mode is suggest-only: it prints what would be removed without deleting anything. Candidate directories are:

- `target/debug/incremental`
- `target/debug/.fingerprint`
- `target/debug/deps`

To preview the estimated space savings without deleting:

```bash
cargo cicd target prune
```

The output includes the current size, the list of candidate directories with their sizes, and the total space that would be freed.

Release artifacts (`target/release`) are never deleted automatically.

```
--apply is not yet available. All prune runs are currently suggest-only.
```

## Verdict thresholds

| Verdict | Condition |
|---------|-----------|
| `pass` | Size is below 70% of the configured maximum |
| `warn` | Size is between 70% and 100% of the maximum |
| `fail` | Size meets or exceeds the maximum |

## When to use it

- Before a long CI build, to confirm there is headroom in the target directory.
- As a weekly maintenance step to reclaim disk space from incremental build artifacts.

## Example output

```bash
$ cargo cicd target show
target directory: target
total size:       4.31 GB
max configured:   20.0 GB
verdict:          pass

$ cargo cicd target prune
target prune plan
=================
current size: 4.31 GB
mode:         suggest (use --apply to execute)

suggested candidates:
  target/debug/incremental (1.20 GB)
  target/debug/.fingerprint (0.34 GB)
  target/debug/deps (2.10 GB)

to execute: cargo cicd target prune --apply
note: release artifacts are never deleted automatically
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Command completed (both show and prune always exit 0) |

## Three-tier architecture

Understanding the tier structure explains why `prune` is safe by default and why `show` has no side effects.

### Tier 1 — Presentation (`TargetNoun`, `TargetShowVerb`, `TargetPruneVerb`)

`TargetNoun` implements `NounCommand`. It declares the noun name (`"target"`) and registers two verbs: `TargetShowVerb` and `TargetPruneVerb`.

Each verb implements `VerbCommand`. The verb's `run` method receives parsed `VerbArgs`, calls the relevant adapter functions, and formats terminal output. No scanning or filesystem mutation happens inside a verb directly.

**Why this matters:** Adding a third verb (e.g. `target stats`) means writing a new struct that implements `VerbCommand`. No existing logic changes.

### Tier 2 — Integration (adapter wiring in `VerbCommand::run`)

`TargetShowVerb::run` calls:
- `TargetScannerAdapter::total_size_gb("target")` — computes total size.
- `TargetScannerAdapter::verdict(size_gb, 20.0)` — computes the verdict string.

`TargetPruneVerb::run` calls `total_size_bytes` on each candidate subdirectory to compute how much space would be freed. It does not delete any files — deletion is gated behind `--apply`, which is not yet wired. The prune verb's current behavior is always suggest-only.

This is the only layer that knows which adapter functions to call and in what order.

### Tier 3 — Domain logic (pure adapter functions)

`TargetScannerAdapter::total_size_bytes(dir: &str) -> u64` walks a directory tree using `walkdir` and sums file sizes. It takes a path, returns a number. No `println!`, no `process::exit`, no mutation.

`TargetScannerAdapter::total_size_gb(dir: &str) -> f64` converts bytes to gigabytes. Pure function.

`TargetScannerAdapter::verdict(size_gb: f64, max_gb: f64) -> &'static str` encodes the threshold logic. Pure function — testable in a unit test with no filesystem.

**Why this matters:** The show path has zero side effects below the verb layer. The prune path's destructive step (deletion) is explicitly gated and not yet reachable, so it cannot be triggered by accident. You can test every verdict threshold with a table-driven unit test without touching disk.

## Notes

- For a full clean, use `cargo clean` directly. `prune` is for incremental maintenance of specific artifact subdirectories.
- The `target_pressure` autonomic policy surfaces the same `warn`/`fail` signal in suggest mode and will recommend running `cargo cicd target prune`.
