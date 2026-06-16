# Documentation Standards for cargo-cicd

This file establishes standards for code documentation, user guides, and examples across the cargo-cicd project. All contributors must follow these conventions to maintain consistency and readability.

---

## 1. Doc Comment Standards

### 1.1 Module-Level Documentation

Every public module must include a module-level doc comment explaining purpose and relationships.

**Template:**

```rust
//! Module-level documentation.
//!
//! This module handles [purpose]. It is responsible for [key responsibilities].
//! Related modules: [`crate::other_module`], [`crate::another_module`].
//!
//! # Examples
//!
//! Typical usage:
//!
//! ```
//! # use cargo_cicd::module_name::SomeType;
//! let value = SomeType::new();
//! value.do_something();
//! ```
```

**Example from project:**

```rust
//! Engine state — the aggregate root of all runtime dimensions.
//!
//! `EngineState` holds the complete state: workspace, toolchain, targets, changed files,
//! test plans, trybuild results, git phase, process events, artifacts, policies, and
//! projection profiles. Adapters populate it from external sources; nouns read from it.
//!
//! Related modules: [`crate::adapters`] (population), [`crate::nouns`] (consumption).
```

### 1.2 Public Function Documentation

Every public function must have:
- One-line summary (first line)
- Blank line
- Detailed explanation (WHY this exists, not WHAT the code does)
- `# Arguments` section (if any)
- `# Returns` section
- `# Panics` section (if applicable)
- `# Errors` section (if it returns Result)
- `# Examples` section (if non-obvious)

**Template:**

