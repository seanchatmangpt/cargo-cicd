# cargo-cicd CLI Reference Guide — Complete Index

A comprehensive, organized reference for the cargo-cicd command-line interface.

**Version:** 26.6.2  
**Last Updated:** 2026-06-14

## Quick Navigation

For different use cases, start here:

| You Want To... | Start Here |
|---|---|
| Get started quickly | [Quick Start Guide](CLI_QUICK_START.md) |
| Look up a specific command | [Complete Command Reference](COMMANDS.md) |
| Remember commands on one page | [Cheat Sheet](CLI_CHEAT_SHEET.md) |
| Solve a problem | [Troubleshooting Guide](CLI_TROUBLESHOOTING.md) |
| Use in CI/CD pipelines | [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md) |
| Set up IDE shortcuts | [IDE Integration](../integration-examples/IDE_INTEGRATION.md) |

---

## Document Structure

### 1. Quick Start (5 minutes)

**File:** [CLI_QUICK_START.md](CLI_QUICK_START.md)

Perfect for new users. Covers:
- Installation
- Basic workflow (first 6 steps)
- Command syntax
- Common one-liners
- Key concepts
- Troubleshooting basics

**Read this first if you're new to cargo-cicd.**

### 2. Complete Command Reference (All Commands)

**File:** [COMMANDS.md](COMMANDS.md)

Comprehensive documentation of every command. Organized by noun:
- `certification` — Display compliance-readiness summary across safety and architecture standards
- `evidence` — Audit process evidence via wasm4pm
- `git` — Git phase management
- `pipeline` — Execute full manufacturing pipeline
- `publish` — Publish state to cicd.toml
- `sbom` — Generate or display the workspace Software Bill of Materials
- `status` — Show workspace CI/CD status
- `target` — Manage target directory
- `test` — Run tests for changed files
- `trybuild` — Manage trybuild fixtures
- `workspace` — Workspace diagnostics

Each command includes:
- Full description
- Usage syntax
- Example output
- Exit codes
- Notes and caveats
- Practical examples

**Read this to understand all available commands in detail.**

### 3. One-Page Cheat Sheet

**File:** [CLI_CHEAT_SHEET.md](CLI_CHEAT_SHEET.md)

Quick reference for common commands and workflows:
- Status & diagnosis commands
- Running tests & fixtures
- Managing target directory
- Publishing & git
- Evidence & auditing
- Common workflows (pre-commit, pre-release, cleanup)
- Flag reference
- Exit codes
- Troubleshooting quick links

**Print this or bookmark it for quick lookups during development.**

### 4. Troubleshooting Guide

**File:** [CLI_TROUBLESHOOTING.md](CLI_TROUBLESHOOTING.md)

Diagnostic and fix guide organized by problem category:
- Installation & setup issues
- Command execution problems
- Workspace issues
- Git-related problems
- Target directory issues
- Evidence & oracle problems
- Testing & fixture issues
- Advanced diagnostics

Each section includes:
- Problem description
- Root causes
- Step-by-step solutions
- Quick reference table

**Read this when something doesn't work.**

### 5. CI/CD Integration Examples

**File:** [../integration-examples/CI_CD_PIPELINES.md](../integration-examples/CI_CD_PIPELINES.md)

Production-ready examples for:
- GitHub Actions (basic, modular, matrix testing, pre-release)
- GitLab CI (basic, with caching)
- Pre-commit hooks
- Docker & containers
- Development workflows (Makefile, pre-push scripts, dev loops)
- Monitoring & observability
- Complete end-to-end pipeline

**Read this to integrate cargo-cicd into your CI/CD system.**

### 6. IDE Integration Guide

**File:** [../integration-examples/IDE_INTEGRATION.md](../integration-examples/IDE_INTEGRATION.md)

Setup instructions for:
- VS Code (tasks, keyboard shortcuts, watch mode)
- JetBrains IDEs (external tools, run configurations)
- Vim/Neovim (make, lua, dispatch, terminal)
- Emacs (compilation mode, ivy, org-mode)
- Sublime Text (build system)
- General editor integration (Makefile, aliases, shell functions)

**Read this to configure your editor for cargo-cicd.**

---

## Command Categories

### Status & Diagnosis

Commands that report state without modifying anything:

