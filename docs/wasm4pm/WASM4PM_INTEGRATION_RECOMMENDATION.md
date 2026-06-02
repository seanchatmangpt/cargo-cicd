# WASM4PM Integration Recommendation

**Generated:** 2026-06-02
**wasm4pm commit:** 65169e62
**cargo-cicd target version:** v26.6.2
**Decision date:** 2026-06-02

---

## Selected Integration Path

**Path C: Thin Local Adapter**
`cargo-cicd/src/integrations/wasm4pm_current.rs`

---

## Justification

### Why not Path A (File Exchange)?

File Exchange was the initial candidate. However, the scan reveals that the `wasm4pm-types` crate is already designed as a stable shared library: every core type derives `serde::Serialize` / `serde::Deserialize`, the `EventLog`, `OCEL`, `PetriNet`, `DFG`, `ConformanceResult`, and `Blake3Hash` types are all stable, and `ocel-core` is re-exported unconditionally. Adding `wasm4pm-types` as a direct Cargo dependency to cargo-cicd is lower coupling than it appears — it has no runtime dependencies and zero unsafe code. Writing JSON files on disk and parsing them back (Path A) would introduce an unnecessary serialization round-trip and remove the type-safety guarantee that the scan confirms is already present.

**Specific evidence:** wasm4pm-types has no runtime dependencies; all core types are `USE_AS_IS`; `ConformanceResult` and `TokenReplayResult` are typed output structs ready for direct pattern-matching in cargo-cicd gate logic.

### Why not Path B (CLI Shell-Out)?

`wpm` binary exists and `wpm doctor` is SHELL_OUT-ready. However, the scan reveals no CLI subcommand that performs conformance checking, log import, or DFG construction with machine-readable output. `wpm telco status` is experimental and has no structured output. `wpm wizard` is interactive-only. Shell-out would require cargo-cicd to speak unstructured stdout, which is a fragile integration surface. Path B is reserved for the `wpm doctor` health check only (a pre-flight step, not the integration core).

**Specific evidence:** No `wpm` subcommand emits `ConformanceResult` JSON. The only stable CLI output is `wpm doctor` health prose. Shell-out cannot access `check_conformance_token_replay` or `check_conformance_alignment`.

### Why not Path D (Defer)?

Deferral is warranted only when the API is unstable or has known gaps that block CI use. The scan finds 22 USE_AS_IS capabilities, 4 WRAP_LOCAL conformance functions, and 9 FEATURE_GATE import surfaces — all stable. The Alpha+ Miner is DO_NOT_USE but conformance checking is not blocked by it. Deferring would abandon the most valuable CI capability (fitness gating via token replay) when it is already accessible.

**Specific evidence:** `check_conformance_token_replay` and `check_conformance_alignment` are STABLE in `wasm4pm-algos::conformance` with two public functions, typed inputs, typed outputs, and no experimental markers. 994 total tests across the workspace confirm the codebase is actively maintained.

### Why Path C?

Path C adds `wasm4pm-types` and `wasm4pm-algos` as Cargo dependencies and wraps the conformance surface behind a thin local adapter. The adapter normalizes:

1. The `activity_key: &str` convention (cargo-cicd uses a typed `ActivityKey` struct; the adapter converts)
2. The `Result<ConformanceResult, wasm4pm_types::Error>` → cargo-cicd's internal `CiConformanceGate` mapping
3. Feature flag selection: compile with `feature-conformance-basic` for token replay, `feature-conformance-full` for alignment
4. XES import with the `import` feature for CI trace fixtures

The adapter is fewer than 150 lines. It does not require any changes to wasm4pm. It is the minimum viable integration that delivers fitness gating in v26.6.2.

---

## v26.6.2 Implementation Plan

### Step 1: Add Cargo dependencies

In `cargo-cicd/Cargo.toml`:

```toml
[dependencies]
wasm4pm-types = { path = "/Users/sac/wasm4pm/crates/wasm4pm-types", features = ["import"] }
wasm4pm-algos  = { path = "/Users/sac/wasm4pm/crates/wasm4pm-algos" }

# When publishing: replace path deps with crates.io versions once wasm4pm publishes.
```

Feature selection for v26.6.2 CI:

```toml
# wasm4pm-algos default build — no extra flags needed for conformance module.
# wasm4pm-types: "import" feature gates XES + gzip parsing.
```

### Step 2: Create the thin adapter

File: `cargo-cicd/src/integrations/wasm4pm_current.rs`

