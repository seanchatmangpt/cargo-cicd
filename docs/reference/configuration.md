# Configuration Reference

`cargo-cicd` is configured via `cicd.toml` at the workspace root. All fields
are optional; defaults are shown below.

## [status]

Controls the `status show` command.

```toml
[status]
# No user-configurable fields. State is written by cargo-cicd.
```

## [target]

Controls the `target show` and `target prune` commands.

```toml
[target]
# Prune artefacts older than this many days. Default: 14.
prune_older_than_days = 14

# If total target size exceeds this, prune is recommended. Default: 10.0 GB.
max_size_gb = 10.0
```

## [test]

Controls the `test changed` command.

```toml
[test]
# Extra flags passed to cargo test. Default: none.
extra_flags = ["--nocapture"]
```

## [trybuild]

Controls the `trybuild changed` command.

```toml
[trybuild]
# Directory containing trybuild fixtures, relative to crate root. Default: "tests/ui".
fixtures_dir = "tests/ui"
```

## [git]

Controls the `git status` and `git close` commands.

```toml
[git]
# Branch to merge into when running git close. Default: "main".
trunk_branch = "main"
```

## [publish]

Controls the `publish run` command.

```toml
[publish]
# Crates to exclude from publish. Default: none.
exclude = ["my-internal-crate"]

# If true, run in dry-run mode and do not actually publish. Default: false.
dry_run = false
```

## [workspace]

Controls the `workspace doctor` command.

```toml
[workspace]
# Maximum allowed version skew between workspace crates sharing a dependency.
# Default: 1 (allow one major version difference).
max_version_skew = 1
```

## Generated fields

The following fields are written by `cargo-cicd` and should not be edited
manually:

```toml
[status]
last_run = "2026-06-02T12:00:00Z"
dirty_files = 0
publish_ready = true

[target]
last_show_run = "2026-06-02T11:00:00Z"
size_bytes = 2_500_000_000
oldest_artifact_days = 7
```
