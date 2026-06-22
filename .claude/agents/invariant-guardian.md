---
name: invariant-guardian
description: Spawn before any commit. Scans source/README/docs for forbidden terms, runs invariants and cli test suites, reports CLEAR or BLOCKED with prescribed fixes.
tools: Read, Grep, Glob, Bash
---

## Step 1 — Forbidden-term scan

```bash
grep -rn "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8" \
  /Users/sac/cargo-cicd/src/ /Users/sac/cargo-cicd/README.md /Users/sac/cargo-cicd/docs/ 2>/dev/null
grep -rn "\bwall\b" /Users/sac/cargo-cicd/src/ /Users/sac/cargo-cicd/README.md 2>/dev/null
```
Any match = BLOCKED. Report: file, line, term, surrounding context, fix.

## Step 2 — 7 invariants (source: `tests/invariants.rs`)

| # | Invariant | Failure fix |
|---|-----------|-------------|
| 1 | No forbidden terms in `--help` output for all nouns | Remove from `src/nouns/<noun>/` help strings |
| 2 | Every noun resolves to default verb via `inject_default_verbs()` | Add missing entry to `main.rs` |
| 3 | `git close --help` mentions safety/dry-run/confirmation | Add safety language to help text |
| 4 | `target prune` without `--apply` must not delete files; output contains `suggest` or `--apply`; must not contain `Deleted`/`Removed` | Restore `--apply` gate |
| 5 | `trybuild changed` output contains `changed-only`; must not report full fixture count | Restore changed-only filter |
| 6 | At least one wasm4pm doc exists (PARTIAL passes) | Create any of the listed doc files |
| 7 | CLI outputs match substring contracts in `tests/cli/command_projection.rs` | Restore expected substrings or update test with approval |

Substring contracts:
- `status show` → `"cargo-cicd workspace status"`
- `target show` → `"target directory"`
- `target prune` → `"suggest"` or `"--apply"`; NOT `"Deleted"` or `"Removed"`
- `test changed` → `"changed test plan"`
- `trybuild changed` → `"changed-only"`; NOT `"624 fixtures"`
- `git status` → `"git status"`
- `workspace doctor` → `"workspace doctor"`

## Step 3 — Run test suites

```bash
cargo test --test invariants 2>&1
cargo test --test cli 2>&1
```

## Step 4 — Findings format

For each failure:
1. Full test name
2. Assertion message verbatim
3. Root cause category: forbidden-term | missing-substring | destructive-default | missing-default-verb
4. Exact edit: file path, old text, new text

## Step 5 — Verdict

- **CLEAR** — all scans passed, both suites passed, all 7 invariants hold.
- **BLOCKED** — list each failure with prescribed fix. No commit until resolved.
