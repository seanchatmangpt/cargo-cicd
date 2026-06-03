# Receipt: VERDICT_PROVENANCE
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

The `ProcessEvent` struct in `src/evidence.rs` carries two distinct verdict fields: `verdict_claimed: String` (set by the cargo-cicd command itself, e.g. "PASS", "WARN", "FAIL") and `verdict_adjudicated: Option<String>` (set only when wasm4pm oracle has externally reviewed the evidence). These are written to separate XES attributes: `cargo_cicd:verdict_claimed` and `wasm4pm:verdict_adjudicated`. This structural separation enforces that claimed verdicts cannot masquerade as adjudicated verdicts. The `evidence:audit` event type is the only event type that sets `verdict_adjudicated`; all other commands set only `verdict_claimed`.

## wasm4pm adjudication

```
grep "verdict_claimed|verdict_adjudicated" src/evidence.rs confirms:
  line 112: pub verdict_claimed: String,
  line 117: pub verdict_adjudicated: Option<String>,
  line 329: cargo_cicd:verdict_claimed  (XES attribute key)
  line 338: wasm4pm:verdict_adjudicated (XES attribute key, only when Some)
```

Structural separation verified. wpm output: DECEPTIVE (process model conformance gap — single-stage trace vs. multi-stage declared model).
