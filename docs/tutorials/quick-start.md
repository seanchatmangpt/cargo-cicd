# cargo-cicd Quick Start

Welcome to cargo-cicd! This guide gets you up and running in 5 minutes.

## What is cargo-cicd?

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces. It keeps your workspace clean, fast, and push-ready by running targeted checks before you push to remote. Think of it as a local pre-flight checklist for your repository.

**Version:** 26.6.19

## Installation

```sh
cargo install cargo-cicd
```

Verify installation:

```sh
cargo cicd --version
# cargo-cicd 26.6.19
```

If the command is not found, ensure `~/.cargo/bin` is on your `$PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

## Basic Workflow

### 1. Check Workspace Health

Run this first to diagnose your workspace:

```sh
cargo cicd workspace
```

Output shows:
- Cargo.toml existence
- Active toolchain
- Git repository status
- cicd.toml state
- Autonomic policy checks

### 2. View Current Status

Get a quick snapshot of your workspace:

```sh
cargo cicd status
```

Output includes:
- Active toolchain
- Target directory size and verdict
- Current branch
- Dirty and untracked files
- Git cleanliness

### 3. Check Target Directory

See how much space the target directory is using:

```sh
cargo cicd target show
```

If the target is too large (>20 GB by default), prune it:

```sh
cargo cicd target prune        # Dry-run (shows what would be deleted)
cargo cicd target prune --apply # Actually delete artifacts
```

### 4. Run Changed Tests

Before committing, test only the files you changed:

```sh
cargo cicd test changed
```

This runs tests for changed source files only, saving time during development.

### 5. Check Git State

Verify your git tree is clean before pushing:

```sh
cargo cicd git status
```

If there are uncommitted changes, review them and stage/commit manually. Then verify:

```sh
cargo cicd git close
```

### 6. Publish Workspace State

Create a `cicd.toml` snapshot of your current workspace:

```sh
cargo cicd publish
```

This records:
- Workspace name and toolchain
- Target directory size
- Changed file counts
- Dirty file status
- Test/fixture counts

## Command Syntax

All commands follow this pattern:

```sh
cargo cicd <noun> [verb] [options]
```

**Nouns** are the main subjects (status, target, test, git, workspace, publish, etc.).
**Verbs** are the actions (show, prune, changed, close, run, etc.).

When a noun has only one main verb, you can omit the verb:

```sh
cargo cicd status           # Implied: status show
cargo cicd publish          # Implied: publish run
cargo cicd workspace        # Implied: workspace doctor
```

## Common One-Liners

Copy these into your development workflow:

```sh
# Full pre-push checklist
cargo cicd workspace && cargo cicd status && cargo cicd target show

# Clean workspace before committing
cargo cicd test changed && cargo cicd git status

# Aggressive cleanup
cargo cicd target prune --apply && cargo cicd workspace

# Full pipeline run (all checks in sequence)
cargo cicd pipeline run
```

## Key Concepts

### cicd.toml

Every `cargo cicd publish` writes a `cicd.toml` file to your workspace root. This is your workspace's CI/CD state snapshot — useful for:
- CI pipelines to read workspace metadata
- Tracking historical state changes
- Debugging workspace issues

### Evidence Logging

cargo-cicd emits process evidence to `target/cargo-cicd/evidence/`. This includes:
- Event logs (JSONL format)
- Process receipts (JSON)
- Audit traces (XES format)

You don't need to interact with this directly, but it's available for auditing and integration with process-mining tools like wasm4pm.

### Default Verbs

Some nouns have a single primary verb that runs by default:

| Noun | Default Verb |
|------|--------------|
| `status` | `show` |
| `publish` | `run` |
| `workspace` | `doctor` |
| `evidence` | `doctor` |

### Flags

Most commands support:

```sh
--help              # Print help for the command
--version           # Print cargo-cicd version
```

The `target prune` command has a special flag:

```sh
--apply             # Execute the prune (default is dry-run only)
```

## Troubleshooting

**"command not found: cargo cicd"**
- Ensure you installed with `cargo install cargo-cicd`
- Add `~/.cargo/bin` to your `$PATH`

**"workspace not found"**
- Ensure you're in a directory with a `Cargo.toml` workspace file
- Use `ls Cargo.toml` to verify

**"git repository not found"**
- Ensure you're in a git repository: `git status` should work
- Some commands require an initialized `.git` directory

**Target directory too large**
- Run `cargo cicd target prune --apply` to clean debug artifacts
- Release builds are never deleted automatically

**wasm4pm oracle not found**
- Some commands integrate with the wasm4pm oracle (`wpm`), which is optional
- If not installed, cargo-cicd continues with a warning
- For full validation, install wasm4pm: https://github.com/seanchatmangpt/wasm4pm

## Next Steps

- Read the [Complete Command Reference](../reference/COMMANDS.md) for all available commands
- Check the [Reference Index](../reference/CLI_REFERENCE_INDEX.md) for quick lookups
- Explore [Integration Examples](../how-to/ci-cd-pipelines.md) for using cargo-cicd in your CI/CD setup
- See [Troubleshooting Guide](../reference/CLI_TROUBLESHOOTING.md) for detailed solutions

## Getting Help

For each command, use `--help`:

```sh
cargo cicd status --help
cargo cicd target prune --help
cargo cicd git close --help
```

For the full CLI reference:

```sh
cargo cicd --help
```

## Learn More

- **Feature Flags:** See [feature-flags.md](../reference/feature-flags.md)
- **cicd.toml Schema:** See [cicd-toml.md](../reference/cicd-toml.md)
- **Evidence Format:** See [evidence-format.md](../reference/evidence-format.md)
