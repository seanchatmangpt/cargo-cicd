# cicd.toml Schema Reference

`cicd.toml` is written to the workspace root by `cargo-cicd`. It serves two
purposes:

1. **Configuration** — user-controlled settings read at command startup.
2. **State** — machine-written records of last-run results.

Do not commit `cicd.toml` to source control. Add it to `.gitignore`.

## Full schema

```toml
# ── Workspace identity ────────────────────────────────────────────────────────

[workspace]
# Name of the workspace (read from root Cargo.toml). Written by cargo-cicd.
name = "my-project"

# Rust toolchain string. Written by cargo-cicd from rust-toolchain.toml.
toolchain = "stable-aarch64-apple-darwin"

# Path to Cargo target directory. Default: "target".
target_dir = "target"

# ── Runtime state ─────────────────────────────────────────────────────────────

[state]
# true if there are uncommitted changes. Written by cargo-cicd.
dirty = false

# Total size of the target directory in GB. Written by cargo-cicd.
target_size_gb = 2.5

# Count of files changed since last recorded green commit. Written by cargo-cicd.
changed_files = 0

# Count of crates with changed tests. Written by cargo-cicd.
changed_tests = 0

# Count of crates with changed trybuild fixtures. Written by cargo-cicd.
changed_trybuild_fixtures = 0

# ── Target directory policy ───────────────────────────────────────────────────

[target]
# Warn if target directory exceeds this size. Default: 10.0 GB.
max_size_gb = 10.0

# Prune artefacts older than this many days. Default: 14.
prune_after_days = 14

# ── Test policy ───────────────────────────────────────────────────────────────

[test.changed]
# Enable changed-file scoping for test runs. Default: true.
enabled = true

# Git ref to compare against when detecting changed files. Default: "origin/main".
base = "origin/main"

# ── Trybuild policy ───────────────────────────────────────────────────────────

[trybuild.changed]
# Enable changed-file scoping for trybuild runs. Default: true.
enabled = true

# Snapshot mode: "changed-only" | "all". Default: "changed-only".
snapshot_mode = "changed-only"

# ── Git phase policy ──────────────────────────────────────────────────────────

[git.phase]
# Require a clean working tree before closing a branch. Default: true.
require_clean_tree = true

# Automatically commit staged evidence files after a phase. Default: false.
commit_after_phase = false

# ── Feature: autonomic ────────────────────────────────────────────────────────
# Requires the `autonomic` feature flag at compile time.

[autonomic]
# Enable autonomic policy suggestions. Default: false.
enabled = false

# Mode: "suggest" | "apply". Default: "suggest".
mode = "suggest"

# ── Events log ────────────────────────────────────────────────────────────────
# Appended by cargo-cicd. Do not edit.

[[events]]
kind = "status"
verdict = "pass"
timestamp = "2026-06-02T12:00:00Z"
```

## Field summary

| Section | Field | Type | Default | Writable by user |
|---------|-------|------|---------|-----------------|
| workspace | name | string | (from Cargo.toml) | no |
| workspace | toolchain | string | (from rust-toolchain.toml) | no |
| workspace | target_dir | string | "target" | yes |
| state | dirty | bool | — | no |
| state | target_size_gb | float | — | no |
| target | max_size_gb | float | 10.0 | yes |
| target | prune_after_days | int | 14 | yes |
| test.changed | enabled | bool | true | yes |
| test.changed | base | string | "origin/main" | yes |
| trybuild.changed | enabled | bool | true | yes |
| trybuild.changed | snapshot_mode | string | "changed-only" | yes |
| git.phase | require_clean_tree | bool | true | yes |
| git.phase | commit_after_phase | bool | false | yes |
| autonomic | enabled | bool | false | yes |
| autonomic | mode | string | "suggest" | yes |
