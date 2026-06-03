# Receipt: RUNTIME_WASM4PM_AUDIT
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

The `Wasm4pmShell` integration in `src/integrations/` detects the `wpm` binary at runtime (searching PATH and known locations including `/Users/sac/wasm4pm/target/release/wpm`). The `status audit` verb shells out to `wpm audit <xes_path>`, captures stdout, parses the verdict line, and emits an `evidence:audit` event back into the XES evidence log with `wasm4pm:verdict_adjudicated` set to the oracle's verdict. This closes the self-certification gap: the binary does not certify its own evidence.

## wasm4pm adjudication

Actual `wpm audit` output on `target/cargo-cicd/evidence/events.xes`:

```
Vision 2030 Conformance Audit Report

Audit Verdict:            DECEPTIVE
Fitness Score:            0.0000
Precision Score:          0.0000

Total Traces Audited:     1
Fitting Traces:           0
Deviating Traces:         1

Sample Deviations:

Trace ID  Fitness  Problems
unknown   0.58     M: null, R: null

Doctrine: If the code says it worked but the event log cannot prove a lawful process happened, then it did not work.
```

wpm binary path: `/Users/sac/wasm4pm/target/release/wpm`
Exit code: 0 (parseable XES accepted; DECEPTIVE is a quality judgment)
