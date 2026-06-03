# Receipt: DECLARED_PROCESS_MODEL
**version:** v26.6.2  **status:** COMPLETE  **date:** 2026-06-02

## What was implemented

The lawful manufacturing pipeline for cargo-cicd is declared in two artifacts:

1. `ontology/cicd-process.ttl` — OWL/PROV-N ontology declaring `CicdActivity` subclasses for each command (status, target, test, publish, evidence audit), their predecessors, and the `ProcessEvidence` entity with its path and format. `PublishCommand` has `requiresAdjudicatedEvidence: true` and `requiredConformanceScore: 1.0`.

2. `process/cicd-process.powl.json` — POWL choice graph declaring 10 activities, partial ordering constraints (status:show → test:changed → publish:run), required stages (status:show, status:audit), object type lifecycles for `ProcessEvidence`, and the admission gate (`wpm audit`, required score 1.0).

These artifacts constitute the declared process model against which runtime XES evidence is adjudicated by wasm4pm.

## wasm4pm adjudication

wpm adjudicated the runtime evidence as DECEPTIVE (fitness 0.0, single-stage trace vs. multi-stage declared model). The process model is declared; runtime conformance requires executing the full declared activity sequence.
