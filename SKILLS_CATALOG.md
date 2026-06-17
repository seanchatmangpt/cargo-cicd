# cargo-cicd Skills Catalog

A comprehensive reference for Claude Code skills designed to automate and streamline workflows in the cargo-cicd project. Each skill encapsulates a distinct operational need and integrates with the Level 5 process-data engine architecture.

---

## 1. cargo-cicd-quick-check

**Trigger Pattern:** `/quick-check` or when doing rapid iteration on features

**Description & Use Case:**
Performs a fast, targeted validation of the working tree: runs type-checking and common unit tests without full build or integration tests. Designed for pre-commit verification and during feature development to catch syntax errors and basic logic bugs quickly.

**Parameters/Configuration:**
- `--with-clippy` (optional): Include Clippy lints (default: false)
- `--target-suite` (optional): Run specific test suite by name (e.g., `invariants`, `autonomic_policies`, `cli`)
- `--feature-scope` (optional): Limit to given features; comma-separated (e.g., `process-data,autonomic`)
- `--max-jobs` (optional): Parallelism limit (default: auto-detect from CPU count)

**Example Invocations:**

```bash
# Fast baseline check: type-check + invariants test
/quick-check

# Include lint analysis for CI readiness
/quick-check --with-clippy

# Verify only autonomic-related tests (process-data + autonomic features)
/quick-check --target-suite autonomic_policies

# Check with specific features enabled
/quick-check --feature-scope process-data --target-suite feature_projection

# Parallel run limit (useful in constrained CI environments)
/quick-check --max-jobs 2
```

**Expected Output:**
```
✓ cargo check: PASS (2.3s)
✓ test::invariants: PASS (3 passed)
✓ test::autonomic_policies: PASS (8 passed)
──────────────────────────────
Summary: 3 suites, 0 failed, 11 passed (5.6s total)
Ready for commit? YES
```

---

## 2. cargo-cicd-release

**Trigger Pattern:** `/release` or when preparing a release cut

**Description & Use Case:**
Validates full release readiness by running all closing gates: invariants, feature projections, wasm4pm evidence gate (if oracle available), policy audits, and receipt validation. This is the authoritative pre-publish check—no release should be cut without it passing.

**Parameters/Configuration:**
- `--require-oracle` (optional): Fail if wasm4pm oracle (`/Users/sac/wasm4pm/target/release/wpm`) is absent (default: graceful Blocked fallback)
- `--strict-policy-mode` (optional): Fail on Warn verdicts, not just Suggest (default: fail only on Suggest/Refuse)
- `--evidence-dir` (optional): Path to evidence output directory (default: `target/cargo-cicd/evidence/`)
- `--receipts-dir` (optional): Path to receipt storage (default: `receipts/`)
- `--parallel` (optional): Run non-blocking suites in parallel (default: sequential for clarity)

**Example Invocations:**

```bash
# Standard release validation (graceful oracle handling)
/release

# Strict mode: require oracle present, fail on warnings
/release --require-oracle --strict-policy-mode

# CI/CD pipeline with explicit paths
/release --evidence-dir /tmp/ci-evidence --receipts-dir /tmp/ci-receipts --parallel

# Quick validation with oracle optional but parallel execution
/release --parallel

# Generate receipt artifacts in custom location
/release --receipts-dir ./release-artifacts/
```

