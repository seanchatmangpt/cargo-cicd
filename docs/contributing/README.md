# Contributor Guide to cargo-cicd

Welcome to the cargo-cicd contributor community! This guide will help you understand the codebase, contribute effectively, and maintain our high standards for code quality and documentation.

## Quick Start

If you just want to get coding:

```bash
rustup update && cargo build && cargo test
git checkout -b feat/your-feature
# ... make changes ...
cargo test && cargo fmt && cargo clippy -- -D warnings
git commit -m "feat(scope): your change"
git push -u origin feat/your-feature
```

Then open a pull request. See [02-pull-request-workflow.md](./02-pull-request-workflow.md) for details.

## The Guides

This contributor guide is organized into 7 focused documents:

### 1. **[Development Setup](./01-development-setup.md)** — Getting Started
- Prerequisites (Rust 1.85+, cargo-make, git)
- One-command setup
- Build and test commands
- Project structure overview
- Troubleshooting

**Read this if:** You're setting up your development environment for the first time.

### 2. **[Pull Request Workflow](./02-pull-request-workflow.md)** — Collaboration
- Branch naming conventions
- Commit message format (feat/fix/docs/etc.)
- PR checklist and description template
- Code review expectations
- Merge process and revert policy

**Read this if:** You're preparing to submit code for review.

### 3. **[Adding Features](./03-adding-features.md)** — Implementation Patterns
- Adding new CLI nouns and verbs
- Creating adapters (external data translators)
- Extending EngineState (workspace state model)
- Adding autonomic policies
- Feature flag usage
- Testing patterns with fixtures

**Read this if:** You're implementing a new feature or extending the system.

### 4. **[Code Style & Patterns](./04-code-style.md)** — Quality Standards
- Rust conventions and formatting
- Naming patterns (modules, types, functions, variables)
- Comment and documentation guidelines
- Module organization and visibility
- Feature-gated code
- Testing code style

**Read this if:** You want to understand our code quality expectations.

### 5. **[Documentation Standards](./05-documentation-standards.md)** — Visibility & Clarity
- Public API documentation (doc comments with examples)
- Feature-gated documentation
- Updating CLAUDE.md for architectural changes
- Visibility levels (public API vs. internal)
- Forbidden terms in public output
- Writing clear, runnable examples

**Read this if:** You're documenting new features or updating architecture docs.

### 6. **[Release Process](./06-release-process.md)** — Shipping
- Semantic versioning (MAJOR.MINOR.PATCH)
- Changelog format (Keep a Changelog style)
- wasm4pm evidence validation gates (required for releases)
- Release checklist and step-by-step process
- Publishing to crates.io
- Hotfix procedures
- Deprecation policy

**Read this if:** You're preparing a release or want to understand our release gates.

### 7. **[Known Gotchas](./07-known-gotchas.md)** — Pitfalls & Prevention
- Forbidden terms in public output
- State mutation patterns to avoid
- Test isolation failures
- Feature flag gating mistakes
- Adapter query anti-patterns
- cicd.toml consistency issues
- Evidence emission requirements
- Troubleshooting quick reference

**Read this if:** A test is failing or you need to debug an issue.

## High-Level Architecture

cargo-cicd is a **Level 5 process-data engine** exposed as a boring Rust CI/CD helper. Here are the core concepts:

### EngineState: Single Source of Truth

All workspace state is modeled in `EngineState` — an immutable snapshot of:
- Workspace metadata (Cargo.toml, members, edition)
- Toolchain (Rust version, components)
- Changed files (git diff, untracked)
- Test plan (which tests to run)
- Git phase (branch, dirty status, ahead/behind)
- Artifacts (binaries, rlibs)
- Policies (autonomic recommendations)

See [CLAUDE.md](../../CLAUDE.md) for the complete architecture.

### Adapters: External Data Translation

Each **adapter** owns one external source and translates it to internal state:

- `GitStatusAdapter` → `GitPhaseState` (reads `git status --porcelain`)
- `TargetScannerAdapter` → `TargetState` (walks `target/` directory)
- `CargoMetadataAdapter` → `WorkspaceState` (reads `Cargo.toml`)
- etc.

Adapters have **no business logic** — only translation. Errors propagate up.

### Nouns & Verbs: CLI Grammar

CLI commands use noun-verb structure:
- **Nouns** are command namespaces: `cargo cicd status`, `cargo cicd target`, `cargo cicd test`
- **Verbs** are subcommands: `show`, `apply`, `prune`, `audit`

Each noun is a module in `src/nouns/` implementing `NounCommand`. Each verb implements `VerbCommand`.

