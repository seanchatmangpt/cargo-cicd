---
name: release-checklist
description: Runs a thorough pre-release gate for cargo-cicd: lint, type-check, full test suite, forbidden-term scan across source and docs, workspace doctor, status check, clean working tree verification, and commit-message format validation. Produces a go/no-go summary table. Use when the user says "release", "ship", "pre-release checklist", or asks whether the workspace is ready to publish.
---

# Release Checklist

Trigger: "release", "ship", "pre-release checklist", or "is the workspace ready to publish".

Run all steps in order. Any NO-GO stops the release.

## Step 1 — Lint and Type-Check

```sh
cargo make check
```

Non-zero exit → surface diagnostics → **NO-GO**.

## Step 2 — Full Test Suite

```sh
cargo test
cargo test --test invariants
cargo test --test cli
cargo test --test feature_projection
cargo build --features autonomic,wasm4pm
cargo test --test wasm4pm_evidence_gate
```

Any failure → show failure output → **NO-GO**.

## Step 3 — Forbidden-Term Scan

```sh
grep -rn -e "ALIVE" -e "Inspection Gate" -e "\bwall\b" -e "Nehemiah" \
     -e "Field8" -e "Instinct8" -e "Cargo Court" -e "\bAGI\b" \
     -e "Truex" -e "CONSTRUCT8" src/ README.md docs/ 2>/dev/null
```

Any match → **NO-GO** (file + line required).

## Step 4 — Workspace Doctor

```sh
cargo cicd workspace doctor
```

ERROR lines → **NO-GO**. WARNING lines → note only.

## Step 5 — Status Check

```sh
cargo cicd status show
```

Confirm: no blockers, toolchain resolved, targets valid.

## Step 6 — Clean Working Tree

```sh
git status
```

Staged or uncommitted changes → **NO-GO**.

## Step 7 — Receipt Doctor

```sh
wpm receipt doctor --format json --strict receipts/*.json
```

Any `"verdict": "Refuse"` → **NO-GO**.

## Step 8 — Commit Message Format

HEAD commit must match:
```
feat|fix|docs|test|chore(core|cli|target|test|git|autonomic|docs|receipts): <description>
```

Mismatch → **NO-GO** (show actual message).

## Summary Table

| # | Gate | Status | Notes |
|---|------|--------|-------|
| 1 | `cargo make check` | GO / NO-GO | |
| 2 | Test suites (invariants, cli, feature_projection, wasm4pm_evidence_gate) | GO / NO-GO | |
| 3 | Forbidden-term scan | GO / NO-GO | List hits |
| 4 | `cargo cicd workspace doctor` | GO / NO-GO | List errors |
| 5 | `cargo cicd status show` | GO / NO-GO | |
| 6 | Clean working tree | GO / NO-GO | |
| 7 | Receipt doctor | GO / NO-GO | |
| 8 | Commit message format | GO / NO-GO | Show actual if wrong |

**Final verdict:** GO (all rows green) or NO-GO (any row red).
