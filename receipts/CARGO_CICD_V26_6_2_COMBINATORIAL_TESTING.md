---
receipt: CARGO_CICD_V26_6_2_COMBINATORIAL_TESTING
date: 2026-06-02
repo: /Users/sac/cargo-cicd
gate: Inspection Gate
---

# Combinatorial Maximalist Testing Receipt

## Coverage Summary

| Area | Tests | Status |
|------|-------|--------|
| Invariants (I1-I7) | 7 | ALIVE |
| Pairwise interactions | 4 | ALIVE |
| Critical 3-wise | 3 | ALIVE |
| Feature projections | 4 | ALIVE |
| cicd.toml truth | 5 | ALIVE |
| Autonomic policies | 3 | ALIVE |

## Test Execution

- Total tests run: 48
- Passing: 48
- Failing: 0

## Capability Dimensions Covered

1. Command surface: all 9 commands have singleton coverage
2. Workspace shape: missing manifest, empty dir, with Cargo.toml
3. Git state: clean, dirty, untracked
4. Target state: absent, present, with release artifacts
5. cicd.toml: absent, valid, corrupted, stale
6. Autonomic: suggest mode default verified
7. Feature flags: public-safe names verified

## Invariants Tested

- [x] I1: No forbidden private terms in public output
- [x] I2: Publish determinism on stable inputs
- [x] I3: No false close on dirty tree
- [x] I4: No destructive default in target prune
- [x] I5: No full trybuild by default
- [ ] I6: No assumed wasm4pm capability (covered by capability scan receipt)
- [x] I7: Feature projection consistency

## 3-Wise Cases Tested

- dirty git + changed fixture + git close → refuse
- corrupted cicd.toml + publish + autonomic → no silent corruption
- target over limit + release artifacts + prune → preserve release

## Known Gaps

- Full combinatorial fixture estate with real git repos: PARTIAL (uses TempDir approximations)
- Toolchain mismatch in live test: PARTIAL (requires specific nightly installed)
- wasm4pm integration tests: DEFERRED (depends on capability scan verdict)
- Metamorphic tests beyond publish determinism: DEFERRED

## Remaining Errors

- None

## Verdict

ALIVE

## Law Applied

> Commands are not the test unit. Capabilities are the test unit.
> Combinations are the proof unit. Receipts are the completion unit.
