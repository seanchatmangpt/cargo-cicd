# Development Setup

Get your environment ready to build and test cargo-cicd.

## Prerequisites

### Required
- **Rust toolchain** 1.85 or later
- **cargo-make** — for convenient build tasks (optional but recommended)
- **Git** — for version control and testing git operations

### Optional but Highly Recommended
- **wasm4pm** — for testing evidence-gate validation (required for release validation)
- **Direnv** or **mise** — for automatic environment loading (if you use them)

### System Requirements
- **Linux, macOS, or Windows** (with WSL for Windows recommended)
- **Disk space:** ~2 GB for full build artifacts
- **RAM:** 4 GB minimum, 8 GB recommended for full test suite with wasm4pm

## One-Command Setup

### macOS/Linux

```bash
# Update Rust to 1.85+
rustup update

# Clone the repo
git clone https://github.com/seanchatmangpt/cargo-cicd
cd cargo-cicd

# Build and verify
cargo build

# Run all tests (without wasm4pm validation)
cargo test
```

### Windows (WSL)
Use the same commands as Linux; WSL provides a full Linux environment.

## Full Setup with All Tools

If you plan to work on release validation or evidence-gate features:

```bash
# 1. Update Rust
rustup update

# 2. Install cargo-make (if not present)
cargo install cargo-make

# 3. Clone and build
git clone https://github.com/seanchatmangpt/cargo-cicd
cd cargo-cicd
cargo make build

# 4. Install wasm4pm (for evidence-gate validation)
# See https://github.com/seanchatmangpt/wasm4pm for detailed setup
# The binary should be available at: /Users/sac/wasm4pm/target/release/wpm (adjust for your system)
```

## Build Commands

### With cargo-make (Recommended)

```bash
# Build release binary
cargo make build

# Check (lint + type-check without building)
cargo make check

# Run all tests
cargo make test

# Full clean build and test
cargo make clean
cargo make build
cargo make test
```

### With plain cargo (Fallback)

```bash
# Build
cargo build

# Build release (optimized)
cargo build --release

# Type-check without building
cargo check

# Run all tests
cargo test

# Run a specific test file
cargo test --test invariants
cargo test --test cli
cargo test --test autonomic_policies

# Run with feature flags
cargo test --features process-data
cargo test --features autonomic
```

## Test Commands Reference

### All Tests

```bash
# Run the full test suite
cargo test

# Run with all features enabled
cargo test --all-features
```

### Integration Tests by Name

```bash
cargo test --test invariants           # Boundary invariants
cargo test --test cli                  # CLI parsing and commands
cargo test --test cicd_toml_truth      # cicd.toml schema validation
cargo test --test autonomic_policies   # Autonomic policy mode
cargo test --test changed_tests        # Changed file detection
cargo test --test git_phase_closure    # Git operations
cargo test --test feature_projection   # Feature flag contracts
```

### Run a Specific Test Function

```bash
cargo test --test invariants test_function_name -- --nocapture
```

### With Feature Flags

```bash
# Test with process-data engine
cargo test --features process-data

# Test with autonomic policies (implies process-data)
cargo test --features autonomic

# Test with contrib features
cargo test --features contrib

# Test with wasm4pm integration
cargo test --features wasm4pm
```

## Verify Your Setup

```bash
# 1. Check Rust version
rustc --version  # Should be 1.85.0 or higher

# 2. Build the binary
cargo build

# 3. Run help to verify it works
./target/debug/cargo-cicd --help

# 4. Run core tests
cargo test --test invariants

# 5. (Optional) Verify wasm4pm integration if installed
# wpm --version
```

## IDE Setup

### VS Code

1. Install the **rust-analyzer** extension
2. Install the **CodeLLDB** extension for debugging
3. Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [
    "--all-targets",
    "--all-features"
  ],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### JetBrains CLion / IntelliJ IDEA

1. Install the **Rust** plugin
2. Enable **Run external linter** in Rust settings
3. Set external linter to **clippy**

### Vim/Neovim

```bash
# Install with rust-analyzer
rustup component add rust-analyzer
```

## Environment Variables

None are required for basic development, but you can set these if helpful:

```bash
# Enable verbose logging during tests
RUST_LOG=debug cargo test

# Run tests without capturing output (see println! output)
cargo test -- --nocapture

# Parallel test execution (default)
cargo test -- --test-threads=4

# Single-threaded testing (use if tests interfere)
cargo test -- --test-threads=1
```

## Troubleshooting

### Error: "rust 1.85 or later required"

Update Rust:
```bash
rustup update
```

### Tests fail with "missing wasm4pm"

You can skip wasm4pm-dependent tests:
```bash
cargo test --test invariants           # These don't require wasm4pm
cargo test --test cli
cargo test --test feature_projection
```

Full evidence-gate tests (`tests/wasm4pm_evidence_gate.rs`) require wasm4pm to be installed. See [Release Process](./06-release-process.md) for wasm4pm setup.

### Out of disk space during build

Clean intermediate artifacts:
```bash
cargo clean          # Remove all build artifacts
cargo make clean     # If using cargo-make
```

### Compilation hangs or is very slow

Try single-threaded compilation:
```bash
cargo build -j 1
```

Or reduce parallel test threads:
```bash
cargo test -- --test-threads=1
```

## Next Steps

- Read [Pull Request Workflow](./02-pull-request-workflow.md) to understand how to structure your contribution
- Check [Code Style & Patterns](./04-code-style.md) to learn the conventions used in this project
- See [Adding Features](./03-adding-features.md) if you're implementing a new capability

## Getting Help

- Review [CLAUDE.md](../../CLAUDE.md) for architecture and internal design
- Check existing tests in `tests/` for patterns and examples
- Open an issue with `[help]` prefix to ask questions
