# cargo-cicd CLI Reference Guide

**A complete, publication-ready reference guide for the cargo-cicd command-line interface.**

**Version:** 26.6.2

---

## Overview

This comprehensive reference guide is designed for developers of all skill levels working with cargo-cicd. Whether you're getting started or optimizing your CI/CD pipeline, you'll find detailed documentation, examples, and solutions here.

### What is cargo-cicd?

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces. It keeps your workspace clean, fast, and push-ready by running targeted checks before you push to remote.

**Key Features:**
- **Status checks** — Workspace health, git state, target directory size
- **Smart testing** — Run only changed tests for faster feedback
- **Target management** — Monitor and clean debug artifacts
- **Git phase closure** — Enforce clean working trees before release
- **Evidence emission** — Process mining integration via wasm4pm
- **State capture** — Publish workspace state to cicd.toml

---

## Document Map

### Starting Out

| Document | Time | For Whom |
|----------|------|----------|
| [**Quick Start Guide**](CLI_QUICK_START.md) | 5 min | New users, first-time setup |
| [**Cheat Sheet**](CLI_CHEAT_SHEET.md) | 1 min | Busy developers, quick reference |

### Reference

| Document | Audience | Purpose |
|----------|----------|---------|
| [**Complete Command Reference**](COMMANDS.md) | All users | Detailed docs of every command |
| [**Troubleshooting Guide**](CLI_TROUBLESHOOTING.md) | Debugging | Solutions for common problems |
| [**CLI Reference Index**](CLI_REFERENCE_INDEX.md) | All users | Master index and learning path |

### Integration

| Document | Focus | For Whom |
|----------|-------|----------|
| [**CI/CD Pipeline Integration**](../integration-examples/CI_CD_PIPELINES.md) | GitHub Actions, GitLab CI, Docker | CI/CD engineers |
| [**IDE Integration**](../integration-examples/IDE_INTEGRATION.md) | VS Code, IntelliJ, Vim, Emacs | IDE users, developers |

---

## Quick Start

### 1. Installation

```bash
cargo install cargo-cicd
cargo cicd --version  # Verify: 26.6.2
```

### 2. First Command

```bash
cargo cicd status
```

Output shows your workspace's CI/CD readiness in seconds.

### 3. Explore

```bash
cargo cicd workspace        # Full health diagnosis
cargo cicd target show      # Check disk usage
cargo cicd git status       # See git state
```

### 4. Integrate

