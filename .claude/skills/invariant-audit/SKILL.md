---
name: invariant-audit
description: Verifies the public-boundary invariants and output-substring contracts for cargo-cicd. Greps source, README, and docs for forbidden terms, runs the invariants/cli/feature_projection test suites, and confirms that each command prints the required output substrings. Use when the user says "check invariants", "public boundary audit", "verify contracts", or "make sure the CLI output is correct" before a release or after changing a noun module.
---

# Invariant Audit

Trigger: "check invariants", "public boundary audit", "verify contracts", or "make sure the CLI output is correct".

## Step 1 — Forbidden Term Scan

Zero matches required across `src/`, `README.md`, `docs/`.

```sh
grep -rn -e "ALIVE" -e "Inspection Gate" -e "\bwall\b" -e "Nehemiah" \
     -e "Field8" -e "Instinct8" -e "Cargo Court" -e "\bAGI\b" \
     -e "Truex" -e "CONSTRUCT8" src/ README.md docs/ 2>/dev/null
```

Any match → **NO-GO**. Remove term; re-run to confirm zero.

## Step 2 — Run Test Suites

```sh
cargo test --test invariants
cargo test --test cli
cargo test --test feature_projection
```

All must exit 0. On failure: read failure output, fix `src/nouns/*.rs` (do not loosen test contracts), re-run.

## Step 3 — Output-Substring Contracts

```sh
cargo cicd status show 2>&1    | grep -q "cargo-cicd workspace status" && echo PASS || echo FAIL
cargo cicd target show 2>&1    | grep -q "target directory"            && echo PASS || echo FAIL
cargo cicd target show 2>&1    | grep -q "GB"                          && echo PASS || echo FAIL
cargo cicd workspace doctor 2>&1 | grep -q "workspace doctor"          && echo PASS || echo FAIL
cargo cicd workspace doctor 2>&1 | grep -q "Cargo.toml"               && echo PASS || echo FAIL
cargo cicd git status 2>&1     | grep -q "git status"                 && echo PASS || echo FAIL
```

Any FAIL → noun output changed. Fix output string in `src/nouns/<noun>.rs`.

## Step 4 — Plain-Mode (Piped) Check

```sh
cargo cicd status show     | cat | grep -P "\x1b" && echo FAIL || echo PASS
cargo cicd target show     | cat | grep -P "\x1b" && echo FAIL || echo PASS
cargo cicd workspace doctor | cat | grep -P "\x1b" && echo FAIL || echo PASS
```

Any FAIL → replace hard-coded `\x1b[...m` with `Style::paint` / `theme::paint` calls.

## Step 5 — Feature Flag Surface

```sh
cargo test --test feature_projection
```

Failure → fix `#[cfg(feature = "...")]` gate in affected source file per `tests/feature_projection.rs`.

## Summary Report Format

```
Forbidden terms:       PASS / FAIL
invariants suite:      PASS / FAIL
cli suite:             PASS / FAIL
feature_projection:    PASS / FAIL
status show contract:  PASS / FAIL
target show contract:  PASS / FAIL
workspace contract:    PASS / FAIL
git status contract:   PASS / FAIL
plain-mode output:     PASS / FAIL
```

For each FAIL: state file + line, fix applied, re-run confirmation.

## Reference Files

- `tests/invariants.rs` — 7 public boundary invariants
- `tests/cli/command_projection.rs` — per-noun projection tests
- `tests/feature_projection.rs` — feature flag surface contract
- `src/ui/caps.rs` — `Style::paint` reads color/unicode capability here
