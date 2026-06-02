---
receipt: CARGO_CICD_V26_6_2_CICD_TOML
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# cicd.toml Contract Receipt

## Schema Sections
- [workspace]: name, toolchain, target_dir
- [state]: dirty, target_size_gb, changed_files, changed_tests, changed_trybuild_fixtures
- [target]: max_size_gb, prune_after_days
- [test.changed]: enabled, base
- [trybuild.changed]: enabled, snapshot_mode
- [git.phase]: require_clean_tree, commit_after_phase
- [autonomic]: enabled, mode
- [[events]]: kind, verdict, details (optional), timestamp (optional)

## Contract Rules
- Emitted by 'cargo cicd publish', not manually written
- Schema stable for v26.6.2 downstream consumers
- Supports richer process-data output via process-data feature
- Does not leak private architecture

## Verdict: ALIVE
