# Runtime Evidence Reconciliation Gate — cargo-cicd v26.6.2

**Date:** 2026-06-02
**Verdict: BLOCKED**

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
`ProcessEvent::new()` hardcodes `timestamp_iso` to `"2026-06-02T00:00:00.000Z"`.
This is a runtime production constructor, not a test fixture.
Every event emitted through the nouns layer (trybuild, test, git, target) carries this fake timestamp.
XES logs emitted by cargo-cicd contain non-temporal evidence.
A process mining conformance check cannot distinguish ordering or measure inter-event duration from the primary emitter path.

**Acceptable occurrences (not defects):**
- `src/integrations/wasm4pm_exchange.rs:605` — hardcoded timestamp in `#[cfg(test)]` block (test-only, acceptable)
- `src/integrations/wasm4pm_exchange.rs:500` — fallback epoch string `"1970-01-01T00:00:00Z"` on `date` command failure (fallback, not primary path)

### DEFECT-2: wpm audit verdict is UNAVAILABLE

**Location:** runtime audit pipeline
`wpm` binary was not found. The audit verdict is `UNAVAILABLE`, not `ACCEPTED`.
No runtime audit subcommand exists in cargo-cicd. Evidence adjudication is test-only.
The evidence file at `target/cargo-cicd/evidence/events.xes` exists but cannot be adjudicated without wpm tooling.

### DEFECT-3: No runtime audit subcommand

**Location:** cargo-cicd command surface
cargo-cicd exposes no `audit` subcommand at runtime.
Evidence adjudication is confined to test code.
There is no path for a release pipeline to obtain an adjudicated verdict from wasm4pm at runtime.

---

## Partials

### PARTIAL-1: Evidence traces not grouped by session/case

No `trace_id` or `case_id` field links multi-command invocations together.
Three commands were invoked (`status`, `target-show`, `git status`), but the XES log contains no structure grouping them as a single session or case.
Cross-command conformance replay is not possible without this grouping.

---

## Required Remediation Steps

### For DEFECT-1 (hardcoded timestamp)

1. Replace the hardcoded `timestamp_iso` literal in `ProcessEvent::new()` (`src/evidence.rs:51`) with a call to the real wall-clock source — use the same `now_rfc3339()` shell-out already present in `src/integrations/wasm4pm_exchange.rs:490-501`, or introduce a `SystemTime`-based implementation.
2. Verify that after the fix, emitted XES events carry real wall-clock timestamps at sub-second precision.
3. Re-run the evidence runtime test and confirm timestamps differ between sequential events.

### For DEFECT-2 and DEFECT-3 (wpm audit unavailable / no runtime audit subcommand)

1. Ensure `wpm` binary is present and on PATH before any publish gate runs.
2. Implement a `cargo cicd audit` subcommand (or equivalent) that invokes `WpmEvidenceOracle` against the emitted evidence file and records the adjudicated verdict into the release receipt.
3. The publish noun (`src/nouns/publish.rs`) must import and call `WpmEvidenceOracle` before writing `verdict = "pass"` into `cicd.toml`. Self-certification is not acceptable.
4. Gate `publish run` on an `ACCEPTED` oracle verdict, not a self-assigned string.

### For PARTIAL-1 (session/case grouping)

1. Assign a `trace_id` (UUID or content hash) at session start.
2. Propagate `trace_id` as the XES `trace` attribute through all events emitted within that invocation.
3. For multi-command sessions, use a stable `case_id` (e.g. git commit hash + session timestamp) to group related command invocations into a single XES trace.

---

## Condition for PUBLISH_READY

This release may not proceed to crates.io until all three BLOCKED defects are resolved and re-verified:

- `ProcessEvent::new()` emits real wall-clock timestamps
- `wpm audit` verdict is `ACCEPTED` (not `UNAVAILABLE`)
- A runtime audit subcommand exists and the publish noun gates on its verdict

The PARTIAL on session grouping must also be closed before the release receipt can carry `PUBLISH_READY`.
