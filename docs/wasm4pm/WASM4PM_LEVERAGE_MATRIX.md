# WASM4PM Leverage Matrix

**Generated:** 2026-06-02
**wasm4pm commit:** 65169e62
**cargo-cicd target version:** v26.6.2

Condensed decision table. All 75 capabilities, sorted by verdict from most actionable to least.
Source of truth: WASM4PM_CAPABILITY_INVENTORY.md

---

## USE_AS_IS (22 capabilities)

These capabilities are stable, well-typed, and can be consumed directly in cargo-cicd without wrapping or patching.

| Capability | Crate / Module | Notes |
|---|---|---|
| `wasm4pm-types` library crate | crates/wasm4pm-types | Foundation; add as Cargo dependency |
| `ocel-core` library crate | crates/ocel-core | Re-exported via wasm4pm-types |
| JSON emit via serde | wasm4pm-types | All core types serializable |
| JSON deserialize via serde | wasm4pm-types | All core types deserializable |
| OCEL JSON import (unconditional) | wasm4pm-types | No feature flag required |
| OCEL NDJSON import (unconditional) | wasm4pm-types | Streaming-friendly |
| `EventLog` type | wasm4pm-types | CI trace carrier |
| `Trace` type | wasm4pm-types | Per-case container |
| `Event` type | wasm4pm-types | Individual pipeline event |
| `AttributeValue` / `Attributes` | wasm4pm-types | Typed attribute map |
| `OCEL` type | ocel-core / wasm4pm-types | Object-centric log carrier |
| `OCELEvent` type | ocel-core / wasm4pm-types | OCEL event with E2O links |
| `OCELObject` type | ocel-core / wasm4pm-types | OCEL object with attributes |
| `PetriNet` type | wasm4pm-types | Process model |
| `Place` / `Transition` / `Arc` types | wasm4pm-types | Petri net components |
| `DFG` type | wasm4pm-types | Directly-follows graph |
| `DFGNode` / `DFGEdge` types | wasm4pm-types | DFG construction |
| `ConformanceResult` type | wasm4pm-types | CI fitness gate output |
| `TokenReplayResult` type | wasm4pm-types | Per-trace replay metrics |
| `ProvenanceChain` type | wasm4pm-types | Pipeline audit receipt chain |
| `Blake3Hash` type | wasm4pm-types | Content-addressed artifact receipts |
| `Error` / `Result<T>` types | wasm4pm-types | Typed error propagation |
| Deployment profile sizes (docs) | wasm4pm workspace | CI artifact size thresholds |
| Workspace feature flag docs | wasm4pm workspace | Feature selection reference |

---

## SHELL_OUT (2 capabilities)

Invoke via `std::process::Command` from cargo-cicd. No library coupling needed.

| Capability | Crate / Module | Invocation | Notes |
|---|---|---|---|
| `wpm doctor` | wasm4pm-cli | `wpm doctor` | Toolchain health check before pipeline run |
| `wpm doctor` (verbose) | wasm4pm-cli | `wpm --verbose doctor` | Verbose health diagnostic |

---

## FILE_EXCHANGE (11 capabilities)

cargo-cicd emits files in these formats; wasm4pm consumes them (or vice versa). No shared library dependency needed in v26.6.2.

| Capability | Crate / Module | Exchange Direction | Format |
|---|---|---|---|
| `pm-core` transition_system types | crates/pm-core | cargo-cicd → wasm4pm | JSON |
| `pm-core` declare types | crates/pm-core | cargo-cicd → wasm4pm | JSON |
| `XESEditableAttribute` | wasm4pm-types | bidirectional | XES/JSON |
| `FlatIncidenceMatrix` | wasm4pm-types | cargo-cicd → wasm4pm | JSON |
| `AlignmentStep` | wasm4pm-algos::conformance | wasm4pm → cargo-cicd | JSON |
| `TraceAlignment` | wasm4pm-algos::conformance | wasm4pm → cargo-cicd | JSON |
| `DeclareModel` / `DeclareConstraint` | wasm4pm-types | cargo-cicd → wasm4pm | JSON |
| `ChoiceGraph` / `ChoiceGraphNode` | wasm4pm-types | wasm4pm → cargo-cicd | JSON |
| `ChoiceGraphError` | wasm4pm-types | wasm4pm → cargo-cicd | JSON |
| `.wasm4pm/config.json` | wasm4pm-cli | cargo-cicd writes, wasm4pm-cli reads | JSON |
| `wpm doctor` stdout format | wasm4pm-cli | wasm4pm → cargo-cicd (parse stdout) | text |

---

## FEATURE_GATE (9 capabilities)

Require selecting the correct Cargo feature before use. Stable once gated correctly.

| Capability | Crate / Module | Required Feature | Notes |
|---|---|---|---|
| XES XML import (`import_xes()`) | wasm4pm-types | `import` | Gate CI fixture imports |
| XES gzip import | wasm4pm-types | `import` + flate2 | Compressed log import |
| `edge` deployment profile (~1.5MB) | wasm4pm | `edge` | Smallest viable conformance bundle |
| `iot` deployment profile (~1MB) | wasm4pm | `iot` | Minimal footprint CI |
| `fog` deployment profile (~2MB) | wasm4pm | `fog` | Full without POWL |
| `feature-conformance-basic` | wasm4pm | `feature-conformance-basic` | Token replay CI gate |
| `feature-conformance-full` | wasm4pm | `feature-conformance-full` | Alignment CI gate |
| `feature-ocel` | wasm4pm | `feature-ocel` | OCEL v2 surfaces |
| `feature-streaming-basic` | wasm4pm | `feature-streaming-basic` | Streaming log input |

