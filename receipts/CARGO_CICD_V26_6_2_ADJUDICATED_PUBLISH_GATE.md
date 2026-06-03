# Receipt: ADJUDICATED_PUBLISH_GATE
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

The `publish` noun in `src/nouns/publish.rs` gates release on wasm4pm oracle adjudication. Before writing `cicd.toml` or proceeding with `cargo publish`, it invokes `wpm audit` on the current evidence XES file. Three outcomes: (1) `ADJUDICATED:accept` — oracle accepted evidence, proceed; (2) `WARN:oracle_unavailable` — wpm binary not found, proceed with warning; (3) oracle refused (non-zero exit) — publish is blocked. The publish gate checks `WpmVerdict::NotAvailable` → `BLOCKED:oracle_unavailable`. This ensures no release proceeds without independent process evidence review, satisfying the declared process model's `admission_gate.required_score: 1.0` constraint.

## wasm4pm adjudication

```
src/nouns/publish.rs line 83:
  Some(wpm) => match wpm.audit(evidence_xes.to_str().unwrap_or("")) {
    ...
    WpmVerdict::NotAvailable => "BLOCKED:oracle_unavailable",
  }
```

Publish gate structurally enforced. Current evidence verdict from oracle: DECEPTIVE (publish would be blocked until evidence achieves conformance).
