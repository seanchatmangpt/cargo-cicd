# Receipt: CARGO_CICD_V26_6_2_GGEN_DOCS_BASELINE

**Date:** 2026-06-02
**Phase:** Two-pass docs manufacturing — Pass 1: ggen baseline docs
**Verdict:** PASS

## Files Manufactured

| File | Kind | Notes |
|---|---|---|
| `docs/tutorials/getting-started.md` | Tutorial | End-to-end first-run walkthrough |
| `docs/how-to/custom-pipeline.md` | How-to | Pipeline customization guide |
| `docs/how-to/wasm4pm-graduation.md` | How-to | Graduating artifacts to wasm4pm |
| `docs/explanation/architecture.md` | Explanation | Crate architecture and design decisions |
| `docs/explanation/two-pass-docs.md` | Explanation | Two-pass docs manufacturing doctrine |

## Gate Verdicts

### run-matrix.sh

| Test | Result |
|---|---|
| basic pipeline | PASS |
| custom stage | PASS |
| wasm4pm evidence | PASS |
| graduation | PASS |
| strict boundary | PASS |

### Pre-publish Gate

| Gate | Verdict |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --all-features --tests` | PASS (126 passed, 0 failed) |
| `cargo doc --no-deps --all-features` | PASS |
| `cargo package --list` | PASS |

## Overall Verdict

PASS — ggen baseline docs surface manufactured clean; all gates green.
