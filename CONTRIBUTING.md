# Contributing to cargo-cicd

cargo-cicd keeps Rust workspaces clean, fast, and push-ready. This guide covers
everything you need to contribute correctly: code conventions, the evidence
emission pattern, how to add verbs and policies, forbidden terms, test
requirements, and the pull request process.

---

## 1. Code Conventions

### Commit Message Format

```
feat(core|cli|target|test|git|autonomic|docs|receipts): description
```

Pick the scope that best describes where the change lands:

| Scope | Covers |
|---|---|
| `core` | EngineState, adapters, evidence emission internals |
| `cli` | Noun/verb handlers, help text, argument parsing |
| `target` | Target directory analysis and cleanup |
| `test` | Test infrastructure, fixtures, harnesses |
| `git` | Git phase tracking, branch detection, closure |
| `autonomic` | Policy engine, policy modules, suggest mode |
| `docs` | CLAUDE.md, CONTRIBUTING.md, reference docs, CHANGELOG |
| `receipts` | wpm receipt artifacts, receipt doctor integration |

Examples:

```
feat(cli): add target repair verb with dry-run safety gate
fix(core): changed_file_detector now handles renamed files
test(git): add regression test for ahead/behind detection
docs(autonomic): document suggest-only policy contract
```

### Rust Style

- **No clippy warnings.** Run `cargo clippy` before every commit. Fix all
  warnings; do not suppress with `#[allow(...)]` unless you include a comment
  explaining why suppression is correct for this specific site.
- **No dead code.** Remove unused functions, fields, and imports. If something
  must exist for a future invariant, gate it with a feature flag and add a
  `// TODO(<issue>):` note.
- **Comments only when the WHY is non-obvious.** Do not restate what the code
  does. Explain an invariant, a workaround for an upstream bug, or a constraint
  imposed by an external system. One sentence is usually enough.
- **No panics in adapters.** Adapters must return defaults on failure. Use
  `unwrap_or_default()`, `unwrap_or_else(|_| …)`, or early-return `None`.
  Only nouns and the CLI entry point may terminate the process.
- **Adapters are stateless.** All adapter methods are `fn(…)` or `&self` with
  no mutable state. Adapters never call other adapters.

---

## 2. Evidence Emission Pattern (Critical)

Every verb implementation MUST emit process evidence. This is non-negotiable:
without evidence, wasm4pm cannot adjudicate, and the release gate fails.

### The Pattern

```rust
use crate::evidence::ProcessEvent;

pub fn run(/* … */) -> anyhow::Result<()> {
    let case_id = "noun_verb_phase".to_string(); // snake_case, stable identifier
    let evidence_dir = crate::evidence::default_evidence_dir();

    // 1. Emit start event BEFORE doing any work
    let (mut start_evt, t0) = ProcessEvent::started("noun verb");
    start_evt.case_id = Some(case_id.clone());
    crate::evidence::append_events(&[start_evt], &evidence_dir);

    // 2. Do the actual work
    let outcome = do_work()?;

    // 3. Determine verdict
    let verdict_str = match outcome {
        Outcome::Clean  => "PASS",
        Outcome::Issues => "WARN",
        Outcome::Fatal  => "FAIL",
    };

    // 4. Emit complete event AFTER work finishes
    let mut complete_evt = ProcessEvent::completed("noun verb", t0, verdict_str);
    complete_evt.case_id = Some(case_id);
    crate::evidence::append_events(&[complete_evt], &evidence_dir);

    Ok(())
}
```

### Rules

| Rule | Requirement |
|---|---|
| **E1** | cargo-cicd never adjudicates itself — only wasm4pm issues verdicts |
| **E2** | XES file must exist on disk before `audit_xes()` is called |
| **E3** | If oracle unavailable and expected verdict is not `Blocked`, panic |
| **E4** | Tests assert only the wasm4pm verdict, never internal cargo-cicd state |
| **E5** | XES groups events by `case_id` into `<trace>` elements |
| **E6** | JSONL emission mirrors XES (same event set, machine-readable) |
| **E7** | `Blocked` is a first-class expectation, not an error |

### Verdict Values

- `"PASS"` — Work completed; all conditions satisfied
- `"WARN"` — Work completed with warnings; execution continues
- `"FAIL"` — Blocking error; work halted
- `"WARN:dry_run"` — Planning only; no mutation occurred
- `"WARN:oracle_unavailable"` — wpm binary not found

---

## 3. Adding a New Verb

Use this checklist whenever you add a verb to an existing noun.

### Step 1 — Define the struct

```rust
// In src/nouns/<noun>.rs
pub struct RepairVerb;
```

### Step 2 — Implement VerbCommand

