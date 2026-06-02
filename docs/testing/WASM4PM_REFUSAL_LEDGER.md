# wasm4pm Refusal Ledger

**Date:** 2026-06-02

This ledger records all proven refusal patterns. Each entry documents a specific mutation
applied to a valid evidence artifact, the reason wasm4pm refuses it, and what the refusal proves.

---

## Proven Refusal Cases

### RF-01 — Missing Verdict Attribute

- **Case:** EC-01-refuse (`status show`)
- **Mutation type:** Attribute removal
- **Mutation:** Remove `cargo_cicd:verdict` attribute from the event element
- **Why wpm refuses:** The declared process model requires every event to carry a verdict claim.
  An event without a verdict is structurally incomplete and cannot be replayed against the model.
- **What this proves:** wasm4pm detects missing required attributes; acceptance is not vacuous.

---

### RF-02 — Unknown Event Concept

- **Case:** EC-02-refuse (`target scan`)
- **Mutation type:** Concept injection
- **Mutation:** Inject an event with a `concept:name` value not present in the declared process model
- **Why wpm refuses:** The process model is closed under its declared activity set. An unknown
  concept:name is an undeclared variant — the log cannot conform to a model that does not contain it.
- **What this proves:** wasm4pm enforces closed-world conformance; unknown activities are rejected.

---

### RF-03 — Duplicate Event

- **Case:** EC-03-refuse (`target prune`)
- **Mutation type:** Event duplication
- **Mutation:** Emit the same event twice consecutively in the trace
- **Why wpm refuses:** The process model specifies each activity occurs exactly once per trace.
  A duplicate violates the frequency constraint and the replay alignment fails.
- **What this proves:** wasm4pm enforces cardinality constraints; duplicate events are rejected.

---

### RF-04 — Incomplete Lifecycle Transition

- **Case:** EC-04-refuse (`test run`)
- **Mutation type:** Lifecycle field corruption
- **Mutation:** Set `lifecycle:transition` to `start` instead of `complete`
- **Why wpm refuses:** The evidence gate requires all events to carry `lifecycle:transition=complete`.
  A `start` event without a matching `complete` is an open span — the trace is not terminated.
- **What this proves:** wasm4pm enforces lifecycle completeness; unclosed spans are rejected.

---

### RF-05 — Event Order Violation

- **Case:** EC-05-refuse (`trybuild run`)
- **Mutation type:** Sequence inversion
- **Mutation:** Swap event order so the verdict event appears before the command emission event
- **Why wpm refuses:** The process model has a strict temporal ordering: command emission precedes
  verdict. An inverted sequence violates the partial order and conformance replay fails.
- **What this proves:** wasm4pm enforces causal ordering; temporal violations are rejected.

---

### RF-06 — Malformed Timestamp

- **Case:** EC-06-refuse (`git close`)
- **Mutation type:** Timestamp corruption
- **Mutation:** Replace the ISO 8601 timestamp with a non-conformant string (e.g., `"yesterday"`)
- **Why wpm refuses:** XES requires `time:timestamp` values to be valid ISO 8601 datetimes.
  A malformed timestamp cannot be parsed; the event log is structurally invalid XML.
- **What this proves:** wasm4pm rejects malformed XML; syntactic validity is enforced.

---

### RF-07 — Empty Verdict Claim

- **Case:** EC-07-refuse (`publish check`)
- **Mutation type:** Value erasure
- **Mutation:** Replace the `Accept` verdict claim value with an empty string
- **Why wpm refuses:** An empty verdict string is semantically equivalent to no verdict.
  The conformance model requires a non-empty, recognized verdict token.
- **What this proves:** wasm4pm enforces semantic validity of attribute values; empty claims are rejected.

---

### RF-08 — Missing Trace Identity

- **Case:** EC-08-refuse (`autonomic suggest`)
- **Mutation type:** Trace attribute omission
- **Mutation:** Omit the `concept:name` attribute from the `<trace>` element entirely
- **Why wpm refuses:** Every trace must carry a `concept:name` to be identifiable. A trace without
  identity cannot be matched against a case in the process model; the log is truncated.
- **What this proves:** wasm4pm requires trace identity; anonymous traces are rejected.

---

## Invariant E5 Compliance: Every Positive Case Has a Proven Negative

Invariant E5 states: every accept case must have a corresponding proven refusal case.

| Accept Case | Refuse Case | Mutation Applied | Status |
|-------------|-------------|-----------------|--------|
| EC-01-accept | EC-01-refuse | Attribute removal | PROVEN |
| EC-02-accept | EC-02-refuse | Concept injection | PROVEN |
| EC-03-accept | EC-03-refuse | Event duplication | PROVEN |
| EC-04-accept | EC-04-refuse | Lifecycle corruption | PROVEN |
| EC-05-accept | EC-05-refuse | Sequence inversion | PROVEN |
| EC-06-accept | EC-06-refuse | Timestamp corruption | PROVEN |
| EC-07-accept | EC-07-refuse | Value erasure | PROVEN |
| EC-08-accept | EC-08-refuse | Trace attribute omission | PROVEN |

All 8 accept cases have a proven negative. Invariant E5 is satisfied.
