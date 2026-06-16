# Definition of Done

This document defines what "done" means for every category of work in
cargo-cicd. A change is not done until every item in the relevant checklist is
satisfied. For work that touches multiple categories (e.g., a bug fix that also
adds a policy), all applicable checklists apply.

---

## Feature DoD

A feature is a new verb, a new noun, or a significant new capability added to
an existing verb.

- [ ] Verb implements the full evidence emission pattern: a `start` event is
      emitted before any work begins and a `complete` event is emitted after
      work finishes, with `case_id` set on both events.
- [ ] Verb is registered in the noun's `verbs()` vec (or equivalent dispatch
      table) so it is reachable from the CLI.
- [ ] At least one CLI test exists in `tests/cli/test_<noun>.rs` that invokes
      the new verb and asserts `output.status.success()`.
- [ ] Additional CLI tests cover each distinct output path (PASS, WARN, FAIL)
      when the verb can produce more than one verdict.
- [ ] Help text for the new verb contains no reserved terms (the canonical
      list is enforced by `invariant_public_boundary_no_forbidden_terms_in_all_help`
      in `tests/invariants.rs`).
- [ ] `cargo test --test invariants` passes (all 7 invariants satisfied).
- [ ] `cargo make test` passes (all test suites).
- [ ] Evidence gate is not broken: `wasm4pm_evidence_gate` tests pass, or all
      evidence tests declare `ExpectedWpmVerdict::Blocked` with a comment
      explaining the offline context.
- [ ] `cargo clippy` produces no new warnings.
- [ ] No dead code introduced; unused items removed or feature-gated.

---

## Bug Fix DoD

- [ ] Root cause is identified and stated in the commit message body (one or
      two sentences; explain WHY the bug occurred, not just what changed).
- [ ] A regression test is added that fails on the unfixed code and passes
      after the fix. The test must be placed in the most specific applicable
      test file (`tests/cli/`, `tests/invariants.rs`, etc.).
- [ ] The fix does not break any previously passing test; `cargo make test`
      passes in full.
- [ ] If the bug affected evidence output or XES content, the wasm4pm verdict
      for the affected operation is still `Accept` after the fix.
- [ ] `cargo clippy` produces no new warnings.
- [ ] If the bug was caused by an adapter silently returning a wrong default,
      a comment is added at the call site explaining the expected failure mode.

---

## Policy Addition DoD

A policy lives in `src/policies/` and runs under the `autonomic` feature flag.

- [ ] Policy module created at `src/policies/<policy_name>.rs` with a public
      `eval(state: &EngineState) -> PolicyEntry` function gated by
      `#[cfg(feature = "autonomic")]`.
- [ ] Policy registered in `src/autonomic/policies.rs::run_all_policies()`.
- [ ] Policy returns one of three verdicts only: `Pass`, `Warn`, or `Skip`.
      It never returns `Fail` and never takes any destructive action — no file
      writes, no process invocations that mutate state.
- [ ] Suggest-mode behavior is documented in the `recommendation` field of the
      returned `PolicyEntry` (a human-readable string describing what the user
      should do, not what the policy did).
- [ ] At least two tests in `tests/autonomic_policies.rs` cover the policy:
      one that triggers `Warn` and one that results in `Pass` (or `Skip` if
      the policy is inapplicable in certain contexts).
- [ ] Tests run cleanly under `cargo test --features autonomic --test autonomic_policies`.
- [ ] `cargo make test` passes in full (policy does not affect non-autonomic
      test suites).

---

## Evidence Gate DoD (Release Gate)

The evidence gate must pass before any release tag is created. This checklist
applies to the release preparation commit and tag.

- [ ] All `cargo make test` suites pass with exit code 0.
- [ ] Evidence has been emitted for the release operation:
      `target/cargo-cicd/evidence/` contains at least one `.xes` file and its
      companion `.jsonl` file from the current session.
- [ ] `wpm receipt doctor --format json --strict receipts/*.json` returns
      `Accept` for all receipt files. No `Refuse` results.
- [ ] `wpm audit target/cargo-cicd/evidence/evt-*.xes` returns `Accept` for
      all evidence files produced by this release. No `Refuse` results.
- [ ] No forbidden terms appear in any `--help` output:
      `cargo test --test invariants` passes the
      `invariant_public_boundary_no_forbidden_terms_in_all_help` test.
- [ ] `cicd.toml` at workspace root reflects the current state and includes
      the evidence events from this release session. The file is committed.
- [ ] `wasm4pm_evidence_gate`, `wasm4pm_evidence_mutation`, and
      `wasm4pm_refusal_cases` test suites all pass (not just `Blocked`; the
      oracle must be available and returning real verdicts for a release).

---

## Release DoD

A release is the act of tagging and pushing a version. All Feature DoD and Bug
Fix DoD items for work included in the release must already be satisfied. The
Release DoD adds the following:

- [ ] All Feature DoD items satisfied for every feature included in the release.
- [ ] All Bug Fix DoD items satisfied for every bug fix included in the release.
- [ ] Evidence Gate DoD satisfied in full (oracle available, verdicts are
      `Accept`, not `Blocked`).
- [ ] `CHANGELOG.md` updated with an entry for this version covering all
      user-visible changes: new features, bug fixes, breaking changes, and
      deprecations.
- [ ] Version bumped in `Cargo.toml` (workspace root) and in `src/main.rs`
      wherever the version constant is declared. Both locations must agree.
- [ ] `cargo make test` passes one final time on the exact commit that will
      be tagged.
- [ ] Working tree is clean: `git status` shows no dirty files, no staged
      changes, and no untracked files that should be committed.
- [ ] Release commit created with message:
      `chore(release): v<VERSION> evidence gate pass`
- [ ] Git tag created and annotated:
      `git tag -a v<VERSION> -m "Release v<VERSION>"`
- [ ] Tag and commit pushed to `origin/main`:
      `git push origin main --tags`

---

## Refactor DoD

A refactor changes internal structure without altering externally observable
behavior. No new features, no bug fixes — structure only.

- [ ] All tests pass without modification after the refactor:
      `cargo make test` exits 0. If a test had to change to accommodate the
      refactor, the change must be reviewed carefully to confirm it does not
      weaken the assertion.
- [ ] No regressions in evidence emission: the same verbs that emitted XES
      before the refactor still emit XES after it, with identical field
      structure.
- [ ] `cargo clippy` produces no new warnings.
- [ ] Adapters remain stateless after the refactor. No mutable state, no
      cross-adapter calls.
- [ ] `EngineState` remains the single aggregate root. Nouns read from it;
      adapters populate it. Nothing else writes to it directly.
- [ ] The seven invariants in `tests/invariants.rs` still pass.
- [ ] Public API surface (noun names, verb names, help text) is unchanged
      unless the refactor was explicitly scoped to modify it.
- [ ] If the refactor moves code between modules, import paths in tests are
      updated and the tests continue to compile without feature-flag changes.

---

## Quick Reference Table

| Work Type | Tests Required | Evidence Required | Oracle Required |
|---|---|---|---|
| Feature (new verb) | CLI tests in `tests/cli/` | Yes — start + complete | No (Blocked OK) |
| Bug Fix | Regression test | Only if evidence was affected | No (Blocked OK) |
| Policy Addition | `tests/autonomic_policies.rs` | No | No |
| Evidence Gate | All suites | Yes — full session | Yes — must Accept |
| Release | All suites | Yes — full session | Yes — must Accept |
| Refactor | All existing suites | No new requirement | No |

---

*Last updated: 2026-06-15*
