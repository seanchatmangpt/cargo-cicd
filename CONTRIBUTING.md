# Contributing to cargo-cicd

Welcome! cargo-cicd is a local-first CI/CD helper that keeps Rust workspaces clean, fast, and push-ready. This guide will help you get started contributing in just a few minutes.

## Quick Start (5 minutes)

Get up and running with your first contribution:

```sh
# Clone the repository
git clone https://github.com/seanchatmangpt/cargo-cicd.git
cd cargo-cicd

# Build the project
cargo make build    # Uses cargo-make (preferred)
# OR
cargo build         # Standard fallback

# Run the test suite
cargo make test
# OR
cargo test

# Open an issue or submit a PR
# Visit: https://github.com/seanchatmangpt/cargo-cicd/issues
```

That's it! You're ready to contribute.

---

## Development Environment Setup (15 minutes)

### Prerequisites

- **Rust 1.85 or later** — Check your version with `rustup --version` and update if needed:
  ```sh
  rustup update
  ```

### Recommended IDE Setup

#### Visual Studio Code + Rust Analyzer (Recommended)

1. Install [Visual Studio Code](https://code.visualstudio.com/)
2. Install the [Rust Analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
3. Optional: Install [Better TOML](https://marketplace.visualstudio.com/items?itemName=bungcip.better-toml) for Cargo.toml highlighting

#### JetBrains IntelliJ IDEA

1. Install [IntelliJ IDEA](https://www.jetbrains.com/idea/)
2. Install the bundled Rust plugin (IntelliJ → Settings → Plugins → Rust)
3. Open the cargo-cicd project folder

#### Emacs + rust-mode

1. Install [rust-mode](https://github.com/rust-lang/rust-mode): `(add-to-list 'load-path "~/.emacs.d/rust-mode")`
2. Install [flycheck-rust](https://github.com/flycheck/flycheck-rust) for real-time linting

### Optional: Setup Tools

#### cargo-make (Recommended)

For faster builds and standardized task running:

```sh
cargo install cargo-make
```

Once installed, use `cargo make` commands throughout this guide.

#### Pre-commit Hooks (Recommended)

Auto-format and lint your code before commit:

```sh
# Copy pre-commit hooks
cp .git/hooks/pre-commit.example .git/hooks/pre-commit  # if available
chmod +x .git/hooks/pre-commit

# Or use pre-commit framework (https://pre-commit.com/):
pip install pre-commit
pre-commit install
```

#### Claude Code (Optional)

If you're using [Claude Code](https://claude.ai/code), the project includes a `CLAUDE.md` file with AI-specific guidance. The harness will automatically pick it up.

---

## Project Structure Quick Tour (10 minutes)

```
cargo-cicd/
├── src/                           # Main source code
│   ├── main.rs                   # CLI entry point (noun-verb grammar)
│   ├── adapters/                 # External data sources (git, cargo, filesystem)
│   ├── nouns/                    # CLI nouns (status, target, test, git, etc.)
│   ├── engine/                   # Level 5 engine state (aggregate root)
│   ├── policies/                 # Autonomic policy rules (suggest mode)
│   ├── autonomic/                # Autonomic subsystem (feature: autonomic)
│   ├── advanced/                 # Advanced capabilities (feature: advanced)
│   └── integrations/             # Integration hooks (wasm4pm, etc.)
│
├── crates/                        # Workspace crates
│   ├── cargo-cicd-core/          # Core domain logic
│   ├── cargo-cicd-lsp/           # LSP server implementation
│   └── cargo-cicd/               # CLI binary
│
├── tests/                         # Integration tests
│   ├── invariants.rs             # Public boundary invariants
│   ├── cli/                      # CLI command tests
│   ├── feature_projection.rs     # Feature flag coverage
│   ├── cicd_toml_truth.rs        # State serialization tests
│   ├── autonomic_policies.rs     # Policy engine tests
│   ├── wasm4pm_*.rs              # Evidence-gate tests
│   └── fixtures/                 # Test workspace fixtures
│
├── templates/                     # Tera templates for code generation
├── ontology/                      # RDF/TTL ontology (source of truth)
├── queries/                       # SPARQL queries for code generation
├── Cargo.toml                     # Workspace manifest
├── CLAUDE.md                      # AI-specific guidance
└── CONTRIBUTING.md               # This file

Key modules:
- **src/adapters/** — GitStatusAdapter, TargetScannerAdapter, CargoMetadataAdapter, etc.
  Each adapter owns one external data source and translates it into EngineState.
- **src/engine/** — EngineState is the aggregate root; it holds all runtime dimensions.
- **src/nouns/** — Verbs read from EngineState; default verbs are injected in main.rs.
- **src/advanced/** — Hyper-fast scanning, caching, observability, dependency graphs (feature-gated).
```

### Key Files to Know

| File | Purpose |
|------|---------|
| `CLAUDE.md` | AI coding guidance, architecture, test hierarchy, forbidden terms |
| `Cargo.toml` | Workspace manifest, feature flags, dependencies |
| `src/main.rs` | CLI entry point; inspect `inject_default_verbs()` for verb routing |
| `src/engine/mod.rs` | EngineState definition (what all adapters populate) |
| `cicd.toml` | Auto-generated state file (git-ignore by default) |

---

## Common Workflows

### Adding a New Adapter

Adapters translate external data sources (git, cargo metadata, filesystem) into `EngineState`.

**Step-by-step example: Adding a new "DependencyGraphAdapter"**

1. **Create the adapter module:**
   ```rust
   // src/adapters/dependency_graph.rs
   use crate::engine::EngineState;
   use anyhow::Result;

   pub struct DependencyGraphAdapter;

   impl DependencyGraphAdapter {
       pub fn scan(state: &mut EngineState) -> Result<()> {
           // Read Cargo.toml, build dependency graph
           let metadata = cargo_metadata::MetadataCommand::new().exec()?;
           
           // Populate state dimensions
           for package in &metadata.packages {
               // ... populate state ...
           }
           
           Ok(())
       }
   }
   ```

2. **Register the adapter in `src/adapters/mod.rs`:**
   ```rust
   pub mod dependency_graph;
   pub use dependency_graph::DependencyGraphAdapter;
   ```

3. **Call the adapter in the appropriate noun verb** (e.g., `src/nouns/workspace/doctor.rs`):
   ```rust
   DependencyGraphAdapter::scan(&mut engine_state)?;
   ```

4. **Add tests** — See "Adding a Test" below.

5. **Update `cicd.toml` schema** if needed (edit `src/cicd_toml.rs`).

### Adding a New Noun/Verb

The CLI uses noun-verb grammar. Each noun is a module in `src/nouns/` implementing `NounCommand`.

**Example: Adding a `cargo cicd lock` noun**

1. **Create the noun module:**
   ```rust
   // src/nouns/lock/mod.rs
   use clap::Subcommand;
   use clap_noun_verb::NounCommand;

   #[derive(Subcommand)]
   pub enum LockVerb {
       /// Update lock file
       Update,
       /// Verify lock is fresh
       Verify,
   }

   pub struct LockNoun;

   impl NounCommand for LockNoun {
       type Verb = LockVerb;
       // Implement handle_verb() ...
   }
   ```

2. **Create verb modules:**
   ```rust
   // src/nouns/lock/update.rs
   use crate::engine::EngineState;
   use anyhow::Result;

   pub fn update(state: &EngineState) -> Result<()> {
       // Lock update logic
       Ok(())
   }
   ```

3. **Register in `src/main.rs`:**
   ```rust
   mod nouns {
       pub mod lock;
       // ... other nouns ...
   }
   ```

4. **Add tests** in `tests/cli/`.

### Adding a Test

Tests live in `/home/user/cargo-cicd/tests/`. Choose the right file:

- **Integration test (new command)** → `tests/cli/command_projection.rs`
- **State invariants** → `tests/invariants.rs`
- **Feature flags** → `tests/feature_projection.rs`
- **cicd.toml serialization** → `tests/cicd_toml_truth.rs`
- **Policy engine** → `tests/autonomic_policies.rs` (requires `autonomic` feature)
- **Evidence gate** → `tests/wasm4pm_evidence_gate.rs` (requires integration with wpm oracle)

**Example: Adding a test for a new adapter**

```rust
#[test]
fn test_dependency_graph_adapter() -> Result<()> {
    // Create a temp workspace
    let temp_dir = tempfile::TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize a minimal Cargo.toml
    let manifest = r#"
        [package]
        name = "test-crate"
        version = "0.1.0"
        edition = "2021"
    "#;
    std::fs::write(workspace_path.join("Cargo.toml"), manifest)?;

    // Create engine state and run adapter
    let mut engine_state = EngineState::new(workspace_path);
    DependencyGraphAdapter::scan(&mut engine_state)?;

    // Assert state was populated
    assert!(!engine_state.dependency_graph.is_empty());
    Ok(())
}
```

**Running tests locally:**

```sh
# All tests
cargo make test
# or
cargo test

# Single test file
cargo test --test invariants

# Single test function
cargo test --test invariants test_dependency_graph_adapter

# With a specific feature
cargo test --features process-data --test autonomic_policies

# All feature combinations (slow)
cargo test --features ""
cargo test --features "process-data"
cargo test --features "autonomic"
cargo test --features "advanced"
cargo test --features "autonomic,advanced"
```

### Running Tests with All Feature Combinations

cargo-cicd has several feature flags. Ensure your changes work with all combinations:

```sh
# Test all feature combinations
for features in "" "process-data" "autonomic" "advanced" "autonomic,advanced"; do
    echo "Testing features: $features"
    cargo test --features "$features" || exit 1
done
```

Or use a helper script:

```bash
#!/bin/bash
# scripts/test-all-features.sh
set -e
cargo test --features ""
cargo test --features "process-data"
cargo test --features "autonomic"
cargo test --features "advanced"
cargo test --features "autonomic,advanced"
echo "All feature combinations passed!"
```

### Building Documentation

```sh
# Build inline docs
cargo doc --open --no-deps

# Documentation comments follow standard Rust convention:
/// Brief description here.
///
/// Longer explanation. Include examples if helpful.
///
/// # Example
/// ```rust
/// let result = my_function()?;
/// ```
///
/// # Errors
/// Returns an error if ... (for functions returning Result)
pub fn my_function() -> Result<()> {
    // ...
}
```

---

## Code Style & Standards

### Rust Conventions

Follow standard Rust idioms. Use `cargo clippy` to catch common mistakes:

```sh
# Run clippy linter
cargo clippy --all-targets --all-features

# Fix common issues automatically
cargo clippy --fix
```

### Code Formatting

All code must be formatted with `rustfmt`:

```sh
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Error Handling

Use `anyhow` for context-aware error propagation and `thiserror` for custom error types:

```rust
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

pub fn risky_operation() -> Result<()> {
    let config = std::fs::read_to_string("config.toml")
        .context("failed to read config.toml")?;
    
    parse_config(&config)
        .context("failed to parse configuration")?;
    
    Ok(())
}
```

### Documentation Comments

Use doc comments for public APIs:

```rust
/// Scans the workspace for changed files since the last known good state.
///
/// This adapter reads the git index and compares file mtimes against
/// cached metadata stored in `cicd.toml`.
///
/// # Arguments
/// * `workspace_root` — Path to the Cargo workspace root
///
/// # Errors
/// Returns an error if:
/// - The workspace is not a valid git repository
/// - File system metadata cannot be read
///
/// # Example
/// ```rust
/// let mut engine_state = EngineState::new(workspace_root);
/// ChangedFileDetector::scan(&mut engine_state)?;
/// assert!(!engine_state.changed_files.is_empty());
/// ```
pub fn scan(state: &mut EngineState) -> Result<()> {
    // ...
}
```

### Commit Message Format

Commits must follow this format:

```
feat(core): add dependency graph adapter

Brief description (under 72 chars).

Longer explanation if needed. Reference issues:
Closes #123
```

Allowed scopes: `core`, `cli`, `target`, `test`, `git`, `autonomic`, `docs`, `receipts`

**Types:**
- `feat:` — New feature
- `fix:` — Bug fix
- `refactor:` — Code reorganization (no logic change)
- `test:` — Test additions/improvements
- `docs:` — Documentation
- `chore:` — Build, CI, tooling

### Forbidden Terms

⚠️ **NEVER use these terms in public docs, CLI help text, or commit messages:**

- ALIVE
- Inspection Gate
- wall
- Nehemiah
- Field8
- Instinct8
- Cargo Court
- AGI
- Truex
- CONSTRUCT8

These are internal implementation details and must not leak into user-facing text.

---

## Testing Requirements

All contributions must include tests. Here's what's expected:

### Coverage Targets

- **New adapters:** 80%+ coverage of public methods
- **New nouns/verbs:** 90%+ of command paths tested
- **Core engine changes:** 85%+ coverage

Run coverage locally:

```sh
# Install tarpaulin (code coverage tool)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

### Unit vs. Integration Tests

- **Unit tests** — Live in `#[cfg(test)]` modules near the code they test
- **Integration tests** — Live in `/tests/` directory, test CLI and public boundaries

Example unit test (in `src/adapters/my_adapter.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_handles_empty_workspace() -> Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let mut state = EngineState::new(temp_dir.path());
        MyAdapter::scan(&mut state)?;
        assert_eq!(state.items.len(), 0);
        Ok(())
    }
}
```

### Feature Flag Testing

Your change must work with all applicable feature combinations:

```sh
# Minimal (no features)
cargo test --features ""

# With process-data
cargo test --features "process-data"

# With autonomic (implies process-data)
cargo test --features "autonomic"

# With advanced (implies process-data)
cargo test --features "advanced"

# All features
cargo test --all-features
```

### Performance Testing

If your change adds computation (scanning, parsing, network I/O), measure performance:

```rust
#[test]
fn bench_adapter_on_large_workspace() -> Result<()> {
    let temp_dir = large_fixture_workspace()?;
    let start = std::time::Instant::now();
    
    let mut state = EngineState::new(temp_dir.path());
    MyAdapter::scan(&mut state)?;
    
    let elapsed = start.elapsed();
    println!("Scanned {} items in {:?}", state.items.len(), elapsed);
    
    // Assert reasonable performance (e.g., < 1 second for typical workspace)
    assert!(elapsed.as_secs() < 1);
    Ok(())
}
```

### Evidence-Gate Tests (Release Closing)

If your change touches critical paths (publish, git close, test changed), you may need to emit process evidence and pass the wasm4pm oracle. See `/home/user/cargo-cicd/CLAUDE.md` for details on the evidence-gate test hierarchy.

---

## Review Process

### Who Reviews

- **Core maintainers:** @seanchatmangpt and team
- **Domain experts:** Adapters reviewed by architecture owner; nouns reviewed by CLI owner
- **All PRs** require at least one approval before merge

### What Reviewers Check

- **Correctness** — Does the code do what it claims?
- **Testing** — Are edge cases covered? Do tests pass locally and in CI?
- **Documentation** — Is the change documented? Are doc comments clear?
- **Performance** — Does it regress on large workspaces?
- **Style** — Does it follow the guidelines in this document?
- **Boundaries** — Does it respect EngineState / adapter / noun separation?
- **Forbidden terms** — No ALIVE, Inspection Gate, etc. in public text

### Timeline Expectations

- **Small fixes (< 50 lines)** — 24 hours
- **Features (50-500 lines)** — 2-3 days
- **Major changes (> 500 lines)** — 1 week (plan first!)

### Responding to Feedback

1. **Acknowledge** the comment (emoji or brief reply)
2. **Discuss** if you disagree — be specific
3. **Make changes** and push (no force-push unless asked)
4. **Re-request review** once changes are made
5. **Resolve threads** when feedback is addressed

---

## Getting Help

### Before Opening an Issue

1. Check existing issues and discussions: https://github.com/seanchatmangpt/cargo-cicd/issues
2. Search closed issues for similar problems
3. Read CLAUDE.md for architectural context

### Filing a Bug Report

Use the issue template:

```markdown
## Description
Brief summary of the bug.

## Steps to Reproduce
1. Clone the repo
2. Run `cargo cicd <command>`
3. Observe error

## Expected Behavior
What should happen instead.

## Actual Behavior
What actually happened.

## Environment
- Rust version: (output of `rustc --version`)
- cargo-cicd version: (output of `cargo cicd --version`)
- OS: (Linux / macOS / Windows)
```

### Asking Questions

- **Architecture questions** → Open a discussion or ask in an issue
- **Build/test failures** → Check that you're on Rust 1.85+ and have cargo-make installed
- **Feature design** → Start with an RFC issue before implementing

### Finding Examples

The best examples are in the tests:

- **CLI parsing** — `tests/cli/command_projection.rs`
- **Adapter patterns** — `tests/fixtures/` and adapter implementations
- **State mutations** — `src/engine/` and adapter implementations
- **Error handling** — `src/adapters/` and error contexts in `anyhow`

---

## Development Checklists

### Before Submitting a PR

- [ ] Code compiles: `cargo build`
- [ ] All tests pass: `cargo make test` (or `cargo test`)
- [ ] All feature combinations pass: See "Running Tests with All Feature Combinations"
- [ ] Code is formatted: `cargo fmt`
- [ ] Clippy passes: `cargo clippy --all-targets --all-features`
- [ ] Doc comments added for public APIs
- [ ] No forbidden terms in public text
- [ ] Commit messages follow format: `feat(scope): description`
- [ ] Tests added for new code
- [ ] Coverage >= 80% for new code

### For Maintainers Merging PRs

- [ ] All CI checks pass
- [ ] At least one approval
- [ ] Commit history is clean (squash if needed)
- [ ] No merge conflicts
- [ ] Evidence tests pass (if closing release)

---

## Quick Reference

### Common Commands

```sh
# Build and test
cargo make build        # or: cargo build
cargo make test         # or: cargo test
cargo make check        # lint + type-check, no build

# Code quality
cargo fmt
cargo clippy --fix
cargo doc --open

# Run specific tests
cargo test --test invariants
cargo test --test feature_projection
cargo test --test autonomic_policies --features autonomic

# Feature combinations
cargo test --features ""
cargo test --features "process-data"
cargo test --features "autonomic"
cargo test --features "advanced"

# Clean build
cargo clean
cargo build --release
```

### Project Boundaries

| Layer | Module | Responsibilities |
|-------|--------|------------------|
| CLI | `src/nouns/` | Parse arguments, route to verbs |
| Verbs | `src/nouns/*/` | Read EngineState, format output |
| Engine | `src/engine/` | Aggregate root, dimensions, queries |
| Adapters | `src/adapters/` | Populate EngineState from external sources |
| Policies | `src/policies/` | Read PolicyState, emit recommendations |
| Advanced | `src/advanced/` | Optional high-performance capabilities |

### Key Abstractions

- **EngineState** — Holds all runtime state; adapters write, nouns read
- **NounCommand** — CLI noun implementation (clap + clap-noun-verb)
- **Adapter** — Translates external data (git, cargo) into EngineState
- **Verb** — Reads EngineState and produces output/side effects
- **PolicyState** — Read-only state for autonomic policies

---

## License

By contributing to cargo-cicd, you agree that your contributions will be licensed under the same terms as the project: [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).

---

## Questions or Feedback?

- Open an issue: https://github.com/seanchatmangpt/cargo-cicd/issues
- Start a discussion: https://github.com/seanchatmangpt/cargo-cicd/discussions
- Email: xpointsh@gmail.com

Happy contributing! 🚀