```bash
cargo cicd status           # Show workspace status
cargo cicd workspace        # Diagnose workspace health
cargo cicd git status       # Show git state
cargo cicd target show      # Show target directory size
cargo cicd test changed     # Plan which tests to run
cargo cicd trybuild changed # Plan which trybuild fixtures to run
```

### Workspace Management

Commands that manage and maintain the workspace:

```bash
cargo cicd target prune [--apply]  # Cleanup target directory
cargo cicd git close               # Verify git phase closure
cargo cicd publish                 # Publish state to cicd.toml
```

### Evidence & Auditing

Commands for process evidence and oracle validation:

```bash
cargo cicd evidence doctor         # Run receipt doctor
cargo cicd evidence audit          # Alias for doctor
cargo cicd status audit            # Audit current XES evidence
```

### Comprehensive Checks

Commands that run multiple checks:

```bash
cargo cicd pipeline run    # Execute full pipeline (all checks in sequence)
```

---

## Noun-Verb Grammar

cargo-cicd uses a noun-verb syntax that mirrors natural language:

```
cargo cicd <noun> [verb] [options]
```

**Nouns** (the subject):
- `certification` — Compliance-readiness across IEC 61508, ISO 26262, SOC2, and TOGAF ADM
- `evidence` — Process evidence and receipts
- `git` — Git repository operations
- `pipeline` — Manufacturing pipeline
- `publish` — Publishing and state
- `sbom` — Software Bill of Materials (CycloneDX JSON)
- `status` — Workspace status
- `target` — Target directory
- `test` — Test execution
- `trybuild` — Trybuild fixtures
- `workspace` — Workspace diagnostics

**Verbs** (the action):
- `show` — Display information
- `prune` — Clean up artifacts
- `changed` — Work with changed files only
- `close` — Enforce phase closure
- `run` — Execute operation
- `doctor` / `audit` — Diagnostic/validation operations

### Default Verbs

Some nouns have default verbs that run if you omit the verb:

| Noun | Default Verb | Example |
|------|--------------|---------|
| `status` | `show` | `cargo cicd status` → `cargo cicd status show` |
| `publish` | `run` | `cargo cicd publish` → `cargo cicd publish run` |
| `workspace` | `doctor` | `cargo cicd workspace` → `cargo cicd workspace doctor` |
| `evidence` | `doctor` | `cargo cicd evidence` → `cargo cicd evidence doctor` |

---

## Command Matrix

Quick reference showing which verbs work with each noun:

| Noun | Verbs | Description |
|------|-------|-------------|
| `certification` | `show` | Compliance-readiness summary (IEC 61508, ISO 26262, SOC2, TOGAF ADM) |
| `evidence` | `doctor`, `audit` | Adjudicate runtime process evidence |
| `git` | `status`, `close` | Git repository state management |
| `pipeline` | `run` | Execute full manufacturing pipeline |
| `publish` | `run` | Publish state to cicd.toml |
| `sbom` | `generate`, `show` | Generate or display CycloneDX JSON Software Bill of Materials |
| `status` | `show`, `audit` | Workspace CI/CD status |
| `target` | `show`, `prune` | Manage target directory |
| `test` | `changed` | Run tests for changed files |
| `trybuild` | `changed` | Run trybuild for changed fixtures |
| `workspace` | `doctor` | Workspace diagnostics |

---

## Common Workflows

### Pre-Commit Checklist

Verify everything before committing:

```bash
cargo cicd workspace      # ① Diagnose workspace
cargo cicd status         # ② Check overall status
cargo cicd test changed   # ③ Plan which tests to run
cargo cicd git status     # ④ Verify git state
```

All should show PASS before committing.

### Pre-Push Validation

Gate push on readiness:

```bash
cargo cicd workspace      # Workspace health check
cargo cicd test changed   # Changed test plan
cargo cicd git close      # Verify tree is clean (required)
cargo cicd publish        # Capture current state
```

All must pass before `git push`.

### Pre-Release Pipeline

Full validation before releasing:

```bash
cargo cicd workspace           # Workspace health
cargo cicd test changed        # Test planning
cargo cicd target show         # Target size check
cargo cicd git status          # Git state
cargo cicd publish run         # Publish state
cargo cicd evidence doctor     # Oracle validation (if available)
cargo cicd pipeline run        # Full integrated check
```

### Development Monitoring

Keep tabs on workspace during development:

