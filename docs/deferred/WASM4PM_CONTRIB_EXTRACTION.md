---
artifact: WASM4PM_CONTRIB_EXTRACTION
date: 2026-06-02
status: DEFERRED
target_checkpoint: post-v26.6.2
---

# wasm4pm-contrib Extraction — Deferred

## Status: DEFERRED

wasm4pm-contrib extraction is explicitly deferred from cargo-cicd v26.6.2.

## Deferred Candidates

- wpm oracle check — defer to wasm4pm-contrib once OrderingLaw evaluation and OracleReport are implemented; the CLI surface and law deserialization already exist
- wpm oracle watch — defer until streaming EarlyStop NDJSON emission is implemented; the interface contract is defined but the body is a single println stub
- wpm mining conformance — defer until model parsing replaces the DFG::new() stub; then FILE_EXCHANGE (emit a .pnml or discovered DFG JSON, shell out for real conformance scores) becomes viable
- wpm mining discover for XES — defer the PATCH_SMALL fix (one-line call to load_eventlog_from_xes instead of empty fallback) to wasm4pm-contrib; once patched this unlocks discovery directly from cargo CI XES artifacts
- OCEL DFG discovery (discover_ocel_dfg / discover_ocel_dfg_per_type) — defer until ocel feature flag stabilizes and ocel-core version drift (26.5.30 vs workspace 26.5.29) is resolved

## Extraction Criteria

1. Stable, tested API (not changing frequently)
2. Genuinely reusable outside cargo-cicd
3. Clear input/output contracts
4. Does not require refactoring wasm4pm
5. Explicit authorization from wasm4pm authority

## Fence Law

> Do not extract until the capability is stable, tested, reusable, and authorized.
