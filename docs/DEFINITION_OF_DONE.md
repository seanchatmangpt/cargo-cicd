---
artifact: DEFINITION_OF_DONE
date: 2026-06-14
version: 26.6.2
---

# Definition of Done — cargo-cicd

**Purpose:** This document defines what it means for a feature, bug fix, refactor, test, documentation, or release to be complete and shippable in cargo-cicd.

All items are categorized as **hard gates** (blockers — must be satisfied before merge/release) or **soft gates** (suggestions — strong best practices but may have documented exceptions).

---

## Feature Definition of Done

A feature is shippable when all the following are satisfied:

### Hard Gates

- [ ] **Code compiles cleanly**
  - Default build passes: `cargo build`
  - All feature combinations compile: `cargo build --all-features`, `cargo build --features process-data`, `cargo build --features autonomic`, `cargo build --features wasm4pm`
  - No clippy warnings on default: `cargo clippy -- -D warnings`
  - No clippy warnings on all features: `cargo clippy --all-features -- -D warnings`

- [ ] **No forbidden terms in public surfaces**
  - Zero matches of: ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8
  - Scan: `src/nouns/**/*`, `src/adapters/**/*`, `src/public.rs`, generated `cicd.toml` comments, help text, README examples
  - Test method: `cargo cicd <noun> <verb> --help` + grep public output

- [ ] **Test coverage: 80%+ of new code (default feature)**
  - Lines added that are covered by tests ≥ 80%
  - Coverage measured via: `cargo tarpaulin --lib --out Html --exclude-files tests/`
  - New test files or new test functions required for each new public noun verb or policy
  - Edge cases must have explicit test cases (not just implicit coverage)

- [ ] **Integration tested with existing adapters and nouns**
  - New code exercises at least one existing adapter: GitStatusAdapter, TargetScannerAdapter, ToolchainDetector, CargoMetadataAdapter, ChangedFileDetector, CicdTomlWriter, TrybuildDetector
  - New noun/verb runs end-to-end in a temp fixture workspace (`tests/fixtures/`)
  - No broken invariants: Run `cargo test --test invariants` — all 7 invariants (I1–I7) must pass

- [ ] **All GitHub CI/CD checks pass**
  - `cargo test` (all suites)
  - `cargo build --all-features`
  - `cargo clippy --all-features -- -D warnings`
  - `cargo fmt --check` (code style)
  - Linter checks (if present)
  - Any custom Actions workflows (e.g., security scanning, dependency audits)

- [ ] **Changelog entry added**
  - Entry in `CHANGELOG.md` or `docs/release/` documenting the feature
  - Format: `- [feature-name]: Brief description. Resolves #123.`
  - Grouped under current version or Unreleased section

- [ ] **Reviewed and approved by maintainer**
  - At least one maintainer review with LGTM or Approved status
  - All review comments resolved or explicitly waived with justification
  - No self-approval of feature PRs (at least one other reviewer required)

### Soft Gates

- [ ] **Performance baseline established**
  - For adapters: Runtime ≤ 100ms per invocation (default case)
  - For policy evaluations: ≤ 50ms per PolicyState analysis
  - For I/O operations: ≤ 200ms per workspace scan
  - Measured via: `cargo test --release -- --nocapture --test-threads=1` on your machine
  - Publish baseline in PR description or comment

- [ ] **Documentation: doc comments + examples**
  - All public functions, nouns, verbs have `/// doc comment`
  - Doc comments include: purpose, use case, example usage
  - Example is a minimal, compilable snippet (or reference to existing example)
  - Architecture notes added to CLAUDE.md if the feature changes module layout or exposes new adapter seams

- [ ] **Security and OWASP check**
  - New code does not parse untrusted input without validation (cicd.toml is trusted; workspace state is external)
  - No hardcoded credentials, API keys, or secrets
  - No unsafe blocks without documented rationale (rare in this codebase)
  - No spawning of shell subprocesses without explicit path validation

---

## Bug Fix Definition of Done

A bug fix is shippable when all the following are satisfied:

### Hard Gates

- [ ] **Root cause identified and documented**
  - Written summary in PR body: "Root cause: [describe the failure mechanism]"
  - Includes: what triggered it, where it manifests, why the fix resolves it
  - Reference to relevant source lines or test case demonstrating the bug

- [ ] **Regression test added**
  - New test that reproduces the bug (fails without the fix)
  - Test must run and fail before applying the fix
  - Test must pass after the fix is applied
  - Commit message or PR notes the before/after status

- [ ] **Existing tests still pass**
  - Full test suite runs: `cargo test`
  - No previously passing tests now fail
  - Invariants still hold: `cargo test --test invariants` (all 7 pass)

