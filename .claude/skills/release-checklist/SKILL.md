---
name: release-checklist
description: Runs a thorough pre-release gate for cargo-cicd: lint, type-check, full test suite, forbidden-term scan across source and docs, workspace doctor, status check, clean working tree verification, and commit-message format validation. Produces a go/no-go summary table. Use when the user says "release", "ship", "pre-release checklist", or asks whether the workspace is ready to publish.
---

# Release Checklist — cargo-cicd Pre-Release Gate

Work through every step in order. Do not mark the gate as **GO** unless all steps pass. Report each step's result inline, then produce a final summary table.

---

## Step 1 — Lint and Type-Check

Run the check step. This must exit zero before anything else is meaningful.

```
cargo make check
```

If it fails: surface the compiler diagnostics, stop, and report **NO-GO**.

---

## Step 2 — Full Test Suite

Run all tests. Pay special attention to the three mandatory integration suites.

```
cargo test
```

Key suites to call out explicitly in the report:

- `invariants` — the seven non-negotiable public-boundary invariants
- `cli` — noun/verb surface contract
- `feature_projection` — feature-flag surface contract

If any suite fails: show the failure output, stop, and report **NO-GO**.

---

## Step 3 — Forbidden-Term Scan

Scan `src/`, `README.md`, and `docs/` for terms that must never appear in the public boundary. The definitive list of forbidden terms is in the **FORBIDDEN in public docs/CLI/help text** section of `/home/user/cargo-cicd/CLAUDE.md`. Read that section now and use each listed term as a search pattern.

Use the Grep tool with `output_mode: "content"` across `src/`, `README.md`, and `docs/`, running one search per term. Any hit — regardless of context, comments, or strings — is a **NO-GO**. Note the file path and line number for each match found.

---

## Step 4 — Workspace Doctor

Run the workspace health check to validate the workspace state and surface any structural issues.

```
cargo cicd workspace doctor
```

Review the output for any ERROR or WARNING lines. Warnings are noted; errors are **NO-GO**.

---

## Step 5 — Status Check

Display the current engine state summary.

```
cargo cicd status show
```

Confirm the reported state is consistent with a clean, releasable workspace (no pending changes flagged as blockers, toolchain resolved, targets valid).

---

## Step 6 — Clean Working Tree

Verify the working tree has no uncommitted changes and no untracked files that belong to the release.

Read the output of `git status` (via the Bash tool or by reading `.git/` state if available). The working tree and index must be clean. Staged but uncommitted changes are a **NO-GO**.

---

## Step 7 — Commit Message Format

Inspect the most recent commit message. It must match the project commit format:

```
feat(core|cli|target|test|git|autonomic|docs|receipts): <description>
```

Accepted scope tokens: `core`, `cli`, `target`, `test`, `git`, `autonomic`, `docs`, `receipts`.

If the HEAD commit message does not match this pattern, flag it as **NO-GO** with the actual message shown.

---

## Step 8 — Go / No-Go Summary Table

After all steps, produce a table:

| # | Gate | Status | Notes |
|---|------|--------|-------|
| 1 | `cargo make check` | GO / NO-GO | |
| 2 | Full test suite (`invariants`, `cli`, `feature_projection`) | GO / NO-GO | |
| 3 | Forbidden-term scan | GO / NO-GO | List any hits |
| 4 | `cargo cicd workspace doctor` | GO / NO-GO | List any errors |
| 5 | `cargo cicd status show` | GO / NO-GO | |
| 6 | Clean working tree | GO / NO-GO | |
| 7 | Commit message format | GO / NO-GO | Show actual message if wrong |

**Final verdict:** GO (all rows green) or NO-GO (any row red).
