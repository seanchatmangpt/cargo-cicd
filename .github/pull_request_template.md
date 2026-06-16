## Summary
<!-- What this PR does and why -->

## Type of Change
- [ ] feat — new feature or verb
- [ ] fix — bug fix
- [ ] docs — documentation only
- [ ] refactor — code change without feature/bug
- [ ] test — adding or improving tests
- [ ] chore — maintenance (deps, config, CI)

## Motivation
<!-- Why this change is needed -->

## Changes Made
<!-- Bullet list of what changed -->

## Evidence Pattern Compliance
- [ ] New verbs emit `ProcessEvent::started()` + `ProcessEvent::completed()` with `case_id`
- [ ] No forbidden terms in help text (ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8)
- [ ] Adapters remain stateless (no business logic)
- [ ] Policies remain in suggest mode only

## Test Plan
- [ ] `cargo make test` passes
- [ ] `cargo test --test invariants` passes
- [ ] Tests added for new behavior
- [ ] Evidence gate: `cargo test --test wasm4pm_evidence_gate` passes (or declares Blocked)

## Definition of Done Checklist
- [ ] Feature DoD: evidence emission pattern followed
- [ ] Bug Fix DoD: regression test added (if applicable)
- [ ] Docs updated (if behavior changed)
- [ ] CHANGELOG.md updated (for release PRs)