- [ ] **Changelog entry added**
  - Entry documents the bug, impact, and fix
  - Format: `- [bug-name]: Fixed issue where X would Y. Resolves #456.`
  - Grouped under "Bug Fixes" or "Fixes" section

- [ ] **Reviewed and approved**
  - At least one maintainer approval
  - Root cause explanation accepted
  - Regression test validated as correct

### Soft Gates

- [ ] **Performance not degraded**
  - If fix adds conditional logic or calls, profile latency before/after
  - If regression possible, add a performance test case to prevent re-introduction

- [ ] **Documentation updated if behavior changed**
  - If user-facing behavior changed, update command docs in `docs/commands/`
  - If policy behavior changed, update `docs/explanation/autonomic-policies.md`
  - If cicd.toml schema changed, update `docs/reference/cicd-toml.md`

- [ ] **Related issues closed**
  - All issues that reference this bug are reviewed
  - Issues without a fix are closed with explanation, or linked to follow-up work

---

## Refactor Definition of Done

A refactor (code reorganization, module extraction, algorithm swap) is shippable when all the following are satisfied:

### Hard Gates

- [ ] **No behavior change — invariants still pass**
  - Run full test suite: `cargo test` — all tests pass, no new failures
  - Run invariant suite: `cargo test --test invariants` — all 7 invariants (I1–I7) pass
  - Generated artifact bytes must be identical (I2 — PublishDeterminism)
  - For CLI refactors: `cargo cicd <noun> <verb> --help` output must remain stable (minor wording OK if same meaning)

- [ ] **Test coverage maintained or improved**
  - Coverage ≥ baseline coverage before refactor (no deletion of tests)
  - New tests added if refactoring exposes new internal seams requiring explicit testing
  - Mutant testing or fuzzing still passes (if project uses these)

- [ ] **Code style consistent**
  - `cargo fmt` applied: `cargo fmt --all`
  - No clippy warnings: `cargo clippy --all-features -- -D warnings`
  - Variable and function naming follows existing conventions

- [ ] **Reviewed and approved**
  - Maintainer approval confirming no behavior change
  - Code complexity assessment (refactoring should reduce or maintain, not increase, cyclomatic complexity)

### Soft Gates

- [ ] **Performance not degraded**
  - Latency histogram comparison before/after (if latency-sensitive code)
  - Heap allocations not increased (if memory-critical path)
  - For adapter refactors: overall workspace scan time ≤ baseline

- [ ] **Documentation updated**
  - CLAUDE.md updated if module structure, adapter seams, or public boundary changed
  - Inline code comments updated if algorithm or control flow changed
  - Architecture or ADR documents updated if refactor impacts decision architecture

---

## Test Definition of Done

A test is mergeable when all the following are satisfied:

### Hard Gates

- [ ] **Test is deterministic (no flakiness)**
  - Run 10 consecutive times locally: `for i in {1..10}; do cargo test test_name || exit 1; done`
  - All 10 runs pass
  - No randomness, no time-dependent assertions, no file system race conditions
  - If concurrency is involved, use `--test-threads=1` or explicit synchronization

- [ ] **Test is isolated**
  - No shared state with other tests (no global mutable statics)
  - Each test has its own `TempDir` or fixture workspace
  - Tests can run in any order or in parallel without cross-talk

- [ ] **Clear, explicit assertions**
  - Each assertion has a descriptive message (not just `assert!(x)`, but `assert!(x, "expected X because Y")`)
  - Assertions validate behavior, not just "does not crash"
  - For command tests: assert on exit code, stdout/stderr content, file system side-effects

- [ ] **Edge cases covered**
  - Empty workspace case tested (if applicable)
  - Large workspace case tested or noted as out-of-scope
  - Error/failure paths tested (not just happy path)
  - Boundary conditions tested (zero items, one item, N items)

- [ ] **Reasonable performance**
  - Test runtime ≤ 1 second (default; longer tests must have `#[ignore]` and be in a separate suite)
  - Slow tests (>1s) are isolated: `#[ignore]` and run separately or noted in CI config

- [ ] **Code reviewed**
  - Test logic reviewed for correctness (does the test actually test what it claims?)
  - Test fixtures are minimal and reusable

### Soft Gates

- [ ] **Test documentation**
  - Test function name clearly states what is being tested
  - Inline comments explain the test scenario or setup if non-obvious
  - If using fixtures, document which fixture and why

---

## Documentation Definition of Done

A documentation file or update is complete when:

### Hard Gates

- [ ] **Clear problem/use-case statement**
  - Opening sentence or section states the problem the doc solves
  - Audience is clear (user, contributor, operator, etc.)

