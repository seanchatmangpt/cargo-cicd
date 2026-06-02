# WASM4PM Capability Inventory

**Generated:** 2026-06-02
**wasm4pm repo:** /Users/sac/wasm4pm
**wasm4pm commit:** 65169e62 fix(debt): resolve debt markers blocking pre-push hook
**cargo-cicd target version:** v26.6.2

This document enumerates every discovered capability across all 11 area categories.
Verdicts: USE_AS_IS / SHELL_OUT / FILE_EXCHANGE / FEATURE_GATE / WRAP_LOCAL / PATCH_SMALL / DEFER_CONTRIB / DO_NOT_USE

---

## Area 1: CLI Commands (`wpm` binary)

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `wpm doctor` | wasm4pm-cli | CLI command | none | stdout health report | STABLE | Verify toolchain readiness before pipeline runs | SHELL_OUT |
| `wpm wizard` | wasm4pm-cli | CLI command (interactive) | stdin prompts | `.wasm4pm/config.json` | STABLE | Not usable in CI — interactive only | DO_NOT_USE |
| `wpm telco status` | wasm4pm-cli | CLI subcommand | none | stdout telco metrics | EXPERIMENTAL | 34ns architecture metrics — not yet CI-relevant | DEFER_CONTRIB |
| `wpm telco map` | wasm4pm-cli | CLI subcommand | none | stdout routing visualization | EXPERIMENTAL | Visualization only; no machine-readable output | DO_NOT_USE |

---

## Area 2: Crates / Modules

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `wasm4pm-types` | crates/wasm4pm-types | Library crate | — | Core type definitions | STABLE | Foundation for all data exchange; EventLog, PetriNet, OCEL, DFG, ConformanceResult | USE_AS_IS |
| `wasm4pm-algos` (alpha module) | crates/wasm4pm-algos | Library module | EventLog | PetriNet | EXPERIMENTAL (placeholder doctest) | Alpha+ discovery — not production-ready | DO_NOT_USE |
| `wasm4pm-algos` (conformance module) | crates/wasm4pm-algos | Library module | EventLog + DFG | ConformanceResult | STABLE | Token replay + alignment conformance — key for pipeline validation | WRAP_LOCAL |
| `pm-core` (transition_system) | crates/pm-core | Library module | — | transition system types | STABLE (30 tests) | Underlying model for process discovery results | FILE_EXCHANGE |
| `pm-core` (declare) | crates/pm-core | Library module | — | DeclareModel, DeclareConstraint | STABLE (27 tests) | Declare conformance checking | FILE_EXCHANGE |
| `wasm4pm-cognition` | crates/wasm4pm-cognition | Library crate | — | Adversarial / oracle breeding | EXPERIMENTAL (breed tests) | Future conformance oracle use; not v26.6.2 ready | DEFER_CONTRIB |
| `ocel-core` | crates/ocel-core | Library crate | — | OCELEvent, OCELObject, OCEL | STABLE | OCEL v2 object-centric log core; re-exported from wasm4pm-types | USE_AS_IS |
| `prolog8` | crates/prolog8 | Library crate | — | Counterfactual / live AAT tests | EXPERIMENTAL (36 tests) | Not relevant to cargo-cicd v26.6.2 scope | DEFER_CONTRIB |

---