```rust
impl VerbCommand for RepairVerb {
    fn name(&self) -> &'static str { "repair" }

    fn about(&self) -> &'static str {
        "Repair target directory issues (e.g., stale locks)"
        // Keep help text under 80 characters.
        // NEVER use any forbidden term (see Section 5).
    }

    fn run(&self, matches: &clap::ArgMatches) -> anyhow::Result<()> {
        let case_id = "target_repair_phase".to_string();
        let evidence_dir = crate::evidence::default_evidence_dir();

        let (mut start_evt, t0) = ProcessEvent::started("target repair");
        start_evt.case_id = Some(case_id.clone());
        crate::evidence::append_events(&[start_evt], &evidence_dir);

        // --- do the actual work here ---
        let verdict_str = "PASS";

        let mut complete_evt = ProcessEvent::completed("target repair", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        crate::evidence::append_events(&[complete_evt], &evidence_dir);

        Ok(())
    }
}
```

### Step 3 — Register the verb in the noun

```rust
// In the noun's verbs() or build() method
fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
    vec![
        Box::new(ShowVerb),
        Box::new(PruneVerb),
        Box::new(RepairVerb), // ← add here
    ]
}
```

### Step 4 — Write at least one CLI test

```rust
// In tests/cli/test_<noun>.rs
#[test]
fn test_target_repair_dry_run_exits_zero() {
    let dir = tempfile::TempDir::new().unwrap();
    init_git_repo(dir.path()); // helper from tests/fixture_workspaces.rs
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .current_dir(dir.path())
        .args(["target", "repair", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
}
```

### Step 5 — Verify no forbidden terms

```bash
cargo run -- target repair --help | grep -iE 'ALIVE|Inspection Gate|wall|Nehemiah|Field8|Instinct8|Cargo Court|AGI|Truex|CONSTRUCT8'
# Must produce no output
```

---

## 4. Adding a New Policy

Policies run in **suggest mode only** — they read state and emit recommendations.
They never mutate files, run commands, or take destructive action.

### Step 1 — Create the policy module

```rust
// src/policies/cargo_lock_age.rs

#[cfg(feature = "autonomic")]
use crate::engine::EngineState;
#[cfg(feature = "autonomic")]
use crate::engine::policy_state::{PolicyEntry, PolicyVerdict};

const MAX_AGE_DAYS: u64 = 30;

#[cfg(feature = "autonomic")]
pub fn eval(state: &EngineState) -> PolicyEntry {
    let lock_path = format!("{}/Cargo.lock", state.workspace.root_path);
    let age_days = lock_age_days(&lock_path).unwrap_or(0);

    let (verdict, recommendation) = if age_days > MAX_AGE_DAYS {
        (
            PolicyVerdict::Warn,
            "Run `cargo update` to refresh the lockfile".to_string(),
        )
    } else {
        (PolicyVerdict::Pass, String::new())
    };

    PolicyEntry {
        policy_name: "cargo_lock_age".to_string(),
        verdict,
        recommendation,
        emitted_at: crate::evidence::now_iso8601(),
    }
}

fn lock_age_days(path: &str) -> Option<u64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let elapsed = modified.elapsed().ok()?;
    Some(elapsed.as_secs() / 86_400)
}
```

### Step 2 — Register in run_all_policies

```rust
// src/autonomic/policies.rs
#[cfg(feature = "autonomic")]
pub fn run_all_policies(state: &EngineState) -> Vec<PolicyEntry> {
    vec![
        crate::policies::target_pressure::eval(state),
        crate::policies::toolchain_mismatch::eval(state),
        crate::policies::trybuild_changed::eval(state),
        crate::policies::branch_behind::eval(state),
        crate::policies::evidence_stale::eval(state),
        crate::policies::publish_not_adjudicated::eval(state),
        crate::policies::git_phase_dirty::eval(state),
        crate::policies::cargo_lock_age::eval(state), // ← add here
    ]
}
```

### Step 3 — Write tests in autonomic_policies.rs

```rust
// tests/autonomic_policies.rs
#[cfg(feature = "autonomic")]
#[test]
fn test_cargo_lock_age_policy_warns_on_stale_lock() {
    // Construct a state with a stale lock path
    let state = EngineState {
        workspace: WorkspaceState {
            root_path: "/path/to/old/workspace".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let entries = run_all_policies(&state);
    let entry = entries.iter().find(|e| e.policy_name == "cargo_lock_age").unwrap();
    assert_eq!(entry.verdict, PolicyVerdict::Warn);
    assert!(entry.recommendation.contains("cargo update"));
}

#[cfg(feature = "autonomic")]
#[test]
fn test_cargo_lock_age_policy_passes_on_fresh_lock() {
    // Construct a state pointing at the current workspace (lock is fresh)
    let state = EngineState::default();
    let entries = run_all_policies(&state);
    let entry = entries.iter().find(|e| e.policy_name == "cargo_lock_age").unwrap();
    assert_eq!(entry.verdict, PolicyVerdict::Pass);
}
```