- [ ] **Code examples compile and run (if applicable)**
  - If doc includes code snippets, they must be valid Rust or shell
  - Example: CLI examples must be runnable as written in a clean workspace
  - If example uses internal types, it's in a doc test or linked to actual source file

- [ ] **Links to related source code**
  - Paths to relevant modules/files are absolute (or relative from repo root, e.g., `src/engine/mod.rs`)
  - Links include specific line numbers or function names for precise reference
  - No dead links (verify with `cargo doc --open` for API docs)

- [ ] **Reviewed by non-author**
  - At least one reviewer who did not write the doc
  - Reviewer confirms clarity and completeness
  - Technical accuracy verified (links still valid, commands still work)

### Soft Gates

- [ ] **Diagram or table if helpful**
  - For architecture docs: include a module dependency diagram
  - For command docs: include a table of verbs and their purpose
  - For process docs: include a flowchart or sequence diagram
  - Format: Markdown tables, Mermaid diagrams (if `mcp__Mermaid_Chart__validate_and_render_mermaid_diagram` available), or ASCII art

- [ ] **Consistent with existing docs**
  - Style matches other docs in `docs/` (headings, code formatting, tone)
  - Follows the template structure if a template exists (ADR, how-to, explanation, etc.)
  - Terminology aligns with CLAUDE.md and project glossary

---

## Release Definition of Done

A release (version bump + crates.io publish) is shippable when all the following are satisfied:

### Hard Gates

- [ ] **All features complete and tested**
  - All features planned for the version are implemented and merged
  - Feature PRs are closed (either merged or rejected with explanation)
  - All feature tests pass: `cargo test --all-features`
  - wasm4pm evidence gate passes (if v26.6.2 or later): `cargo test --features wasm4pm` includes `tests/wasm4pm_evidence_gate.rs` — all tests pass with ALIVE verdict from wpm oracle

- [ ] **Changelog complete and reviewed**
  - `CHANGELOG.md` or `docs/release/CHANGELOG_<VERSION>.md` exists
  - Entries for all features, bug fixes, refactors in the release
  - Format: grouped by type (Features, Bug Fixes, Breaking Changes, Refactors, Documentation)
  - Reviewed and approved by at least one maintainer

- [ ] **Version bumped (semver)**
  - `Cargo.toml` version field updated
  - Update follows semver: MAJOR.MINOR.PATCH (or pre-release/build metadata if applicable)
  - Version matches tag name (if using tags)
  - Commit message references version and release notes: `chore: release v<VERSION>`

- [ ] **Git tag created**
  - Lightweight or annotated tag created: `git tag v<VERSION>`
  - Tag message (if annotated) includes: release date, high-level summary, link to release notes
  - Tag is signed if project policy requires: `git tag -s v<VERSION>`

- [ ] **GitHub release created**
  - Release notes drafted (auto-generated from changelog + git log, then reviewed)
  - Release notes include: features, bug fixes, breaking changes, known issues
  - Binaries or artifacts attached (if applicable)
  - Pre-release flag set (if alpha/beta/rc)

- [ ] **No forbidden terms in release materials**
  - Release notes, changelog, README examples contain zero matches of forbidden terms
  - Scan with: `grep -r 'ALIVE\|Inspection Gate\|Nehemiah' docs/release/ CHANGELOG.md`

- [ ] **All GitHub CI/CD checks pass on release branch**
  - Final commit before tag passes: `cargo test`, `cargo build --all-features`, `cargo clippy --all-features`, `cargo fmt --check`
  - All Actions workflows (if present) complete successfully

### Soft Gates

- [ ] **Crates.io published (if public crate)**
  - `cargo publish --dry-run` completes successfully
  - `cargo publish` executed (not just dry-run)
  - Crate is live on crates.io and installable: `cargo install cargo-cicd --version <VERSION>`

- [ ] **Announcement drafted**
  - High-level summary of what changed (for blog post, release notes, or social media)
  - Format: 100-200 words, highlights key features or fixes
  - Reviewed for tone and marketing alignment (if applicable)

- [ ] **Documentation site updated (if applicable)**
  - Docs website reflects the new version (if docs are versioned)
  - Links to the release on GitHub are added to the docs homepage

---

## Workflow: How to Apply These Gates

### For Feature PRs

1. **Before opening PR:**
   - Self-check: Does the feature pass all hard gates? (Code compiles, no forbidden terms, tests added, coverage ≥80%)
   - Run: `cargo test`, `cargo clippy --all-features -- -D warnings`, `cargo fmt`

2. **In PR description:**
   - List which soft gates you've addressed (performance, docs, security review)
   - Link to related issues
   - Provide a summary of what the feature does and how to test it

3. **After review feedback:**
   - Resolve comments or add justification
   - Re-run full test suite
   - Request re-review

