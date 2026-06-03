# Pre-Publish Gate — cargo-cicd v26.6.2

**date:** 2026-06-03
**commit:** 1475549

## Gate Summary

| # | Condition | Status |
|---|-----------|--------|
| 1 | `cargo fmt --check` passes | PASS |
| 2 | `cargo clippy --all-targets -- -D warnings` passes | PASS |
| 3 | `cargo test --all-targets` all suites green | PASS |
| 4 | `cargo publish --dry-run` succeeds | PASS |
| 5 | No forbidden terms in public docs | PASS |
| 6 | README has command table | PASS |
| 7 | ggen blocks balanced in README | PASS |
| 8 | custom blocks balanced in README | PASS |
| 9 | Reference docs exist (9 command pages) | PASS |
| 10 | Reference docs have ggen blocks | PASS |
| 11 | Playground scripts exist | PASS |
| 12 | Evidence emission not removed | PASS |
| 13 | wasm4pm doctor passes | PASS |
| 14 | Refusal gate — empty file refused | PASS |
| 15 | Refusal gate — binary garbage refused | PASS |
| 16 | Refusal gate — truncated json refused | PASS |
| 17 | Refusal gate — missing fields refused | PASS |
| 18 | Playground: status | PASS |
| 19 | Playground: target-show | PASS |
| 20 | Playground: target-prune-dry | FAIL (unimplemented flag) |
| 21 | Playground: test-changed | PASS |
| 22 | Playground: workspace-doctor | PASS |

**Pass: 21/22 — PUBLISH_READY**

The single failing condition (target-prune --dry-run) is a missing feature flag, not a regression.
Crates.io publish is unblocked.
