# WASM4PM Deferred Contrib Extraction

**Generated:** 2026-06-02
**wasm4pm commit:** 65169e62
**cargo-cicd target version:** v26.6.2

ALL items in this document are DEFERRED. None are to be acted on in v26.6.2.
Revisit at v26.7.0 or later when preconditions are met.

---

## DEFERRED — Do not act on in v26.6.2

---

### DEFER_CONTRIB_001: Heuristic Miner

**What it is:** A frequency-based process discovery algorithm that produces DFG / PetriNet from event logs. More robust than Alpha+ in the presence of noise.

**Source crate:** `wasm4pm` (feature-flag: `heuristic_miner`)

**What cargo-cicd would gain:** Automatic process model discovery from CI pipeline traces, enabling model-vs-log conformance checking without a pre-defined reference model. cargo-cicd could discover its own process model from observed trace data.

**Preconditions for extraction:**
1. Heuristic Miner feature flag must be confirmed stable (currently inferred from feature list only — no direct test coverage verified)
2. Public API must emit `DFG` or `PetriNet` with a stable function signature
3. cargo-cicd must add a model discovery pipeline stage (not in v26.6.2 scope)

**Estimated effort:** 2–3 days (verify API, write adapter, write CI fixture)

**Status:** DEFERRED

---

### DEFER_CONTRIB_002: Inductive Miner

**What it is:** A divide-and-conquer process discovery algorithm that guarantees sound process trees / Petri nets. Produces fitness=1 models by construction.

**Source crate:** `wasm4pm` (feature-flag: `inductive_miner`)

**What cargo-cicd would gain:** Sound model discovery for pipeline conformance checking. The guaranteeed-fitness property makes it ideal for CI: any deviation from the discovered model is a genuine conformance defect, not a model quality issue.

**Preconditions for extraction:**
1. Inductive Miner feature flag must be confirmed stable
2. Public API must emit `PetriNet` or process tree type with stable signature
3. cargo-cicd model discovery stage must exist
4. ProcessTree type (from pm-core) must be confirmed serializable to JSON for file exchange

**Estimated effort:** 3–4 days

**Status:** DEFERRED

---

### DEFER_CONTRIB_003: Adversarial Fixture Library (breed_adversarial.rs)

**What it is:** 38-test adversarial log breeding suite in `wasm4pm-cognition/tests/breed_adversarial.rs`. Generates edge-case event logs that stress-test conformance algorithms (loops, skips, deadlocks, artificial parallelism).

**Source crate:** `crates/wasm4pm-cognition`

**What cargo-cicd would gain:** A library of negative test fixtures for CI conformance gating. Instead of writing adversarial fixtures by hand, cargo-cicd could import the adversarial log patterns as known-bad inputs and verify that conformance gates correctly reject them.

**Preconditions for extraction:**
1. `wasm4pm-cognition` must stabilize its public API (currently experimental)
2. Adversarial fixture format must be confirmed serializable to JSON / XES for file exchange
3. cargo-cicd negative test fixture library must be created as a target location
4. Licensing / extraction agreement with wasm4pm maintainers

**Estimated effort:** 4–5 days

**Status:** DEFERRED

---

### DEFER_CONTRIB_004: Oracle Gap Patterns (breed_oracle_gaps.rs)

**What it is:** 31-test oracle gap breeding suite in `wasm4pm-cognition/tests/breed_oracle_gaps.rs`. Identifies gaps between what a conformance oracle claims and what token replay produces.

**Source crate:** `crates/wasm4pm-cognition`

**What cargo-cicd would gain:** Validated oracle gap patterns for CI conformance calibration. cargo-cicd could use these patterns to detect when its conformance gate is over-reporting or under-reporting fitness.

**Preconditions for extraction:**
1. Same as DEFER_CONTRIB_003 (wasm4pm-cognition stabilization)
2. Oracle gap patterns must be documented as stable canonical test cases, not experimental exploration artifacts
3. cargo-cicd conformance calibration stage must exist

**Estimated effort:** 3–4 days (after DEFER_CONTRIB_003)

**Status:** DEFERRED

---

### DEFER_CONTRIB_005: OCEL v2 Integration Fixture (ocel_v2.rs)

**What it is:** OCEL v2 integration test in `tests/ocel_v2.rs`. Demonstrates the full OCEL v2 object-centric log construction, event-to-object linking, and object-to-object relationship patterns.

