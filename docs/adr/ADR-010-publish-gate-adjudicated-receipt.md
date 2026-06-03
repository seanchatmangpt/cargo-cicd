# ADR-010: Publish Gate Requires Adjudicated Receipt

**Status:** Accepted
**Date:** 2026-06-03

## Context

`cargo cicd publish run` is the release command. It must not proceed unless independent evidence of correct process execution has been adjudicated. Internal test results (even 100% passing) do not constitute adjudicated evidence — they only confirm code paths exist. The distinction matters: a pipeline that emits no evidence at all can still pass internal tests.

## Decision

`cargo cicd publish run` gates release on an adjudicated receipt from the wasm4pm oracle. The gate is implemented in `src/nouns/publish.rs` via `ReceiptDoctor::emit_and_adjudicate()` and is not conditional on any feature flag.

Gate outcomes:

| Oracle Response | Publish Outcome | cicd.toml |
|-----------------|-----------------|-----------|
| `state: Admitted` | ADJUDICATED:accept — proceed | `publish_ready = true` |
| Non-zero exit (Refused) | AndonPull — blocked | `publish_ready = false` |
| wpm binary not found | WARN:oracle_unavailable — proceed with warning | `publish_ready = false` |

The gate checks `WpmVerdict::NotAvailable` → `BLOCKED:oracle_unavailable` structurally in code, ensuring even the NotAvailable case is handled explicitly.

## Rationale

A publish gate that only checks internal test state is trivially defeated by a codebase that has tests but no evidence emission. The oracle adjudication requirement means that the pipeline must have actually run, emitted XES evidence, and had that evidence structurally validated by an independent system. This is the minimum bar for a Level 5 process-data engine claiming a release.

## Consequences

- Release closure requires `RECEIPT_DOCTOR:accepted` in the publish run output.
- The declared process model's `requiredConformanceScore: 1.0` and `requiresAdjudicatedEvidence: true` are enforced at runtime, not just in the ontology.
- The wasm4pm evidence gate tests (`tests/wasm4pm_evidence_gate.rs`) are the closing tests for every release.
- No release may claim completion based solely on `cargo make test` passing.

## Violation

A publish that proceeds without `RECEIPT_DOCTOR:accepted` is an unadjudicated release. It violates the evidence gate invariant (ADR-002), the pipeline trace requirement (ADR-008), and this decision. Such a release must be reverted and re-released with proper adjudication.