## Area 3: Feature Flags

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `browser` (default, ~2.78MB) | wasm4pm | Cargo feature | — | Full WASM bundle | STABLE | Too large for CI artifact gating; use only for browser targets | DO_NOT_USE |
| `edge` (~1.5MB) | wasm4pm | Cargo feature | — | conformance + discovery + streaming | STABLE | Smallest viable CI bundle with conformance | FEATURE_GATE |
| `iot` (~1MB) | wasm4pm | Cargo feature | — | minimal bundle | STABLE | Minimal footprint CI test | FEATURE_GATE |
| `fog` (~2MB) | wasm4pm | Cargo feature | — | full except POWL | STABLE | Use when POWL not needed | FEATURE_GATE |
| `feature-conformance-basic` | wasm4pm | Cargo capability flag | — | basic conformance surface | STABLE | Token replay only — sufficient for CI fitness gating | FEATURE_GATE |
| `feature-conformance-full` | wasm4pm | Cargo capability flag | — | full alignment conformance | STABLE | Alignment-based CI gating | FEATURE_GATE |
| `feature-discovery-advanced` | wasm4pm | Cargo capability flag | — | advanced discovery | EXPERIMENTAL | Not needed in v26.6.2 | DEFER_CONTRIB |
| `feature-ml` | wasm4pm | Cargo capability flag | — | ML surfaces | EXPERIMENTAL | No CI use case yet | DEFER_CONTRIB |
| `feature-ocel` | wasm4pm | Cargo capability flag | — | OCEL v2 surfaces | STABLE | OCEL log consumption | FEATURE_GATE |
| `feature-powl` | wasm4pm | Cargo capability flag | — | POWL surfaces | EXPERIMENTAL | Not needed in v26.6.2 | DEFER_CONTRIB |
| `feature-streaming-basic` | wasm4pm | Cargo capability flag | — | streaming log input | STABLE | Stream CI event logs | FEATURE_GATE |
| `feature-gpu` | wasm4pm | Cargo capability flag | — | GPU acceleration stub | STUB | No GPU in CI | DO_NOT_USE |
| `feature-rayon` | wasm4pm | Cargo capability flag | — | parallel processing stub | STUB | Stub only | DO_NOT_USE |
| `cognition` | wasm4pm | Cargo capability flag | — | cognition surfaces | EXPERIMENTAL | Not v26.6.2 ready | DEFER_CONTRIB |
| `bcinr` | wasm4pm | Cargo capability flag | — | BCINR algorithm surface | EXPERIMENTAL | Not v26.6.2 ready | DEFER_CONTRIB |

---

## Area 4: File Formats

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| XES XML import | wasm4pm-types (`import` feature) | Format parser | XES XML file | EventLog | STABLE | Standard process log format for CI fixtures | FEATURE_GATE |
| XES gzip import | wasm4pm-types (`import` feature + flate2) | Format parser | .xes.gz file | EventLog | STABLE | Compressed log import in CI | FEATURE_GATE |
| JSON emit (serde) | wasm4pm-types | Format serializer | any core type | JSON bytes | STABLE | Universal serialization for file exchange | USE_AS_IS |
| JSON deserialize (serde) | wasm4pm-types | Format deserializer | JSON bytes | any core type | STABLE | Universal deserialization from file exchange | USE_AS_IS |
| OCEL JSON import | wasm4pm-types | Format parser | OCEL JSON | OCEL struct | STABLE | OCEL v2 unconditional import | USE_AS_IS |
| OCEL NDJSON import | wasm4pm-types | Format parser | OCEL NDJSON | OCEL struct | STABLE | NDJSON streaming OCEL import | USE_AS_IS |
| `.wasm4pm/config.json` | wasm4pm-cli | Config format | wizard prompts | deployment config | STABLE | Could be used to parameterize CI build profiles | FILE_EXCHANGE |

---

## Area 5: Import / Export Surfaces

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `import_xes()` | wasm4pm-types | Import function | XES file path / reader | EventLog | STABLE (requires `import` feature) | Import CI trace fixtures from XES | FEATURE_GATE |
| `EventLog` serde round-trip | wasm4pm-types | Export/import | EventLog | JSON | STABLE | Lossless JSON serialization for file exchange | USE_AS_IS |
| `OCEL` serde round-trip | ocel-core / wasm4pm-types | Export/import | OCEL | JSON | STABLE | OCEL round-trip for cargo-cicd event logs | USE_AS_IS |
| `ProvenanceChain` | wasm4pm-types | Export struct | — | Blake3Hash chain | STABLE | Receipt / provenance chain for pipeline audit trail | USE_AS_IS |
| `Blake3Hash` | wasm4pm-types | Hash type | bytes | hash digest | STABLE | Content-addressing in cargo-cicd artifact receipts | USE_AS_IS |
| `ChoiceGraph` / `ChoiceGraphNode` | wasm4pm-types | Graph type | — | ChoiceGraph | STABLE | Process decision surfaces; not yet CI-relevant | FILE_EXCHANGE |

---

