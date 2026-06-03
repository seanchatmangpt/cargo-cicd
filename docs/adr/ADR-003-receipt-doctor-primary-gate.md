# ADR-003: ReceiptDoctor as Primary Publish Gate

**Status:** Accepted
**Date:** 2026-06-03

## Context

cargo-cicd needs a publish gate that is structurally enforced, not advisory. An internal test suite that passes does not constitute an adjudicated receipt. The gate must invoke an external oracle that has no knowledge of internal test state.

## Decision

`ReceiptDoctor::emit_and_adjudicate()` is the primary and mandatory publish gate. It:

1. Builds an OCEL 2.0 compliant receipt JSON from the current event log.
2. Invokes `wpm receipt doctor --format json --strict <receipt>`.
3. Parses the `state` field from the JSON response.
4. Returns `RECEIPT_DOCTOR:accepted` only if `state == "Admitted"`.
5. Returns `AndonPull` (publish blocked) if the oracle refuses.
6. Returns `WARN:oracle_unavailable` (proceed with warning) if wpm is not found.

The publish noun in `src/nouns/publish.rs` calls this gate before writing `cicd.toml` or invoking `cargo publish`.

## Rationale

Internal test passage is a necessary but not sufficient condition for release. The receipt doctor provides an independent structural check that the evidence record is well-formed and admits the declared commands. No internal state can substitute for this check — the oracle must be consulted.

## Consequences

- `publish_ready = true` in `cicd.toml` only if the oracle returns `Admitted`.
- `cargo cicd evidence doctor` must pass before `cargo cicd publish run` will proceed.
- The receipt format is `algorithms`-based OCEL2; `CanonicalHashVerifier` is skipped (no `receipt_hash` field) — structural correctness only.
- Receipt path: `target/cargo-cicd/evidence/receipts/latest.json`.

## Violation

If publish proceeds based on internal test state alone, or if the oracle check is gated behind a feature flag, the publish gate is void. Any release without `RECEIPT_DOCTOR:accepted` is unadjudicated and must be treated as unverified.