**Expected Output:**
```
═══════════════════════════════════════════════════════════════
RELEASE VALIDATION GATE — cargo-cicd v26.6.2
═══════════════════════════════════════════════════════════════

Stage 1: Public Boundary Invariants
─────────────────────────────────────
✓ No forbidden terms in help text (INVARIANT 1)
✓ No destructive defaults (INVARIANT 4)
✓ No false close safety violations (INVARIANT 3)
✓ Git close requires confirmation (INVARIANT 5)
Status: PASS (4/4)

Stage 2: Feature Projection Surface
─────────────────────────────────────
✓ Default features build successfully
✓ process-data feature gates rich export
✓ autonomic implies process-data
✓ wasm4pm implies process-data
✓ No forbidden terms in feature names
Status: PASS (5/5)

Stage 3: Autonomic Policy Audit
─────────────────────────────────────
✓ Target pressure checker operational
✓ Toolchain mismatch detection working
✓ Git phase closure ready
✓ Trybuild change detection accurate
Status: PASS (4/4) — 0 Warnings, 0 Refuse verdicts

Stage 4: wasm4pm Evidence Gate
─────────────────────────────────────
Oracle Status: AVAILABLE (/Users/sac/wasm4pm/target/release/wpm)
✓ Evidence emitted as XES format
✓ Status show → Accept
✓ Target show → Accept
✓ Target prune plan → Accept
Evidence Dir: target/cargo-cicd/evidence/
Status: PASS (3/3)

Stage 5: Receipt Validation
─────────────────────────────────────
✓ Receipt doctor passed (wpm receipt doctor --format json --strict)
  Fitness: 0.98 (TRUTHFUL threshold: ≥0.95)
  Precision: 0.97
Status: PASS

═══════════════════════════════════════════════════════════════
RELEASE VERDICT: ACCEPT
═══════════════════════════════════════════════════════════════
All gates passed. Safe to publish v26.6.2.
Artifacts written to: receipts/CARGO_CICD_V26_6_2_RELEASE_CERTIFICATE.md
```

---

## 3. wasm4pm-adjudicate

**Trigger Pattern:** `/adjudicate` or when submitting evidence to the oracle

**Description & Use Case:**
Submits process evidence (XES format) to the wasm4pm oracle for adjudication. Handles oracle availability gracefully (fallback to Blocked when oracle absent), parses verdicts (Accept/Refuse/Variance/Blocked), and formats receipt doctor output. Essential for wasm4pm evidence-gate tests and release certification.

**Parameters/Configuration:**
- `--evidence-path` (required): Path to XES evidence file
- `--oracle-bin` (optional): Path to wpm binary (default: `/Users/sac/wasm4pm/target/release/wpm`)
- `--format` (optional): Output format — `json`, `plaintext`, or `receipt` (default: `plaintext`)
- `--verdict-key` (optional): Expected verdict key for assertion (e.g., `Accept`, `Refuse`)
- `--fitness-threshold` (optional): Minimum fitness score for TRUTHFUL (default: 0.95)
- `--output-receipt` (optional): Write receipt to this path (e.g., `receipts/v26.6.2.md`)

**Example Invocations:**

```bash
# Simple adjudication of an evidence file
/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes

# Assert a specific verdict and fail if mismatch
/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes \
            --verdict-key Accept

# JSON output for downstream CI processing
/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes \
            --format json

# Generate receipt artifact
/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes \
            --output-receipt receipts/RELEASE_VERDICT.md

# Custom oracle binary and strict thresholds
/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes \
            --oracle-bin /home/user/custom-wpm \
            --fitness-threshold 0.98 \
            --format receipt
```

**Expected Output:**

**plaintext mode:**
```
Adjudicating evidence: target/cargo-cicd/evidence/ci-run.xes
Oracle: /Users/sac/wasm4pm/target/release/wpm

Evidence Analysis:
  - Event count: 47
  - Activity types: [status show, target show, test run, git phase, publish run]
  - Timestamp range: 2026-06-14T09:00:00Z to 2026-06-14T09:15:23Z

Oracle Verdict: ACCEPT
  Fitness: 0.987 (TRUTHFUL — ≥0.95)
  Precision: 0.992
  Recall: 0.981
  XES health: VALID

Receipt doctor output:
{
  "verdict": "Accept",
  "fitness": 0.987,
  "precision": 0.992,
  "anomalies": []
}

Status: ACCEPT ✓
```

**json mode:**
```json
{
  "status": "success",
  "evidence_path": "target/cargo-cicd/evidence/ci-run.xes",
  "oracle_path": "/Users/sac/wasm4pm/target/release/wpm",
  "verdict": "Accept",
  "fitness": 0.987,
  "precision": 0.992,
  "recall": 0.981,
  "fitness_threshold": 0.95,
  "verdict_key_matched": true,
  "anomalies": []
}
```