Choose your integration:
- **Pre-commit hook** → [Troubleshooting Guide](CLI_TROUBLESHOOTING.md#pre-commit-hooks)
- **VS Code** → [IDE Integration](../integration-examples/IDE_INTEGRATION.md#vs-code)
- **GitHub Actions** → [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md#github-actions)
- **Makefile** → [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md#makefile-integration)

---

## Commands at a Glance

### Status & Diagnosis (Read-Only)

```bash
cargo cicd status                # Workspace status (fast)
cargo cicd workspace             # Full health diagnosis
cargo cicd git status            # Git repository state
cargo cicd target show           # Target directory size
cargo cicd test changed          # Plan which tests to run
cargo cicd trybuild changed      # Plan trybuild fixtures
```

### Workspace Management (May Modify)

```bash
cargo cicd target prune [--apply]  # Cleanup debug artifacts
cargo cicd git close               # Verify phase can close
cargo cicd publish                 # Publish state to cicd.toml
```

### Evidence & Oracle

```bash
cargo cicd evidence doctor         # Validate process receipt
cargo cicd status audit            # Audit current evidence
```

### Full Pipeline

```bash
cargo cicd pipeline run    # Run all checks sequentially
```

---

## Common Workflows

### Before Committing

```bash
cargo cicd workspace      # ✓ Check workspace health
cargo cicd status         # ✓ Check status
cargo cicd test changed   # ✓ Plan changed tests
cargo cicd git status     # ✓ Check git state
```

### Before Pushing

```bash
cargo cicd workspace      # ✓ Workspace health
cargo cicd test changed   # ✓ Test planning
cargo cicd git close      # ✓ Enforce clean tree
```

### Before Releasing

```bash
cargo cicd pipeline run   # ✓ Full integrated check
cargo cicd evidence doctor # ✓ Oracle validation
```

### Monitoring Workspace

```bash
# Run periodically during development
watch -n 30 'cargo cicd status'
```

---

## Document Guide

### When You Want To...

| Goal | Document | Section |
|------|----------|---------|
| Get started in 5 minutes | Quick Start | [Installation](CLI_QUICK_START.md#installation) |
| Understand a specific command | Command Reference | [Nouns and Verbs](COMMANDS.md#nouns-and-verbs) |
| Remember commands quickly | Cheat Sheet | [Status & Diagnosis](CLI_CHEAT_SHEET.md#status--diagnosis) |
| Solve a problem | Troubleshooting | [Installation Issues](CLI_TROUBLESHOOTING.md#installation--setup-issues) |
| Set up CI/CD | CI/CD Integration | [GitHub Actions](../integration-examples/CI_CD_PIPELINES.md#github-actions) |
| Configure your editor | IDE Integration | [VS Code](../integration-examples/IDE_INTEGRATION.md#vs-code) |
| Find anything | Reference Index | [Quick Navigation](CLI_REFERENCE_INDEX.md#quick-navigation) |

### Document Details

#### [Quick Start Guide](CLI_QUICK_START.md)
**5 minutes** — Get cargo-cicd installed and running

Contains:
- Installation instructions
- Basic workflow (6 step introduction)
- Command syntax explanation
- Common one-liners
- Key concepts overview
- Troubleshooting basics
- Next steps

**Perfect for:** New users, first-time setup

---

#### [Complete Command Reference](COMMANDS.md)
**Reference** — Every command, fully documented

Contains:
- Full description of each noun
- All verbs with explanations
- Usage examples
- Output examples
- Exit codes
- Notes and caveats
- Practical usage scenarios
- Tips and tricks

**Perfect for:** Understanding all available commands in detail

Organization:
- Organized by noun (evidence, git, pipeline, publish, status, target, test, trybuild, workspace)
- Each noun has all its verbs documented
- Global options and flags
- Environment variables
- Exit codes

---

#### [One-Page Cheat Sheet](CLI_CHEAT_SHEET.md)
**1 page** — Quick reference for common commands

Contains:
- Installation quick check
- Status & diagnosis commands
- Running tests & fixtures
- Managing target directory
- Publishing & git
- Evidence & auditing
- Common workflows
- Flag reference
- File paths
- Exit codes
- Troubleshooting quick links
- Noun→verb mapping

**Perfect for:** Busy developers, quick lookups, printing/bookmarking

---

#### [Troubleshooting Guide](CLI_TROUBLESHOOTING.md)
**Advanced** — Solutions for common problems

Contains:
- Installation & setup issues
- Command execution problems
- Workspace issues
- Git-related problems
- Target directory issues
- Evidence & oracle problems
- Testing & fixture issues
- Advanced diagnostics
- Quick reference table
- When all else fails

**Perfect for:** Debugging, problem-solving, diagnostics

---

#### [CI/CD Pipeline Integration](../integration-examples/CI_CD_PIPELINES.md)
**Production-Ready Examples** — Integrate cargo-cicd into your CI/CD system

Contains:
- GitHub Actions (basic, modular, pre-release, matrix testing)
- GitLab CI (basic, with caching)
- Pre-commit hooks
- Docker & containers
- Development workflows (scripts, Makefile, loops)
- Monitoring & observability
- Complete end-to-end example
- Tips & best practices

**Perfect for:** CI/CD engineers, release automation, production deployments

---

#### [IDE Integration](../integration-examples/IDE_INTEGRATION.md)
**Editor-Specific Setup** — Configure cargo-cicd in your IDE

Contains:
- VS Code (tasks, shortcuts, watch mode)
- JetBrains IDEs (external tools, run configs)
- Vim/Neovim (make, lua, dispatch)
- Emacs (compilation mode, ivy, org-mode)
- Sublime Text (build system)
- General editor integration
- Workflow combinations
- Troubleshooting

**Perfect for:** IDE users, editor customization, productivity

---

#### [CLI Reference Index](CLI_REFERENCE_INDEX.md)
**Master Index** — Navigation, learning paths, and comprehensive overview

Contains:
- Quick navigation guide
- Document structure
- Command categories
- Noun-verb grammar
- Command matrix
- Common workflows
- Flag reference
- File locations
- Exit codes
- Performance notes
- Key concepts
- Learning path (beginner → advanced)
- Tips & best practices
- Troubleshooting links
- Additional resources

**Perfect for:** Finding anything, learning progressively, reference

---

## Feature Overview

### Read-Only Operations (Safe)

These commands only report state; they don't modify anything:

```bash
cargo cicd status                # Show workspace status
cargo cicd workspace doctor      # Full health diagnosis
cargo cicd git status            # Show git repository state
cargo cicd target show           # Show target directory size
cargo cicd test changed          # Plan which tests to run
cargo cicd trybuild changed      # Plan which fixtures to run
```

**Safe to run at any time.** Useful for continuous monitoring.

### Safe Modifications (Dry-Run by Default)

These commands can modify state, but default to dry-run:

```bash
cargo cicd target prune          # Preview cleanup (dry-run only)
cargo cicd target prune --apply  # Execute cleanup
```

**Prune default is dry-run.** Must explicitly use `--apply` to delete.

### Destructive Operations (Require Confirmation)

These require explicit setup or refuse if conditions aren't met:

```bash
cargo cicd git close       # Refuses if tree is dirty
cargo cicd publish run     # Fails if oracle refuses
cargo cicd evidence doctor # Fails if oracle refuses
```

**Design philosophy:** Never silently hide dirty state.

---

## Key Concepts

### cicd.toml

A TOML configuration file written to your workspace root. Contains:
- Workspace metadata (name, toolchain, target dir)
- Workspace state (target size, dirty flag, changed file counts)
- Event log (timestamp, activity, verdict for each operation)

Useful for:
- CI/CD pipelines reading workspace metadata
- Tracking state history over time
- Debugging workspace issues

### Evidence & Process Mining

cargo-cicd emits process evidence for auditing:
- `events.jsonl` — Line-delimited JSON events
- `events.xes` — XML Event Stream (for process mining)
- `receipts/latest.json` — Latest process receipt

Used with wasm4pm oracle for validation.

### wasm4pm Oracle (Optional)

Optional external tool for evidence validation:
- Validates process receipts
- Detects policy violations
- Validates XES traces

**Optional:** cargo-cicd continues with warnings if not installed.

### Changed Files Detection

Intelligently identifies changed files since base ref (default: `origin/main`):
- Used for test planning
- Used for trybuild fixture planning
- Customizable via cicd.toml

---

## Installation & Verification

### Install

```bash
cargo install cargo-cicd
```

### Verify

```bash
cargo cicd --version
# Expected output: cargo-cicd 26.6.2

cargo cicd status
# Should show workspace status
```

### Update

```bash
cargo install --force cargo-cicd
```

---

## Workspace Requirements

Minimum requirements:
- `Cargo.toml` — Rust workspace manifest (required)
- `.git/` — Git repository (required for git commands)

Optional:
- `rust-toolchain.toml` — Pinned Rust version (recommended)
- `cicd.toml` — Generated by `cargo cicd publish` (generated on demand)

---

## Common Issues & Quick Fixes

| Problem | Quick Fix |
|---------|-----------|
| "command not found" | `cargo install cargo-cicd` |
| "workspace not found" | `cd` to directory with `Cargo.toml` |
| "git repository not found" | `git init` or navigate to repo |
| Target too large | `cargo cicd target prune --apply` |
| Git tree is dirty | `git add . && git commit -m "message"` |
| wpm not found | Optional; install if needed |

See [Troubleshooting Guide](CLI_TROUBLESHOOTING.md) for detailed solutions.

---

## Version Information

- **Current Version:** 26.6.2
- **Rust Version:** 1.70+ (recommended)
- **Platforms:** Linux, macOS, Windows

Check your version:
```bash
cargo cicd --version
```

---

## Further Learning

### Documentation Map

- **Getting Started** → [Quick Start Guide](CLI_QUICK_START.md)
- **All Commands** → [Complete Command Reference](COMMANDS.md)
- **Quick Lookup** → [Cheat Sheet](CLI_CHEAT_SHEET.md)
- **Troubleshooting** → [Troubleshooting Guide](CLI_TROUBLESHOOTING.md)
- **Navigation** → [Reference Index](CLI_REFERENCE_INDEX.md)
- **CI/CD** → [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md)
- **IDEs** → [IDE Integration](../integration-examples/IDE_INTEGRATION.md)

### Related Documentation

- [Architecture](../SOLUTION_ARCHITECTURE.md) — How cargo-cicd works
- [cicd.toml Schema](../reference/cicd-toml.md) — Configuration format
- [Evidence Format](../reference/evidence-format.md) — Evidence format specification
- [Autonomic Policies](../explanation/autonomic-policies.md) — Policy-based suggestions

### Community

- **Repository:** https://github.com/seanchatmangpt/cargo-cicd
- **Issues:** https://github.com/seanchatmangpt/cargo-cicd/issues
- **Discussions:** https://github.com/seanchatmangpt/cargo-cicd/discussions

---

## Quick Reference

### Most Common Commands

```bash
# Before committing
cargo cicd workspace
cargo cicd status
cargo cicd test changed
cargo cicd git status

# Before pushing
cargo cicd git close

# Before releasing
cargo cicd pipeline run

# Monitor during development
cargo cicd target show
cargo cicd status
```

### Keyboard Shortcuts (VS Code Example)

```
Ctrl+Shift+C  →  cargo cicd status
Ctrl+Shift+W  →  cargo cicd workspace
Ctrl+Shift+T  →  cargo cicd test changed
Ctrl+Shift+P  →  cargo cicd pipeline run
```

See [IDE Integration](../integration-examples/IDE_INTEGRATION.md) for your editor.

---

## Document Statistics

| Document | Size | Reading Time |
|----------|------|--------------|
| Quick Start | 5.7 KB | 5 min |
| Command Reference | 40+ KB | 30+ min |
| Cheat Sheet | 4.4 KB | 1 min |
| Troubleshooting | 21 KB | 15+ min |
| CI/CD Integration | 21 KB | 20+ min |
| IDE Integration | 18 KB | 15+ min |
| Reference Index | 16 KB | 10+ min |
| **Total** | **~126 KB** | **90+ min** |

---

## How to Use This Guide

### If You're New (First 30 minutes)

1. Read [Quick Start Guide](CLI_QUICK_START.md) (5 min)
2. Run your first commands (5 min)
3. Review [Cheat Sheet](CLI_CHEAT_SHEET.md) (2 min)
4. Try basic workflows (10+ min)

### If You're Setting Up CI/CD (1-2 hours)

1. Read [Quick Start Guide](CLI_QUICK_START.md) (5 min)
2. Check [CI/CD Integration](../integration-examples/CI_CD_PIPELINES.md) examples (30 min)
3. Configure your CI system (30+ min)
4. Test and verify (15+ min)

### If You're Troubleshooting (30+ minutes)

1. Check [Troubleshooting Guide](CLI_TROUBLESHOOTING.md) for your error (10 min)
2. Follow suggested solutions (10+ min)
3. Verify fix works (5+ min)
4. Escalate if needed (contact community)

### If You Need Everything (Complete reference)

Start with [Reference Index](CLI_REFERENCE_INDEX.md) for navigation and learning path.

---

## Getting Help

### For Quick Help

```bash
cargo cicd --help
cargo cicd <command> --help
```

### For Detailed Information

Consult the appropriate guide:
- **Getting started** → [Quick Start](CLI_QUICK_START.md)
- **Specific command** → [Command Reference](COMMANDS.md)
- **Quick lookup** → [Cheat Sheet](CLI_CHEAT_SHEET.md)
- **Problem-solving** → [Troubleshooting](CLI_TROUBLESHOOTING.md)
- **Finding anything** → [Reference Index](CLI_REFERENCE_INDEX.md)

### For Community Support

- GitHub Issues: https://github.com/seanchatmangpt/cargo-cicd/issues
- GitHub Discussions: https://github.com/seanchatmangpt/cargo-cicd/discussions

---

## About This Guide

**Created:** 2026-06-14  
**Version:** 26.6.2  
**Status:** Complete and production-ready

This comprehensive reference guide includes:
- ✓ Quick Start (5 minutes)
- ✓ Complete Command Reference (all commands)
- ✓ One-Page Cheat Sheet
- ✓ Troubleshooting Guide
- ✓ CI/CD Integration Examples
- ✓ IDE Integration Guide
- ✓ Master Reference Index

Suitable for publication and distribution to users.

---

## Navigation

- **Start Here:** [Quick Start Guide](CLI_QUICK_START.md)
- **Find Anything:** [Reference Index](CLI_REFERENCE_INDEX.md)
- **All Commands:** [Command Reference](COMMANDS.md)
- **Need Help:** [Troubleshooting](CLI_TROUBLESHOOTING.md)
