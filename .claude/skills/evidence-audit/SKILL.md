---
name: evidence-audit
description: Explains and runs the process-evidence adjudication pipeline for cargo-cicd. Checks that XES event files exist under target/cargo-cicd/evidence/, invokes the receipt doctor via `cargo cicd evidence doctor`, runs `cargo cicd status audit`, and interprets the Accept or Refuse verdict from the wpm oracle. Notes that a missing wpm oracle exits non-zero with a BLOCKED diagnostic (expected in local dev environments). Use when the user says "audit evidence", "check receipts", "adjudicate", or asks whether process evidence was accepted.
---

# Evidence Audit

Trigger: user says "audit evidence", "check receipts", "adjudicate", or asks if process evidence was accepted.

Evidence format: OCEL 2.0 JSON (`.ocel.json`). XES (`.xes`) is legacy — do not extend. wpm shell-out only; never linked.

## Step 1 — Confirm Evidence Files Exist

```sh
ls target/cargo-cicd/evidence/
```

- Empty or absent → report **BLOCKED — no evidence files found**. Run `cargo cicd status show` to emit evidence, then re-audit.
- Files present → list paths and sizes, proceed.

## Step 2 — Receipt Doctor

```sh
cargo cicd evidence doctor
# internally: wpm receipt doctor --format json --strict <receipt.json>
```

| Output | Action |
|--------|--------|
| `"verdict": "Accept"` | Proceed to Step 3 |
| `"verdict": "Refuse"` | Show `reasons` array. Stop. |
| Non-zero + `BLOCKED` / `oracle unavailable` | wpm absent at `$(which wpm)`. Expected locally. Note and continue. |

## Step 3 — Status Audit

```sh
cargo cicd status audit
# internally: wpm audit <file.ocel.json> per file
```

| Output | Action |
|--------|--------|
| All files `OK` / `Accept` | Proceed to verdict |
| Any `Refuse` | Show file path + error detail. Stop. |
| Non-zero + `BLOCKED` | Same as Step 2 oracle-absent case. Note and continue. |

## Step 4 — Combined Verdict

| Condition | Verdict |
|-----------|---------|
| Both steps Accept | **PASSED** |
| Either Refuse | **FAILED** — fix `src/evidence.rs` or receipt schema, re-emit, re-audit |
| Either BLOCKED / oracle unavailable | **DEFERRED (local)** — CI must have wpm; local non-fatal |
| Evidence dir empty | **BLOCKED** — emit evidence first |

## Failure Remediation

- **Refuse / missing attributes**: fix `src/evidence.rs`, re-run emitting command, re-audit.
- **Timestamp order violation**: check `ProcessEvent::started` / `ProcessEvent::completed` call ordering.
- **Receipt schema mismatch**: verify `wasm4pm_compat::receipt::Receipt` fields match wpm expectation.
- **Oracle absent**: `which wpm` must resolve; build wasm4pm-cli if needed.

Invariant E1: cargo-cicd never adjudicates itself. Only wpm issues verdicts.
