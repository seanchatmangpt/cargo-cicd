# DAY3_FRUIT_CANDIDATES — cargo-cicd v26.6.2

**Date:** 2026-06-03
**Re-verified:** 2026-06-02 (Day 3 synthesis agent, git HEAD 00d29c2)
**Purpose:** Rank Day 3 work candidates by readiness, impact, and scope so the highest-value, lowest-risk item is selected first.

---

## FruitScore Formula

```
FruitScore = (impact * proof_readiness * user_visibility) / (risk * scope)
```

All dimensions scored 1–5:

| Dimension | Meaning |
|---|---|
| impact | How much does completing this improve the system? |
| proof_readiness | How close is the existing code to done? |
| user_visibility | How visible is the result to a user? |
| risk | How likely is this to break something or encounter unknown blockers? |
| scope | How large is the change? |

Higher FruitScore = more fruit, less work, lower risk.

---

## Candidate Table

| Rank | Candidate | Impact | Proof Readiness | User Visibility | Risk | Scope | FruitScore | Status |
|---|---|---|---|---|---|---|---|---|
| 1 | LSP editor diagnostics proof | 3 | 4 | 4 | 2 | 2 | **12.0** | PARTIAL — clap wiring gap only |
| 2 | CICD-WPM-004 + regression fixture | 4 | 3 | 3 | 2 | 3 | **6.0** | PARTIAL — catalog present; runtime_court.rs not wired |
| 3 | Publish gate as adjudicated receipt | 4 | 3 | 4 | 3 | 3 | **5.33** | PARTIAL — fixture present; no test; no receipt schema |
| 4 | Spec Kit integration | 3 | 1 | 3 | 2 | 5 | **0.9** | NOT STARTED |
| 5 | LSP fixture coverage expansion | 2 | 4 | 2 | 1 | 3 | **5.33** | PARTIAL — 13 codes lack fixture tests |

---

## Candidate Details

### 1. LSP editor diagnostics proof — FruitScore 12.0

**Description:** Prove `cargo cicd lsp explain` works end-to-end. Wire the `code` positional arg through clap-noun-verb 26.6.2 `build_command()`. Confirm CICD-GIT-001 and at least 7 catalog codes are reachable. Produce JSON proof of an initialize response and one diagnostic.

**Status:** PARTIAL — run logic implemented; positional arg not wired through `build_command()`.

**Enabling surfaces:**
- `lsp.rs` `additional_args()` already declares `clap::Arg::new("code")`
- CICD_CATALOG has 22 entries including CICD-WPM-004
- `explain_diagnostic_code()` helper covers at least 7 codes

**Blocking gaps:**
- `build_command()` in clap-noun-verb 26.6.2 does not forward `additional_args()` positional args
- `code` arg is unreachable at runtime without that wiring

**First step:** Locate `build_command()` in clap-noun-verb 26.6.2 crate and add positional arg forwarding from `additional_args()`; verify with `cargo cicd lsp explain CICD-GIT-001` producing JSON receipt.

---

### 2. CICD-WPM-004 + regression fixture — FruitScore 6.0

**Description:** Wire `verdict_key_mismatch` (CICD-WPM-004) into `analyzers/runtime_court.rs` so it fires when `overall_fitness` key is absent or misnamed in wpm audit output. Add regression fixture proving the diagnostic is emitted correctly.

**Status:** PARTIAL — catalog entry present; `analyzers/runtime_court.rs` WPM-004 not wired; rendered_surface fixture present but no test file.

**Enabling surfaces:**
- CICD-WPM-004 catalog entry at `lsp.rs:169`
- wpm binary present at `/Users/sac/wasm4pm/target/release/wpm` (version 26.5.29)
- `audit_key_regression_protected=true` implies fitness key shape is known

**Blocking gaps:**
- `runtime_court.rs` does not emit CICD-WPM-004 on `verdict_key_mismatch`
- No test file for rendered_surface fixture
- wpm not in PATH (WPM_PATH env var or known scan path required for CI)

**First step:** Open `analyzers/runtime_court.rs`, add WPM-004 emission branch on `verdict_key_mismatch`, write regression test using the existing rendered_surface fixture, confirm with `cargo test` on that fixture.

---

### 3. Publish gate as adjudicated receipt — FruitScore 5.33

**Description:** Replace the boolean cicd.toml publish gate with a receipt that carries wpm verdict + commit hash + timestamp. `cargo publish` readiness is adjudicated by wpm receipt doctor judgment, not a static flag. The receipt must be emitted, persisted, and replayable.

**Status:** PARTIAL — `analyzers/publish.rs` fixture present but no test file; wpm binary present but not in PATH.

**Enabling surfaces:**
- wpm binary at `/Users/sac/wasm4pm/target/release/wpm` (26.5.29)
- `Wasm4pmShell` shell-out pattern is established
- `analyzers/publish.rs` fixture present

**Blocking gaps:**
- No test file for publish analyzer
- Receipt schema for publish gate not defined
- wpm receipt doctor `--strict` output contract not documented in cargo-cicd

**First step:** Define the publish receipt struct (`wpm_verdict`, `commit_hash`, `timestamp`, `fitness_at_adjudication`), wire it into `analyzers/publish.rs`, write a test against the existing fixture, confirm receipt is emitted to evidence dir.

---

### 4. Spec Kit integration — FruitScore 0.9

**Description:** Add `.specify` constitution + first spec + task-to-evidence trace mapping. `speckit_present=false`. CICD-SPEC-002 catalog entry exists as forward declaration only. Entirely greenfield.

**Status:** NOT STARTED.

**Blocking gaps:**
- No `.specify` constitution format defined
- No spec noun/verb in CLI
- No task-to-evidence trace mapping schema
- No test fixtures
- Entirely greenfield — no existing code to leverage

**First step:** Define the `.specify/constitution.toml` schema (project name, spec format version, required fields), write a parsing module, add `cargo cicd spec show` verb that reads it and emits an evidence event.

---

### 5. Conformance 1.0 feedback closure — FruitScore 0.625

**Description:** Resolve the VARIANCE between internal oracle (fitness 0.9636, TRUTHFUL) and external wpm audit on the same events.xes (fitness 0.8194, VARIANCE). Identify whether the discrepancy is a model mismatch, a XES filter difference, or a replay algorithm difference.

**Status:** VARIANCE — oracle and wpm audit disagree on same XES; root cause unknown.

**Blocking gaps:**
- Root cause of 0.9636 vs 0.8194 gap is uninvestigated
- Precision metric absent (null, not documented)
- No closed-loop model feedback plan exists

**First step:** Run wpm audit on events.xes with verbose trace output; compare the 1 deviating trace (oracle) against the 2 missing traces (wpm) to identify which activities differ between oracle model and wpm replay model.

---

## Recommendation

**Day 3 primary target: LSP editor diagnostics proof (FruitScore 12.0)**

The run logic and CICD_CATALOG lookup are already implemented. The only gap is wiring the `code` positional arg through `build_command()` — a bounded, local fix with no external dependencies, no schema changes, no binary deps, and high user visibility (makes 22 catalog codes externally usable via CLI). Proof is a single JSON receipt from `cargo cicd lsp explain CICD-GIT-001`.

CICD-WPM-004 (6.0) and Publish gate (5.3) are next-tier but both require additional setup (PATH resolution or new receipt schema). Conformance closure (0.625) and Spec Kit (0.9) are low-readiness and should not be Day 3 targets.