## Area 6: Process Algorithms

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| Alpha+ Miner (`discover_alpha`) | wasm4pm-algos::alpha | Discovery algorithm | EventLog | PetriNet | EXPERIMENTAL (placeholder, O(n+m²), missing parallel/choice handling) | Not production-ready; do not use in v26.6.2 | DO_NOT_USE |
| Token Replay Conformance (`check_conformance_token_replay`) | wasm4pm-algos::conformance | Conformance algorithm | EventLog + DFG | ConformanceResult | STABLE | Core CI fitness gate | WRAP_LOCAL |
| Alignment Conformance (`check_conformance_alignment`) | wasm4pm-algos::conformance | Conformance algorithm | EventLog + DFG | ConformanceResult | STABLE | Alignment-based CI fitness gate | WRAP_LOCAL |
| Heuristic Miner | wasm4pm (feature-flag gated) | Discovery algorithm | EventLog | DFG / PetriNet | STABLE (inferred from feature flag) | Potentially useful for CI model discovery | DEFER_CONTRIB |
| Inductive Miner | wasm4pm (feature-flag gated) | Discovery algorithm | EventLog | ProcessTree | STABLE (inferred) | Potentially useful for CI model discovery | DEFER_CONTRIB |
| Genetic / ILP / A* algorithms | wasm4pm (feature-flag gated) | Discovery algorithms | EventLog | PetriNet | EXPERIMENTAL | No CI use case in v26.6.2 | DO_NOT_USE |
| ACO / PSO / Simulated Annealing | wasm4pm (feature-flag gated) | Metaheuristic algorithms | EventLog | PetriNet | EXPERIMENTAL | No CI use case | DO_NOT_USE |

---

## Area 7: Conformance / Replay

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `ConformanceResult` type | wasm4pm-types | Result struct | — | fitness + precision metrics | STABLE | Core output type for CI gating decisions | USE_AS_IS |
| `TokenReplayResult` type | wasm4pm-types | Result struct | — | token replay trace metrics | STABLE | Per-trace replay fitness for CI diagnostics | USE_AS_IS |
| `AlignmentStep` type | wasm4pm-algos::conformance | Algorithm struct | — | per-step alignment | STABLE | Detailed alignment diagnostics | FILE_EXCHANGE |
| `TraceAlignment` type | wasm4pm-algos::conformance | Algorithm struct | — | per-trace alignment | STABLE | Per-trace alignment for CI conformance reporting | FILE_EXCHANGE |
| `check_conformance_token_replay` | wasm4pm-algos::conformance | Public function | EventLog + DFG + activity_key | ConformanceResult | STABLE | Main CI token replay invocation | WRAP_LOCAL |
| `check_conformance_alignment` | wasm4pm-algos::conformance | Public function | EventLog + DFG + activity_key | ConformanceResult | STABLE | Main CI alignment invocation | WRAP_LOCAL |

---

## Area 8: OCEL / XES / Petri / DFG Types

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `EventLog` | wasm4pm-types | Core type | — | XES-style flat log | STABLE | CI trace carrier | USE_AS_IS |
| `Trace` | wasm4pm-types | Core type | — | ordered event sequence | STABLE | Per-case trace in CI | USE_AS_IS |
| `Event` | wasm4pm-types | Core type | — | event with attributes | STABLE | Individual CI pipeline event | USE_AS_IS |
| `AttributeValue` / `Attributes` | wasm4pm-types | Core type | — | typed attribute map | STABLE | Structured event metadata | USE_AS_IS |
| `XESEditableAttribute` | wasm4pm-types | Core type | — | mutable XES attribute | STABLE | XES round-trip editing | FILE_EXCHANGE |
| `OCEL` | ocel-core / wasm4pm-types | Core type | — | object-centric log | STABLE | OCEL v2 carrier for CI object-centric pipelines | USE_AS_IS |
| `OCELEvent` | ocel-core / wasm4pm-types | Core type | — | OCEL event with E2O links | STABLE | CI OCEL event | USE_AS_IS |
| `OCELObject` | ocel-core / wasm4pm-types | Core type | — | OCEL object with attributes | STABLE | CI OCEL object | USE_AS_IS |
| `PetriNet` | wasm4pm-types | Core type | — | places + transitions + arcs | STABLE | Process model for conformance | USE_AS_IS |
| `Place` / `Transition` / `Arc` | wasm4pm-types | Core types | — | Petri net components | STABLE | Compose Petri nets for CI model checking | USE_AS_IS |
| `FlatIncidenceMatrix` | wasm4pm-types | Core type | — | incidence matrix representation | STABLE | Efficient Petri net computation | FILE_EXCHANGE |
| `DFG` | wasm4pm-types | Core type | — | directly-follows graph | STABLE | Primary model for token replay CI gate | USE_AS_IS |
| `DFGNode` / `DFGEdge` | wasm4pm-types | Core types | — | DFG components | STABLE | DFG construction from CI trace data | USE_AS_IS |
| `DeclareModel` / `DeclareConstraint` | wasm4pm-types | Core types | — | declarative process model | STABLE (27 tests) | Declare-based CI constraint checking | FILE_EXCHANGE |

