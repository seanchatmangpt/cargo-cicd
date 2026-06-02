---
receipt: CARGO_CICD_V26_6_2_CICD_TOML
date: 2026-06-02
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# cicd.toml Contract Receipt

## Emission
- **Command:** `cargo cicd publish run`
- **Path:** `/Users/sac/cargo-cicd/cicd.toml`
- **Emitted:** yes (verified 2026-06-02)
- **Output observed:** `published cicd.toml` + workspace summary to stdout

## Schema Sections

| Section | Fields | Purpose |
|---|---|---|
| `[workspace]` | name, toolchain, target_dir | Identity and build environment |
| `[state]` | dirty, target_size_gb, changed_files, changed_tests, changed_trybuild_fixtures | Current workspace state snapshot |
| `[target]` | max_size_gb, prune_after_days | Target directory policy |
| `[test.changed]` | enabled, base | Changed-test detection config |
| `[trybuild.changed]` | enabled, snapshot_mode | Trybuild fixture change config |
| `[git.phase]` | require_clean_tree, commit_after_phase | Phase closure enforcement |
| `[autonomic]` | enabled | Policy engine toggle |

## Sample Content (emitted 2026-06-02)

```toml
[workspace]
name = "cargo-cicd"
toolchain = "stable-aarch64-apple-darwin"
target_dir = "target"

[state]
dirty = true
target_size_gb = 1.78
changed_files = 0
changed_tests = 0
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
```

## Contract Rules
- Emitted by `cargo cicd publish run`, not manually written
- Schema stable for v26.6.2 downstream consumers
- `process-data` feature enables richer `[[events]]` output (not default)
- Does not leak private architecture or internal state machine details

## Verdict: ALIVE