**receipt mode:**
```markdown
# Evidence Adjudication Receipt — v26.6.2

**Oracle Verdict:** Accept  
**Fitness:** 0.987 (TRUTHFUL)  
**Precision:** 0.992  
**Recall:** 0.981  
**Timestamp:** 2026-06-14T09:15:47Z  

Evidence path: target/cargo-cicd/evidence/ci-run.xes  
Oracle: /Users/sac/wasm4pm/target/release/wpm  
Command: wpm receipt doctor --format json --strict <evidence.xes>  

## Verdict Analysis

All process activities conform to the Level 5 gate contract. No anomalies detected.
```

---

## 4. fixture-generator

**Trigger Pattern:** `/generate-fixture` or when building test scenarios

**Description & Use Case:**
Generates reproducible test fixtures: Rust workspace layouts, cicd.toml configurations, git repository states, and trybuild UI test suites. Accelerates test development and regression suite expansion. Supports both minimal fixtures (single crate) and complex scenarios (multi-crate with divergent git histories).

**Parameters/Configuration:**
- `--type` (required): Fixture type — `workspace`, `cicd-config`, `git-history`, `trybuild-suite`, or `full-scenario`
- `--name` (required): Fixture name/ID (e.g., `multi_crate_changed_only`)
- `--output-dir` (required): Where to write fixture (e.g., `tests/fixtures/`)
- `--crates` (optional): Comma-separated crate names (default: `cargo-cicd`)
- `--git-commits` (optional): Number of commits to simulate (default: 5)
- `--cicd-sections` (optional): Sections to populate — `workspace`, `state`, `target`, `events` (default: all)
- `--trybuild-count` (optional): Number of compile-fail/pass-ui test files (default: 10)
- `--template` (optional): Use existing fixture as template/seed (e.g., `tests/fixtures/baseline`)

**Example Invocations:**

```bash
# Create a simple workspace fixture
/generate-fixture --type workspace \
                  --name simple_single_crate \
                  --output-dir tests/fixtures/

# Complex multi-crate scenario with git history
/generate-fixture --type full-scenario \
                  --name multi_workspace_diverged \
                  --crates core,lsp,cli \
                  --git-commits 10 \
                  --output-dir tests/fixtures/

# Generate trybuild suite for UI test regression
/generate-fixture --type trybuild-suite \
                  --name trybuild_changed_only \
                  --trybuild-count 50 \
                  --output-dir tests/fixtures/

# Generate populated cicd.toml with full state
/generate-fixture --type cicd-config \
                  --name populated_state \
                  --cicd-sections workspace,state,target,events \
                  --output-dir tests/fixtures/

# Git history fixture: simulate upstream divergence
/generate-fixture --type git-history \
                  --name upstream_diverged \
                  --git-commits 20 \
                  --template tests/fixtures/baseline \
                  --output-dir tests/fixtures/

# Seed from existing fixture to avoid duplication
/generate-fixture --type workspace \
                  --name variant_of_baseline \
                  --template tests/fixtures/baseline \
                  --crates core,lsp \
                  --output-dir tests/fixtures/
```

**Expected Output:**
```
Generating fixture: multi_workspace_diverged
Type: full-scenario
Output: tests/fixtures/multi_workspace_diverged/

Creating workspace layout:
  ✓ Cargo.toml (root, resolver = 2)
  ✓ Cargo.toml (crates/cargo-cicd-core)
  ✓ Cargo.toml (crates/cargo-cicd-lsp)
  ✓ Cargo.toml (crates/cargo-cicd-cli)

Creating git history:
  ✓ Initial commit (root project setup)
  ✓ Commit 1: core feature (added 12 files)
  ✓ Commit 2: lsp integration (modified 3 files)
  ✓ Commit 3: cli refactor (modified 8 files, removed 2)
  [...]
  ✓ Commit 10: state alignment (modified 5 files)

Creating cicd.toml state:
  ✓ [workspace] section (3 members, resolved)
  ✓ [state] section (toolchain: stable, profile: release)
  ✓ [target] section (pressure: 2.3 GiB / 10 GiB)
  ✓ [[events]] (47 ProcessEvent entries)

Creating trybuild UI tests (if applicable):
  [N/A for this scenario type]

Fixture ready at: tests/fixtures/multi_workspace_diverged/
  README: tests/fixtures/multi_workspace_diverged/README.md
  Workspace: tests/fixtures/multi_workspace_diverged/Cargo.toml
  Git history: .git/ (commits 0-10)
  CI config: tests/fixtures/multi_workspace_diverged/cicd.toml
  Manifest: tests/fixtures/multi_workspace_diverged/FIXTURE.json
```

