# Development Setup

This guide will get you up and running with cargo-cicd development in minutes.

## Prerequisites

### Required
- **Rust 1.85 or later** (as specified in `Cargo.toml`)
  - Install or update: `rustup update`
- **Git** (for repository interaction and adapters)
- **cargo-make** (optional but strongly recommended)
  - Install: `cargo install cargo-make`

### Optional but Recommended
- **wasm4pm** (for release validation; path: `/Users/sac/wasm4pm/target/release/wpm`)
  - Needed only when running evidence-gate tests or preparing releases
- **A Rust IDE** (VS Code with rust-analyzer recommended)

## One-Command Setup

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/cargo-cicd
cd cargo-cicd

# Run setup (updates Rust, builds project, runs tests)
rustup update && cargo build && cargo test
```

That's it! You can now develop.

## Build Commands

### Using cargo-make (Preferred)

If you have `cargo-make` installed:

```bash
# Build the project
cargo make build

# Check code (lint + type-check, no build)
cargo make check

# Run all tests
cargo make test

# Clean build artifacts
cargo make clean
```

### Using cargo Directly

If `cargo-make` is unavailable:

```bash
# Build
cargo build

# Type-check and lint
cargo check

# Run all tests
cargo test

# Build with optimizations
cargo build --release
```

## Test Commands

### Integration Tests

Run specific integration test suites:

```bash
# Public boundary invariants (no forbidden terms, safety, etc.)
cargo test --test invariants

# CLI command projection test
cargo test --test cli

# cicd.toml correctness
cargo test --test cicd_toml_truth

# Autonomic policies
cargo test --test autonomic_policies

# Changed test detection
cargo test --test changed_tests

# Git phase closure
cargo test --test git_phase_closure

# Feature flag projection
cargo test --test feature_projection

# wasm4pm evidence gate (requires wasm4pm binary)
cargo test --test wasm4pm_evidence_gate
```

### Unit Tests

Run all unit tests (within each module):

```bash
cargo test --lib
```

### Specific Test Function

```bash
# Run a single test function by name
cargo test --test invariants invariant_public_boundary
```

### With Feature Flags

```bash
# Test with process-data feature enabled
cargo test --features process-data

# Test with autonomic feature (implies process-data)
cargo test --features autonomic

# Test with wasm4pm feature (for release validation)
cargo test --features wasm4pm
```

### Full Test Suite

```bash
# Run everything (lib + integration tests)
cargo test

# Run with all features
cargo test --all-features
```

## Common Development Workflows

### Starting a New Feature

```bash
# Update Rust first
rustup update

# Create a feature branch
git checkout -b feat/my-feature

# Build and verify setup
cargo build

# Run tests to establish baseline
cargo test

# Now implement your feature...
```

### Debugging a Test Failure

```bash
# Run the failing test with output visible
cargo test test_name -- --nocapture

# Or with rust backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture
```

### Before Committing

```bash
# Check formatting (requires rustfmt)
cargo fmt --check

# Check linting (requires clippy)
cargo clippy -- -D warnings

# Run all tests
cargo test

# Try a clean build
cargo clean && cargo build && cargo test
```

## Project Structure

```
cargo-cicd/
├── src/
│   ├── main.rs              # CLI entry point, noun registration
│   ├── lib.rs               # Library exports
│   ├── cicd_toml.rs         # cicd.toml schema & parsing
│   ├── evidence.rs          # Process event emission (XES)
│   ├── session.rs           # Session lifecycle
│   ├── nouns/               # CLI noun (command) implementations
│   │   ├── status.rs        # `cargo cicd status`
│   │   ├── target.rs        # `cargo cicd target`
│   │   ├── test.rs          # `cargo cicd test`
│   │   └── ...
│   ├── adapters/            # External source adapters
│   │   ├── git_status.rs    # Git repository state
│   │   ├── target_scanner.rs # target/ directory scanning
│   │   ├── toolchain_detector.rs # Rust toolchain info
│   │   └── ...
│   ├── engine/              # EngineState and dimensions
│   │   ├── mod.rs           # EngineState aggregate root
│   │   ├── workspace_state.rs
│   │   ├── git_phase_state.rs
│   │   └── ...
│   ├── state/               # Additional state structures
│   ├── autonomic/           # Autonomic policies (feature-gated)
│   ├── policies/            # Policy implementations
│   └── integrations/        # External integrations (wasm4pm, etc.)
├── tests/
│   ├── invariants.rs        # Public boundary tests
│   ├── cli/                 # CLI command tests
│   ├── feature_projection.rs # Feature flag surface tests
│   ├── cicd_toml_truth.rs   # cicd.toml correctness
│   ├── fixtures/            # Test fixture workspaces
│   └── ...
├── Cargo.toml               # Workspace & package config
├── CLAUDE.md                # Internal architecture (detailed)
├── CONTRIBUTING.md          # Quick reference (links here)
└── docs/
    └── contributing/        # This contributor guide
        ├── 01-development-setup.md
        ├── 02-pull-request-workflow.md
        ├── ...
```

## Troubleshooting

### Rust Version Too Old

```bash
# Check your current version
rustc --version

# Update to latest stable
rustup update

# Or use nightly if needed
rustup install nightly
rustup default nightly
```

### "cargo-make not found"

Install it:
```bash
cargo install cargo-make
```

Or use plain `cargo` commands instead (all `cargo make X` commands have `cargo X` equivalents).

### Build Fails on Linux

Ensure you have build tools:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"
```

### Tests Timeout

Some tests (especially wasm4pm tests) can take a minute. Use:
```bash
# Increase timeout (in seconds)
cargo test -- --test-threads=1
```

### Git Commands Fail in Tests

Ensure git is initialized in the test directory. Most tests use `FixtureWorkspace` which handles this automatically. If writing custom tests, initialize git:
```rust
std::process::Command::new("git")
    .args(&["init"])
    .current_dir(&test_dir)
    .output()?;
```

## Next Steps

Once setup is complete:

1. **Read [CLAUDE.md](../../CLAUDE.md)** for a deep dive into architecture
2. **Review [02-pull-request-workflow.md](./02-pull-request-workflow.md)** before making your first commit
3. **Check [03-adding-features.md](./03-adding-features.md)** if you're implementing new functionality
4. **Browse tests/** for examples of how features are tested

Happy coding!
