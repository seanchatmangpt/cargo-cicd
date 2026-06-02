# CARGO_CICD_V26_6_2_WASM4PM_CAPABILITY_SCAN

**Receipt type:** Capability scan receipt
**Date:** 2026-06-02
**Scanner:** Claude Code (claude-sonnet-4-6), session 2026-06-02

---

## Scan Parameters

| Field | Value |
|---|---|
| date | 2026-06-02 |
| wasm4pm repo path | /Users/sac/wasm4pm |
| wasm4pm commit | 65169e62 fix(debt): resolve debt markers blocking pre-push hook |
| cargo-cicd target version | v26.6.2 |
| scan areas | 11 (CLI, crates, features, formats, import/export, algorithms, conformance, types, tests, docs, errors) |

---

## Findings

| Metric | Count |
|---|---|
| Total capabilities inventoried | 75 |
| Capabilities accepted (USE_AS_IS) | 22 |
| Capabilities accepted (SHELL_OUT) | 2 |
| Capabilities accepted (FILE_EXCHANGE) | 11 |
| Capabilities accepted (FEATURE_GATE) | 9 |
| Capabilities accepted (WRAP_LOCAL) | 4 |
| Capabilities accepted (PATCH_SMALL) | 2 |
| Capabilities deferred (DEFER_CONTRIB) | 14 |
| Capabilities rejected (DO_NOT_USE) | 11 |
| Total accepted (all non-DO_NOT_USE non-DEFER) | 50 |

---

## Selected Integration Path

**Path C: Thin Local Adapter**

File: `cargo-cicd/src/integrations/wasm4pm_current.rs`

Core: `wasm4pm-types` + `wasm4pm-algos::conformance` as direct Cargo dependencies.
Pre-flight: `wpm doctor` shell-out.
Receipt chain: `ProvenanceChain` + `Blake3Hash` from wasm4pm-types.

---

## Deferred Contrib Candidates

14 candidates documented in `/Users/sac/cargo-cicd/docs/deferred/WASM4PM_CONTRIB_EXTRACTION.md`:

1. DEFER_CONTRIB_001 — Heuristic Miner
2. DEFER_CONTRIB_002 — Inductive Miner
3. DEFER_CONTRIB_003 — Adversarial Fixture Library
4. DEFER_CONTRIB_004 — Oracle Gap Patterns
5. DEFER_CONTRIB_005 — OCEL v2 Integration Fixture
6. DEFER_CONTRIB_006 — Telco Structured Output
7. DEFER_CONTRIB_007 — Feature-Discovery-Advanced Surface
8. DEFER_CONTRIB_008 — POWL Surface
9. DEFER_CONTRIB_009 — ML / Prediction Surface
10. DEFER_CONTRIB_010 — Cognition Surface
11. DEFER_CONTRIB_011 — BCINR Algorithm Surface
12. DEFER_CONTRIB_012 — Live Counterfactual Testing
13. DEFER_CONTRIB_013 — Statistical Library
14. DEFER_CONTRIB_014 — Combined CF Property Tests

---

## Known Gaps

1. **Alpha+ Miner is placeholder:** `wasm4pm_algos::alpha::discover_alpha` has a placeholder doctest and missing parallel/choice handling. DO_NOT_USE until production implementation.

2. **No machine-readable CLI output for conformance:** `wpm` binary has no subcommand that emits `ConformanceResult` as JSON. Shell-out cannot replace library-level conformance access.

3. **`import_xes` function signature unverified:** The scan infers the XES import function exists from the `import` feature flag and `quick-xml` / `flate2` dependencies. The exact public function signature (`import_xes`, `EventLog::from_xes`, etc.) must be verified before the adapter is written.

4. **Named refusal reasons unverified:** The scan infers named refusal types exist in `wasm4pm-algos` from the wasm4pm-compat architecture. Direct source inspection of `wasm4pm-algos` refusal types was not performed.

5. **`wpm` not confirmed on PATH in CI environments:** `wpm doctor` pre-flight is optional in the v26.6.2 plan. If `wpm` is absent, the pre-flight step must degrade gracefully.

6. **wasm4pm-algos path dependency:** v26.6.2 uses path dependencies (`path = "/Users/sac/wasm4pm/..."`). Publishing to crates.io is a precondition for any use outside the local machine.

7. **WASM bundle targets not evaluated:** The scan covers Rust library API only. WASM bundle emission (wasm-pack, cdylib targets) for browser/edge/fog/iot profiles was not evaluated for cargo-cicd use. WASM targets are out of scope for v26.6.2.

---

## Verdict

**ALIVE**

The scan finds 50 accepted capabilities (USE_AS_IS through PATCH_SMALL), a stable conformance surface with typed inputs and outputs, and a clear thin-adapter integration path requiring fewer than 150 lines of adapter code. No blocking gaps prevent v26.6.2 integration. The Alpha+ Miner gap (DO_NOT_USE) does not block conformance gating. The missing `import_xes` signature (Gap 3) is a 30-minute verification task, not a blocker.

---

## Documents Created

| Document | Path |
|---|---|
| Capability Inventory (75 capabilities, 11 areas) | /Users/sac/cargo-cicd/docs/wasm4pm/WASM4PM_CAPABILITY_INVENTORY.md |
| Leverage Matrix (sorted by verdict) | /Users/sac/cargo-cicd/docs/wasm4pm/WASM4PM_LEVERAGE_MATRIX.md |
| Integration Recommendation (Path C) | /Users/sac/cargo-cicd/docs/wasm4pm/WASM4PM_INTEGRATION_RECOMMENDATION.md |
| Deferred Contrib Extraction (14 candidates) | /Users/sac/cargo-cicd/docs/deferred/WASM4PM_CONTRIB_EXTRACTION.md |
| This receipt | /Users/sac/cargo-cicd/receipts/CARGO_CICD_V26_6_2_WASM4PM_CAPABILITY_SCAN.md |
