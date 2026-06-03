# Conformance Evidence Law

This document explains how cargo-cicd establishes process conformance and how
the four evidence layers relate to trace class separation.

## Four-Layer Evidence Law

Process conformance is not a single assertion. It is a chain of four distinct
evidence layers. Every layer must hold. A gap at any layer breaks the chain.

### Layer 1 — Event Exists

A `ProcessEvent` was emitted with a real system timestamp at the moment the
activity occurred. Hardcoded or fabricated timestamps invalidate this layer.

The event carries:
- A case identifier linking it to a process instance
- An activity name drawn from the declared process model
- A wall-clock timestamp from `SystemTime::now()`

### Layer 2 — Trace Exists

Individual events accumulate into a trace. The trace is the full ordered
history of a process instance. cargo-cicd appends events to a JSONL log keyed
by session `case_id`. At audit time, the full XES event log is rebuilt from
this accumulated record.

A trace that was never persisted cannot be replayed. A trace replayed from
incomplete data produces unreliable fitness scores.

### Layer 3 — Court Judges

The process mining court (`wpm receipt doctor --strict`) replays the trace
against the declared process model and emits a structured verdict. The verdict
state must reach `Admitted` for the trace to count as conforming.

The court does not trust assertions. It replays token flow against the model
and measures deviation.

### Layer 4 — Consumer Reads Correctly

The verdict JSON contains authoritative fields with specific names. A consumer
that reads the wrong field name receives a structurally valid but operationally
false result.

**The authoritative fitness field is `overall_fitness`.** Reading `fitness`
instead silently returns null or zero. This was the root cause of the
CONFORMANCE-1.0 baseline reading 0.0000 DECEPTIVE despite correct underlying
evidence.

Law: a structurally correct receipt can be operationally false if the judgment
key is unread or misread.

## Verdict Schema Contract

| Field | Type | Meaning |
|---|---|---|
| `overall_fitness` | f64 (nullable) | Token-replay fitness. Authoritative. Never substitute `fitness`. |
| `precision` | f64 (nullable) | Explicit null when not computed. Never silently zero. |
| `verdict` | string | `TRUTHFUL` / `VARIANCE` / `DECEPTIVE` |
| `token_deviation` | string (nullable) | Missing and remaining token summary, e.g. `M:0 R:0` |
| `trace_class` | string (nullable) | `pipeline_run` or `live_workspace_trace` |
| `model_source` | string (nullable) | How the Petri net was derived |
| `receipt_ref` | string (nullable) | Receipt identifier that was adjudicated |

## Trace Class Separation

Not all traces should be held to the same conformance standard. cargo-cicd
distinguishes two trace classes:

### `pipeline_run`

A deliberate, full-pipeline execution: `status → test → publish → audit`.
This trace is constructed to exercise the declared process model completely.
It is the appropriate target for `TRUTHFUL` fitness claims.

Current pipeline_run fitness: **0.9636 TRUTHFUL**

### `live_workspace_trace`

The accumulated history of all commands issued in a workspace session,
including exploratory, diagnostic, and repair commands that are not part of
the declared pipeline. This trace honestly represents what happened, not what
the process model says should happen.

`VARIANCE` on a live_workspace_trace is **not a failure**. It is the honest
reporting of accumulated ambient activity.

Current live_workspace_trace fitness: **0.8194 VARIANCE**

**Conflating these two classes produces misleading conformance readings.**
A `VARIANCE` verdict on a live trace must not be interpreted as a pipeline
regression.

## Open Gates

| Gate | Status | Requirement for 1.0 |
|---|---|---|
| Precision metric | PARTIAL | simd_token_replay must compute and emit precision |
| Closed-loop model feedback | OPEN | DFG→Petri-net needs feedback path to reduce token deviation |
| Trace-class separation | CLOSED | TraceClass enum enforces the distinction at the type level |
| Verdict schema contract | CLOSED | WpmVerdict struct names all authoritative fields |

## See Also

- `docs/lsp/DIAGNOSTICS.md` — LSP diagnostic codes including CICD-WPM-004
- `receipts/CARGO_CICD_V26_6_2_CONFORMANCE_1_0_CHECKPOINT.md` — conformance state receipt
- `receipts/CARGO_CICD_V26_6_2_VERDICT_PROVENANCE.md` — verdict provenance chain