---

## WRAP_LOCAL (4 capabilities)

Stable API but requires a thin local adapter in `cargo-cicd/src/integrations/` to handle the `activity_key` parameter convention and `Result` mapping.

| Capability | Crate / Module | Wrap Reason | Location |
|---|---|---|---|
| `check_conformance_token_replay` | wasm4pm-algos::conformance | Needs `activity_key` param normalization + CI `ConformanceResult` mapping | `src/integrations/wasm4pm_current.rs` |
| `check_conformance_alignment` | wasm4pm-algos::conformance | Same as token replay | `src/integrations/wasm4pm_current.rs` |
| `wasm4pm-algos` conformance module | crates/wasm4pm-algos | Activity key convention + error type bridging | `src/integrations/wasm4pm_current.rs` |
| `wasm4pm-algos` (DFG model input) | crates/wasm4pm-algos | DFG construction from CI trace data requires local builder | `src/integrations/wasm4pm_current.rs` |

---

## PATCH_SMALL (2 capabilities)

Stable and relevant, but require a small targeted patch to be usable in cargo-cicd.

| Capability | Crate / Module | Patch Required | Notes |
|---|---|---|---|
| `dispatch_smoke.rs` integration test | workspace | Extract as fixture template; remove wasm4pm-internal imports | Study pattern for cargo-cicd CI invocation |
| Named refusal reasons (wasm4pm-algos) | wasm4pm-algos | Verify named reason types exist; add `#[derive(Debug)]` if missing | Structured refusal model for CI diagnostics |

---

## DEFER_CONTRIB (14 capabilities)

Potentially valuable. Not ready for v26.6.2. Revisit after stabilization.

| Capability | Crate / Module | Why Deferred | Revisit Condition |
|---|---|---|---|
| `wpm telco status` | wasm4pm-cli | Experimental; no machine-readable output | When telco emits structured JSON |
| Heuristic Miner | wasm4pm (feature-gated) | No CI model discovery use case yet | When cargo-cicd adds model discovery stage |
| Inductive Miner | wasm4pm (feature-gated) | Same as heuristic miner | Same condition |
| `feature-discovery-advanced` | wasm4pm | No CI use case in v26.6.2 | When CI discovery needed |
| `feature-ml` | wasm4pm | No CI ML use case yet | When CI prediction stage added |
| `feature-powl` | wasm4pm | Experimental POWL surfaces | When POWL stable |
| `cognition` feature | wasm4pm | Experimental | When PAPERLAW_CROWN_ALIVE_004 graduates |
| `bcinr` feature | wasm4pm | Experimental | When stabilized |
| `wasm4pm-cognition` crate | crates/wasm4pm-cognition | Adversarial breed tests not yet CI-applicable | When negative test fixture library stabilizes |
| `prolog8` crate | crates/prolog8 | Counterfactual / AAT tests out of v26.6.2 scope | When live counterfactual CI gate added |
| Integration test: `ocel_v2.rs` | workspace | OCEL v2 fixture pattern; study then adapt | Adapt for cargo-cicd OCEL fixture library |
| Integration test: `breed_adversarial.rs` | wasm4pm-cognition | Adversarial log pattern library | When negative test fixtures needed |
| Integration test: `breed_oracle_gaps.rs` | wasm4pm-cognition | Oracle gap patterns | Same condition |
| Integration test: `aat_live_counterfactual.rs` | prolog8 | Out of scope | When counterfactual CI gate added |
| Integration test: `combine_cf_properties.rs` | workspace | Out of scope | When property-based CI testing added |

---

## DO_NOT_USE (11 capabilities)

Either interactive-only, experimental with known gaps, stub, or not CI-relevant.

| Capability | Crate / Module | Reason |
|---|---|---|
| `wpm wizard` | wasm4pm-cli | Interactive stdin — cannot run in CI |
| `wpm telco map` | wasm4pm-cli | Visualization only; no machine-readable output |
| `browser` feature (~2.78MB) | wasm4pm | Too large for CI; browser target only |
| `feature-gpu` | wasm4pm | GPU stub — no GPU in CI |
| `feature-rayon` | wasm4pm | Parallel processing stub only |
| Alpha+ Miner (`discover_alpha`) | wasm4pm-algos::alpha | Experimental placeholder; missing parallel/choice handling |
| Genetic algorithm | wasm4pm (feature-gated) | Experimental; no CI use case |
| ILP algorithm | wasm4pm (feature-gated) | Experimental; no CI use case |
| A* algorithm | wasm4pm (feature-gated) | Experimental; no CI use case |
| ACO / PSO / Simulated Annealing | wasm4pm (feature-gated) | Experimental metaheuristics; no CI use case |
| 994 `#[test]` suite (raw) | workspace | wasm4pm internal tests; not cargo-cicd fixtures |
