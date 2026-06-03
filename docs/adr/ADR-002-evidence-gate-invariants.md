# ADR-002: Evidence Gate Invariants

**Status:** Accepted
**Date:** 2026-06-03

## Context

cargo-cicd is a Level 5 process-data engine. Its correctness claim is not that tests pass — it is that the declared manufacturing pipeline was actually executed in lawful order and the evidence of that execution was independently adjudicated. Internal tests can pass while the runtime process never follows the declared model.

## Decision

Every command emits process evidence as XES (XML Event Stream) to `target/cargo-cicd/evidence/`. Release is gated on wasm4pm adjudicating that evidence as conforming to the declared process model. The following invariants hold unconditionally:

1. Every command emits at least one `ProcessEvent` with a real UTC timestamp.
2. Evidence is written to `target/cargo-cicd/evidence/events.xes` before any publish gate check.
3. The wasm4pm oracle must be consulted before publish proceeds.
4. Accept/Refuse/NotAvailable are the only valid oracle verdicts — no other interpretation is permitted.
5. A single-stage trace with fitness 0.0 results in DECEPTIVE classification and blocks release.

## Rationale

The Van der Aalst Constitution: if the event log cannot prove a lawful process happened, then it did not happen. Internal smoke tests confirm code paths exist; they cannot confirm that the runtime process followed the declared model. Only an adjudicated XES trace can make that claim.

## Consequences

- Tests in `tests/wasm4pm_evidence_gate.rs` are the closing tests — non-closing tests do not satisfy the release criterion.
- `wasm4pm` feature flag gates richer runtime integration but does NOT gate the evidence-gate law.
- The declared process model in `ontology/cicd-process.ttl` and `process/cicd-process.powl.json` is the ground truth for conformance scoring.
- Single-command invocations will produce DECEPTIVE verdicts until the full declared sequence is executed.

## Violation

If evidence emission is removed, if oracle adjudication is bypassed, or if internal test passage alone is treated as a release criterion, then cargo-cicd's correctness claim is fraudulent and no release is valid.