---

## 5. policy-audit

**Trigger Pattern:** `/policy-audit` or when evaluating operational state

**Description & Use Case:**
Runs the autonomic policy engine against current workspace state (or snapshot) and reports all verdicts: target pressure, toolchain mismatch, git phase closure readiness, trybuild change detection. Generates a policy report (CSV/JSON) suitable for dashboards, decision gates, or metrics collection.

**Parameters/Configuration:**
- `--snapshot-path` (optional): Path to JSON/TOML snapshot of workspace state (default: live scan)
- `--policy-set` (optional): Which policies to run — `all`, `target`, `toolchain`, `git`, `trybuild` (default: `all`)
- `--mode` (optional): Policy execution mode — `suggest`, `audit`, or `enforce` (default: `suggest`)
- `--output-format` (optional): Report format — `csv`, `json`, `markdown`, or `plaintext` (default: `markdown`)
- `--output-path` (optional): Write report to file (default: stdout)
- `--thresholds` (optional): Override policy thresholds as JSON (e.g., `{"target_gb": 15.0, "toolchain_version": "1.85"}`)

**Example Invocations:**

```bash
# Audit current workspace state, suggest mode
/policy-audit

# Audit specific policies only
/policy-audit --policy-set target,toolchain

# Generate JSON metrics for CI dashboards
/policy-audit --output-format json --output-path metrics.json

# Enforce mode with custom thresholds (would fail on Suggest)
/policy-audit --mode enforce --thresholds '{"target_gb": 12.0}'

# CSV export for spreadsheet analysis
/policy-audit --output-format csv --output-path policy-audit.csv

# Replay audit from a saved snapshot
/policy-audit --snapshot-path target/cargo-cicd/state-snapshot.json \
              --output-format markdown
```

**Expected Output:**

**markdown mode (default):**
```markdown
# Autonomic Policy Audit — 2026-06-14T10:23:45Z

## Summary
- Policies run: 4
- Verdicts: 3 Pass, 1 Warn, 0 Suggest, 0 Refuse
- Recommendation: All systems within operational envelope

## Target Pressure Policy
**Verdict:** Pass  
**Current:** 2.3 GiB / 10 GiB (23%)  
**Threshold:** 80% → Warn, 100%+ → Suggest prune  
**Recommendation:** Within limits; no action required.

## Toolchain Mismatch Policy
**Verdict:** Pass  
**System toolchain:** stable-x86_64-unknown-linux-gnu (1.85.0)  
**Project pinned:** (none — uses system)  
**Recommendation:** No mismatch detected.

## Git Phase Closure Policy
**Verdict:** Warn  
**Current HEAD:** feature/wasm4pm-integration (8 commits ahead of main)  
**Upstream status:** Diverged  
**Recommendation:** Push to origin or close phase (git phase close) before publish.

## Trybuild Change Detection Policy
**Verdict:** Pass  
**UI tests changed:** 0  
**Compile-fail suite status:** All passing  
**Recommendation:** No test regressions; trybuild suite is healthy.

---
**Generated by:** policy-audit v26.6.2  
**Snapshot:** Live (from current workspace)  
**Mode:** suggest (recommendations only, no enforcement)
```

**json mode:**
```json
{
  "timestamp": "2026-06-14T10:23:45Z",
  "workspace_root": "/home/user/cargo-cicd",
  "policy_mode": "suggest",
  "policies": [
    {
      "name": "target_pressure",
      "verdict": "Pass",
      "current_value": 2.3,
      "threshold_warn": 8.0,
      "threshold_suggest": 10.0,
      "unit": "GiB",
      "recommendation": "Within limits; no action required."
    },
    {
      "name": "toolchain_mismatch",
      "verdict": "Pass",
      "system_toolchain": "stable-x86_64-unknown-linux-gnu",
      "pinned_toolchain": null,
      "recommendation": "No mismatch detected."
    },
    {
      "name": "git_phase_closure",
      "verdict": "Warn",
      "head_branch": "feature/wasm4pm-integration",
      "commits_ahead": 8,
      "upstream_status": "diverged",
      "recommendation": "Push to origin or close phase before publish."
    },
    {
      "name": "trybuild_changed",
      "verdict": "Pass",
      "ui_tests_changed": 0,
      "compile_fail_status": "all_passing",
      "recommendation": "No test regressions; trybuild suite is healthy."
    }
  ],
  "summary": {
    "total": 4,
    "pass": 3,
    "warn": 1,
    "suggest": 0,
    "refuse": 0
  }
}
```