```bash
# Run periodically (e.g., every 5 minutes)
cargo cicd status
cargo cicd target show
cargo cicd git status
```

### Aggressive Workspace Cleanup

Free up disk space:

```bash
cargo cicd target prune --apply    # Remove debug artifacts
cargo clean                         # Fully clean target/
cargo build                         # Rebuild
cargo cicd publish                 # Capture new state
```

---

## Flags & Options

### Global Options

Available on all commands:

```bash
--help, -h              # Show help for command
--version               # Show cargo-cicd version
```

### Command-Specific Flags

#### target prune

```bash
--apply                 # Execute cleanup (default is dry-run)
```

---

## Environment Variables

### WPM_PATH

Set location of wasm4pm binary:

```bash
export WPM_PATH=/path/to/wpm
cargo cicd evidence doctor
```

---

## File Locations

### Configuration & State

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest (required) |
| `cicd.toml` | Workspace CI/CD state (generated) |
| `rust-toolchain.toml` | Rust toolchain pinning (optional) |
| `.git/` | Git repository (required for git operations) |

### Evidence & Artifacts

| Path | Purpose |
|------|---------|
| `target/cargo-cicd/evidence/` | Process evidence directory |
| `target/cargo-cicd/evidence/events.jsonl` | Event log (line-delimited JSON) |
| `target/cargo-cicd/evidence/events.xes` | XES trace (XML Event Stream) |
| `target/cargo-cicd/evidence/receipts/latest.json` | Latest process receipt |
| `target/cargo-cicd/evidence/.session` | Session ID |

---

## Exit Codes

All commands follow standard Unix exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success (command completed as expected) |
| 1 | Failure (command encountered an error or refused operation) |
| 2 | Invalid workspace (Cargo.toml not found) |
| 3 | Readiness check failed (not ready for operation) |

---

## Performance Notes

### Command Execution Time

Typical execution times (first run may be slower due to compilation):

| Command | Time | Notes |
|---------|------|-------|
| `status` | ~50ms | Very fast, read-only |
| `target show` | ~30ms | Very fast, read-only |
| `workspace doctor` | ~100ms | Includes policy checks |
| `git status` | ~20ms | Very fast, read-only |
| `test changed` | ~50ms | Planning only, doesn't run tests |
| `trybuild changed` | ~50ms | Planning only, doesn't run fixtures |
| `publish run` | ~100ms | May include oracle call |
| `pipeline run` | ~5-30s | Runs all checks + oracle validation |

### Caching

cargo-cicd leverages Rust's build cache:
- First run: slower (binary compilation)
- Subsequent runs: faster (cached)
- Clean build: `cargo clean` then re-run

---

## Versioning

Current version: **26.6.2**

Check your version:
```bash
cargo cicd --version
```

Update to latest:
```bash
cargo install --force cargo-cicd
```

---

## Key Concepts

### cicd.toml

A TOML file written to workspace root by `cargo cicd publish run`. Contains:
- Workspace metadata (name, toolchain, target directory)
- Workspace state (target size, dirty flag, changed file counts)
- Event log (timestamp, activity, verdict for each operation)

Useful for:
- CI/CD pipelines to read workspace metadata
- Tracking state changes over time
- Debugging workspace issues

### Evidence & Process Mining

cargo-cicd emits process evidence to `target/cargo-cicd/evidence/`:
- `events.jsonl` — Line-delimited JSON events
- `events.xes` — XML Event Stream for process mining
- `receipts/latest.json` — Latest process receipt

Used for:
- Auditing operations
- Process mining with wasm4pm
- Tracking evidence for releases

### wasm4pm Oracle

Optional external tool for evidence validation:
- Receipt Doctor (`wpm receipt doctor`) — Validates process receipts
- Audit (`wpm audit`) — Validates XES traces
- Detects policy violations and process anomalies

**Optional:** cargo-cicd continues with warnings if wpm is not found.

### Changed Files Detection

Intelligently identifies files that have changed since base ref (default: `origin/main`):
- Changed test files → affects `test changed`
- Changed trybuild fixtures → affects `trybuild changed`
- Changed Rust files → affects test planning

**Base ref is configurable in `cicd.toml [test]` section.**

### Phase Closure

Git-specific concept: ensuring working tree is clean before release:
- `git status` — Reports tree state (clean/dirty)
- `git close` — Enforces clean tree; refuses if dirty

