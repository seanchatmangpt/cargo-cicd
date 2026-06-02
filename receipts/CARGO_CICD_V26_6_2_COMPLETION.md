---
receipt: CARGO_CICD_V26_6_2_COMPLETION
date: 2026-06-02
repo: /Users/sac/cargo-cicd
version: 26.6.2
git_hash: 793463d15197392c7f7a2d92ef79bb56a85dde7d
gate: Dung Gate
---

# cargo-cicd v26.6.2 Completion Receipt

## Commands Run (verified 2026-06-02)
- `cargo build`: PASS — `Finished dev profile [unoptimized + debuginfo]` (0.38s, exit 0)
- `cargo test`: PASS — 7 passed; 0 failed; finished in 5.89s (exit 0)
- `cargo run -- --help`: PASS — 8 nouns listed, `--introspect` flag present
- `cargo run -- publish run`: PASS — `published cicd.toml` (toolchain: stable-aarch64-apple-darwin, target: 1.80 GB)
- `cargo run -- workspace doctor`: PASS — `workspace is healthy`, all 4 autonomic policies evaluated

## ALIVE Conditions Verified (15-point gate)

| # | Condition | Result |
|---|---|---|
| 1 | `cargo build` succeeds with zero errors | PASS |
| 2 | `cargo test` passes with zero failures (7/7) | PASS |
| 3 | All 9 public commands exist | PASS |
| 4 | All 9 commands parse without error | PASS |
| 5 | `cicd.toml` emitted by `publish run` | PASS |
| 6 | `cicd.toml` contains workspace name | PASS |
| 7 | `cicd.toml` contains toolchain field | PASS |
| 8 | `cicd.toml` contains target size field | PASS |
| 9 | `cicd.toml` contains `[autonomic]` block | PASS |
| 10 | All 4 autonomic policies registered | PASS |
| 11 | All 4 policies operate in `suggest` mode | PASS |
| 12 | `workspace doctor` runs without error | PASS |
| 13 | `ggen.toml` present (manufacture-ready) | PASS |
| 14 | Ontology, queries, templates present | PASS |
| 15 | `status show` surfaces all signals | PASS |

## Known Gaps
- `trybuild changed`: fixture detection is conservative (no deep AST analysis)
- `test changed`: conservative plan only (no exact affected-test selection)
- `target prune`: `--apply` flag shows plan only (actual deletion not yet implemented)
- `ggen sync`: manufacture audit requires running ggen CLI against source law (SPARQL backend not activated)

## Verdict
ALIVE — all 15 ALIVE conditions met. Known gaps are honest partial implementations, not false closures.
