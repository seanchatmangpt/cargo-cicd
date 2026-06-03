# Receipt: Runtime Evidence Reconciliation — cargo-cicd v26.6.2

**Date:** 2026-06-02
**Verdict: BLOCKED**
**Receipt type:** Release gate reconciliation

---

## Summary Table

| Gate | Status | Notes |
|---|---|---|
| Build | PASS | cargo-cicd v26.6.2 compiled successfully (3.35s) |
| Runtime timestamp integrity | BLOCKED | `ProcessEvent::new()` in `src/evidence.rs:51` hardcodes `timestamp_iso` to `"2026-06-02T00:00:00.000Z"` in the production runtime constructor — not a test |
| wpm audit verdict | BLOCKED | `wpm` binary not found; audit verdict is UNAVAILABLE, not ACCEPTED |
| Runtime audit subcommand | BLOCKED | No `audit` subcommand exists at runtime; evidence adjudication is test-only |
| Evidence lifecycle pairs | FAIL | Only `complete` transitions emitted; no `start` events; lifecycle pairs absent |
| Session/case grouping | PARTIAL | No `trace_id` or `case_id` links multi-command invocations together |
| Evidence file coverage | FAIL | Only the final command (`git status`) produced an XES event; `status` and `target-show` emitted no evidence |
| Publish gate adjudication | FAIL | `src/nouns/publish.rs` self-assigns `verdict = "pass"` without consulting `WpmEvidenceOracle`; wasm4pm integration deferred to v26.6.3+ |
| Release receipt completeness | PARTIAL | Receipt carries `PARTIAL` (not `PUBLISH_READY`); uncommitted working-tree changes and exit-0 re-verification pending |

---

## Defects (BLOCKED Conditions)

### DEFECT-1: Hardcoded timestamp in production runtime constructor

**Location:** `src/evidence.rs:51`

`ProcessEvent::new()` hardcodes `timestamp_iso` to `"2026-06-02T00:00:00.000Z"` in the runtime production constructor.
Every event emitted through the nouns layer (trybuild, test, git, target) carries this fake timestamp.
XES logs emitted by cargo-cicd contain non-temporal evidence.
A process mining conformance check cannot distinguish ordering or measure inter-event duration from the primary emitter path.

Acceptable occurrences (not defects):
- `src/integrations/wasm4pm_exchange.rs:605` — hardcoded timestamp in `#[cfg(test)]` block (test-only)
- `src/integrations/wasm4pm_exchange.rs:500` — fallback epoch `"1970-01-01T00:00:00Z"` on `date` command failure (fallback, not primary path)

### DEFECT-2: wpm audit verdict is UNAVAILABLE

**Location:** runtime audit pipeline

`wpm` binary not found. Audit verdict is `UNAVAILABLE`, not `ACCEPTED`. No runtime audit subcommand exists.

### DEFECT-3: No runtime audit subcommand

**Location:** cargo-cicd command surface

cargo-cicd exposes no `audit` subcommand at runtime. Evidence adjudication is test-only. No path exists for a release pipeline to obtain an adjudicated verdict from wasm4pm at runtime.

---

## Partials

### PARTIAL-1: Evidence traces not grouped by session/case

No `trace_id` or `case_id` field links multi-command invocations. Three commands were invoked but the XES log contains no structure grouping them as a single session or case.

---

## Required Remediation Steps

### DEFECT-1

1. Replace the hardcoded `timestamp_iso` literal in `ProcessEvent::new()` (`src/evidence.rs:51`) with a real wall-clock call — use `now_rfc3339()` already present in `src/integrations/wasm4pm_exchange.rs:490-501` or introduce a `SystemTime`-based implementation.
2. Verify emitted XES events carry real wall-clock timestamps at sub-second precision.
3. Confirm timestamps differ between sequential events in the runtime evidence test.

### DEFECT-2 and DEFECT-3

1. Ensure `wpm` binary is present and on PATH before any publish gate runs.
2. Implement a `cargo cicd audit` subcommand that invokes `WpmEvidenceOracle` against the emitted evidence file and records the adjudicated verdict.
3. `src/nouns/publish.rs` must call `WpmEvidenceOracle` before writing `verdict = "pass"` into `cicd.toml`.
4. Gate `publish run` on an `ACCEPTED` oracle verdict.

### PARTIAL-1

1. Assign a `trace_id` at session start and propagate it through all events in that invocation.
2. Use a stable `case_id` to group related command invocations into a single XES trace.

---

## Condition for PUBLISH_READY

This release may not proceed to crates.io until:

- `ProcessEvent::new()` emits real wall-clock timestamps (DEFECT-1 closed)
- `wpm audit` verdict is `ACCEPTED` (DEFECT-2 closed)
- A runtime audit subcommand exists and publish gates on its verdict (DEFECT-3 closed)
- Session/case grouping is present in emitted evidence (PARTIAL-1 closed)