### Step 4 — Run with the feature enabled

```bash
cargo test --features autonomic --test autonomic_policies
```

---

## 5. Forbidden Terms

The following terms MUST NEVER appear in any public output: help text, status
output, error messages, or log lines that a user can see. The invariant test
`invariant_public_boundary_no_forbidden_terms_in_all_help()` scans all
`--help` output and will fail the build if any of these are found.

| Forbidden Term | Why |
|---|---|
| `ALIVE` | Level 5 engine status marker; internal only |
| `Inspection Gate` | Manufacturing subsystem identity |
| `wall` | Jargon from manufacturing pipeline |
| `Nehemiah` | Code name for manufacturing layer (expose only as `ggen`) |
| `Field8` | Internal capacity measurement |
| `Instinct8` | Autonomic reasoning subsystem; not exposed in suggest mode |
| `Cargo Court` | Internal adjudication metaphor |
| `AGI` | AI system classification |
| `Truex` | Internal truth engine |
| `CONSTRUCT8` | Manufacturing directive system |

If you are unsure whether a term is safe, run:

```bash
cargo run -- <noun> <verb> --help | grep -iF '<term>'
```

If the grep produces any output, the term must not appear there.

---

## 6. Test Requirements

### What Every New Verb Needs

- At least one test in `tests/cli/test_<noun>.rs` that invokes the verb and
  asserts `output.status.success()`.
- A test for each significant output path (PASS, WARN, FAIL) if the verb can
  produce more than one verdict.
- If the verb emits evidence, a test that confirms the XES file is created in
  `target/cargo-cicd/evidence/`.

### Evidence Tests — Assert the Oracle, Not Internal State

```rust
// WRONG — asserts on internal cargo-cicd state
assert_eq!(state.target.total_size_bytes, expected_bytes);

// CORRECT — asserts on wasm4pm verdict
assert_eq!(wpm_verdict, ExpectedWpmVerdict::Accept);
```

Evidence tests live in `tests/wasm4pm_evidence_gate.rs`,
`tests/wasm4pm_evidence_mutation.rs`, and `tests/wasm4pm_refusal_cases.rs`.
Tests in those files must use `ExpectedWpmVerdict::Blocked` when running in
environments without the `wpm` binary.

### The Seven Invariants (tests/invariants.rs)

These must always pass. Never introduce code that breaks them:

1. No forbidden terms in any `--help` output
2. No destructive action without `--confirm`
3. No full trybuild run by default (conservative mode)
4. Noun names are lowercase ASCII
5. Binary name is `cargo-cicd`
6. `cargo cicd status` exits 0 (baseline health check)
7. `git close` emits safety warnings (no silent close)

Run them with:

```bash
cargo test --test invariants
```

### Before Opening a PR

```bash
cargo make test            # all test suites
cargo clippy               # no warnings
cargo build --features autonomic,wasm4pm  # feature flags compile
```

---

## 7. Pull Request Process

### Title Format

PR titles follow the same format as commit messages:

```
feat(cli): add target repair verb with dry-run safety gate
fix(core): changed_file_detector handles renamed files correctly
```

### Checklist Before Opening

- [ ] `cargo make test` passes locally (all suites)
- [ ] `cargo clippy` produces no warnings
- [ ] No forbidden terms in help output (`cargo test --test invariants`)
- [ ] Evidence gate passes, or all evidence tests declare
      `ExpectedWpmVerdict::Blocked` with a comment explaining why wpm is
      unavailable in this context
- [ ] New verbs have CLI tests in `tests/cli/`
- [ ] New policies have tests in `tests/autonomic_policies.rs`
- [ ] Commit message uses the correct scope and format

### Review Criteria

Reviewers will check:

1. Evidence emission pattern is present and complete (start + complete, with
   `case_id` set on both events).
2. No forbidden terms anywhere in changed files.
3. Adapters remain stateless and silently fail.
4. EngineState remains the single aggregate root — nouns read from it, adapters
   populate it, nothing else writes to it directly.
5. Tests assert on behavior and oracle verdicts, not on internal structs.

### After Merge

The CI pipeline runs `cargo make test` including the evidence gate. If wpm is
available in CI, the gate runs fully. If not, tests must declare `Blocked`.
Releases require a full evidence gate pass with a live oracle.