---

## Area 9: Tests / Fixtures

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| 994 total `#[test]` annotations | workspace | Test suite | — | pass/fail | MIXED | Validates wasm4pm itself; not directly cargo-cicd fixtures | DO_NOT_USE |
| Integration test: `ocel_v2.rs` | workspace | Integration test | OCEL v2 data | pass/fail | STABLE | OCEL v2 integration pattern; study for cargo-cicd OCEL fixtures | DEFER_CONTRIB |
| Integration test: `breed_adversarial.rs` (38 tests) | wasm4pm-cognition | Integration test | adversarial logs | pass/fail | EXPERIMENTAL | Adversarial pattern library — future negative test fixture source | DEFER_CONTRIB |
| Integration test: `breed_oracle_gaps.rs` (31 tests) | wasm4pm-cognition | Integration test | oracle gaps | pass/fail | EXPERIMENTAL | Oracle gap patterns | DEFER_CONTRIB |
| Integration test: `dispatch_smoke.rs` | workspace | Integration test | — | pass/fail | STABLE | Smoke test pattern for cargo-cicd CI invocation reference | PATCH_SMALL |
| Integration test: `aat_live_counterfactual.rs` (36 tests) | prolog8 | Integration test | counterfactual traces | pass/fail | EXPERIMENTAL | Not relevant to v26.6.2 | DEFER_CONTRIB |
| Integration test: `combine_cf_properties.rs` | workspace | Integration test | — | pass/fail | EXPERIMENTAL | Not relevant to v26.6.2 | DEFER_CONTRIB |

---

## Area 10: Docs / Examples

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `wpm doctor` stdout format | wasm4pm-cli | CLI documentation | — | health check prose | STABLE | Reference for cargo-cicd toolchain verification | SHELL_OUT |
| `.wasm4pm/config.json` schema | wasm4pm-cli | Config documentation | — | deployment profile schema | STABLE | Could parameterize cargo-cicd wasm4pm build stage | FILE_EXCHANGE |
| Workspace `Cargo.toml` feature docs | wasm4pm | Inline docs | — | feature flag descriptions | STABLE | Source of truth for feature selection in cargo-cicd | USE_AS_IS |
| Deployment profile sizes (browser=2.78MB, fog=2MB, edge=1.5MB, iot=1MB, mobile=500KB) | wasm4pm | Inline docs | — | bundle size reference | STABLE | CI artifact size gating thresholds | USE_AS_IS |

---

## Area 11: Error / Refusal Model

| Capability | Crate | Type | Input | Output | Stability | cargo-cicd Relevance | Verdict |
|---|---|---|---|---|---|---|---|
| `Error` type | wasm4pm-types | Error enum | — | structured error | STABLE | Typed error propagation in cargo-cicd integration code | USE_AS_IS |
| `Result<T>` alias | wasm4pm-types | Type alias | — | `Result<T, Error>` | STABLE | Standard return type for all wasm4pm API calls | USE_AS_IS |
| `ChoiceGraphError` | wasm4pm-types | Error type | — | choice graph error | STABLE | Graph construction error surface | FILE_EXCHANGE |
| Named refusal reasons (wasm4pm-algos) | wasm4pm-algos | Refusal types | — | named law violation | PARTIAL (inferred from architecture) | Structured refusal model analogous to wasm4pm-compat; verify before use | PATCH_SMALL |

---

## Verdict Summary

| Verdict | Count |
|---|---|
| USE_AS_IS | 22 |
| SHELL_OUT | 2 |
| FILE_EXCHANGE | 11 |
| FEATURE_GATE | 9 |
| WRAP_LOCAL | 4 |
| PATCH_SMALL | 2 |
| DEFER_CONTRIB | 14 |
| DO_NOT_USE | 11 |
| **Total** | **75** |