---

## 6. feature-flag-matrix

**Trigger Pattern:** `/feature-matrix` or when validating feature flag surface

**Description & Use Case:**
Tests all combinations of feature flags (default, `process-data`, `autonomic`, `contrib`, `wasm4pm`) to ensure:
1. Each combination builds without error
2. No forbidden terms leak into public output for any combination
3. Feature dependencies (e.g., `autonomic` → `process-data`) are enforced
4. Public API surface is consistent across all valid combinations

Generates a matrix report (CSV/HTML) showing build status, forbidden-term detection, and API coverage per combination.

**Parameters/Configuration:**
- `--skip-build` (optional): Skip compilation, only test public boundary (default: false)
- `--skip-tests` (optional): Skip test suites, only check build (default: false)
- `--help-scan` (optional): Check `--help` output for forbidden terms across all combos (default: true)
- `--report-format` (optional): `csv`, `html`, `json`, or `markdown` (default: `markdown`)
- `--output-path` (optional): Write report to file (default: stdout)
- `--parallel-jobs` (optional): Number of parallel builds (default: CPU count)

**Example Invocations:**

```bash
# Full matrix: build + tests + forbidden-term scan
/feature-matrix

# Quick surface check (no build/test, help-text only)
/feature-matrix --skip-build --skip-tests

# HTML report for stakeholder review
/feature-matrix --report-format html --output-path feature-matrix.html

# CSV for metrics tracking
/feature-matrix --report-format csv --output-path feature-coverage.csv

# Parallel builds with detailed logging
/feature-matrix --parallel-jobs 4

# Subset: just validate forbidden-term invariant
/feature-matrix --help-scan --skip-build --skip-tests
```

**Expected Output:**

**markdown mode (default):**
```markdown
# Feature Flag Matrix — Validation Report

## Summary
- Total combinations: 32
- All build: ✓ 32/32
- All pass invariants: ✓ 32/32
- API surface consistent: ✓
- Forbidden terms: ✓ None detected

## Feature Dependency Validation
- `autonomic` → `process-data`: ✓
- `contrib` → `process-data`: ✓
- `wasm4pm` → `process-data`: ✓
- `default` (empty): ✓

## Build Matrix

| Features | Build | Tests | Forbidden Terms | Public Boundary | Notes |
|----------|-------|-------|-----------------|-----------------|-------|
| (none) | ✓ | ✓ | ✓ | ✓ | Baseline — no rich export |
| `process-data` | ✓ | ✓ | ✓ | ✓ | Rich engine state available |
| `autonomic` | ✓ | ✓ | ✓ | ✓ | Policy engine active |
| `autonomic,process-data` | ✓ | ✓ | ✓ | ✓ | Redundant (autonomic implies process-data) |
| `wasm4pm` | ✓ | ✓ | ✓ | ✓ | Evidence gate active |
| `contrib` | ✓ | ✓ | ✓ | ✓ | Contribution mode |
| `autonomic,wasm4pm` | ✓ | ✓ | ✓ | ✓ | Both engines active |
| ... (28 more combinations) | | | | | |

## Public Boundary Invariants (All Combinations)
- INVARIANT 1 — Forbidden terms: ✓ PASS (no ALIVE, Nehemiah, CONSTRUCT8, etc. detected)
- INVARIANT 3 — No false close: ✓ PASS
- INVARIANT 4 — No destructive default: ✓ PASS
- INVARIANT 5 — No full trybuild by default: ✓ PASS

## Test Coverage by Feature

| Feature | Unit Tests | Integration Tests | wasm4pm Gate |
|---------|------------|-------------------|--------------|
| (default) | 7 | 12 | N/A |
| `process-data` | 7 | 12 | Gated |
| `autonomic` | 8 | 13 | Gated |
| `wasm4pm` | 7 | 12 | Required |

---
**Report generated:** 2026-06-14T10:45:12Z  
**Duration:** 4m 23s  
**Environment:** Linux 6.18.5, 8 CPU cores  
```

