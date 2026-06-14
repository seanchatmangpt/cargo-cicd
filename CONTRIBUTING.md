# Contributing to cargo-cicd

Thank you for your interest in contributing to cargo-cicd! This guide will help you get started with development, understand our processes, and contribute effectively.

## Quick Links

- **[Development Setup](./docs/contributing/01-development-setup.md)** — Prerequisites, one-command setup, build & test
- **[Pull Request Workflow](./docs/contributing/02-pull-request-workflow.md)** — Branch naming, commit format, code review process
- **[Adding Features](./docs/contributing/03-adding-features.md)** — Feature flags, extending EngineState, adapters
- **[Code Style & Patterns](./docs/contributing/04-code-style.md)** — Rust conventions, naming, comments, module organization
- **[Documentation Standards](./docs/contributing/05-documentation-standards.md)** — Feature docs, CLAUDE.md, visibility levels
- **[Release Process](./docs/contributing/06-release-process.md)** — Versioning, changelog, wasm4pm validation
- **[Known Gotchas](./docs/contributing/07-known-gotchas.md)** — Forbidden terms, state mutation, test isolation

## High-Level Overview

**cargo-cicd** is a Level 5 process-data engine exposed as a Rust CI/CD helper. The public-facing slogan is:

> "cargo-cicd keeps Rust workspaces clean, fast, and push-ready."

The architecture revolves around:
- **EngineState** — the aggregate root containing all runtime state dimensions
- **Adapters** — translate external sources (git, cargo, filesystem) into internal state
- **Nouns** — CLI commands organized by noun-verb grammar (e.g., `cargo cicd status show`)
- **cicd.toml** — workspace state carrier file
- **Feature flags** — `process-data`, `autonomic`, `wasm4pm`, `contrib`

## For Impatient Contributors

```bash
# Clone and enter the repo
git clone https://github.com/seanchatmangpt/cargo-cicd && cd cargo-cicd

# One-command setup (see Development Setup for details)
rustup update && cargo build

# Run tests
cargo test

# Create a feature branch
git checkout -b feat/your-feature-name

# Commit with proper format
git commit -m "feat(core): add your feature

Detailed description of what and why.

https://claude.ai/code/session_XX"

# Push and open a PR
git push -u origin feat/your-feature-name
```

## What Can You Contribute?

- **Bug fixes** — use `fix(scope):` commit prefix
- **Features** — use `feat(scope):` commit prefix
- **Refactoring** — use `refactor(scope):` commit prefix
- **Tests** — use `test(scope):` commit prefix
- **Documentation** — use `docs(scope):` commit prefix
- **Build & CI** — use `ci(scope):` commit prefix

**Valid scopes:** `core`, `cli`, `target`, `test`, `git`, `autonomic`, `docs`, `receipts`

## Important Notes

- **Forbidden terms in public docs/CLI:** ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8
- **Test requirements:** Internal smoke tests must also pass wasm4pm validation gates for any release
- **Feature flags:** Most new features should gate behind `process-data`, `autonomic`, or `contrib` flags
- **State mutation:** All state changes flow through adapters; nouns are read-only consumers

See the detailed guides linked above for comprehensive coverage of each topic.

## Questions?

Check [CLAUDE.md](./CLAUDE.md) for architecture details and internal conventions, or open an issue to ask the maintainers.

Happy contributing!
