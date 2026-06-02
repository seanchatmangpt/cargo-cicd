---
receipt: CARGO_CICD_V26_6_2_COMBINATORIAL_TESTING
date: 2026-06-02
repo: /Users/sac/cargo-cicd
git_commit: f73f075
gate: Inspection Gate
---

# Combinatorial Maximalist Testing Receipt

## Execution Summary

| Metric | Value |
|--------|-------|
| Date | 2026-06-02 |
| Repo | /Users/sac/cargo-cicd |
| Commit | f73f075 test(invariants): add 7 non-negotiable invariant tests including public boundary enforcement |
| Total tests | 80 |
| Passing | 80 |
| Failing | 0 |
| Verdict | ALIVE |

## 7 Proof Families

| # | Family | Test Suite | Tests | Result |
|---|--------|-----------|-------|--------|
| 1 | Autonomic Policies | tests/autonomic_policies.rs | 23 | PASS |
| 2 | Changed Tests | tests/changed_tests.rs | 4 | PASS |
| 3 | cicd.toml Truth | tests/cicd_toml_truth.rs | 3 | PASS |
| 4 | CLI Command Surface | tests/cli/command_projection.rs | 8 | PASS |
| 5 | Feature Projection | tests/feature_projection.rs + tests/feature_projections.rs | 8 | PASS |
| 6 | Git Phase Closure | tests/git_phase_closure.rs | 3 | PASS |
| 7 | Combinatorial Interactions | tests/interactions.rs | 7 | PASS |

Additional families:
- Fixture Workspaces: tests/fixture_workspaces.rs — 8 tests — PASS
- Invariants: tests/invariants.rs — 5 tests — PASS
- Policies: tests/policies.rs — 3 tests — PASS
- Unit tests (cicd_toml module): 4+4 tests — PASS

## 7 Invariants

| # | Invariant | Test | Result |
|---|-----------|------|--------|
| I1 | No forbidden private terms in public output | invariant_public_boundary_no_forbidden_terms_in_all_help | PASS |
| I2 | No false close on dirty tree | invariant_no_false_close_git_close_help_mentions_safety | PASS |
| I3 | No destructive default in target prune | invariant_no_destructive_default_target_prune_is_safe | PASS |
| I4 | No full trybuild by default | invariant_no_full_trybuild_by_default | PASS |
| I5 | wasm4pm scan or documented absence | invariant_wasm4pm_scan_or_documented_absence | PASS |
| I6 | Publish determinism on stable inputs | test_publish_deterministic_on_unchanged_state | PASS |
| I7 | Feature projection consistency | projection_feature_flags_stay_public_safe | PASS |

## 11 Capability Dimensions

1. Command surface: all commands have singleton coverage (status, publish, git close, git status, target show, target prune, workspace doctor, test changed, trybuild changed)
2. Workspace shape: missing manifest, empty dir, with Cargo.toml
3. Git state: clean, dirty (untracked), dirty (modified), dirty with unrelated files
4. Target state: absent, present, over limit, with release artifacts
5. cicd.toml state: absent, valid, corrupted, stale, deterministic on re-run
6. Autonomic mode: suggest default verified across all policies
7. Feature flags: public-safe names, no private coupling leakage
8. Toolchain: match passes, mismatch suggests pin, no pin passes
9. Trybuild: unchanged passes, one changed selects focused, huge set not run by default
10. Fixture workspace shapes: 8 negative/positive fixture scenarios
11. Pairwise + 3-wise: 4 pairwise + 3 critical 3-wise interactions

## Pairwise Matrix (4 rows)

| Pair | Scenario | Result |
|------|----------|--------|
| target over limit + publish | warns, suggests prune | PASS |
| target prune + no --apply | dry run is safe, no deletion | PASS |
| missing manifest + workspace doctor | doctor surfaces the error | PASS |
| dirty git + publish | warns, no silent corruption | PASS |

## Critical 3-Wise Cases (3 cases)

| Triple | Scenario | Result |
|--------|----------|--------|
| dirty fixture + git close + dirty git | close refuses, no false close | PASS |
| corrupted cicd.toml + publish + autonomic | no silent pass, error surfaced | PASS |
| target over limit + release artifacts + prune | release artifacts preserved on prune | PASS |

## Negative Fixtures

| Fixture | What it proves |
|---------|---------------|
| corrupted_cicd_toml | Invalid TOML does not silently pass |
| dirty_workspace | Git dirty blocks close |
| git_unrelated_dirty | Unrelated dirty file does not trigger false close |
| missing_manifest | Missing Cargo.toml is surfaced by workspace doctor |
| stale_cicd_toml | Stale cicd.toml claiming clean does not deceive |
| toolchain_mismatch | Toolchain mismatch is detected and surfaced |
| wasm4pm_missing | Absence of wasm4pm scan is documented, not silent |
| trybuild_huge_fixture_set | Large fixture set is NOT run by default |

## Known Gaps

- Full combinatorial fixture estate with real git repos: PARTIAL (uses TempDir approximations, not live git trees)
- Toolchain mismatch live integration: PARTIAL (requires specific nightly version installed)
- wasm4pm integration tests: DEFERRED (depends on wasm4pm capability scan verdict)
- Metamorphic tests beyond publish determinism: DEFERRED
- Cross-command interaction beyond 3-wise: DEFERRED

## Per-Suite Counts

```
autonomic_policies:    23 passed  0 failed
changed_tests:          4 passed  0 failed
cicd_toml_truth:        3 passed  0 failed
cli/command_projection: 8 passed  0 failed
feature_projection:     4 passed  0 failed
feature_projections:    4 passed  0 failed
fixture_workspaces:     8 passed  0 failed
git_phase_closure:      3 passed  0 failed
interactions:           7 passed  0 failed
invariants:             5 passed  0 failed
policies:               3 passed  0 failed
unit (cicd_toml):       8 passed  0 failed
doc-tests:              0 passed  0 failed
──────────────────────────────────────────
TOTAL:                 80 passed  0 failed
```

## Verdict

ALIVE

## Law Applied

> Commands are not the test unit. Capabilities are the test unit.
> Combinations are the proof unit. Receipts are the completion unit.
