# cargo-cicd Contributor Quick Start

Welcome to cargo-cicd. This document is the first thing a new contributor should read. It covers prerequisites, the first five minutes, architecture, common tasks, slash commands, and the key files you need to know.

For deeper reference, read `CLAUDE.md` (single source of truth) and `.claude/ARCHITECTURE.md` (visual diagrams and data flow).

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [First 5 Minutes](#2-first-5-minutes)
3. [Understanding the Architecture](#3-understanding-the-architecture)
4. [Common Tasks](#4-common-tasks)
5. [Claude Code Slash Commands](#5-claude-code-slash-commands)
6. [Key Files to Know](#6-key-files-to-know)
7. [Forbidden Terms](#7-forbidden-terms-never-use-in-help-text)
8. [Commit Format](#8-commit-format)
9. [Getting Help](#9-getting-help)

---

## 1. Prerequisites

### Required

**Rust stable toolchain**

cargo-cicd targets stable Rust. Install via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustc --version  # should print rustc 1.7x.0 or newer
```

**Nightly toolchain (for trybuild tests)**

Some compiler error snapshot tests (`trybuild changed`) require nightly:

```bash
rustup install nightly
rustup show  # verify both stable and nightly appear
```

**cargo-make**

All canonical build and test commands use cargo-make. Install it once:

```bash
cargo install cargo-make
cargo make --version  # verify
```

Without cargo-make, you can fall back to bare `cargo build` and `cargo test`, but the Makefile.toml workflows are the authoritative entry points.

### Optional (required only for release gating)

**wpm oracle (wasm4pm)**

The `wpm` binary adjudicates all process evidence before release. It is not required for local development. Evidence gate tests that cannot reach the oracle report `ExpectedWpmVerdict::Blocked`, which is a valid offline state — not a failure.

To install:

```bash
# Build from the wasm4pm source repository
cd /path/to/wasm4pm
cargo build --release
export PATH="/path/to/wasm4pm/target/release:$PATH"
wpm --version
```

**ggen (code generation)**

The CLI grammar is manufactured from an RDF/Turtle ontology. If you add or rename nouns or verbs, you need `ggen` to regenerate derived files (README sections, test scaffolding, reference docs). For code changes that do not touch the ontology, `ggen` is not required.

```bash
cargo install ggen  # or build from source
ggen --version
```

---

## 2. First 5 Minutes

Run these five steps in order. If any step fails, stop and read the error before continuing.

### Step 1: Clone the repo

```bash
git clone <repo-url> cargo-cicd
cd cargo-cicd
```

### Step 2: Build the binary

```bash
cargo make build
```

Expected output: compilation succeeds, no errors. Warnings about unused code are normal during active development. The binary is placed at `target/debug/cargo-cicd`.

Fallback if cargo-make is unavailable:

```bash
cargo build
```

Verify the binary was produced:

```bash
./target/debug/cargo-cicd --version
# cargo-cicd 26.6.2
```

### Step 3: Lint check (no build artifacts required)

```bash
cargo make check
```

This runs `cargo check` (type-checking) and `cargo clippy` (lint). It is faster than a full build and catches most errors immediately. Should exit 0 on a clean checkout.

### Step 4: Run the binary on the repo itself

```bash
./target/debug/cargo-cicd status
```

This invokes the `status show` verb (the default verb for the `status` noun) against the current workspace. Expected output: a workspace health snapshot showing your branch name, git phase, toolchain version, and target directory size.

If you see errors about "no Cargo.toml found", you are not in a workspace root. Run from the `cargo-cicd/` directory.

### Step 5: Run the 7 public boundary invariants

```bash
cargo test --test invariants
```

Expected output: all 7 tests pass. These invariants enforce the non-negotiable public contract of the CLI:

1. No forbidden terms in any `--help` output
2. No destructive action without `--confirm`
3. No full trybuild run by default (conservative mode)
4. Noun names are lowercase ASCII
5. Binary name is `cargo-cicd`
6. `status show` exits 0
7. `git close` prints a safety warning

If any invariant fails on a fresh checkout, something is broken upstream. Check recent commits and open an issue before proceeding.

---

## 3. Understanding the Architecture

### The Manufacturing Pipeline

The CLI grammar is **manufactured, not handwritten**. You do not write clap argument structs by hand. Instead:

```
ontology/cargo-cicd-capabilities.ttl   (RDF/Turtle — the ground truth)
    |
    v  [ggen reads ggen.toml + SPARQL rules + Tera templates]
    |
    +-> src/nouns/*.rs           (noun/verb module stubs)
    +-> tests/cli/               (CLI test scaffolding)
    +-> README.md                (generated command reference)
    +-> docs/reference/commands/ (per-command markdown)
```

Consequence: if you add a new verb to the ontology and run `ggen`, the module stub appears automatically. You only implement the business logic. If you skip `ggen` and write a noun by hand, your code will diverge from the ontology and future `ggen` runs will overwrite it.

### Noun-Verb CLI Grammar

The binary exposes a strict noun-verb grammar. Every command is `cargo cicd <noun> <verb> [flags]`. There are 10 nouns:

| Noun | Default verb | Purpose |
|------|-------------|---------|
| `status` | `show` | Workspace health snapshot |
| `git` | — | Git phase tracking and closure |
| `test` | — | Selective test execution by changed files |
| `trybuild` | — | Compiler error snapshot tests |
| `target` | — | Target directory analysis and cleanup |
| `workspace` | `doctor` | Workspace-wide diagnostics |
| `publish` | `run` | Artifact publishing gate |
| `evidence` | `doctor` | Process evidence emission and adjudication |
| `pipeline` | — | Sequential execution of all CI/CD activities |
| `lsp` | — | Language server for IDE integration |

Default verb injection means bare nouns work as shortcuts:

```bash
cargo cicd status          # same as: cargo cicd status show
cargo cicd workspace       # same as: cargo cicd workspace doctor
cargo cicd evidence        # same as: cargo cicd evidence doctor
cargo cicd publish         # same as: cargo cicd publish run
```

This injection is implemented in `src/main.rs::inject_default_verbs()`.

Verb categories:
- **Read-only** (`show`, `status`, `explain`, `doctor`) — never mutate state
- **Dry-run** (`prune --dry-run`) — planning only, no side effects
- **Execution** (`run`, `close`) — may mutate state; require `--confirm` for destructive operations
- **Adjudication** (`audit`) — submits evidence to wasm4pm oracle

### EngineState as Aggregate Root

All runtime state lives in `EngineState`, defined in `src/engine/mod.rs`. It is the single struct that nouns and policies read from. Adapters populate it. Nothing outside of adapters writes to it during initialization.

```
EngineState {
    workspace     <- CargoMetadataAdapter
    toolchain     <- ToolchainDetector
    target        <- TargetScannerAdapter
    changed_files <- ChangedFileDetector
    test_plan     <- derived from changed_files
    trybuild      <- TrybuildDetector
    git_phase     <- GitStatusAdapter
    process_events<- populated by verbs at runtime
    artifacts     <- populated by verbs at runtime
    policies      <- populated by policy runner
    projection    <- feature flag contract
}
```

Construction: `EngineState::from_workspace()` calls all adapters in sequence. Adapter failures are **silently swallowed** — partial data is preferred over a crash. This means if `git` is not available, `git_phase` defaults to empty values rather than panicking.

### Adapters as Pure Translators

Adapters are stateless. Every method is either `fn()` (static) or `fn(&self)` (no mutable state). They translate one external source into one `EngineState` dimension.

Rules:
1. One adapter = one external source
2. Adapters never call other adapters
3. Adapters never panic — they return `anyhow::Result` and callers swallow errors
4. Adapters contain no business logic — only translation

The slowest adapter is `TargetScannerAdapter`, which performs a recursive `walkdir` traversal over the entire `target/` directory. On large workspaces this can take 1–5 seconds. The result is cached in `cicd.toml`.

### Evidence Emission Pattern

Every verb that does work must emit process evidence. The pattern is always:

```
start event → [perform work] → complete event → [optional oracle adjudication]
```

Evidence is emitted as:
- `target/cargo-cicd/evidence/*.xes` — XES (XML Event Stream) for wasm4pm
- `target/cargo-cicd/evidence/*.jsonl` — JSONL companion (same events, machine-readable)

Tests assert on the **wasm4pm verdict** (`Accept`/`Refuse`/`Blocked`), never on internal cargo-cicd state. This is enforced by invariant E4.

### The cicd.toml State Carrier

`cicd.toml` in the workspace root is the persistent state carrier. It is written by `CicdTomlWriter` after major operations and read back by nouns and policies. It is **not** a config file — it will be overwritten by the next verb run. Do not rely on manual edits surviving.

### Feature Flags

```
default = []
process-data = []           # Level 5 engine internals (opt-in)
autonomic = [process-data]  # Policy suggestions (implies process-data)
contrib   = [process-data]  # Developer diagnostics
wasm4pm   = [process-data]  # Oracle integration
```

The default build produces a minimal binary with no Level 5 engine. All non-default features imply `process-data`. The engine is opt-in by design so that end users who install the binary do not get internal machinery by default.

---

## 4. Common Tasks

### 4.1 Add a New Verb to an Existing Noun

**Scenario:** Add `target repair` to the `target` noun.

**Step 1 — Edit the ontology**

```turtle
# ontology/cargo-cicd-capabilities.ttl
cc:target-repair a skos:Concept ;
    cc:noun "target" ;
    cc:verb "repair" ;
    cc:cliCommand "cargo cicd target repair" ;
    dcterms:description "Repair target directory issues (e.g., stale locks)" .
```

**Step 2 — Regenerate from ontology**

```bash
ggen
```

This updates `README.md`, test scaffolding in `tests/cli/`, and reference docs. Check `git diff` to see what changed.

**Step 3 — Implement the verb handler**

```rust
// src/nouns/target.rs
pub struct RepairVerb;

impl VerbCommand for RepairVerb {
    fn run() -> Result<()> {
        let state = EngineState::from_workspace();
        // ... implement repair logic reading from state ...
        Ok(())
    }
}
```

**Step 4 — Register the verb in the noun**

```rust
// In TargetNoun::new() or the noun builder
self.add_verb(RepairVerb)
```

**Step 5 — Emit evidence (mandatory)**

Every verb that does work must emit a `ProcessEvent`. Use the builder pattern from `src/evidence.rs`:

```rust
let case_id = read_or_create_session_id()?;

let start_event = ProcessEvent::started()
    .case_id(case_id.clone())
    .activity_name("target_repair")
    .timestamp(Utc::now())
    .build();

// ... perform repair work ...

let end_event = ProcessEvent::completed()
    .case_id(case_id.clone())
    .activity_name("target_repair")
    .timestamp(Utc::now())
    .build();

engine_state.append_events(vec![start_event, end_event])?;
```

Read-only verbs (no side effects) may skip evidence. All mutations and decisions must emit evidence.

**Step 6 — Write tests**

At minimum, one smoke test per verb:

```rust
// tests/cli/test_target.rs
#[test]
fn test_target_repair_dry_run() {
    let dir = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["target", "repair", "--dry-run"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dry") || stdout.contains("WARN:dry_run"));
}

#[test]
fn test_target_repair_requires_confirm() {
    let dir = TempDir::new().unwrap();
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["target", "repair"])  // no --confirm
        .output()
        .unwrap();

    // Should refuse without --confirm for destructive operations
    assert!(!output.status.success());
}
```

**Step 7 — Verify invariants still pass**

```bash
cargo test --test invariants
```

The `invariant_public_boundary_no_forbidden_terms_in_all_help` test will scan your new verb's `--help` output. Make sure no forbidden term appears in the description you wrote.

---

### 4.2 Fix a Failing Test

**Step 1 — Identify the failing test**

Run the specific test suite in isolation:

```bash
cargo test --test invariants -- --nocapture
# or:
cargo test --test cli -- --nocapture
```

The output will name the specific failing test function.

**Step 2 — Run just that test function**

```bash
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help -- --nocapture
```

**Step 3 — Understand what the test is asserting**

Read the test file. For invariants, see `tests/invariants.rs`. For CLI tests, see `tests/cli/test_<noun>.rs`. Each test documents what it enforces.

**Step 4 — Trace the failure through the architecture**

For most failures, the path is:
- CLI test fails → the verb output is wrong → check `src/nouns/<noun>.rs`
- Invariant fails for forbidden term → search `rg "<term>" src/` → fix help text
- Adapter-related failure → check the relevant adapter in `src/adapters/`
- State-related failure → check `src/engine/<dimension>_state.rs`

**Step 5 — Enable debug logging for adapter failures**

If an adapter is silently failing:

```bash
RUST_LOG=debug cargo run -- status show 2>&1 | head -40
```

Adapters log their errors at `debug` level rather than panicking. This output will reveal what is failing silently.

**Step 6 — Add a regression test**

After fixing the bug, add a test that would have caught it:

```rust
#[test]
fn test_status_show_detects_dirty_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), b"dirty").unwrap();

    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["status", "show"])
        .output()
        .unwrap();

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("dirty") || text.contains("WARN"));
}
```

**Step 7 — Run the full suite to confirm no regressions**

```bash
cargo make test
```

---

### 4.3 Run the Evidence Gate Locally

The evidence gate verifies that cargo-cicd emits well-formed process evidence and that the wasm4pm oracle adjudicates it correctly.

**Without wpm (offline mode)**

```bash
cargo test --test wasm4pm_evidence_gate
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

If wpm is not on PATH, these tests will complete with `ExpectedWpmVerdict::Blocked`. This is expected for offline development. The tests will not fail — `Blocked` is a first-class expectation.

**With wpm (full evidence gate)**

First verify wpm is available:

```bash
which wpm
wpm --version
```

Then run:

```bash
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

Evidence files are emitted to `target/cargo-cicd/evidence/`. You can inspect them:

```bash
ls -la target/cargo-cicd/evidence/
# Shows *.xes and *.jsonl files

cat target/cargo-cicd/evidence/evt-*.xes
# Shows the XES XML event stream
```

To manually audit an XES file:

```bash
wpm audit target/cargo-cicd/evidence/evt-*.xes
# Output: Accept / Refuse / Blocked
```

**Diagnosing oracle refusals**

If the oracle returns `Refuse` on a happy-path test:

1. Inspect the XES file for malformed or missing fields
2. Compare against the expected format in `src/evidence.rs`
3. Check that `case_id` is present — missing case_id creates an evidence gap
4. Check that both `start` and `complete` lifecycle events are present in the same trace
5. Verify the `verdict_claimed` value is `PASS`, `WARN`, or `FAIL` (not empty)

---

### 4.4 Debug a Failing Adapter

Adapters silently return defaults on failure. This means a broken adapter looks like missing data, not an error. To diagnose:

**Step 1 — Identify which state dimension is wrong**

Run `status show` and compare against what you expect:

```bash
cargo run -- status show
```

For example, if the branch name is blank, `GitStatusAdapter` is failing. If the workspace name is wrong, `CargoMetadataAdapter` is the culprit.

**Step 2 — Enable debug logging**

```bash
RUST_LOG=debug cargo run -- status show 2>&1 | grep -i "adapter\|error\|failed"
```

**Step 3 — Test the underlying command manually**

Each adapter wraps an external command. Run the underlying command directly:

```bash
# GitStatusAdapter
git status --porcelain
git rev-parse --abbrev-ref HEAD
git rev-list --count HEAD ^origin/main

# ToolchainDetector
rustc --version
rustup show active-toolchain

# ChangedFileDetector
git diff origin/main --name-only

# CargoMetadataAdapter
head -20 Cargo.toml
```

If the underlying command fails (wrong directory, no git repo, etc.), the adapter will silently return defaults.

**Step 4 — Check the adapter source**

Each adapter is in `src/adapters/`. Find the method that populates the failing dimension and trace through it:

```
src/adapters/git_status.rs           -> GitPhaseState
src/adapters/cargo_metadata.rs       -> WorkspaceState
src/adapters/toolchain_detector.rs   -> ToolchainState
src/adapters/target_scanner.rs       -> TargetState
src/adapters/changed_file_detector.rs-> ChangedFileState
src/adapters/trybuild_detector.rs    -> TrybuildState
src/adapters/cicd_toml_writer.rs     -> cicd.toml persistence
```

**Step 5 — Confirm the adapter's silent-failure contract**

After fixing the adapter, verify it still returns a default rather than panicking when the underlying command is unavailable:

```rust
// Good: silent failure
pub fn branch() -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()  // empty string, not panic
}
```

---

### 4.5 Add a New Autonomic Policy

Policies run in `suggest` mode only. They read `EngineState` and emit recommendations. They never take action.

**Step 1 — Create the policy module**

```rust
// src/policies/cargo_lock_age.rs
use crate::engine::EngineState;
use crate::engine::policy_state::{PolicyEntry, PolicyVerdict};
use crate::evidence::now_iso8601;

const MAX_AGE_DAYS: u64 = 30;

#[cfg(feature = "autonomic")]
pub fn eval(state: &EngineState) -> PolicyEntry {
    let lock_path = format!("{}/Cargo.lock", state.workspace.root_path);
    let age_days = std::fs::metadata(&lock_path)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().unwrap_or_default().as_secs() / 86400)
        .unwrap_or(0);

    let (verdict, recommendation) = if age_days > MAX_AGE_DAYS {
        (
            PolicyVerdict::Warn,
            format!(
                "Cargo.lock is {} days old. Run `cargo update` to refresh.",
                age_days
            ),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyEntry {
        policy_name: "cargo_lock_age".to_string(),
        verdict,
        recommendation,
        emitted_at: now_iso8601(),
    }
}
```

**Step 2 — Register in the policy runner**

```rust
// src/autonomic/policies.rs
#[cfg(feature = "autonomic")]
pub fn run_all_policies(state: &EngineState) -> Vec<PolicyEntry> {
    vec![
        crate::policies::git_phase_dirty::eval(state),
        crate::policies::target_pressure::eval(state),
        crate::policies::cargo_lock_age::eval(state),  // add here
        // ... other policies ...
    ]
}
```

**Step 3 — Declare in the module registry**

```rust
// src/policies/mod.rs
pub mod cargo_lock_age;
```

**Step 4 — Write a test**

```rust
// tests/autonomic_policies.rs
#[test]
#[cfg(feature = "autonomic")]
fn test_cargo_lock_age_policy_detects_stale_lock() {
    let state = EngineState { /* setup with old Cargo.lock */ };
    let policies = run_all_policies(&state);
    let entry = policies.iter()
        .find(|p| p.policy_name == "cargo_lock_age")
        .expect("policy not found");
    assert_eq!(entry.verdict, PolicyVerdict::Warn);
    assert!(entry.recommendation.contains("cargo update"));
}
```

**Step 5 — Verify the feature gate compiles**

```bash
cargo build --features autonomic
cargo test --features autonomic --test autonomic_policies
```

---

### 4.6 Write an Integration Test

Integration tests use `assert_cmd` and `tempfile` to run the binary in an isolated temporary workspace.

**Pattern for a fixture-based integration test:**

```rust
// tests/cli/test_publish.rs
use tempfile::TempDir;
use assert_cmd::Command;

fn write_minimal_cargo_toml(dir: &std::path::Path, with_license: bool) {
    let license_line = if with_license { r#"license = "MIT""# } else { "" };
    let content = format!(
        r#"
[package]
name = "test_crate"
version = "0.1.0"
description = "Test crate"
{}
"#,
        license_line
    );
    std::fs::write(dir.join("Cargo.toml"), content).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "").unwrap();
}

#[test]
fn test_publish_run_with_complete_metadata() {
    let dir = TempDir::new().unwrap();
    write_minimal_cargo_toml(dir.path(), true);

    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["publish", "run"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS") || stdout.contains("ready"));
}

#[test]
fn test_publish_run_missing_license_warns() {
    let dir = TempDir::new().unwrap();
    write_minimal_cargo_toml(dir.path(), false);  // no license

    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["publish", "run"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WARN") || stdout.contains("license"));
}
```

Run just this test:

```bash
cargo test --test cli test_publish_run_with_complete_metadata
```

**Critical: assert on oracle verdict, not internal state**

When writing evidence gate tests, the single most important rule is:

```rust
// WRONG — do not assert on internal cargo-cicd state
assert_eq!(state.target.total_size_bytes, expected_size);

// CORRECT — assert on the wasm4pm verdict
assert_eq!(wpm_verdict, WpmVerdict::Accept);
```

This is invariant E4 and it is enforced in code review.

---

## 5. Claude Code Slash Commands

The `.claude/commands/` directory contains slash commands you can invoke with Claude Code. Each command is a markdown file with a precise description of what to do.

### Available Commands

**`/build`** — Build the binary and verify it was produced

Checks for `cargo-make`, runs `cargo make build` (or `cargo build` as fallback), detects warnings and errors, verifies the binary exists at `target/debug/cargo-cicd`, and runs `--version` to confirm the binary links correctly.

Source: `.claude/commands/build.md`

---

**`/test`** — Run cargo-cicd test suites

Understands the stratified test hierarchy. Runs Tier 1 (unit/smoke) tests first, then Tier 2 (evidence gate) tests. Reports which suites passed or failed. Knows about offline behaviour (`ExpectedWpmVerdict::Blocked`) and explains when wpm is required.

Key detail: always runs `invariants` first. If invariants fail, the other suites are not informative.

Source: `.claude/commands/test.md`

---

**`/git`** — Git phase management

Guides through checking `cargo cicd git status`, understanding the `GitPhaseState` dimensions (dirty, staged, untracked, ahead, behind), and safely running `git close`. Includes troubleshooting for merge conflicts, diverged branches, and detached HEAD.

Source: `.claude/commands/git.md`

---

**`/release`** — Full release workflow for v26.6.2

Walks through all 12 release steps in order:
1. Pre-flight git clean check
2. Full test suite (`cargo make test`)
3. Invariants (no forbidden terms)
4. Feature flag compilation check
5. Evidence gate (wasm4pm)
6. wpm receipt validation
7. README currency check (ggen)
8. CHANGELOG update
9. Version check in `Cargo.toml` and `src/main.rs`
10. Final release commit
11. Annotated tag creation
12. Push to origin with tags

Do not skip steps. Each one is a gate.

Source: `.claude/commands/release.md`

---

### Operations Without Dedicated Command Files

These operations do not have command files yet. Run them directly:

**Check** (lint + type-check):
```bash
cargo make check
```

**Evidence** (run evidence gate):
```bash
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases
```

**Status** (workspace health snapshot):
```bash
cargo cicd status show
# or just:
cargo cicd status
```

**Workspace** (full workspace diagnostics with policies):
```bash
cargo cicd workspace doctor
# or just:
cargo cicd workspace
```

---

## 6. Key Files to Know

### Entry Points

| File | Role |
|------|------|
| `src/main.rs` | Binary entry point. Contains `inject_default_verbs()` which maps bare nouns to their default verbs. Dispatches to noun modules. |
| `src/lib.rs` | Public API surface. Re-exports types used by external callers and sub-crates. |
| `Cargo.toml` | Root workspace manifest. Defines feature flags, workspace members, and `[[bin]]` target. |
| `Makefile.toml` | cargo-make task definitions. `cargo make build`, `cargo make check`, `cargo make test` are defined here. |

### CLI Grammar (Noun Modules)

All noun implementations live in `src/nouns/`. Each file = one noun.

| File | Noun | Key verbs |
|------|------|-----------|
| `src/nouns/status.rs` | `status` | `show` (default) |
| `src/nouns/git.rs` | `git` | `status`, `close`, `phase` |
| `src/nouns/test.rs` | `test` | `changed` |
| `src/nouns/trybuild.rs` | `trybuild` | `changed`, `full` |
| `src/nouns/target.rs` | `target` | `show`, `prune` |
| `src/nouns/workspace.rs` | `workspace` | `doctor` (default) |
| `src/nouns/publish.rs` | `publish` | `run` (default) |
| `src/nouns/evidence.rs` | `evidence` | `doctor` (default), `audit` |
| `src/nouns/pipeline.rs` | `pipeline` | `run` |
| `src/nouns/lsp.rs` | `lsp` | `explain` |
| `src/nouns/mod.rs` | — | Noun registry; registers all nouns with clap |

### Engine State

| File | What it holds |
|------|--------------|
| `src/engine/mod.rs` | `EngineState` struct definition and `from_workspace()` constructor |
| `src/engine/workspace_state.rs` | Workspace name, root path, members, toolchain, Rust edition |
| `src/engine/toolchain_state.rs` | Active toolchain, rustc version |
| `src/engine/target_state.rs` | Target directory path and total size in bytes |
| `src/engine/changed_file_state.rs` | Base ref, changed `.rs` files, test files, trybuild fixtures |
| `src/engine/git_phase_state.rs` | Branch name, dirty/staged/untracked files, ahead/behind counts |
| `src/engine/process_event_state.rs` | List of emitted `ProcessEvent` structs for the current session |
| `src/engine/policy_state.rs` | `PolicyEntry` structs from autonomic policy evaluation |
| `src/engine/trybuild_state.rs` | Fixture sets, changed fixtures |
| `src/engine/test_plan_state.rs` | Estimated test count, conservative mode flag |
| `src/engine/projection_profile.rs` | Feature flag surface contract |

### Adapters

| File | External source | Populates |
|------|----------------|-----------|
| `src/adapters/cargo_metadata.rs` | `Cargo.toml` (line-by-line scan) | `WorkspaceState` |
| `src/adapters/manifest_parser.rs` | `Cargo.toml` (TOML parsing) | Package names, metadata |
| `src/adapters/git_status.rs` | `git status --porcelain` | `GitPhaseState` |
| `src/adapters/toolchain_detector.rs` | `rustc --version` | `ToolchainState` |
| `src/adapters/target_scanner.rs` | Recursive `walkdir` over `target/` | `TargetState` |
| `src/adapters/changed_file_detector.rs` | `git diff origin/main --name-only` | `ChangedFileState` |
| `src/adapters/trybuild_detector.rs` | `tests/ui/` filesystem scan | `TrybuildState` |
| `src/adapters/cicd_toml_writer.rs` | — (writes only) | Serializes `EngineState` to `cicd.toml` |

### Evidence and Oracle

| File | Role |
|------|------|
| `src/evidence.rs` | `ProcessEvent` struct, XES serialization, `emit_xes_event()`, `append_events()`, invariants E1–E7 |
| `src/integrations/wasm4pm_shell.rs` | Shell invocation of `wpm audit` and `wpm receipt doctor` |
| `src/integrations/wasm4pm_current.rs` | Current oracle state and XES serialization format |
| `src/session.rs` | Session ID generation via `read_or_create_session_id()` |

### Policies

| File | Policy | Trigger |
|------|--------|---------|
| `src/policies/git_phase_dirty.rs` | `git_phase_dirty` | Dirty or staged files present |
| `src/policies/target_pressure.rs` | `target_pressure` | Target dir exceeds size threshold |
| `src/policies/toolchain_mismatch.rs` | `toolchain_mismatch` | rustc version differs from lockfile expectation |
| `src/policies/trybuild_changed.rs` | `trybuild_changed` | trybuild fixtures changed since base |
| `src/policies/branch_behind.rs` | `branch_behind` | Local branch behind `origin/main` by N commits |
| `src/policies/evidence_stale.rs` | `evidence_stale` | Last evidence emission older than threshold |
| `src/policies/publish_not_adjudicated.rs` | `publish_not_adjudicated` | Publish happened but no wasm4pm verdict exists |
| `src/policies/mod.rs` | — | Policy registry |

### Tests

| File | What it validates |
|------|------------------|
| `tests/invariants.rs` | 7 non-negotiable public boundary invariants |
| `tests/cli/` | Noun/verb CLI parsing, dispatch, and output |
| `tests/feature_projection.rs` | Feature flag surface contract |
| `tests/cicd_toml_truth.rs` | `cicd.toml` serialization/deserialization round-trip |
| `tests/autonomic_policies.rs` | Policy evaluation logic |
| `tests/changed_tests.rs` | Changed-file classification accuracy |
| `tests/git_phase_closure.rs` | Git state detection |
| `tests/wasm4pm_evidence_gate.rs` | Happy-path evidence to wasm4pm `Accept` |
| `tests/wasm4pm_evidence_mutation.rs` | Corrupt evidence to wasm4pm `Refuse` |
| `tests/wasm4pm_refusal_cases.rs` | Edge cases: oracle unavailable, malformed XES |
| `tests/wasm4pm_harness.rs` | Shared test harness for evidence gate tests |
| `tests/fixtures/` | Fixture workspaces used by integration tests |

### Manufacturing / Ontology

| File | Role |
|------|------|
| `ontology/cargo-cicd-capabilities.ttl` | RDF/Turtle capability definitions — ground truth for noun/verb grammar |
| `ggen.toml` | Code generation config: SPARQL rules, Tera template paths, output destinations |
| `queries/*.sparql` | SPARQL inference rules for capability projection |
| `templates/README.md.tera` | Tera template for the generated README |
| `templates/docs/reference-command.md.tera` | Tera template for per-command reference docs |

### Generated/Derived Files

These files are generated by `ggen` and must not be edited by hand:

| File | Generated from |
|------|---------------|
| `README.md` (command reference sections) | `templates/README.md.tera` + ontology |
| `docs/reference/commands/*.md` | `templates/docs/reference-command.md.tera` + ontology |

### Workspace Artifacts

| File | Role |
|------|------|
| `cicd.toml` | Persistent state carrier. Written by `CicdTomlWriter`. Not committed (in `.gitignore`). |
| `target/cargo-cicd/evidence/` | XES and JSONL evidence files emitted by verbs |
| `receipts/` | wasm4pm receipt artifacts from past adjudications |
| `CHANGELOG.md` | Human-maintained changelog; must be updated before each release |

---

## 7. Forbidden Terms (NEVER Use in Help Text)

The following 10 terms are banned from all public output — CLI help text, status messages, error messages, and any string that could reach the user. They are internal implementation details that must not be disclosed.

The invariant `invariant_public_boundary_no_forbidden_terms_in_all_help` in `tests/invariants.rs` scans **every** noun and verb `--help` output. A single occurrence blocks the release.

| Term | Reason |
|------|--------|
| `ALIVE` | Level 5 engine status marker; internal only |
| `Inspection Gate` | Manufacturing subsystem identity |
| `wall` | Jargon from the manufacturing pipeline |
| `Nehemiah` | Code name for the manufacturing layer; expose only as `ggen` |
| `Field8` | Internal capacity measurement; not user-facing |
| `Instinct8` | Autonomic reasoning subsystem; not exposed in suggest mode |
| `Cargo Court` | Internal adjudication metaphor |
| `AGI` | AI system classification; not disclosed in CLI output |
| `Truex` | Internal truth engine; only XES/evidence models are exposed |
| `CONSTRUCT8` | Manufacturing directive system |

**How to diagnose a forbidden term leak:**

1. Find which noun/verb produced the leak:
   ```bash
   for noun in status git test trybuild target workspace publish evidence pipeline lsp; do
     cargo run -- $noun --help 2>&1 | grep -E "ALIVE|Inspection Gate|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8"
   done
   ```

2. Search the source:
   ```bash
   rg "ALIVE" src/
   rg "Nehemiah" src/
   # etc.
   ```

3. Replace the forbidden term with its approved public alternative:
   ```rust
   // WRONG
   println!("ALIVE status: {}", state.engine_alive);

   // CORRECT
   println!("Process state: {}", state.is_complete);
   ```

4. Re-run the invariant:
   ```bash
   cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help
   ```

---

## 8. Commit Format

Every commit to this repository must follow this format:

```
<type>(<scope>): <description>
```

**Types:**
- `feat` — new feature or capability
- `fix` — bug fix
- `test` — adding or updating tests
- `docs` — documentation only
- `chore` — maintenance (release commits, dependency updates)
- `refactor` — code restructuring without behavior change

**Scopes:**

| Scope | Use when |
|-------|----------|
| `core` | Changes to `src/engine/`, `src/evidence.rs`, `src/session.rs` |
| `cli` | Changes to `src/nouns/`, `src/main.rs` |
| `target` | Changes to target-related adapters or noun |
| `test` | Changes to test files |
| `git` | Changes to git adapter or git noun |
| `autonomic` | Changes to `src/autonomic/` or `src/policies/` |
| `docs` | Changes to `CLAUDE.md`, `.claude/`, `docs/` |
| `receipts` | Changes to `receipts/` or oracle integration |

**Examples:**

```
feat(core): add ProcessEvent serialization for XES traces
fix(cli): ensure status noun injects default show verb
docs(autonomic): clarify policy suggestion lifecycle
test(wasm4pm): add mutation tests for verdict handling
chore(release): v26.6.2 evidence gate pass
refactor(target): extract scanner logic into TargetScannerAdapter
```

The scope is mandatory. A commit without a scope will be flagged in review.

---

## 9. Getting Help

### Primary References (in order)

1. **`CLAUDE.md`** — Single source of truth for architecture, patterns, commands, and invariants. If something contradicts `CLAUDE.md`, `CLAUDE.md` wins.

2. **`.claude/ARCHITECTURE.md`** — Visual diagrams of data flow, EngineState composition, adapter composition, evidence lifecycle, and the noun-verb registry. Start here for architecture questions.

3. **`.claude/PATTERNS.md`** — Code pattern reference. Covers the noun-verb pattern, evidence emission pattern, adapter pattern, EngineState aggregate root, policy evaluation, LSP analyzer pattern, feature flag guards, error handling, and testing patterns. Start here for "how do I implement X" questions.

4. **`.claude/QUICKSTART.md`** — This document. Covers onboarding steps, common tasks, and day-to-day commands.

### When the Docs Don't Cover It

If you have a question not answered by the four documents above:

1. **Search the source** — Most design decisions are visible in the existing code. Use `rg` to search for similar patterns.

2. **Run the tests with `--nocapture`** — Test output often reveals design intent:
   ```bash
   cargo test --test invariants -- --nocapture
   ```

3. **Check the ontology** — The capability definitions in `ontology/cargo-cicd-capabilities.ttl` are the ground truth for what the CLI is supposed to do.

4. **Check git history** — `git log --oneline src/nouns/<noun>.rs` shows the rationale behind past decisions.

### Common Mistakes to Avoid

**Do not write noun/verb modules by hand without running `ggen` first.**
The ontology drives code generation. Adding a verb in `src/nouns/` without updating the ontology means the verb will not appear in README, reference docs, or test scaffolding. It will also be overwritten the next time `ggen` runs.

**Do not assert on internal cargo-cicd state in evidence tests.**
Evidence gate tests must assert on the wasm4pm verdict. Asserting on `state.target.size` or `state.git_phase.dirty_files` is an architectural violation.

**Do not swallow adapter errors with `.ok()`.**
Adapters should return `anyhow::Result` and let callers handle partial state. Using `.ok()` loses context about what failed.

**Do not skip the forbidden-terms invariant.**
Even a single forbidden term in help text is a hard release block. Check `tests/invariants.rs` after any change to noun descriptions or help strings.

**Do not commit to main directly.**
All changes go through pull requests. The release branch is protected.

**Do not use `--no-verify` to skip hooks.**
Pre-commit hooks enforce formatting and lint. If a hook fails, fix the underlying issue.

---

## Appendix: Quick Reference Commands

```bash
# Build
cargo make build                          # preferred
cargo build                               # fallback

# Check (lint + type-check)
cargo make check

# Run all tests
cargo make test

# Run invariants only (fastest gate)
cargo test --test invariants

# Run a specific test function
cargo test --test invariants invariant_public_boundary_no_forbidden_terms_in_all_help

# Run with feature flags
cargo test --features autonomic
cargo test --features wasm4pm
cargo test --features autonomic,wasm4pm

# Run evidence gate
cargo test --test wasm4pm_evidence_gate -- --nocapture
cargo test --test wasm4pm_evidence_mutation
cargo test --test wasm4pm_refusal_cases

# Run the binary
cargo run -- status show
cargo run -- workspace doctor
cargo run -- git status
cargo run -- evidence doctor

# Debug adapter failures
RUST_LOG=debug cargo run -- status show

# Manually audit evidence
wpm audit target/cargo-cicd/evidence/evt-*.xes
wpm receipt doctor --format json --strict receipts/*.json

# Regenerate from ontology
ggen

# Feature flag compilation check
cargo build --features autonomic,wasm4pm,contrib
```

---

**Last Updated:** 2026-06-16

**Version:** cargo-cicd 26.6.2