**html mode:** (generates interactive table with expandable rows for details)

---

## 7. workspace-health

**Trigger Pattern:** `/workspace-health` or for comprehensive system diagnostics

**Description & Use Case:**
Performs a complete health audit of the workspace: scans Cargo.toml for consistency, verifies all adapters can connect to their external sources (git, cargo metadata, toolchain), checks cicd.toml structure, ensures no stale evidence or receipt artifacts, validates test fixture integrity, and probes the Level 5 engine state initialization.

Outputs a detailed health report (color-coded) and returns exit code 0 (healthy) or 1 (unhealthy).

**Parameters/Configuration:**
- `--deep-scan` (optional): Run full file system crawl (slower, finds orphans) (default: false)
- `--repair` (optional): Auto-fix fixable issues (e.g., stale evidence cleanup) (default: false)
- `--report-format` (optional): `plaintext`, `json`, `markdown`, or `junit` (default: `plaintext`)
- `--output-path` (optional): Write report to file (default: stdout)
- `--check-fixtures` (optional): Validate test fixtures integrity (default: true)
- `--check-engine-state` (optional): Initialize EngineState and report (default: true)

**Example Invocations:**

```bash
# Quick health check (surface diagnostics)
/workspace-health

# Full deep scan with fixture validation
/workspace-health --deep-scan --check-fixtures

# Auto-repair: clean stale evidence and fix recoverable issues
/workspace-health --repair

# JSON output for programmatic handling
/workspace-health --report-format json --output-path health-report.json

# JUnit XML for CI/CD integration
/workspace-health --report-format junit --output-path health-report.xml

# Comprehensive audit with markdown report
/workspace-health --deep-scan --check-engine-state --report-format markdown \
                  --output-path WORKSPACE_HEALTH.md
```

**Expected Output:**