**Design:** Does NOT automatically commit. Requires manual staging/committing.

---

## Learning Path

### Beginner (0-30 minutes)

1. [Quick Start](CLI_QUICK_START.md) — Installation and basic workflow
2. Run basic commands: `cargo cicd status`, `cargo cicd workspace`
3. Review [Cheat Sheet](CLI_CHEAT_SHEET.md) for common patterns

### Intermediate (30-120 minutes)

1. Read [Complete Command Reference](COMMANDS.md) — Understand all commands
2. Try workflows: pre-commit checklist, pre-push validation
3. Set up IDE integration: [IDE Integration](../integration-examples/IDE_INTEGRATION.md)

### Advanced (120+ minutes)

1. Integrate into CI/CD: [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md)
2. Read [Architecture](../SOLUTION_ARCHITECTURE.md) for internal details
3. Troubleshoot issues: [Troubleshooting Guide](CLI_TROUBLESHOOTING.md)
4. Explore evidence and wasm4pm integration

---

## Tips & Best Practices

1. **Always run workspace doctor first:** Catches issues early
2. **Use git close as a gate:** Prevents dirty trees from being pushed
3. **Integrate into pre-commit hooks:** Catches issues before they propagate
4. **Monitor target size:** Prevents disk space issues
5. **Use changed commands in development:** Faster feedback loops
6. **Run full pipeline before release:** Comprehensive validation
7. **Keep cicd.toml in version control:** Track state history
8. **Use IDE shortcuts:** Speed up development workflow

---

## Troubleshooting Quick Links

Common issues and where to find solutions:

| Problem | Solution |
|---------|----------|
| "command not found" | [Installation Issues](CLI_TROUBLESHOOTING.md#installation--setup-issues) |
| Workspace errors | [Workspace Issues](CLI_TROUBLESHOOTING.md#workspace-issues) |
| Git problems | [Git-Related Problems](CLI_TROUBLESHOOTING.md#git-related-problems) |
| Target too large | [Target Directory Issues](CLI_TROUBLESHOOTING.md#target-directory-issues) |
| wpm not found | [Evidence & Oracle Problems](CLI_TROUBLESHOOTING.md#evidence--oracle-problems) |
| Test failures | [Testing & Fixture Issues](CLI_TROUBLESHOOTING.md#testing--fixture-issues) |

---

## Additional Resources

### Related Documentation

- [Architecture](../SOLUTION_ARCHITECTURE.md) — How cargo-cicd works internally
- [cicd.toml Schema](../reference/cicd-toml.md) — Configuration file format
- [Evidence Format](../reference/evidence-format.md) — Process evidence format
- [Feature Flags](../reference/feature-flags.md) — Compile-time options
- [Autonomic Policies](../explanation/autonomic-policies.md) — Policy-based suggestions

### Contributing

- [Development Setup](../contributing/01-development-setup.md)
- [Pull Request Workflow](../contributing/02-pull-request-workflow.md)
- [Code Style](../contributing/04-code-style.md)

### Community

- **GitHub:** https://github.com/seanchatmangpt/cargo-cicd
- **Issues:** https://github.com/seanchatmangpt/cargo-cicd/issues
- **Discussions:** https://github.com/seanchatmangpt/cargo-cicd/discussions

---

## Getting Help

### Quick Help

For any command, use `--help`:

```bash
cargo cicd --help
cargo cicd status --help
cargo cicd target prune --help
```

### Detailed Documentation

- **Quick Start:** [CLI_QUICK_START.md](CLI_QUICK_START.md)
- **Full Reference:** [COMMANDS.md](COMMANDS.md)
- **Quick Lookup:** [CLI_CHEAT_SHEET.md](CLI_CHEAT_SHEET.md)
- **Problem Solving:** [CLI_TROUBLESHOOTING.md](CLI_TROUBLESHOOTING.md)

### Community Support

- GitHub Discussions for questions
- GitHub Issues for bug reports
- Pull requests for contributions

---

## Document Maintenance

This reference guide was generated from the cargo-cicd source code and documentation.

- **Last Updated:** 2026-06-14
- **Version:** 26.6.2
- **Maintainer:** cargo-cicd community

For the most up-to-date documentation, visit the [GitHub repository](https://github.com/seanchatmangpt/cargo-cicd).