```rust
/// Scans the workspace for changed files since the last green commit.
///
/// This adapter compares the current working tree against the last known good
/// commit hash stored in `cicd.toml`. Only files matching `**/*.rs` are tracked
/// (ignoring build artifacts, vendor dirs, and gitignored paths).
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `last_green_commit` - Commit hash to compare against
///
/// # Returns
///
/// A list of changed `.rs` file paths, or an error if git commands fail.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace root is not a git repository
/// - The commit hash does not exist
/// - Git command execution fails
///
/// # Examples
///
/// ```
/// # use cargo_cicd::adapters::ChangedFileDetector;
/// # use std::path::Path;
/// let detector = ChangedFileDetector::new();
/// let changed = detector.scan(
///     Path::new("."),
///     "abc1234"
/// )?;
/// println!("Changed files: {}", changed.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn scan_changed_files(
    workspace_root: &Path,
    last_green_commit: &str,
) -> Result<Vec<PathBuf>, ScanError> {
    // ...
}
```

### 1.3 Struct Documentation

Every public struct must document fields and invariants.

**Template:**

```rust
/// Workspace metadata from `Cargo.toml`.
///
/// Caches the result of `cargo metadata` for the duration of a session.
/// Invariant: `name` and `version` are never empty after construction.
/// Invariant: `members` contains at least one crate.
#[derive(Debug, Clone)]
pub struct WorkspaceState {
    /// Name of the workspace (or primary crate if workspace is unnamed).
    pub name: String,

    /// Workspace version from `Cargo.toml`.
    pub version: String,

    /// Resolved member crate paths, relative to workspace root.
    pub members: Vec<PathBuf>,

    /// Timestamp when metadata was last resolved.
    pub resolved_at: SystemTime,
}
```

### 1.4 Enum Documentation

Document enum purpose, each variant's meaning, and invariants.

**Template:**

```rust
/// Outcome of a test run.
///
/// Produced by test adapters; read by test planning logic.
/// Invariant: `All(true)` and `Some(n)` where n > 0 are mutually exclusive.
#[derive(Debug, Clone, Copy)]
pub enum TestOutcome {
    /// All selected tests passed.
    All(bool),

    /// Some tests failed; `passed` is the count of passing tests.
    Some { passed: usize, failed: usize },

    /// Test run was skipped (no applicable tests for this crate).
    Skipped,
}
```

### 1.5 Trait Documentation

Document trait purpose, implementor contract, and default behavior.

**Template:**

```rust
/// Adapters populate dimensions of `EngineState` from external sources.
///
/// Each adapter owns one external source (git, filesystem, cargo metadata, etc.)
/// and translates it into the internal state model. Adapters are read-only;
/// they do not perform mutations or side effects.
///
/// # Contract
///
/// Implementations must:
/// - Be deterministic (same input → same output)
/// - Not perform mutations (except `populate()`)
/// - Fail fast on transient errors (permission denied, git not available)
pub trait Adapter: Send + Sync {
    /// Populate a dimension of engine state.
    fn populate(&self, state: &mut EngineState) -> Result<(), AdapterError>;

    /// Human-readable name of the adapter (e.g., "GitStatusAdapter").
    fn name(&self) -> &'static str;
}
```

---

## 2. Code Example Standards

### 2.1 Compilation and Testing

All examples in doc comments are compiled and tested by `cargo test --doc`. Examples must:
- Compile without errors
- Run without panicking
- Use `# use` for hidden imports
- Be 5–10 lines (short and sweet)

**Good example:**

```rust
/// ```
/// # use cargo_cicd::engine::EngineState;
/// let mut state = EngineState::default();
/// state.workspace.name = "my_workspace".to_string();
/// assert_eq!(state.workspace.name, "my_workspace");
/// ```
```

**Bad example (too long, toy code):**

```rust
/// ```
/// // Don't do this: example is 30 lines of boilerplate
/// let mut state = EngineState::default();
/// // ... 25 lines of setup ...
/// ```
```

### 2.2 Realistic Examples

Examples should be realistic: copy-paste-able code that does something useful.

**Template for adapter usage:**

```rust
/// # Examples
///
/// ```
/// # use cargo_cicd::adapters::CargoMetadataAdapter;
/// # use std::path::Path;
/// let adapter = CargoMetadataAdapter::new();
/// let mut state = cargo_cicd::EngineState::default();
/// adapter.populate(&mut state)?;
/// println!("Workspace: {}", state.workspace.name);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

### 2.3 Hidden Imports

Use `# use` to hide boilerplate imports that distract from the example:

```rust
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use tempfile::TempDir;
/// let dir = TempDir::new()?;
/// let path = dir.path();
/// assert!(path.exists());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

---

## 3. Doc Strings for Non-Obvious Code

Include doc comments for public methods, functions, and associated types. Private items (functions, fields) may have less documentation, but any non-obvious private logic should be explained.

**Private method with comment:**

```rust
impl EngineState {
    /// Private helper: computes cache invalidation key from workspace + commit.
    fn cache_key(&self) -> String {
        format!("{}:{}", self.workspace.name, self.git_phase.current_branch)
    }
}
```

---

## 4. Module-Level Documentation

Every public module needs a `//!` block at the top explaining purpose and structure.

**Location:** First lines of `src/module_name/mod.rs`

**Template:**

```rust
//! Adapters translate external state into `EngineState` dimensions.
//!
//! Each adapter owns one external data source:
//! - `GitStatusAdapter` — git repository state
//! - `TargetScannerAdapter` — target directory size/age
//! - `CargoMetadataAdapter` — workspace manifest and member crates
//! - `ChangedFileDetector` — files changed since last green
//! - `ToolchainDetector` — Rust version, targets, components
//! - `TrybuildDetector` — trybuild test fixture status
//! - `CicdTomlWriter` — reads/writes `cicd.toml` state carrier
//!
//! Adapters are stateless and idempotent. They do not depend on each other.
//!
//! # Usage
//!
//! Use adapters when implementing nouns or policy engines. Example:
//!
//! ```
//! # use cargo_cicd::adapters::{Adapter, GitStatusAdapter};
//! let adapter = GitStatusAdapter::new();
//! let mut state = cargo_cicd::EngineState::default();
//! adapter.populate(&mut state)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
```

---

## 5. User-Facing Documentation

User docs live in `docs/` and `README.md`. They target users who run `cargo cicd` from the CLI.

### 5.1 Quick Start (5 min)

**File:** `docs/tutorials/GETTING_STARTED.md`

**Template:**

```markdown
# Getting Started with cargo-cicd

## Install

```sh
cargo install cargo-cicd
```

## Your First Run

Navigate to a Rust workspace:

```sh
cd ~/my_rust_project
cargo cicd status show
```

Expected output: workspace summary (dirty files, last commit, test status).

## Common Tasks

### Check workspace health

```sh
cargo cicd workspace doctor
```

### Run only changed tests

```sh
cargo cicd test changed
```

### View target directory size

```sh
cargo cicd target show
```

### Full command reference

See `cargo cicd --help` or [commands.md](../reference/commands.md).
```

### 5.2 Architecture Overview

**File:** `docs/ARCHITECTURE.md`

Explain:
- How pieces fit together
- Data flow (adapters → engine → nouns)
- Feature flags and their effects
- Integration points (wasm4pm, cicd.toml)

**Example section:**

```markdown
## Engine State

The heart of cargo-cicd is `EngineState`, a struct holding all runtime dimensions:

- **WorkspaceState** — crate names, versions, members
- **ToolchainState** — Rust version, targets, components
- **TargetState** — target directory size and age
- **ChangedFileState** — files changed since last green
- **TestPlanState** — which tests to run
- **TrybuildState** — trybuild fixture results
- **GitPhaseState** — branch, commit, ahead/behind counts
- **ProcessEventState** — events emitted during this run
- **ArtifactState** — build artifacts and checksums
- **PolicyState** — autonomic policy recommendations
- **ProjectionProfile** — feature flag combinations

### Data Flow

1. **Adapter phase**: External sources → EngineState
   - `GitStatusAdapter` reads .git/
   - `CargoMetadataAdapter` runs `cargo metadata`
   - `ChangedFileDetector` compares commits
2. **Processing phase**: EngineState → decisions
   - Policy engines read PolicyState
   - Test planner reads ChangedFileState
3. **Noun phase**: EngineState → CLI output + events
   - Nouns format and display state
   - Events are emitted to cicd.toml

See [src/adapters/mod.rs](../src/adapters/mod.rs) for adapter contracts.
```

### 5.3 API Reference

**Auto-generated from doc comments** via `cargo doc --open`.

Ensure all public types have doc comments (covered in section 1).

### 5.4 Troubleshooting Guide

**File:** `docs/TROUBLESHOOTING.md`

**Template:**

```markdown
# Troubleshooting

## "cargo cicd: command not found"

**Symptom:** Running `cargo cicd` fails with command not found.

**Fix:**

```sh
cargo install cargo-cicd
```

Then verify:

```sh
cargo cicd --version
```

## "wasm4pm oracle not found"

**Symptom:** `cargo cicd evidence doctor` fails: "wpm: command not found"

**Cause:** The wasm4pm binary is not in PATH or not installed.

**Fix:**

Set the WPM_BIN environment variable:

```sh
export WPM_BIN=/path/to/wpm
cargo cicd evidence doctor
```

Or install wasm4pm globally and ensure it's in PATH.

## "cicd.toml is stale"

**Symptom:** Commands warn "cicd.toml is stale (older than last commit)"

**Fix:**

Regenerate cicd.toml:

```sh
cargo cicd status show  # Forces re-scan
```

Or manually remove and regenerate:

```sh
rm cicd.toml
cargo cicd status show
```
```

### 5.5 Performance Tips

**File:** `docs/PERFORMANCE.md`

```markdown
# Performance Tips

## Use feature flags for heavy workspaces

The default binary is lean. For large workspaces (100+ crates), enable advanced:

```sh
cargo install cargo-cicd --features advanced
```

Advanced includes:
- Parallel filesystem scanning (`ignore` + `rayon`)
- Caching (`moka`)
- Observability (`tracing`)

## Run changed tests, not all tests

Reduces CI time by 80% on large monorepos:

```sh
cargo cicd test changed    # Only changed crates
cargo test                 # All crates (slower)
```

## Enable target directory pruning

Reclaim 50%+ of target/ on large workspaces:

```sh
cargo cicd target prune --dry-run  # See what would be freed
cargo cicd target prune             # Free it
```

## Inspect cicd.toml

The `cicd.toml` state carrier shows elapsed time per phase:

```sh
cat cicd.toml | grep elapsed_ms
```

Slow adapters can be identified and optimized.
```

### 5.6 Contributing Guide

**File:** `CONTRIBUTING.md`

```markdown
# Contributing to cargo-cicd

## Developer Setup

```sh
git clone https://github.com/yourusername/cargo-cicd
cd cargo-cicd
cargo make build
cargo make test
```

## Project Structure

- `crates/cargo-cicd/` — main library and CLI
- `crates/cargo-cicd-lsp/` — LSP server (optional)
- `docs/` — user documentation
- `ontology/` — SPARQL + TTL for noun generation
- `tests/` — integration tests (fixtures in `tests/fixtures/`)

## Making a Change

1. **Write a test first** (TDD):

```rust
#[test]
fn test_my_feature() {
    // ...
}
```

2. **Implement the feature**

3. **Update doc comments** (see DOCUMENTATION.md)

4. **Run checks**:

```sh
cargo make check
cargo make test
```

5. **Commit** with proper format:

```sh
git commit -m "feat(core): add my feature

Description of why this change.

Closes #123"
```

See [#Commit Format](#commit-format) below.

## Commit Message Format

Format: `type(scope): description`

**Types:** feat, fix, refactor, docs, test, chore
**Scopes:** core, cli, target, test, git, autonomic, docs, receipts

Example:

```
feat(cli): add --json flag to status show
refactor(core): simplify changed-file detection
docs(cli): improve git close examples
```

**Never:** Use ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 in commit messages.

## Testing

```sh
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test invariants

# All tests (feature: process-data)
cargo test --features process-data

# Doc comment examples
cargo test --doc
```

## Linting

```sh
cargo clippy --all-targets
cargo fmt --check
```

## Before Submitting

- [ ] Tests pass (`cargo make test`)
- [ ] Doc comments on all public items
- [ ] Commit message follows format
- [ ] No forbidden terms in public docs
- [ ] Examples compile (`cargo test --doc`)
```

---

## 6. README.md Standards

The README is the first touchpoint. It must:
1. Solve the problem statement (headline)
2. Show a 30-second example
3. List features
4. Link to full docs
5. Installation + quick start

**Template:**

```markdown
# cargo-cicd

`cargo-cicd` is a local-first CI/CD helper for Rust workspaces.
It keeps workspaces clean, fast, and push-ready.

## Problem Solved

You maintain a Rust workspace. You need to:
- Know what's changed since the last green commit
- Run only the tests for changed crates
- Check workspace health (duplicate deps, version skew)
- Free disk space from stale build artifacts
- Close git phases with confidence

cargo-cicd does all of this locally, without CI runners or network calls.

## Install

```sh
cargo install cargo-cicd
```

## Usage (30 seconds)

```sh
cd ~/my_rust_project
cargo cicd status show       # Workspace summary
cargo cicd test changed      # Run changed tests
cargo cicd workspace doctor  # Diagnose health
cargo cicd target prune      # Free disk space
```

See [docs/tutorials/GETTING_STARTED.md](docs/tutorials/GETTING_STARTED.md) for a full walkthrough.

## Features

- **Changed test planning** — Run only tests for changed crates
- **Workspace doctor** — Detect duplicate dependencies, version skew, toolchain mismatch
- **Git phase closure** — Merge with evidence and confidence
- **Target directory management** — View and prune stale artifacts
- **Local-first** — No CI runners, no network calls
- **Process evidence** — Emit structured events (XES format) for audit/replay
- **Autonomic policies** — Get recommendations for workspace health

## Architecture

```
Adapters (git, cargo, filesystem)
    ↓
EngineState (all runtime dimensions)
    ↓
Nouns (status, test, target, publish, …)
    ↓
Events (emitted to cicd.toml)
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Command Reference

| Command | Does |
|---------|------|
| `cargo cicd status show` | Workspace summary |
| `cargo cicd test changed` | Run changed tests |
| `cargo cicd git close` | Close branch with evidence |
| `cargo cicd target show` | Show target directory size |
| `cargo cicd target prune` | Remove stale artifacts |
| `cargo cicd workspace doctor` | Diagnose health |

Full reference: `cargo cicd --help` or [docs/reference/commands.md](docs/reference/commands.md).

## Documentation

- [Getting Started](docs/tutorials/GETTING_STARTED.md) (5 min)
- [Architecture](docs/ARCHITECTURE.md) (15 min)
- [Command Reference](docs/reference/commands.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Performance Tips](docs/PERFORMANCE.md)
- [Contributing](CONTRIBUTING.md)

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
```

---

## 7. Changelog Standards

**File:** `CHANGELOG.md`

Format: [Keep a Changelog](https://keepachangelog.com) v1.0.0

**Template:**

```markdown
# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `cargo cicd evidence doctor` now accepts `--strict` flag for receipt validation
- New `ParallelScan` adapter for 50% faster workspace scanning on large monorepos
- Feature flag `autonomic` now enables policy recommendations (suggest mode)

### Changed

- Changed file detection now ignores `.git/objects/` by default
- `cicd.toml` events now include `elapsed_ms` for performance tuning

### Fixed

- Fixed spurious "stale cicd.toml" warnings on non-git directories
- Fixed panic when `Cargo.toml` is missing `[workspace]` section

### Deprecated

- `--legacy-output` flag is deprecated; use `--format text` instead

### Removed

- Dropped support for Rust < 1.70

### Security

- Updated dependencies to patch [CVE-2024-1234](https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2024-1234)

## [0.2.5] — 2026-06-14

### Added

- `--json` output flag for all commands
- `cargo cicd trybuild changed` command
- LSP server at `crates/cargo-cicd-lsp/`

### Fixed

- Handle workspaces with non-ASCII member names

### Changed

- Nouns now emit events to `cicd.toml` immediately (not deferred)

[Unreleased]: https://github.com/yourusername/cargo-cicd/compare/v0.2.5...HEAD
[0.2.5]: https://github.com/yourusername/cargo-cicd/releases/tag/v0.2.5
```

---

## 8. Commit Message Conventions

Format: `type(scope): description`

### Types

- `feat` — New feature
- `fix` — Bug fix
- `refactor` — Code reorganization (no behavior change)
- `docs` — Documentation only
- `test` — Test additions/updates
- `chore` — Dependency updates, build config, tooling
- `perf` — Performance improvement (implies refactor)

### Scopes

- `core` — Engine, state, adapters
- `cli` — Nouns, verbs, command structure
- `target` — Target directory scanning and pruning
- `test` — Test planning and execution
- `git` — Git phase, branch closure
- `autonomic` — Policies, suggestions, recommendations
- `docs` — Documentation, examples, tutorials
- `receipts` — Event emission, cicd.toml format

### Examples

```
feat(core): add parallel workspace scanning

Enable gitignore-aware scanning using ignore + rayon.
Reduces workspace scan time by 50% on large monorepos.

fix(test): handle changed tests in virtual workspaces

Fixes #456: test planning now correctly identifies changed
tests in workspaces with [workspace] sections but no root
crate.

refactor(cli): extract common argument parsing

Consolidate --json, --format, --verbose flags in shared
argument struct.

docs(git): improve git close examples

Add example showing git close workflow with stale branches.

Closes #789
```

### Template

```
type(scope): one-liner (lowercase, no period)

Longer explanation (wrapped at 72 chars). Explain the WHY,
not the WHAT (code shows that).

Reference issues and PRs:

Closes #123
Related to #456
```

### Forbidden Terms (Never in commits)

Never use these in public commits: ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8

---

## 9. Checklist for Reviewers

When reviewing code:

- [ ] All public items have doc comments
- [ ] Doc comments explain WHY, not WHAT
- [ ] Examples in docs are realistic and compile
- [ ] Panics/Errors/Safety sections present (if applicable)
- [ ] Module docs explain purpose and related modules
- [ ] Commit message follows `type(scope): description` format
- [ ] No forbidden terms in public-facing text
- [ ] Tests pass: `cargo make test && cargo test --doc`
- [ ] Linting passes: `cargo clippy && cargo fmt --check`

---

## 10. Enforcement

### Automated Checks

```sh
# Lint + format check
cargo clippy --all-targets
cargo fmt --check

# Doc comment examples
cargo test --doc

# Full test suite
cargo make test

# Forbid terms (in CI)
grep -r "ALIVE\|Inspection Gate\|wall\|Nehemiah" \
  --include="*.md" --include="*.rs" docs/ src/
```

### Pre-Commit Hook (Optional)

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cargo fmt --check || exit 1
cargo clippy --all-targets || exit 1
```

---

## 11. Examples by Type

### Writing a New Adapter

**File:** `src/adapters/my_adapter.rs`

```rust
//! Adapts [external source] into EngineState.
//!
//! Example: `GitStatusAdapter` reads `.git/refs/` and populates `GitPhaseState`.

use crate::engine::EngineState;

/// Reads [external source] and populates engine state.
///
/// This adapter is responsible for detecting and reporting [what it detects].
/// It is deterministic and idempotent.
///
/// # Examples
///
/// ```
/// # use cargo_cicd::adapters::{Adapter, MyAdapter};
/// let adapter = MyAdapter::new();
/// let mut state = cargo_cicd::EngineState::default();
/// adapter.populate(&mut state)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MyAdapter;

impl MyAdapter {
    /// Create a new adapter instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MyAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

### Writing a New Noun

**File:** `src/nouns/my_noun.rs`

```rust
//! The `my-noun` command reads `EngineState` and produces formatted output.
//!
//! Example: `status show` reads `EngineState` and displays workspace summary.

use crate::engine::EngineState;

/// Displays [information] about the workspace.
///
/// This noun is responsible for [what it does]. It reads from `EngineState`
/// and produces human-readable or JSON output.
///
/// # Arguments
///
/// * `state` - Engine state to read from
/// * `format` - Output format (text or JSON)
///
/// # Returns
///
/// Formatted output as a string.
///
/// # Examples
///
/// ```
/// # use cargo_cicd::nouns::my_noun;
/// # use cargo_cicd::EngineState;
/// let state = EngineState::default();
/// let output = my_noun::show(&state, "text")?;
/// println!("{}", output);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn show(state: &EngineState, format: &str) -> Result<String, String> {
    match format {
        "text" => format_text(state),
        "json" => format_json(state),
        _ => Err(format!("Unknown format: {}", format)),
    }
}

fn format_text(state: &EngineState) -> Result<String, String> {
    // ...
    Ok("...".to_string())
}

fn format_json(state: &EngineState) -> Result<String, String> {
    // ...
    Ok("{}".to_string())
}
```

---

## 12. Document Maintenance

Keep documentation in sync with code:

1. **After adding a public item:** Add doc comment
2. **After renaming:** Update links in docs and examples
3. **Before release:** Review CHANGELOG for completeness
4. **Quarterly:** Audit README + architecture docs for drift

---

## 13. Testing Documentation

All doc comment examples are tested by `cargo test --doc`. To run just doc tests:

```sh
cargo test --doc
```

To debug a failing doc test:

```sh
cargo test --doc my_function -- --nocapture
```

---

**Version:** 1.0  
**Last Updated:** 2026-06-14  
**Maintainer:** cargo-cicd contributors  
