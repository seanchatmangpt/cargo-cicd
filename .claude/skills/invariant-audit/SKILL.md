---
name: invariant-audit
description: Verifies the public-boundary invariants and output-substring contracts for cargo-cicd. Greps source, README, and docs for forbidden terms, runs the invariants/cli/feature_projection test suites, and confirms that each command prints the required output substrings. Use when the user says "check invariants", "public boundary audit", "verify contracts", or "make sure the CLI output is correct" before a release or after changing a noun module.
---

# Invariant Audit

Follow these steps in order to audit cargo-cicd's public-boundary invariants.

## Step 1 — Scan for forbidden terms

These terms must never appear in `src/`, `README.md`, or `docs/`. Run each grep and confirm zero matches.

```sh
# Grep source files
grep -r "ALIVE" src/ README.md docs/ 2>/dev/null || true
grep -r "Inspection Gate" src/ README.md docs/ 2>/dev/null || true
grep -r "\bwall\b" src/ README.md docs/ 2>/dev/null || true
grep -r "Nehemiah" src/ README.md docs/ 2>/dev/null || true
grep -r "Field8" src/ README.md docs/ 2>/dev/null || true
grep -r "Instinct8" src/ README.md docs/ 2>/dev/null || true
grep -r "Cargo Court" src/ README.md docs/ 2>/dev/null || true
grep -r "\bAGI\b" src/ README.md docs/ 2>/dev/null || true
grep -r "Truex" src/ README.md docs/ 2>/dev/null || true
grep -r "CONSTRUCT8" src/ README.md docs/ 2>/dev/null || true
```

If any match is found: locate the file and line, remove or replace the term with public-safe language, and re-run the grep to confirm zero matches before continuing.

## Step 2 — Run the three invariant test suites

```sh
cargo test --test invariants
cargo test --test cli
cargo test --test feature_projection
```

All three must exit 0. If any test fails:
1. Read the failure output carefully — it will name the specific assertion and command.
2. Identify whether the breakage is in source (`src/nouns/*.rs`) or in the test fixture.
3. Fix the source; do not loosen the test contract without team approval.

## Step 3 — Verify output-substring contracts

The following contracts are enforced by `tests/cli/command_projection.rs` and `tests/invariants.rs`. Verify each manually after any noun output change:

| Command | Required output substring(s) |
|---|---|
| `cargo cicd status show` | `"cargo-cicd workspace status"` |
| `cargo cicd target show` | `"target directory"` and `"GB"` |
| `cargo cicd workspace doctor` | `"workspace doctor"` and `"Cargo.toml"` |
| `cargo cicd git status` | `"git status"` |
| `cargo cicd evidence doctor` | verdict line from `wpm receipt doctor` |

Run each command in a subshell and confirm the substring is present:

```sh
cargo cicd status show 2>&1 | grep -q "cargo-cicd workspace status" && echo "PASS" || echo "FAIL"
cargo cicd target show 2>&1 | grep -q "target directory" && echo "PASS" || echo "FAIL"
cargo cicd target show 2>&1 | grep -q "GB" && echo "PASS" || echo "FAIL"
cargo cicd workspace doctor 2>&1 | grep -q "workspace doctor" && echo "PASS" || echo "FAIL"
cargo cicd workspace doctor 2>&1 | grep -q "Cargo.toml" && echo "PASS" || echo "FAIL"
cargo cicd git status 2>&1 | grep -q "git status" && echo "PASS" || echo "FAIL"
```

## Step 4 — Check plain-mode output (piped)

Color must be absent and all glyphs must be ASCII when output is piped. Run:

```sh
cargo cicd status show | cat | grep -P "\\x1b" && echo "FAIL: ANSI escape in plain mode" || echo "PASS: plain mode clean"
cargo cicd target show | cat | grep -P "\\x1b" && echo "FAIL: ANSI escape in plain mode" || echo "PASS: plain mode clean"
cargo cicd workspace doctor | cat | grep -P "\\x1b" && echo "FAIL: ANSI escape in plain mode" || echo "PASS: plain mode clean"
```

If ANSI escapes appear in piped output, the offending noun is using raw escape strings instead of `crate::ui::style::Style::paint()` or `crate::ui::theme::paint()`. Fix by replacing any hard-coded `\x1b[...m` sequences with `Style::paint` calls.

## Step 5 — Feature flag surface contract

Run the feature-projection test to confirm that gated APIs are not accidentally exposed in the default build:

```sh
cargo test --test feature_projection
```

If the test fails, read `tests/feature_projection.rs` for the exact surface contract that was violated, and fix the feature-gate (`#[cfg(feature = "...")]`) in the affected source file.

## Step 6 — Report and fix

After completing all steps, produce a brief report:

```
Forbidden terms: PASS / FAIL (list any matches)
invariants suite: PASS / FAIL
cli suite:        PASS / FAIL
feature_projection: PASS / FAIL
status show contract: PASS / FAIL
target show contract: PASS / FAIL
workspace doctor contract: PASS / FAIL
git status contract: PASS / FAIL
plain-mode output: PASS / FAIL
```

For each FAIL, state the file and line, the fix applied, and re-run the affected test or grep to confirm the fix.

## Reference: test files

- `tests/invariants.rs` — 7 non-negotiable public boundary invariants.
- `tests/cli/command_projection.rs` — per-noun projection tests asserting parse + output.
- `tests/feature_projection.rs` — feature flag surface contract.
- `src/nouns/*.rs` — noun implementations; output must use only `crate::ui` primitives.
- `src/ui/caps.rs` — color/unicode capability detection; `Style::paint` reads this.