**Source crate:** workspace integration tests

**What cargo-cicd would gain:** A reference fixture for OCEL v2 log construction in cargo-cicd. cargo-cicd could adopt the same OCEL v2 construction pattern to emit object-centric pipeline traces for process mining.

**Preconditions for extraction:**
1. OCEL v2 fixture must be separated from wasm4pm-internal test harness
2. cargo-cicd must add OCEL v2 log emission to its pipeline trace exporter
3. The E2O and O2O link construction patterns must be documented as stable API

**Estimated effort:** 2 days

**Status:** DEFERRED

---

### DEFER_CONTRIB_006: Telco Status Structured Output

**What it is:** `wpm telco status` subcommand reporting 34ns architecture metrics. Currently emits prose stdout only.

**Source crate:** `wasm4pm-cli`

**What cargo-cicd would gain:** Machine-readable telco latency metrics for CI performance gating. cargo-cicd could assert 34ns architecture compliance as a CI gate.

**Preconditions for extraction:**
1. `wpm telco status` must emit structured JSON output (currently prose only)
2. The 34ns architecture metrics must be defined as stable thresholds
3. cargo-cicd must have a latency gate stage

**Estimated effort:** 1 day (after wasm4pm-cli adds `--json` flag to `telco status`)

**Status:** DEFERRED

---

### DEFER_CONTRIB_007: Feature-Discovery-Advanced Surface

**What it is:** `feature-discovery-advanced` Cargo feature in the wasm4pm workspace. Exposes advanced process discovery beyond basic DFG/PetriNet.

**Source crate:** `wasm4pm`

**What cargo-cicd would gain:** Access to advanced discovery algorithms for complex pipeline trace analysis.

**Preconditions for extraction:**
1. Feature must be confirmed stable (currently EXPERIMENTAL in scan)
2. API surface must be documented
3. cargo-cicd advanced discovery use case must be defined

**Estimated effort:** Unknown until API is documented

**Status:** DEFERRED

---

### DEFER_CONTRIB_008: POWL Surface (feature-powl)

**What it is:** Partially-ordered workflow language surface, gated by `feature-powl`. Expresses concurrent process behavior beyond strict sequence.

**Source crate:** `wasm4pm`

**What cargo-cicd would gain:** POWL-based process modeling for CI pipelines with genuine parallelism (parallel test stages, concurrent build jobs).

**Preconditions for extraction:**
1. `feature-powl` must be confirmed stable
2. POWL types must have stable serde round-trip
3. wasm4pm-compat POWL type law (from PAPERLAW_CROWN_ALIVE_004) must be confirmed compatible

**Estimated effort:** 5–7 days

**Status:** DEFERRED

---

### DEFER_CONTRIB_009: ML / Prediction Surface (feature-ml)

**What it is:** Machine learning capability surface gated by `feature-ml`, with sub-flags `ml_classify`, `ml_cluster`, `ml_forecast`, `ml_anomaly`. Provides predictive process monitoring capabilities.

**Source crate:** `wasm4pm`

**What cargo-cicd would gain:** Predictive CI failure detection (forecast build failures from partial trace data), anomaly detection in pipeline behavior.

**Preconditions for extraction:**
1. ML feature flags must be confirmed stable (currently EXPERIMENTAL)
2. Training data format must be defined
3. cargo-cicd must have a predictive monitoring use case
4. Model persistence format must be stable

**Estimated effort:** 10+ days

**Status:** DEFERRED

---

### DEFER_CONTRIB_010: Cognition Surface

**What it is:** `cognition` Cargo feature in wasm4pm. Adversarial breeding and oracle reasoning surfaces.

**Source crate:** `wasm4pm` / `crates/wasm4pm-cognition`

**What cargo-cicd would gain:** Automated oracle reasoning for CI conformance decisions. Instead of static thresholds, cognition could reason about whether a conformance result represents a genuine defect.

**Preconditions for extraction:**
1. `cognition` feature must stabilize (currently EXPERIMENTAL)
2. PAPERLAW_CROWN_ALIVE_004 graduation must be confirmed (per memory: sealed with 196 compile-fail + 406 compile-pass receipts)
3. Cognition API must expose a stable `reason_about_conformance()` surface
4. Integration with wasm4pm-compat type law must be validated

**Estimated effort:** Unknown; depends on cognition API stability

**Status:** DEFERRED

---

