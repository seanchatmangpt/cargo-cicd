# ADR-008: Pipeline Traces vs. Ambient Traces

**Status:** Accepted
**Date:** 2026-06-03

## Context

Two categories of XES traces can appear in the evidence directory: pipeline traces (events emitted by the declared manufacturing pipeline in lawful order with real timestamps) and ambient traces (events from any invocation of any command at any time, not necessarily in the declared order). Ambient traces may incidentally contain the right activity names without following the declared sequence.

## Decision

Only pipeline traces are admissible evidence for release closure. An admissible trace must:

1. Follow the declared activity sequence from the process model (`status:show → status:audit → test:changed → publish:run`).
2. Carry real UTC timestamps in strictly ascending order.
3. Have been emitted by the canonical pipeline execution, not by isolated command invocations.
4. Achieve conformance fitness >= the required score declared in `ontology/cicd-process.ttl` (currently 1.0).

Ambient traces — those that contain matching activity names but not the declared sequence — produce fitness 0.0 and are classified DECEPTIVE.

## Rationale

A trace that mentions `publish:run` without the preceding stages provides no evidence that the pipeline ran correctly. It only proves that the publish command was invoked in isolation. The declared process model exists precisely to require the full sequence. Accepting ambient traces would make the evidence gate trivially satisfiable by any single command invocation.

## Consequences

- Single-command test runs produce traces classified as DECEPTIVE — this is expected and correct.
- Release closure requires running the full declared pipeline in a single session.
- `process/cicd-process.powl.json` defines the lawful partial order; any trace that violates it receives a reduced fitness score.
- Evidence emission tests (Tier 1) verify structure only; conformance tests (Tier 2) verify pipeline order.

## Violation

If ambient traces are accepted as conforming evidence, any isolated command invocation constitutes a passing release gate. This makes the declared process model decorative rather than enforceable, and the Van der Aalst Constitution is violated.