### cicd.toml: State Carrier

`cicd.toml` in the workspace root caches state and emitted events. It's written by adapters, read by nouns.

### Feature Flags

- **`process-data`** — enables Level 5 engine internals (EngineState, adapters, policies)
- **`autonomic`** — implies `process-data`; enables policy/suggest mode
- **`wasm4pm`** — implies `process-data`; enables evidence-gate integration
- **`contrib`** — implies `process-data`; contributor-only utilities

## Common Tasks

### I want to add a new CLI command

1. Create a new noun module in `src/nouns/my_noun.rs`
2. Implement `NounCommand` trait with verbs
3. Register in `src/main.rs`
4. Write tests in `tests/cli/`
5. Check: no forbidden terms in help text (`cargo test --test invariants`)

See [03-adding-features.md](./03-adding-features.md) for step-by-step.

### I want to add a new state dimension

1. Define state struct in `src/engine/my_dimension.rs`
2. Add to `EngineState` in `src/engine/mod.rs`
3. Create adapter in `src/adapters/my_source.rs`
4. Update CLAUDE.md with the new dimension
5. Test with `FixtureWorkspace`

See [03-adding-features.md](./03-adding-features.md) for details.

### I want to add a policy

1. Create policy in `src/policies/my_policy.rs`
2. Implement decision logic reading `PolicyState` and other dimensions
3. Register in autonomic mode (feature-gated behind `autonomic`)
4. Test policy verdicts
5. Document in CLAUDE.md

See [03-adding-features.md](./03-adding-features.md) for patterns.

### I found a bug

1. Create a test that reproduces it
2. Fix the bug
3. Verify test passes
4. Commit with `fix(scope): description`
5. Open a PR

### My test is failing

1. Check [07-known-gotchas.md](./07-known-gotchas.md) for common causes
2. Run test with output: `cargo test test_name -- --nocapture`
3. Use `FixtureWorkspace` to isolate from external state
4. Check git state in the test: `git status --porcelain`
5. Inspect the fixture directory if needed

### A review mentioned "forbidden term"

1. The term (ALIVE, Nehemiah, etc.) appeared in public output
2. Find it: `grep -r "ALIVE" src/`
3. Remove or reword it
4. Re-run invariants test: `cargo test --test invariants invariant_public_boundary`

## Important Rules

1. **No silent failures** — Adapters propagate errors; nouns don't swallow them
2. **No destructive operations by default** — `--confirm` flag required for `prune`, etc.
3. **Immutable snapshots** — `EngineState` is read-only in nouns; mutations flow through adapters
4. **Feature gating** — New engine code goes behind `process-data`, new policies behind `autonomic`
5. **Public boundaries** — No forbidden terms in help text, stdout, or error messages
6. **Test isolation** — Tests use `FixtureWorkspace`, never depend on real git/filesystem state
7. **Evidence gates** — Releases require wasm4pm validation (evidence-gate tests must pass)

## External Documentation

- **[CLAUDE.md](../../CLAUDE.md)** — Internal architecture deep-dive, state models, adapter patterns
- **[README.md](../../README.md)** — Public-facing project description
- **[ARCHITECTURE.md](../../ARCHITECTURE.md)** — Design principles and rationale

## Getting Help

- **For architecture questions** — Read [CLAUDE.md](../../CLAUDE.md) first
- **For style questions** — See [04-code-style.md](./04-code-style.md)
- **For test failures** — Check [07-known-gotchas.md](./07-known-gotchas.md)
- **For release questions** — Read [06-release-process.md](./06-release-process.md)
- **For design decisions** — Open an issue or ask a maintainer

## Before You Commit

```bash
# Update Rust
rustup update

# Build
cargo build

# Run all tests
cargo test

# Check formatting
cargo fmt --check

# Check lints
cargo clippy -- -D warnings

# Run invariants (no forbidden terms)
cargo test --test invariants

# Commit with proper format
git commit -m "feat(scope): description

Detailed explanation if needed.

https://claude.ai/code/session_XX"

# Push
git push -u origin feat/your-feature
```

## Next Steps

1. **Read [01-development-setup.md](./01-development-setup.md)** to set up your environment
2. **Read [CLAUDE.md](../../CLAUDE.md)** to understand the architecture
3. **Find an issue to work on** or plan a feature
4. **Follow [02-pull-request-workflow.md](./02-pull-request-workflow.md)** when submitting code
5. **Reference [03-adding-features.md](./03-adding-features.md)** for implementation patterns

## License

cargo-cicd is licensed under MIT OR Apache-2.0. By contributing, you agree to license your contributions under the same terms.

Happy contributing!
