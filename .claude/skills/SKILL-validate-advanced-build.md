---
name: validate-advanced-build
description: "Validates that advanced build features work correctly across multiple feature flag combinations. Tests cargo check, cargo test, and cargo clippy with different feature combinations to identify compatibility issues."
---

# Advanced Build Validator Skill

## Overview

The `validate-advanced-build` skill ensures that all advanced feature combinations in `cargo-cicd` compile, test, and lint correctly. It tests five key feature combinations and reports which are safe for production use.

## What It Does

This skill validates that the following feature combinations work without errors:

1. **advanced-only**: Just the `advanced` feature
2. **advanced-autonomic**: `advanced` + `autonomic` policies
3. **advanced-wasm4pm**: `advanced` + `wasm4pm` evidence gate
4. **advanced-contrib**: `advanced` + `contrib` features
5. **advanced-all**: All features combined (`advanced` + `autonomic` + `wasm4pm` + `contrib`)

For each combination, it runs:
- `cargo check --features <combo>` — Type-check without building
- `cargo test --lib --features <combo>` — Run library unit tests
- `cargo clippy --features <combo>` — Lint with clippy (deny warnings)

## Parameters

### `--quick`
Skip clippy and tests, only run `cargo check`. Useful for rapid iteration during development. Completes in 1-2 minutes instead of 5-10.

### `--verbose`
Show full output from all commands, including compiler diagnostics and test logs. Useful for debugging failures.

### `--fix`
Attempt to auto-correct clippy warnings. Uses `cargo clippy --fix --allow-dirty --allow-staged` to apply fixes automatically.

## Usage Examples

### Quick validation during development
```bash
validate-advanced-build --quick
```
Runs only `cargo check` for each combination. ~1-2 minutes.

### Full validation before release
```bash
validate-advanced-build
```
Runs all checks (check, test, clippy) for each combination. ~5-10 minutes.

### Auto-fix clippy issues with details
```bash
validate-advanced-build --verbose --fix
```
Runs all validations, shows detailed output, and attempts to auto-fix clippy warnings.

## Output

The skill produces a **compatibility matrix** like this:

```
[1/5] Testing: advanced-only
  ① cargo check: ✅ PASS
  ② cargo test:  ✅ PASS
  ③ cargo clippy: ✅ PASS
✅ advanced-only: COMPATIBLE

[2/5] Testing: advanced-autonomic
  ① cargo check: ✅ PASS
  ② cargo test:  ✅ PASS
  ③ cargo clippy: ✅ PASS
✅ advanced-autonomic: COMPATIBLE

...

════════════════════════════════════════════════════════════
SUMMARY
════════════════════════════════════════════════════════════
Total combinations tested: 5
Fully compatible:        5✅
Partially tested:        0⚠️
Incompatible:            0❌
Duration: 487s

✅ All tested combinations are compatible!
   You can safely use any combination of: advanced, autonomic, wasm4pm, contrib
```

## Output Interpretation

### Status Symbols

- **✅ PASS**: Validation step succeeded
- **❌ FAIL**: Validation step failed (error found)
- **⚠️ SKIP**: Validation step was skipped (due to `--quick` flag)

### Overall Compatibility

- **COMPATIBLE** (✅): All enabled validation steps passed
- **INCOMPATIBLE** (❌): One or more validation steps failed
- **PARTIAL** (⚠️): Some steps skipped (e.g., due to `--quick` mode)

### Exit Codes

- `0` — All tested combinations are compatible ✅
- `1` — One or more combinations are incompatible ❌
- `2` — System error or timeout occurred

## Artifact Files

The skill saves detailed reports to `target/cargo-cicd/validation-reports/`:

- `validation-{timestamp}.json` — Machine-parseable report (detailed results for each step)
- `validation-summary-{timestamp}.txt` — Human-readable summary
- Last 10 reports are retained for history

## Feature Dependency Graph

Understanding feature relationships:

