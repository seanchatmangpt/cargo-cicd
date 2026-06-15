---
description: Drive a release: lint, test, surface scan, workspace checks, git cleanliness, go/no-go summary.
argument-hint: [version]
allowed-tools: Bash, Read, Grep
---

You are running a pre-release gate for the cargo-cicd workspace. The target version is: **$ARGUMENTS**

Work through the checklist below in order. After every step record PASS, FAIL, or WARN and the relevant output. At the end, print a go/no-go table and a one-sentence recommendation.

---

## Step 1 — Lint and type-check

Run the full lint and type-check pipeline:

```
cargo make check
```

If this fails, the release is blocked. Record the first error line.

---

## Step 2 — Full test suite

Run all tests:

```
cargo make test
```

Record the summary line (`test result: ok. X passed; Y failed`). Any failure blocks the release.

---

## Step 3 — Forbidden-term surface scan

Read `CLAUDE.md` under the section "FORBIDDEN in public docs/CLI/help text" to get the exact list of terms that must never appear in public-facing output.

Then search every `.rs`, `.toml`, and `.md` file under `src/`, `tests/`, and `.claude/` using the Grep tool. For each forbidden term, run a case-sensitive search. Report every match with its file path and line number.

Any hit is a blocker — forbidden terms in public surface files must be removed before release.

---

## Step 4 — Workspace doctor

Run the workspace health check:

```
cargo cicd workspace doctor
```

Read the output and flag any ERROR-level diagnostics. WARN-level items are noted but do not block.

---

## Step 5 — Status check

Run:

```
cargo cicd status show
```

Note the reported workspace state and whether any dimension is degraded.

---

## Step 6 — Clean git tree

Run:

```
cargo cicd git status
```

The release requires a completely clean working tree — no modified, untracked, or staged files. If any dirty paths appear, list them and mark the step FAIL.

If `$ARGUMENTS` was supplied, also confirm the HEAD commit matches the expected release tag:

```bash
git tag --list "v$ARGUMENTS" 2>/dev/null || echo "tag not yet created"
```

The commit message at HEAD must match the format: `feat(core|cli|target|test|git|autonomic|docs|receipts): description`.

---

## Step 7 — Evidence directory sanity

Check that `target/cargo-cicd/evidence/` exists and contains at least one `.xes` file from the current session:

```bash
ls -1 target/cargo-cicd/evidence/*.xes 2>/dev/null | tail -5 || echo "no evidence files found"
```

A missing evidence directory is a WARN (first run may not have emitted yet); missing files after a full test run is a FAIL.

---

## Final summary

Print a markdown table:

| Step | Check | Result |
|------|-------|--------|
| 1 | `cargo make check` | … |
| 2 | `cargo make test` | … |
| 3 | Forbidden-term scan | … |
| 4 | `workspace doctor` | … |
| 5 | `status show` | … |
| 6 | Clean git tree | … |
| 7 | Evidence directory | … |

Then state one of:
- **GO** — all steps PASS; safe to tag and publish.
- **NO-GO** — list the blocking steps and what must be fixed before retrying.