**plaintext mode (default):**
```
┌─────────────────────────────────────────────────────────────┐
│           WORKSPACE HEALTH REPORT — cargo-cicd              │
│           2026-06-14T10:50:33Z (v26.6.2)                    │
└─────────────────────────────────────────────────────────────┘

WORKSPACE STRUCTURE
───────────────────
✓ Root Cargo.toml present and valid
✓ Workspace resolver: 2 (correct)
✓ Members: 3 crates registered
  - . (cargo-cicd)
  - crates/cargo-cicd-core
  - crates/cargo-cicd-lsp
✓ Rust version: 1.85 (meets MSRV)
✓ Edition: 2021 (compatible)

DEPENDENCIES & VERSIONS
───────────────────────
✓ clap: 4.x (correct — for CLI parsing)
✓ clap-noun-verb: 26.6.2 (exact match)
✓ serde: 1.x + derive (correct)
✓ toml: 0.8 (compatible)
✓ anyhow: 1.x (correct)
✓ walkdir: 2 (file traversal OK)
✓ assert_cmd: 2 (dev dependency OK)
✓ tempfile: 3 (dev dependency OK)

EXTERNAL ADAPTERS
──────────────────
✓ GitStatusAdapter: Can connect (repo at /home/user/cargo-cicd/.git)
✓ TargetScannerAdapter: Can scan (target/ exists, readable)
✓ ToolchainDetector: rustc 1.85.0 stable detected
✓ CargoMetadataAdapter: Can invoke cargo metadata
✓ ChangedFileDetector: Git history accessible
✓ CicdTomlWriter: Can write to workspace root
✓ TrybuildDetector: UI test suites found (tests/fixtures/trybuild_*/)

FEATURE FLAGS
──────────────
✓ process-data: gate present, not default
✓ autonomic: implies process-data ✓
✓ contrib: implies process-data ✓
✓ wasm4pm: implies process-data ✓
✓ No forbidden feature names (ALIVE, cell8, etc.) ✓

cicd.toml STATE
────────────────
File: /home/user/cargo-cicd/cicd.toml
✓ [workspace] section: 3 members recorded
✓ [state] section: toolchain, profile, timestamp present
✓ [target] section: pressure, limits valid
✓ [[events]]: 47 ProcessEvent entries (latest: 2026-06-14T09:15:23Z)
✓ No schema violations

LEVEL 5 ENGINE STATE
─────────────────────
✓ EngineState::new() initializes successfully
✓ WorkspaceState populated: 3 crates, 2 binaries, 5 libs
✓ ToolchainState populated: stable 1.85.0, profile: release
✓ TargetState: 2.3 GiB / 10 GiB (23% pressure)
✓ ChangedFileState: 0 files modified since HEAD
✓ TestPlanState: 27 tests scanned, 0 excluded
✓ TrybuildState: 87 UI tests present, 0 compile-fail regressions
✓ GitPhaseState: HEAD on feature/wasm4pm-integration (8 commits ahead of main)
✓ ProcessEventState: 47 events, latest verdict: PASS
✓ ArtifactState: 0 leaked receipts, evidence dir clean
✓ PolicyState: All 4 policies initialized
✓ ProjectionProfile: Feature flags aligned with engine

TEST FIXTURES
──────────────
✓ tests/fixtures/ directory accessible
✓ 7 fixture scenarios found:
  ✓ simple_single_crate (workspace)
  ✓ multi_workspace_diverged (full-scenario)
  ✓ trybuild_changed_only (trybuild-suite)
  ✓ trybuild_huge_set (trybuild-suite, 50 files)
  ✓ git_baseline (git-history)
  ✓ cicd_config_template (cicd-config)
  ✓ upstream_diverged (git-history, seeded)
✓ All fixtures have README.md ✓
✓ All fixtures have FIXTURE.json manifest ✓

ARTIFACT HYGIENE
──────────────────
✓ target/cargo-cicd/evidence/: clean (last use 2026-06-14T09:15:23Z)
✓ receipts/: 8 valid receipts, none orphaned
✓ .git/: healthy (47 commits, HEAD resolvable)
✓ No stale temporary files detected

TESTS & INVARIANTS
───────────────────
Scanned test suites:
✓ invariants: 5 assertions, all depend on public boundary
✓ autonomic_policies: 8 assertions, all policies tested
✓ changed_tests: 4 assertions, impact analysis OK
✓ cli: 20 assertions, command projection complete
✓ feature_projection: 4 assertions, feature gates validated
✓ cicd_toml_truth: 3 assertions, schema enforcement
✓ git_phase_closure: 2 assertions, closure semantics
✓ wasm4pm_evidence_gate: 3 assertions, oracle submission OK
✓ wasm4pm_evidence_mutation: 2 assertions, evidence integrity
✓ wasm4pm_refusal_cases: 3 assertions, refusal path exercised

BUILD COMMANDS
───────────────
✓ cargo build: functional
✓ cargo check: functional
✓ cargo test: functional (27 suites, 162 tests)
✓ cargo clippy: warnings present but not blocking

SUMMARY
─────────
Status: HEALTHY ✓

Components:   30/30 ✓
Adapters:     7/7 ✓
Features:     5/5 ✓
Tests:        27/27 ✓
Fixtures:     7/7 ✓
Artifacts:    0 issues ✓

Exit code: 0 (healthy)
Recommendation: Workspace is ready for development and release.

┌─────────────────────────────────────────────────────────────┐
│ Generated by workspace-health v26.6.2                       │
│ Duration: 12.3s | Deep scan: disabled | Repair: disabled   │
└─────────────────────────────────────────────────────────────┘
```

**json mode:**
```json
{
  "timestamp": "2026-06-14T10:50:33Z",
  "version": "26.6.2",
  "status": "healthy",
  "exit_code": 0,
  "summary": {
    "components_checked": 30,
    "components_healthy": 30,
    "adapters_checked": 7,
    "adapters_healthy": 7,
    "fixtures_checked": 7,
    "fixtures_healthy": 7,
    "tests_found": 27,
    "issues_found": 0
  },
  "workspace": {
    "root": "/home/user/cargo-cicd",
    "members": 3,
    "rust_version": "1.85",
    "edition": "2021"
  },
  "adapters": [
    {
      "name": "GitStatusAdapter",
      "status": "healthy",
      "message": "Can connect (repo at /home/user/cargo-cicd/.git)"
    },
    {
      "name": "ToolchainDetector",
      "status": "healthy",
      "message": "rustc 1.85.0 stable detected"
    }
  ],
  "engine_state": {
    "workspace_state": "initialized",
    "crate_count": 3,
    "test_count": 27,
    "target_pressure_gb": 2.3,
    "git_phase": "feature/wasm4pm-integration",
    "commits_ahead": 8
  },
  "recommendations": [
    "Workspace is ready for development and release."
  ]
}
```