### DEFER_CONTRIB_011: BCINR Algorithm Surface

**What it is:** `bcinr` Cargo feature. Likely a specialized conformance or discovery algorithm.

**Source crate:** `wasm4pm`

**What cargo-cicd would gain:** Unknown until API is documented.

**Preconditions for extraction:**
1. `bcinr` must be documented (name expansion, algorithm description)
2. Must be confirmed stable
3. cargo-cicd use case must be identified

**Estimated effort:** Unknown

**Status:** DEFERRED

---

### DEFER_CONTRIB_012: Live Counterfactual Testing (aat_live_counterfactual.rs)

**What it is:** 36-test live counterfactual AAT suite in `prolog8/tests/aat_live_counterfactual.rs`. Tests what-if process variants.

**Source crate:** `crates/prolog8`

**What cargo-cicd would gain:** Counterfactual CI testing — "what would have happened if this build step was skipped?" process reasoning.

**Preconditions for extraction:**
1. `prolog8` crate must stabilize
2. Counterfactual reasoning API must be documented
3. cargo-cicd counterfactual CI use case must be defined

**Estimated effort:** Unknown

**Status:** DEFERRED

---

### DEFER_CONTRIB_013: Statistical Library (feature-statrs / feature-hand-rolled-stats)

**What it is:** `feature-statrs` (uses the `statrs` crate) and `feature-hand-rolled-stats` (no external dependency) provide statistical computation for conformance metrics.

**Source crate:** `wasm4pm`

**What cargo-cicd would gain:** Statistical confidence intervals on conformance fitness scores. Instead of a single point estimate, cargo-cicd could gate on "fitness is 0.95 ± 0.02 at 95% confidence."

**Preconditions for extraction:**
1. Statistical API surface must be confirmed public
2. Must integrate with `ConformanceResult` type
3. cargo-cicd statistical gating use case must be defined

**Estimated effort:** 2–3 days

**Status:** DEFERRED

---

### DEFER_CONTRIB_014: Combined Counterfactual Property Tests (combine_cf_properties.rs)

**What it is:** Workspace integration test `tests/combine_cf_properties.rs`. Tests combined counterfactual properties across algorithm variants.

**Source crate:** workspace

**What cargo-cicd would gain:** Property-based test patterns for CI conformance algorithm validation.

**Preconditions for extraction:**
1. Test suite must stabilize
2. Property definitions must be documented
3. cargo-cicd property-based testing framework must exist

**Estimated effort:** 3–4 days

**Status:** DEFERRED

---

## Summary

| ID | Capability | Source Crate | Effort | Precondition Blocker |
|---|---|---|---|---|
| DEFER_CONTRIB_001 | Heuristic Miner | wasm4pm (feature) | 2–3d | API stability confirmation |
| DEFER_CONTRIB_002 | Inductive Miner | wasm4pm (feature) | 3–4d | API stability + ProcessTree serde |
| DEFER_CONTRIB_003 | Adversarial Fixture Library | wasm4pm-cognition | 4–5d | wasm4pm-cognition stabilization |
| DEFER_CONTRIB_004 | Oracle Gap Patterns | wasm4pm-cognition | 3–4d | After DEFER_CONTRIB_003 |
| DEFER_CONTRIB_005 | OCEL v2 Fixture | workspace tests | 2d | OCEL v2 extraction from test harness |
| DEFER_CONTRIB_006 | Telco Structured Output | wasm4pm-cli | 1d | wpm telco --json flag |
| DEFER_CONTRIB_007 | Feature-Discovery-Advanced | wasm4pm | Unknown | API documentation |
| DEFER_CONTRIB_008 | POWL Surface | wasm4pm | 5–7d | feature-powl stability |
| DEFER_CONTRIB_009 | ML / Prediction Surface | wasm4pm | 10+d | ML feature stability |
| DEFER_CONTRIB_010 | Cognition Surface | wasm4pm / wasm4pm-cognition | Unknown | cognition API stability |
| DEFER_CONTRIB_011 | BCINR Algorithm | wasm4pm | Unknown | Documentation |
| DEFER_CONTRIB_012 | Live Counterfactual Testing | prolog8 | Unknown | prolog8 stabilization |
| DEFER_CONTRIB_013 | Statistical Library | wasm4pm | 2–3d | Public statistical API |
| DEFER_CONTRIB_014 | CF Property Tests | workspace | 3–4d | Test suite stabilization |