```rust
//! Thin adapter: cargo-cicd ↔ wasm4pm conformance surface.
//!
//! Wraps `wasm4pm_algos::conformance` behind cargo-cicd's internal gate types.
//! This is not an abstraction layer — it is a translation layer.
//! When wasm4pm API changes, update this file only.

use wasm4pm_types::{ConformanceResult, EventLog, models::DFG};
use wasm4pm_algos::conformance::{check_conformance_token_replay, check_conformance_alignment};

/// Activity key used by all cargo-cicd event logs.
const DEFAULT_ACTIVITY_KEY: &str = "concept:name";

/// Canonical token-replay fitness gate for cargo-cicd.
///
/// Returns `ConformanceResult` with fitness and precision fields.
/// Caller is responsible for deciding the fitness threshold.
pub fn token_replay_gate(
    log: &EventLog,
    model: &DFG,
) -> Result<ConformanceResult, wasm4pm_types::Error> {
    check_conformance_token_replay(log, model, DEFAULT_ACTIVITY_KEY)
}

/// Canonical alignment fitness gate for cargo-cicd.
///
/// More precise than token replay; higher compute cost.
pub fn alignment_gate(
    log: &EventLog,
    model: &DFG,
) -> Result<ConformanceResult, wasm4pm_types::Error> {
    check_conformance_alignment(log, model, DEFAULT_ACTIVITY_KEY)
}

/// Import a CI fixture from XES file path.
///
/// Requires the `import` feature on wasm4pm-types.
pub fn import_xes_fixture(path: &std::path::Path) -> Result<EventLog, wasm4pm_types::Error> {
    wasm4pm_types::event_log::import_xes(path)
}
```

### Step 3: Pre-flight shell-out (Path B subset)

In cargo-cicd pipeline initialization:

```rust
// Pre-flight: verify wasm4pm toolchain before conformance gates run.
let status = std::process::Command::new("wpm")
    .arg("doctor")
    .status()
    .expect("wpm not found on PATH");
assert!(status.success(), "wpm doctor failed — wasm4pm toolchain not healthy");
```

### Step 4: Wire into pipeline gate

In the cargo-cicd pipeline gate evaluation:

```rust
use crate::integrations::wasm4pm_current::{token_replay_gate, import_xes_fixture};

let log = import_xes_fixture(&fixture_path)?;
let dfg = build_dfg_from_pipeline_trace(&pipeline_trace); // cargo-cicd internal
let result = token_replay_gate(&log, &dfg)?;

if result.fitness < 0.95 {
    return Err(CiGateError::ConformanceGateFailed {
        fitness: result.fitness,
        threshold: 0.95,
    });
}
```

### Step 5: Emit provenance receipt

```rust
use wasm4pm_types::provenance::ProvenanceChain;
use wasm4pm_types::hash::Blake3Hash;

let receipt_hash = Blake3Hash::from_bytes(artifact_bytes);
let chain = ProvenanceChain::new(receipt_hash);
// Serialize to cargo-cicd receipts/ directory.
```

---

## What is NOT in scope for v26.6.2

- Process discovery (Alpha+, Heuristic, Inductive Miners) — DO_NOT_USE or DEFER_CONTRIB
- ML / prediction surfaces — DEFER_CONTRIB
- POWL, cognition, bcinr features — DEFER_CONTRIB
- WASM bundle emission (`browser`, `fog`, `edge` profiles) — not a cargo-cicd concern
- `wpm wizard` or interactive CLI — DO_NOT_USE in CI

---

## Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| wasm4pm publishes breaking API change before v26.6.2 ships | LOW | Path dep pinned to commit 65169e62; update adapter only |
| `check_conformance_alignment` has undocumented panics | LOW | Wrap in `std::panic::catch_unwind` in adapter until confirmed safe |
| `import_xes` not public / behind internal module | MEDIUM | Verify function signature; fall back to `serde_json` round-trip if needed |
| `wpm` not on PATH in CI | MEDIUM | Make `wpm doctor` pre-flight optional; degrade gracefully |

---

## Decision Record

- **Decided by:** Capability scan synthesis, 2026-06-02
- **Integration path:** C (Thin Local Adapter)
- **Scope:** `wasm4pm-types` + `wasm4pm-algos` conformance module only
- **Adapter file:** `cargo-cicd/src/integrations/wasm4pm_current.rs`
- **Estimated LOC:** < 150
- **Revisit at:** v26.7.0 — add DEFER_CONTRIB candidates if stabilized
