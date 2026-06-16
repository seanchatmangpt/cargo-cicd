---
description: Run the public-boundary invariant suites and report which hold, with remediation tips on failure.
allowed-tools: Bash, Read, Grep
---

You are verifying that cargo-cicd's public boundaries are intact. Run the three invariant test suites, parse the results, and produce a structured report.

---

## Step 1 — Run the invariant suites

Run all three suites together so failures in one do not suppress the others:

```bash
cargo test --test invariants 2>&1 | tee /tmp/ccicd_invariants.txt
cargo test --test cli       2>&1 | tee /tmp/ccicd_cli.txt
cargo test --test feature_projection 2>&1 | tee /tmp/ccicd_feature_projection.txt
```

Capture the full output of each.

---

## Step 2 — Parse results

For each suite, extract the summary line (e.g. `test result: ok. 12 passed; 0 failed`) and list every `FAILED` test by name.

```bash
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_invariants.txt
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_cli.txt
grep -E '^test .* \.\.\. (FAILED|ok)' /tmp/ccicd_feature_projection.txt
```

---

## Step 3 — Report invariant status

Print a table with one row per invariant test. The 7 non-negotiable public boundary invariants defined in `tests/invariants.rs` are:

| # | Invariant | Status |
|---|-----------|--------|
| 1 | CLI binary is reachable as `cargo cicd` | … |
| 2 | `status show` exits 0 and emits structured output | … |
| 3 | Unknown noun returns non-zero exit code | … |
| 4 | `--help` on every noun exits 0 | … |
| 5 | No forbidden terms in any `--help` output | … |
| 6 | `workspace doctor` exits 0 in a valid workspace | … |
| 7 | `target show` exits 0 and lists at least one target | … |

Fill the Status column from the test output (PASS / FAIL / NOT RUN).

For the `cli` suite, list any noun/verb combinations that failed their smoke test.

For the `feature_projection` suite, note which feature-flag surface contracts were violated (e.g. a symbol present under `autonomic` that leaked into the default build).

---

## Step 4 — Remediation tips

For each FAILED test, provide a concrete remediation path:

**Invariant 1 (binary unreachable)**
- Check `cargo build` succeeded and the binary is in `target/debug/cargo-cicd`.
- Confirm `.cargo/config.toml` registers `cicd` as an alias or the binary name is correct.

**Invariant 2 / 6 / 7 (command exits non-zero)**
- Run the failing command manually: `cargo cicd status show`, `cargo cicd workspace doctor`, `cargo cicd target show`.
- Check `src/nouns/<noun>.rs` for a panic or unwrap that aborts on an empty workspace.
- Inspect `cicd.toml` — a missing or corrupt file can cause an early exit.

**Invariant 3 (unknown noun does not return non-zero)**
- Check `main.rs` error handling; `clap-noun-verb` should propagate unknown-noun errors automatically.

**Invariant 4 / 5 (`--help` failures)**
- Run `cargo cicd <noun> --help` for the failing noun and look for a panic or forbidden term.
- Search `src/nouns/<noun>.rs` for any hardcoded string that contains a forbidden term.

**feature_projection failures**
- A feature-flag surface leak means a `#[cfg(feature = "...")]` guard is missing.
- Find the leaking symbol with `grep -rn '<symbol>' src/` and wrap it in the correct `#[cfg(...)]`.

---

## Step 5 — Overall verdict

State:
- **All invariants hold** — public boundary is intact; safe to continue.
- **N invariant(s) failed** — list them; these must be fixed before any release gate is run.
