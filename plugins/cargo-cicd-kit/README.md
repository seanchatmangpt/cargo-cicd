# cargo-cicd-kit

A Claude Code plugin for **cargo-cicd** Rust workspaces.

## What this plugin provides

| Asset | Type | Purpose |
|---|---|---|
| `/cicd-status` | Slash command | Run `cargo cicd status` and `cargo cicd workspace doctor`, then summarise workspace health |
| `cicd-doctor` | Subagent | Full CI/CD readiness diagnostic: status, target, git, workspace, and evidence checks |
| `workspace-clean` | Skill | Safely prune the `target/` directory via `cargo cicd target show` then `cargo cicd target prune` |
| `hooks.json` | SessionStart hook | Prints a one-line "cargo-cicd-kit loaded" banner at session start |

## Install from the marketplace

```sh
# 1. Register the marketplace (run once per project)
/plugin marketplace add ./cargo-cicd-marketplace

# 2. Install the kit
/plugin install cargo-cicd-kit
```

## Usage

```
/cicd-status             # quick workspace health snapshot
/cicd-doctor             # deep CI/CD readiness report (via subagent)
/workspace-clean         # dry-run target prune
/workspace-clean --apply # execute the prune
```

## Requirements

- `cargo-cicd` binary reachable on `$PATH` (or installed with `cargo install cargo-cicd`)
- Rust toolchain matching the workspace's `rust-toolchain.toml`
