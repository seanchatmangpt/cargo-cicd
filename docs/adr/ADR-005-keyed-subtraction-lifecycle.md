# ADR-005: Keyed Subtraction Receipt Lifecycle

**Status:** Accepted
**Date:** 2026-06-03

## Context

As cargo-cicd commands run repeatedly in a workspace, receipts accumulate in `target/cargo-cicd/evidence/receipts/`. Without a clear lifecycle policy, stale receipts from previous runs remain alongside current ones, creating ambiguity about which receipt represents the current state. A phantom record (a receipt without a corresponding live event) can cause the oracle to admit evidence that does not reflect actual execution.

## Decision

Receipts follow keyed subtraction: each receipt key maps to exactly one live record. Emitting a new receipt for an existing key replaces the prior record — there is no accumulation. An empty key slot is always preferable to a stale record.

```
emit(key="publish:run", event) → receipts[key] = event   // replace, not append
```

The `latest.json` receipt is always the single authoritative current record for its key. When the oracle is consulted, it sees exactly the receipts that correspond to live events in the current session.

## Rationale

Accumulated phantom receipts would allow an oracle to admit evidence based on past successful runs even when the current run has not yet reached the required stages. Keyed subtraction ensures the receipt state always reflects the current execution. The oracle cannot be fooled by historical records.

## Consequences

- Receipt writes are idempotent for a given key within a session.
- No receipt accumulation directory needs to be managed or pruned.
- `latest.json` always reflects the most recent `emit_and_adjudicate()` call.
- Test fixtures that inject receipts must use the same keyed replacement semantics.

## Violation

If receipts accumulate without key-based replacement, the oracle may admit evidence that includes phantom records from prior sessions. This produces false Accept verdicts and defeats the evidence gate. Release closure based on such verdicts is invalid.
