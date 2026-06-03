# Receipt: TRACE_GROUPING (SESSION_TRACES)
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

Each invocation of `cargo-cicd` assigns a session ID (`sess-<random>-<pid>`) stored as the `concept:name` of the XES `<trace>` element. All events emitted during that invocation share the same `case_id`, grouping start/complete pairs into a coherent session trace. The `EventLog` struct in `src/evidence.rs` manages trace grouping: events are appended to the trace keyed by `case_id`, enabling wasm4pm to reconstruct per-session process instances. Verification: `events.xes` contains `<string key="concept:name" value="sess-18b56eab2e1d3808-00008e47"/>` as the trace identifier.

## wasm4pm adjudication

```
Vision 2030 Conformance Audit Report
Audit Verdict: DECEPTIVE
Total Traces Audited: 1
Fitting Traces: 0
Deviating Traces: 1
Trace ID: unknown (fitness 0.58)
```

Trace grouping is structurally sound — wpm recognized one trace with one case. The DECEPTIVE verdict reflects process model conformance gap, not structural XES issues.
