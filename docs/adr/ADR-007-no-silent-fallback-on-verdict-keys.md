# ADR-007: No Silent Fallback on Verdict Keys

**Status:** Accepted
**Date:** 2026-06-03

## Context

When parsing wpm oracle JSON output, the `state` key contains the verdict (`Admitted`, `Refused`, etc.). Code under time pressure may use `unwrap_or("Admitted")` or similar fallbacks so that a missing key does not block progress. This is a silent failure mode: if the oracle output format changes, or if parsing fails, the gate always passes.

## Decision

Absent verdict keys are errors, not fallback cases. The `state` key must be present in the oracle output. If it is absent, the gate returns an error — it does not default to Accept, Admitted, or any passing verdict.

```rust
// CORRECT
let verdict = output.get("state")
    .ok_or_else(|| anyhow::anyhow!("wpm output missing 'state' key"))?;

// WRONG — silent fallback
let verdict = output.get("state").unwrap_or("Admitted");
```

The only permitted fallback is `NotAvailable` — when the wpm binary is not found on the system. This is a distinct code path that emits `WARN:oracle_unavailable`, not a JSON parse fallback.

## Rationale

A silent fallback on missing verdict keys means that any oracle output format change, any parse error, or any test fixture that omits the key would produce a false Accept. The gate exists specifically to prevent unreviewed evidence from reaching release. A gate that silently passes on parse error is equivalent to no gate.

## Consequences

- Oracle output parsing must handle the `state` key's absence as a hard error.
- Test fixtures for wpm output must include valid `state` keys.
- Integration tests that mock wpm must return well-formed JSON with `state` present.
- The `WpmVerdict` enum has exactly three variants: `Accept`, `Refuse`, `NotAvailable` — no `Unknown` or `ParseError` variants that could be silently treated as passing.

## Violation

A silent fallback allows the publish gate to pass even when the oracle output is malformed, truncated, or from a different API version. Any release proceeding under a silently-passed gate is unadjudicated by definition.
