# cargo-cicd Workflow Checklists

This file provides step-by-step checklists for every recurring workflow in cargo-cicd development. Complete each checklist in order — later items depend on earlier ones passing.

---

## Pre-Commit Checklist

Run before every `git commit`. These gates catch problems before they reach CI.

- [ ] **Format:** `cargo make fmt` — no formatting diff remains
- [ ] **Lint:** `cargo make lint` — zero clippy warnings
- [ ] **Tests:** `cargo make test` — all test suites exit 0
- [ ] **Invariants:** `cargo test --test invariants` — all 7 public boundary invariants pass
- [ ] **Forbidden terms:** no forbidden term appears in any `--help` output (covered by invariants above, but verify manually when adding help strings)
- [ ] **Evidence pattern:** any new verb follows start → work → complete → emit; no verb claims its own final verdict
- [ ] **Git state:** `cargo cicd git status` — review dirty files before committing

---

## Pre-Push Checklist

Run after all commits are ready and before `git push`. These gates validate workspace-level health.

- [ ] All pre-commit items above pass
- [ ] **Workspace health:** `cargo cicd workspace doctor` — no blocking diagnostics
- [ ] **Evidence gate:** `cargo cicd evidence doctor` — evidence is valid, or declare `ExpectedWpmVerdict::Blocked` if oracle is unavailable
- [ ] **Commit messages:** every commit follows `feat(scope): description` format; scope is one of `core`, `cli`, `target`, `test`, `git`, `autonomic`, `docs`, `receipts`
- [ ] **No secrets:** no `.env`, credential files, or private keys staged for push

---

## Pre-Release Checklist

Run before tagging any release. The oracle must be available and return `Accept`. No release ships without a passing evidence gate.

- [ ] All pre-push items above pass
- [ ] **Oracle available:** `wpm --version` returns a version string
- [ ] **Evidence gate tests:** `cargo test --test wasm4pm_evidence_gate -- --nocapture` exits 0
- [ ] **Mutation tests:** `cargo test --test wasm4pm_evidence_mutation` exits 0
- [ ] **Refusal cases:** `cargo test --test wasm4pm_refusal_cases` exits 0
- [ ] **Oracle accepts evidence:** `wpm audit target/cargo-cicd/evidence/evt-*.xes` returns `Accept` for all files
- [ ] **Receipts valid:** `wpm receipt doctor --format json --strict receipts/*.json` returns `Accept` for all receipts
- [ ] **Release build:** `cargo build --release --features autonomic,wasm4pm` succeeds with no errors
- [ ] **CHANGELOG.md:** updated with all features and fixes since last release
- [ ] **Version bump:** version incremented in `Cargo.toml` and `src/main.rs`
- [ ] **Git clean:** `git status` shows no dirty files and no untracked files
- [ ] **Tag created:** `git tag -a vX.Y.Z -m "Release vX.Y.Z — evidence adjudicated by wasm4pm"`
- [ ] **Tag pushed:** `git push origin main --tags`

---

## New Verb Checklist

Follow this checklist when adding a verb to an existing noun. Complete steps in order.

### Ontology & Generation
- [ ] **Ontology entry added:** new `skos:Concept` block in `ontology/cargo-cicd-capabilities.ttl` with `cc:noun`, `cc:verb`, `cc:cliCommand`, and `dcterms:description`
- [ ] **ggen run:** `ggen` executed; README, test scaffolding, and reference docs regenerated
- [ ] **No manual edits to generated files:** only edit source templates or the ontology, then re-run `ggen`

### Implementation
- [ ] **Verb struct defined:** `pub struct MyNounMyVerbVerb;` in the appropriate noun module
- [ ] **VerbCommand trait implemented:** `impl VerbCommand for MyNounMyVerbVerb` with a `run()` method
- [ ] **evidence_dir() called:** evidence output directory created at start of `run()`
- [ ] **case_id created:** unique identifier generated to link the start and complete event pair
- [ ] **ProcessEvent::started() emitted:** start event written before any work begins
- [ ] **Work performed:** noun logic executes and result is captured
- [ ] **Verdict determined:** exactly one of `PASS`, `WARN`, or `FAIL` selected based on work outcome
- [ ] **ProcessEvent::completed() emitted:** complete event written with verdict, duration, and case_id
- [ ] **append_events() called:** events persisted to XES and JSONL evidence files
- [ ] **Verb registered:** new verb struct added to the parent noun's `verbs()` vec

### Testing
- [ ] **CLI test file created or updated:** test in `tests/cli/` for the new verb (happy path at minimum)
- [ ] **Destructive verb has --confirm test:** if verb modifies state, test that it refuses without `--confirm` and proceeds with it
- [ ] **Dry-run test:** if verb supports `--dry-run`, test that no side effects occur
- [ ] **Evidence test:** verify XES file is emitted with correct `verdict_claimed`
- [ ] **`cargo make test` passes:** full suite exits 0 with the new verb in place

### Documentation
- [ ] **Help text written:** verb description in clap attributes is clear, accurate, and contains no forbidden terms
- [ ] **GLOSSARY.md updated:** if the verb introduces a new concept, add it to `docs/GLOSSARY.md`

---

## New Policy Checklist

Follow this checklist when adding an autonomic policy. Policies are read-only — they never take destructive action.

