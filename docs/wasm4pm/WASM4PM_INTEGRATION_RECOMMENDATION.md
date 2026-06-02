---
artifact: WASM4PM_INTEGRATION_RECOMMENDATION
date: 2026-06-02
version: 26.6.2
selected_path: SHELL_OUT
---

# wasm4pm Integration Recommendation

## Decision

**Selected path:** SHELL_OUT

**Rationale:** The wpm binary is at /Users/sac/wasm4pm/target/release/wpm (v26.5.29) with confirmed functional implementations for doctor, telco status, audit (real SIMD token replay on XES), receipt doctor (--audience ci --format json), lean, spc status, and autoprocess. These are all safe to shell out with zero coupling to wasm4pm internals. The FILE_EXCHANGE path is implicitly part of this approach — cargo-cicd emits XES event log files and receipt JSON files on disk, and wpm consumes them via CLI arguments. Deeper library coupling is unnecessary and would violate the do-not-refactor constraint.

## Implementation Plan

For cargo-cicd v26.6.2, implement a four-stage shell-out pipeline against /Users/sac/wasm4pm/target/release/wpm:

1. PRE-FLIGHT: `wpm doctor` — run from the cargo project root with a synthetic Cargo.toml check. Exit non-zero propagates to CI failure. Expected: all PASS.

2. CONFORMANCE AUDIT GATE: After cargo test produces an XES event log artifact (written to a known output path such as target/process-intelligence/ci-run.xes), shell out: `wpm audit target/process-intelligence/ci-run.xes --activity-key concept:name`. Parse the fitness/precision scores from stdout. Gate on fitness >= 0.95 for TRUTHFUL verdict. Fitness between 0.70 and 0.95 is VARIANCE (warn, do not fail). Below 0.70 is DECEPTIVE — fail the CI job. The XES fixture must be written by the cargo test harness as a file exchange artifact before this stage runs.

3. RECEIPT DOCTOR GATE: After any checkpoint receipt is emitted (JSON file), shell out: `wpm receipt doctor <receipt.json> --audience ci --format json`. Parse the JSON response. Gate on `state != "Refused"` and zero `Deny`-severity findings. Any `Deny` finding propagates to CI failure. Warnings are collected and attached to the build summary.

4. TELCO HEALTH: `wpm telco status` — parse Operational State field, fail if not ACTIVE. This is a lightweight sanity check confirming the wasm4pm runtime is not in a degraded state before CI results are trusted.

Do NOT invoke `wpm mining conformance`, `wpm oracle check`, or `wpm oracle watch` — all three are confirmed stubs that return exit code 0 regardless of input and would produce false-positive CI passes.

## Blockers

- wpm oracle check is a confirmed stub — AndonPull detection cannot gate CI until OrderingLaw evaluation and OracleReport emission are implemented in wasm4pm-algos
- wpm mining conformance stubs model loading to DFG::new() — any CI gate using this command would always produce a meaningless conformance result regardless of the actual model file
- wpm doctor reports FAIL for Cargo.toml not found and src/ directory not found when invoked outside a wasm4pm source tree — the cargo-cicd project must either supply a .wasm4pm config pointing to the correct root or accept these FAIL lines as non-blocking for non-wasm4pm projects
- XES event log fixture authorship is not wasm4pm's responsibility — cargo-cicd must implement its own OTel-to-XES emission step before wpm audit has a real input to consume
- wpm binary path is not on $PATH — CI steps must reference the absolute path /Users/sac/wasm4pm/target/release/wpm or install via cargo install

## Fence Law

> The first wasm4pm integration is not an adapter.
> The first integration is a capability map.

This document IS the first integration.
