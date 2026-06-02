---
artifact: INVARIANTS
date: 2026-06-02
---

# cargo-cicd Invariants

These must hold everywhere, regardless of command or feature combination.

## I1 — Public Boundary
No public command/help/README output contains forbidden private terms.
Forbidden: ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8

## I2 — Publish Determinism
If workspace state does not change, cargo cicd publish produces stable process state.

## I3 — No False Close
git close must not claim closed if dirty state remains.

## I4 — No Destructive Default
target prune must not delete by default without confirmation.

## I5 — No Full Trybuild By Default
trybuild changed must not run the full fixture estate by default.

## I6 — No Assumed wasm4pm Capability
wasm4pm integration must not use a capability not discovered and classified.

## I7 — Feature Projection Consistency
Enabling process-data/autonomic/wasm4pm may add records but must not contradict default facts.
