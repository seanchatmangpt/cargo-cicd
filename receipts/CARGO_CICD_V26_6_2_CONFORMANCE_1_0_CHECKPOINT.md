---
# CONFORMANCE-1.0 Checkpoint — cargo-cicd v26.6.2

**Date:** 2026-06-03
**Status:** PARTIAL — 0.9636 TRUTHFUL achieved; 1.0 feedback closure pending

## Conformance Journey

| State | Score | Verdict | Cause |
|---|---|---|---|
| Baseline | 0.0000 | DECEPTIVE | Hardcoded timestamps, test-only oracle |
| After runtime fixes | 0.0000 | DECEPTIVE | Verdict reader read wrong key (fitness vs overall_fitness) |
| After key alignment | 0.9636 | TRUTHFUL | Pipeline trace, real timestamps, declared activities |
| Live ambient trace | 0.8194 | VARIANCE | Honest: accumulated history ≠ clean pipeline |

## The Decisive Discovery

The system had structurally valid conformance output, but the audit reader was looking for
key "fitness" while simd_token_replay emitted "overall_fitness".

**Law established:**
> Evidence is not enough. The verdict path must be schema-aligned all the way to the reader.
> A structurally correct receipt can be operationally false if the judgment key is unreadable.

## Four Evidence Layers (Now Separated)

1. Event exists — ProcessEvent + append_events, real SystemTime::now() timestamps
2. Trace exists — JSONL accumulation, session case_id, XES rebuilt from full log
3. Court judges — wpm receipt doctor --strict → ACCEPTED, state: Admitted
4. Consumer reads correctly — overall_fitness key, explicit precision: null

## Verdict Schema Contract

The authoritative keys emitted by wpm:

| Field | Type | Notes |
|---|---|---|
| overall_fitness | f64 (nullable) | Authoritative. Consumers must read this, never "fitness" |
| precision | f64 (nullable) | Explicit null when not computed — never silent 0.0 |
| verdict | string | TRUTHFUL / VARIANCE / DECEPTIVE |
| token_deviation | string (nullable) | e.g. "M:0 R:0" |
| trace_class | string (nullable) | pipeline_run / live_workspace_trace |
| model_source | string (nullable) | DFG-derived Petri net source |
| receipt_ref | string (nullable) | Receipt that was adjudicated |

## Trace Class Separation

**pipeline_run** — deliberate full-pipeline execution (status → test → publish → audit)
- May target TRUTHFUL fitness
- Current: 0.9636

**live_workspace_trace** — accumulated ambient command history
- Honestly reports VARIANCE — not a failure
- Current: 0.8194

Conflating these two produces misleading conformance readings.

## CONFORMANCE-1.0 Gates (Next Checkpoint)

| Gate | Status | Notes |
|---|---|---|
| Precision metric | PARTIAL | simd_token_replay does not compute precision; explicit null emitted |
| Closed-loop model feedback | OPEN | DFG→Petri-net without feedback path; M:2 R:1 token deviation remains |
| Trace-class separation | CLOSED | TraceClass enum, pipeline vs ambient documented |
| Verdict schema contract | CLOSED | WpmVerdict struct with authoritative field names |
| LSP CICD-WPM-004 | CLOSED | Diagnostic code defined, regression fixture added |

## Current Honest Statement

> Fitness is live at 0.9636 TRUTHFUL on pipeline traces.
> Precision is not yet computed — explicitly null, not silently zero.
> Full 1.0 conformance requires closed-loop model feedback path.
> VARIANCE on ambient traces is expected and honest.
