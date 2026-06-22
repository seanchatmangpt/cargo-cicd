---
description: Run the public-boundary invariant suites and report which hold, with remediation tips on failure.
allowed-tools: Bash, Read, Grep
---

Trigger: user requests invariant check or public boundary audit before release.

## Steps

```bash
cargo test --test invariants 2>&1 | tee /tmp/ccicd_invariants.txt
cargo test --test cli 2>&1 | tee /tmp/ccicd_cli.txt
cargo test --test feature_projection 2>&1 | tee /tmp/ccicd_feature_projection.txt
```

Parse each output:
```bash
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_invariants.txt
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_cli.txt
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_feature_projection.txt
```

## Required invariants (`tests/invariants.rs`)

| # | Invariant |
|---|----------|
| 1 | CLI binary reachable as `cargo cicd` |
| 2 | `status show` exits 0, emits structured output |
| 3 | Unknown noun returns non-zero exit |
| 4 | `--help` on every noun exits 0 |
| 5 | No forbidden terms in any `--help` output |
| 6 | `workspace doctor` exits 0 in valid workspace |
| 7 | `target show` exits 0, lists ≥1 target |

## Failure remediation

| Invariant | Fix |
|-----------|-----|
| 1 | Confirm binary at `target/debug/cargo-cicd`; check `.cargo/config.toml` alias |
| 2, 6, 7 | Run command manually; check for panic/unwrap in `src/nouns/<noun>.rs`; inspect `cicd.toml` |
| 3 | Check `main.rs` error propagation from clap-noun-verb |
| 4, 5 | Run `cargo cicd <noun> --help`; grep `src/nouns/<noun>.rs` for forbidden terms |
| feature_projection | Add missing `#[cfg(feature = "...")]` guard to leaking symbol |

Forbidden terms: `ALIVE` `Nehemiah` `CONSTRUCT8` `Instinct8` `Inspection Gate` `Cargo Court` `AGI` `Truex` `Field8` `wall`

## Verdict

- All pass → **public boundary intact**
- Any fail → **list failures; block release gate**
