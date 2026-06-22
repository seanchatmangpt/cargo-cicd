---
description: Inspect build artifacts with target show, preview what target prune would free, and explain the --apply flag.
allowed-tools: Bash, Read
---

Trigger: user wants to reclaim disk space from the Rust build cache.

## Steps

```bash
cargo cicd target show
cargo cicd target prune
```

1. Run `cargo cicd target show` → capture total disk usage, stale artifacts, and whether `target/cargo-cicd/evidence/` is in inventory.
2. Run `cargo cicd target prune` (dry-run by default) → capture reclaimable bytes by artifact kind.
3. Present dry-run output to user. Do NOT run `--apply` without explicit user confirmation.
4. When user confirms:
   ```bash
   cargo cicd target prune --apply
   ```
   This is irreversible.

## Facts

- `target/cargo-cicd/evidence/` is NEVER touched by prune, even with `--apply`. Evidence files require manual removal after `cargo cicd evidence doctor` review.
- `target/release/` artifacts are NOT auto-deleted. Require `--apply --include-release` to remove.
- With `[autonomic]` in `cicd.toml`, prune defaults to suggest mode. `--apply` overrides for the single invocation.
- Check active autonomic mode: `cargo cicd status show` → `PolicyState` section.
