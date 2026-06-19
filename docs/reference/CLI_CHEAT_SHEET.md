# cargo-cicd Cheat Sheet

Quick one-page reference for the most common cargo-cicd commands.

## Installation

```sh
cargo install cargo-cicd
cargo cicd --version   # Verify: 26.6.2
```

## Status & Diagnosis

```sh
cargo cicd status              # Show workspace status
cargo cicd target show         # Check target directory size
cargo cicd workspace           # Diagnose workspace health
cargo cicd git status          # Show git state (branch, dirty files)
```

## Running Tests & Fixtures

```sh
cargo cicd test changed        # Run tests for changed files only
cargo cicd trybuild changed    # Run trybuild for changed fixtures
cargo cicd pipeline run        # Run all checks in sequence
```

## Managing Target Directory

```sh
cargo cicd target show                    # View target size
cargo cicd target prune                   # Preview cleanup (dry-run)
cargo cicd target prune --apply           # Actually delete artifacts
```

## Publishing & Git

```sh
cargo cicd publish             # Create cicd.toml snapshot
cargo cicd git status          # Check if tree is clean
cargo cicd git close           # Verify phase can close
```

## Evidence & Auditing

```sh
cargo cicd evidence doctor     # Run receipt doctor audit
cargo cicd evidence audit      # Alias for doctor
cargo cicd status audit        # Audit current evidence
```

## Compliance & Supply Chain

```sh
cargo cicd certification show     # IEC 61508 / ISO 26262 / SOC2 / TOGAF summary
cargo cicd sbom generate          # Generate CycloneDX SBOM → sbom.json
cargo cicd sbom show              # Display generated SBOM
```

## Common Workflows

### Pre-Commit Checklist

```sh
cargo cicd workspace && \
cargo cicd status && \
cargo cicd test changed && \
cargo cicd git status
```

### Full Pre-Release Pipeline

```sh
cargo cicd workspace
cargo cicd test changed
cargo cicd target show
cargo cicd git status
cargo cicd publish
cargo cicd evidence doctor
```

### Aggressive Workspace Cleanup

```sh
cargo cicd target prune --apply
cargo clean
cargo build
cargo cicd publish
```

### Monitor Workspace During Development

```sh
while true; do
  cargo cicd status
  sleep 30
done
```

## Flag Reference

| Flag | Description | Example |
|------|-------------|---------|
| `--help` | Show help for a command | `cargo cicd status --help` |
| `--version` | Show cargo-cicd version | `cargo cicd --version` |
| `--apply` | Execute prune (not dry-run) | `cargo cicd target prune --apply` |

## File Paths

| Path | Purpose |
|------|---------|
| `cicd.toml` | Workspace state snapshot (at root) |
| `target/cargo-cicd/evidence/` | Event logs and receipts |
| `sbom.json` | Generated CycloneDX SBOM |
| `Cargo.toml` | Workspace manifest (required) |
| `rust-toolchain.toml` | Pinned Rust version (optional) |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failed (check stderr) |
| 2 | Invalid workspace |
| 3 | Readiness check failed |

## Noun → Verb Mapping

Default verbs (verb can be omitted):

| Command | Expands To |
|---------|-----------|
| `cargo cicd status` | `cargo cicd status show` |
| `cargo cicd publish` | `cargo cicd publish run` |
| `cargo cicd workspace` | `cargo cicd workspace doctor` |
| `cargo cicd evidence` | `cargo cicd evidence doctor` |

## Key Concepts

**cicd.toml** — Snapshot of workspace state (target size, changed files, toolchain, git status)

**Evidence Log** — Process trace stored in `target/cargo-cicd/evidence/events.jsonl`

**Receipt Doctor** — wasm4pm oracle that validates workspace readiness for publish

**Changed Files** — Files modified since `origin/main` (default base ref, customizable in cicd.toml)

**Phase Closure** — Git verification that tree is clean before release

**SBOM** — Software Bill of Materials in CycloneDX format (`sbom.json`), generated via `cargo-cyclonedx`

## Troubleshooting Quick Links

| Problem | Solution |
|---------|----------|
| Command not found | `cargo install cargo-cicd` |
| Not a workspace | Run from directory with `Cargo.toml` |
| Target too large | `cargo cicd target prune --apply` |
| Git tree dirty | `git add .` and `git commit` first |
| wpm not found | Optional; install wasm4pm for full validation |
| sbom generate fails | `cargo install cargo-cyclonedx` |

## Environment Variables

```sh
WPM_PATH=/path/to/wpm         # Path to wasm4pm binary (if not on PATH)
```

## Quick Test

Verify installation and basic functionality:

```sh
cd /path/to/workspace
cargo cicd workspace
cargo cicd status
cargo cicd target show
cargo cicd git status
```

All should pass with [OK] or [PASS] verdicts.

## Learn More

- [Quick Start](CLI_QUICK_START.md) — Installation and basic workflow
- [Complete Command Reference](COMMANDS.md) — Full documentation of all commands
- [Troubleshooting Guide](CLI_TROUBLESHOOTING.md) — Solutions for common issues
- [Integration Examples](../integration-examples/CI_CD_PIPELINES.md) — Using cargo-cicd in CI/CD