---

## Skill Invocation Reference

All skills are invoked via the `/skill-name` syntax in Claude Code. Examples:

```bash
# In Claude Code terminal or chat:
/quick-check
/quick-check --with-clippy --feature-scope autonomic

/release --require-oracle --strict-policy-mode --parallel

/adjudicate --evidence-path target/cargo-cicd/evidence/ci-run.xes \
            --verdict-key Accept --output-receipt receipts/RELEASE.md

/generate-fixture --type full-scenario \
                  --name multi_workspace_diverged \
                  --crates core,lsp,cli \
                  --output-dir tests/fixtures/

/policy-audit --policy-set target,toolchain --output-format json

/feature-matrix --report-format html --output-path feature-matrix.html

/workspace-health --deep-scan --repair --report-format markdown
```

---

## Integration with CI/CD Pipelines

These skills are designed for both interactive development and automated CI/CD:

### GitHub Actions Example
```yaml
- name: Quick Check
  run: cargo-code /quick-check --with-clippy

- name: Release Validation
  run: cargo-code /release --parallel --evidence-dir /tmp/evidence

- name: Feature Matrix
  run: cargo-code /feature-matrix --report-format csv --output-path matrix.csv

- name: Workspace Health Report
  run: cargo-code /workspace-health --deep-scan --output-path health.json
```

### Local Pre-Commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo-code /quick-check || exit 1
cargo-code /policy-audit --mode suggest || exit 1
```

---

## Troubleshooting & Support

### Common Issues

**Q: Why does `/release` exit with `Blocked` instead of `Accept`?**
A: The wasm4pm oracle (`/Users/sac/wasm4pm/target/release/wpm`) is not found. Either:
1. Install the oracle binary, or
2. Run `/release` without `--require-oracle` (default handles gracefully)

**Q: How do I debug a policy audit verdict?**
A: Use `--snapshot-path` to replay a previous state:
```bash
# Capture state
/workspace-health --report-format json --output-path state.json
# Later, replay
/policy-audit --snapshot-path state.json --output-format json
```

**Q: Can I generate fixtures from a template?**
A: Yes! Use `--template`:
```bash
/generate-fixture --type workspace \
                  --name variant_of_baseline \
                  --template tests/fixtures/baseline \
                  --output-dir tests/fixtures/
```

**Q: How are feature-matrix results used for release gates?**
A: Embed the matrix check in your CI release job:
```bash
/feature-matrix --skip-tests  # Fast: just build + boundary check
exit_code=$?
if [ $exit_code -ne 0 ]; then
  echo "Feature matrix validation failed"
  exit 1
fi
```

---

## Glossary & Concepts

**Evidence**: XES (XML Event Stream) logs emitted by the Level 5 process-data engine, used by the wasm4pm oracle for adjudication.

**Receipt**: Artifact written by `CicdTomlWriter`, records process events and wasm4pm verdict (Accept/Refuse/etc.).

**Oracle**: The wasm4pm process engine (`wpm` binary) that evaluates evidence fitness and precision.

**Policy**: Autonomic decision rule (e.g., target pressure, toolchain mismatch). Verdict: Pass/Warn/Suggest/Refuse.

**Fixture**: Reproducible test workspace (Cargo.toml, git history, cicd.toml, trybuild suite).

**Feature Projection**: Mapping of feature flags (process-data, autonomic, wasm4pm) to runtime capabilities.

**Invariant**: Non-negotiable public boundary constraint (e.g., no forbidden terms, no destructive defaults).

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14  
**Maintained By:** cargo-cicd development team  
**Contact:** xpointsh@gmail.com