```
advanced
├── process-data
├── ignore (crate: ignore v0.4)
├── rayon (parallel processing)
├── blake3 (hashing)
├── tracing (instrumentation)
├── tracing-subscriber (logging)
├── miette (diagnostics)
├── thiserror (error types)
├── moka (caching)
├── bitcode (serialization)
├── petgraph (graph algorithms)
├── jiff (dates/times)
├── hdrhistogram (histograms)
└── aho-corasick (string matching)

autonomic
└── process-data

contrib
└── process-data

wasm4pm
└── process-data
```

All features implicitly enable `process-data`. The skill tests whether this dependency graph has any conflicts.

## Common Issues and Solutions

### Compilation Error in a Specific Combination

**What it means**: Two features enable incompatible dependency versions or conflicting feature combinations.

**How to debug**:
```bash
# Re-run with full output
validate-advanced-build --verbose

# Or manually check the combination
cargo check --features advanced,autonomic
```

**What to do**: Review the error message. Usually involves:
- Conflicting dependency versions
- Mutually exclusive feature flags in dependencies
- Conditional compilation code that doesn't handle the combination

### Test Failures

**What it means**: A test fails with a specific feature combination, likely due to feature-gated code paths.

**How to debug**:
```bash
cargo test --lib --features advanced,autonomic -- --nocapture <test_name>
```

**What to do**: Check whether the failing test is properly guarded with `#[cfg(feature = "...")]` or needs adjustment for the feature combination.

### Clippy Denials

**What it means**: Clippy found warnings that are treated as errors (`-D warnings`).

**How to fix automatically**:
```bash
validate-advanced-build --fix
```

**How to fix manually**:
```bash
# See the warnings
cargo clippy --features advanced,autonomic -- -A deny_warnings

# Auto-fix them
cargo clippy --features advanced,autonomic --fix --allow-dirty --allow-staged

# Re-run clippy to verify
cargo clippy --features advanced,autonomic -- -D warnings
```

## When to Use This Skill

### Before Committing Feature-Gated Code
Ensure your changes don't break other feature combinations:
```bash
validate-advanced-build --quick
```

### Before Opening a Pull Request
Full validation to catch all issues:
```bash
validate-advanced-build
```

### Before Release
Final validation with auto-fixes:
```bash
validate-advanced-build --verbose --fix
```

### In CI/CD
Add to your CI pipeline to catch regressions:
```bash
validate-advanced-build
# Exit code tells you if the build is ready
```

## Performance Expectations

| Mode | Time | What's Tested |
|------|------|---------------|
| Quick | 1-2 min | cargo check only |
| Standard | 5-10 min | check + test + clippy |
| With --fix | 7-15 min | check + test + clippy with auto-fixes |

Times are for all 5 feature combinations combined.

## Integration with CI/CD

The skill is designed for CI/CD integration:

```bash
#!/bin/bash
set -e

# Quick validation on every commit
validate-advanced-build --quick

# Full validation on PRs
if [ "$GITHUB_EVENT_NAME" = "pull_request" ]; then
  validate-advanced-build
fi

# Auto-fix before release
if [ "$GITHUB_REF" = "refs/heads/main" ]; then
  validate-advanced-build --verbose --fix
fi
```

The exit code (0 = success, non-zero = failure) makes it easy to gate releases.

## Files

- `validate-advanced-build.json` — Skill definition (schema, parameters, documentation)
- `validate-advanced-build.sh` — Implementation script
- `target/cargo-cicd/validation-reports/` — Generated reports
- `.claude/settings.json` — Can register this skill as a Claude Code action

## Related Commands

```bash
# Check just one feature combination
cargo check --features advanced,autonomic

# Run tests with a feature combo
cargo test --lib --features advanced,wasm4pm

# Lint with a feature combo
cargo clippy --features advanced,contrib --all-targets -- -D warnings

# See all available features
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "cargo-cicd") | .features'
```