4. **Merge approval:**
   - At least one maintainer approves
   - All hard gates confirmed PASS
   - Soft gates confirmed or documented (e.g., "Performance testing deferred to v26.7 due to large scope")

### For Bug Fix PRs

1. **In PR body:**
   - Write a root cause analysis (why the bug happened)
   - Link the regression test
   - Describe how to verify the fix

2. **Before merging:**
   - Regression test is merged alongside the fix
   - All existing tests pass
   - Invariants hold

### For Refactor PRs

1. **In PR body:**
   - Explain what was refactored and why
   - Assert that no behavior change occurred (link to passing invariant tests)
   - Provide before/after code complexity metrics (lines of code, function count, cyclomatic complexity)

2. **Before merging:**
   - Full test suite passes
   - Invariants pass (especially I2 — PublishDeterminism)
   - Clippy and fmt checks pass

### For Release PRs

1. **Create release branch:**
   - Branch off main: `git checkout -b release/v<VERSION>`
   - Update `Cargo.toml` version
   - Create or update `CHANGELOG.md`
   - Commit: `chore: release v<VERSION>`

2. **Create PR to main:**
   - Title: `Release v<VERSION>`
   - Body: Link to changelog, list of commits since last release
   - Ensure all CI checks pass

3. **After approval:**
   - Merge to main
   - Create git tag: `git tag -s v<VERSION> -m "Release v<VERSION>"`
   - Push tag: `git push origin v<VERSION>`
   - Create GitHub release from tag

---

## Exceptions and Waivers

A hard gate may be waived **only if**:

1. **Documented in the PR body or issue:**
   - Clear explanation of why the gate cannot be met
   - Justification that the benefit outweighs the risk

2. **Approved by at least two maintainers:**
   - Both must explicitly acknowledge the waiver
   - Waiver is recorded in the merge commit message or issue

3. **A follow-up task is created and linked:**
   - Issue or PR that addresses the waived gate in the next release or next sprint
   - Example: "Performance testing deferred to v26.7 per #999"

---

## Tools & Commands Reference

### Quick Verification Checklist

```bash
# 1. Build all features
cargo build --all-features

# 2. Run all tests
cargo test

# 3. Run invariants
cargo test --test invariants

# 4. Check code style and lint
cargo fmt --check
cargo clippy --all-features -- -D warnings

# 5. Check for forbidden terms (from CLAUDE.md)
grep -r 'ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8' src/ docs/commands/ 2>/dev/null || echo "Clean"

# 6. Measure coverage (requires tarpaulin)
cargo tarpaulin --lib --out Html --exclude-files tests/

# 7. Run wasm4pm evidence gate (if applicable)
cargo test --features wasm4pm --test wasm4pm_evidence_gate
```

### For Release Checklist

```bash
# 1. Verify version in Cargo.toml
grep '^version' Cargo.toml

# 2. Verify changelog exists
ls -la CHANGELOG.md docs/release/CHANGELOG_*.md 2>/dev/null | head -5

# 3. Dry-run publish
cargo publish --dry-run

# 4. Create and sign tag
git tag -s v<VERSION> -m "Release v<VERSION>"

# 5. Final test on clean checkout (optional but recommended)
git clone . /tmp/cargo-cicd-release-check
cd /tmp/cargo-cicd-release-check
cargo test --all-features
```

---

## Related Documents

- **CLAUDE.md** — Project mission, architecture, feature flags, test hierarchy
- **INVARIANTS.md** — 7 non-negotiable public boundary invariants that all tests must trace to
- **CRATES_IO_RELEASE_CHECKLIST.md** — Detailed pre-publish validation for crates.io
- **WASM4PM_EVIDENCE_GATE.md** — Evidence emission and wasm4pm oracle integration for release closure
- **ARCHITECTURE.md** — Noun-verb CLI grammar, adapter pattern, EngineState design

---

## Glossary

| Term | Definition |
|------|-----------|
| **Hard gate** | Blocker — must be satisfied before merge; no exceptions without documented waiver and 2-maintainer approval |
| **Soft gate** | Best practice — strongly recommended but may be deferred if justified and a follow-up task is created |
| **Invariants** | 7 non-negotiable public boundary properties (I1–I7) that all code must maintain |
| **Coverage** | % of lines in new code exercised by tests; target: ≥80% |
| **Regression test** | Test that reproduces the bug and would fail without the fix |
| **Forbidden terms** | Internal project terms not allowed in public surfaces: ALIVE, Inspection Gate, Nehemiah, etc. |
| **Evidence gate** | wasm4pm oracle acceptance test that validates process evidence (XES format) before release |

---

**Last Updated:** 2026-06-14  
**Version:** 26.6.2  
**Maintainers:** cargo-cicd team