### Module Setup
- [ ] **Policy module created:** `src/policies/your_policy_name.rs` exists
- [ ] **Feature gate applied:** module is `#[cfg(feature = "autonomic")]`
- [ ] **eval() function signature correct:** `pub fn eval(state: &EngineState) -> PolicyEntry`
- [ ] **PolicyEntry fields complete:** `policy_name`, `verdict`, `recommendation`, `emitted_at` all set

### Verdict Logic
- [ ] **Mode checked:** policy inspects `state.projection` or runtime config before taking any action
- [ ] **No destructive action:** policy never writes files, runs commands, or modifies state — suggest mode only
- [ ] **PolicyVerdict used correctly:**
  - `Pass` — condition not triggered
  - `Warn` — condition detected, recommendation emitted
  - `Suggest` — soft recommendation
  - `Skip` — policy inapplicable to this workspace
- [ ] **Recommendation is actionable:** the `recommendation` string tells the user exactly which `cargo cicd` command to run

### Registration & Testing
- [ ] **Registered in run_all_policies():** `your_policy::eval(state)` added to the dispatch list in `src/autonomic/policies.rs`
- [ ] **PolicyState populated:** policy result appears in `PolicyState::entries` after `run_all_policies()` runs
- [ ] **Unit test written:** `tests/autonomic_policies.rs` contains at least one test for the triggered case and one for the pass case
- [ ] **Fixture state used:** tests construct an `EngineState` with controlled values rather than reading live workspace state
- [ ] **`cargo test --features autonomic --test autonomic_policies` passes**

---

## New Adapter Checklist

Follow this checklist when adding a new `EngineState` adapter.

- [ ] **Adapter module created:** `src/adapters/your_adapter.rs`
- [ ] **No internal state:** all methods are `&self` or free functions — no `mut self`, no stored results
- [ ] **Silent failure:** adapter returns defaults (empty string, zero, empty vec) on any error — never panics, never propagates errors
- [ ] **No cross-adapter calls:** adapter does not call other adapters; each adapter is independently invocable
- [ ] **EngineState wired:** `EngineState::from_workspace()` in `src/engine/mod.rs` calls the new adapter and stores the result in the appropriate state dimension
- [ ] **New state dimension added (if needed):** new field added to `EngineState`, with a corresponding `src/engine/your_dimension_state.rs` module
- [ ] **Performance documented:** adapter speed category noted (fast/medium/slow) in `src/adapters/mod.rs` registry comment
- [ ] **Test written:** adapter tested in isolation with controlled inputs

---

## Bug Fix Checklist

Follow this checklist when fixing a defect.

- [ ] **Failing test identified:** a test that reproduces the bug exists (or write one first)
- [ ] **Root cause located:** the defective field, adapter, or noun handler identified by file and line
- [ ] **Minimal fix applied:** change is scoped to the defective location; no unrelated refactors bundled
- [ ] **Regression test added:** a new test that would have caught this bug before the fix
- [ ] **Evidence pattern unbroken:** if the fix touches a verb, the start/complete evidence emission is still correct
- [ ] **Forbidden terms check:** fix introduces no forbidden terms in any user-visible string
- [ ] **`cargo make test` passes**
- [ ] **Commit message follows format:** `fix(scope): short description of what was wrong`

---

## Evidence Emission Checklist

Follow this checklist when implementing or reviewing the evidence emission pattern inside a verb's `run()` method.

- [ ] `evidence_dir()` called to create output directory
- [ ] `case_id` generated (unique per invocation, links start and complete)
- [ ] `ProcessEvent` created with `lifecycle_transition = "start"` and `verdict_claimed = "PASS"` (provisional)
- [ ] Start event serialized to XES and JSONL before work begins
- [ ] Work performed; outcome captured
- [ ] `verdict_claimed` updated to reflect actual outcome (`PASS`, `WARN`, or `FAIL`)
- [ ] `ProcessEvent` created with `lifecycle_transition = "complete"`, final verdict, `duration_ms`, and `case_id`
- [ ] Complete event serialized to XES and JSONL
- [ ] `append_events()` persists both events to `ProcessEventState`
- [ ] `CicdTomlWriter` writes updated `EngineState` to `cicd.toml` (for execution verbs)
- [ ] Verb does **not** call `wpm audit` directly — oracle invocation is test-layer responsibility
- [ ] Tests assert `wpm_verdict`, never `state.some_field`

---

## Feature Flag Checklist

Follow this checklist when adding or modifying a feature flag.

- [ ] **Flag declared in Cargo.toml:** new flag listed under `[features]` with correct dependency chain
- [ ] **Implication chain correct:** any flag requiring `process-data` lists it as a dependency (e.g., `autonomic = ["process-data"]`)
- [ ] **Code gated correctly:** new code under the flag wrapped in `#[cfg(feature = "your-flag")]`
- [ ] **Default build unaffected:** `cargo build` (no features) still compiles and passes invariants
- [ ] **Feature build passes:** `cargo build --features your-flag` succeeds
- [ ] **Tests added:** `cargo test --features your-flag` exercises the new flag's behaviour
- [ ] **ProjectionProfile updated:** `src/engine/projection_profile.rs` reflects the new flag in the surface contract
- [ ] **feature_projection test updated:** `tests/feature_projection.rs` asserts the new flag's expected surface
