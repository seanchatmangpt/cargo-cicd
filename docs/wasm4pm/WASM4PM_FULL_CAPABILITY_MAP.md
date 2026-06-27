# wasm4pm Full Capability Map

**Date:** 2026-06-02
**Doctrine:** Inventory first. Classify second. Leverage third.
**Scope:** All crates in the wasm4pm ecosystem reachable from cargo-cicd, plus the `wpm` CLI command surface.

---

## Leverage Classification Key

| Label | Meaning |
|---|---|
| `USE_AS_IS` | Depend directly; call the public API without modification |
| `WRAP_LOCAL` | Write a thin adapter crate/module; the underlying capability is solid but the API shape requires translation |
| `FEATURE_GATE` | Usable only after explicitly enabling a Cargo feature flag; default build omits it |
| `SHELL_OUT` | Call as a subprocess; no library coupling possible or advisable |
| `FILE_EXCHANGE` | Integrate via file I/O (input files / output files); no direct API coupling |
| `PATCH_SMALL` | Requires a small upstream or local patch before use; single-session fix |
| `DEFER_CONTRIB` | Not usable now; requires upstream work before integration is safe |
| `DO_NOT_USE` | Broken, misleadingly named, or structurally incompatible; exclude from integration |

---

## Summary Table

| Crate / Command | Purpose | Overall Leverage | Recommended Use |
|---|---|---|---|
| `wasm4pm-types` | Canonical binary data structures (event logs, Petri nets, conformance results, provenance) | `WRAP_LOCAL` | Direct path dep for stable re-exported types; thin re-exports for dense_kernel primitives |
| `wasm4pm-algos` | Branchless process mining algorithms (discovery, conformance, receipt verification) | `WRAP_LOCAL` | Thin adapter for DFG discovery, DFG token replay, BLAKE3 receipt verification |
| `wasm4pm-utils` | Zero-allocation utility primitives (indexing, bitsets, hash tables, SCC, MCTS, Jaccard) | `WRAP_LOCAL` | Pure-function primitives direct; newtype wrappers for KBitSet/DenseIndex |
| `ocel-core` | OCEL 2.0 in-memory type system, OCEDO/OCPQ accessors, flattening, ND-JSON intake | `USE_AS_IS` | Direct path dep; call validate, flatten, NDJsonStream directly |
| `pm-core` | Formally-grounded process mining type library (no_std + alloc) | `WRAP_LOCAL` | Adapter crate normalizing ActivityName/DurationNs/Frequency newtypes across modules |
| `wasm4pm-cli` | Official `wpm` binary CLI (discovery, conformance, receipts, SPC, oracle, wizard) | `SHELL_OUT` | Shell out to receipt subcommand family with `--format json`; hard exit-code gates |
| `miniml-core` | WASM-targeted classical ML, optimization, process-sequence analytics | `WRAP_LOCAL` | Direct imports for `optimization::*`; `_impl` adapters for regression/classification |
| `wasm4pm-cognition` | AutoSystems cognition kernel (AI breeds, BLAKE3 receipt chain, adversarial detectors) | `WRAP_LOCAL` | EvidenceSource adapter; direct Pareto/cost-law calls; run_contract for proof gates |
| `prolog8` | Byte-capped Datalog/Prolog proof engine (arity/body/var ≤ 8) | `WRAP_LOCAL` | CognitionAdapter struct owning catalog construction and FactBlock8 translation |
| `wasm4pm-macros` | Proc-macro attributes for POWL-model conformance test instrumentation | `WRAP_LOCAL` | Path/git dev-dep under nightly-2026-04-15; feature-gated wrapper re-exporting attributes |
| `ocpq` | OCPQ runtime: binding boxes, query trees, BASIC predicates over OCEL logs | `WRAP_LOCAL` | `ocpq_eval_json()` direct; binding-count guard before `BindingBox::output()` on large logs |
| `tps-metrics` | TPS metrics (takt, lead time, mura, value stream, andon) from git history | `SHELL_OUT` | Subprocess with `--json`; suppress `error_rate_per_kloc` until backend integrated |
| `wasm4pm` | Core WASM cdylib/rlib (discovery, conformance, POWL, OCEL binary, graduation bridge) | `WRAP_LOCAL` | Vendor/workspace-join for path dep; wrap mining + conformance + PowerMiner APIs |

---

## Crate-by-Crate Analysis

---

### wasm4pm-types

**Purpose:** Defines canonical binary data structures shared across all wasm4pm crates: event logs, process models, conformance results, provenance chains, and hashing primitives.

**Nightly required:** No
**Key blockers:**
- Path dependency on `ocel-core` at `../ocel-core` — external crates must be in the same workspace or vendor the path dep
- `import` feature flag required for XES and OCEL parsing; not enabled by default
- `version.workspace = true` — exact version only determinable from root `Cargo.toml`

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| XES event log import via streaming parser | `import::xes::XesParser`, streaming API | `FEATURE_GATE` | Requires `features = ["import"]`; also pulls in `quick-xml` and `flate2`; ocel-core transitive path dep must be resolved |
| OCEL 2.0 JSON and NDJSON import | `import::ocel::*`, re-exports from ocel-core | `FEATURE_GATE` | ocel.rs shim re-exports ocel-core types; transitive path dep applies |
| Petri net representation with incidence matrix | `PetriNet`, `Place`, `Transition`, `Arc`, `FlatIncidenceMatrix` | `USE_AS_IS` | Fully re-exported at crate root; NaN-safe arithmetic; cached incidence matrix; MDL scoring; no nightly |
| Directly-follows graph construction | `DFG`, `DFGNode`, `DFGEdge` | `USE_AS_IS` | Minimal, complete, serde-derived; re-exported at crate root |
| DECLARE constraint model | `DeclareModel`, `DeclareConstraint` | `USE_AS_IS` | Simple, complete, serde-derived; re-exported at crate root |
| Token replay and conformance result types | `ConformanceResult`, `TokenReplayResult` | `USE_AS_IS` | NaN-safe, regression-hardened (PR #54), builder pattern; fully re-exported |
| BLAKE3 provenance chain | `ProvenanceChain`, `Blake3Hash`, `blake3_hex()`, `blake3_combined()`, `canonical_json()` | `USE_AS_IS` | Highest-confidence surface; complete, tested, regression-hardened (PR #54, PR #66) |
| Dense activity index (FNV1a + K-bit set) | `dense_kernel::DenseIndex`, `dense_kernel::KBitSet`, `dense_kernel::PackedKeyTable` | `WRAP_LOCAL` | Not re-exported at crate root; access via `wasm4pm_types::dense_kernel::*`; add thin re-export wrapper at crate root |
| ChoiceGraph (non-block-structured decision DAG) | `ChoiceGraph`, `ChoiceGraphNode`, `ChoiceGraphError` | `USE_AS_IS` | Re-exported at crate root; DFS acyclicity + Start/End path reachability checks; paper-grounded (arXiv:2505.07052 Def. 1) |
| Powl8Op and FieldMask primitives | `powl8_op::Powl8Op`, `mask::FieldMask` | `WRAP_LOCAL` | Not re-exported at crate root; in-development status; `Powl8Op` has 9 discriminants + `TryFrom<u8>`; `FieldMask` has PR #71 correctness fix (1u64 shift) |

**Recommended integration path:** Add `wasm4pm-types` as a workspace path dependency. Depend directly on the stable re-exported surface (`ConformanceResult`, `TokenReplayResult`, `ProvenanceChain`, `Blake3Hash`, `PetriNet`, `DFG`, `DeclareModel`, `ChoiceGraph`). Enable the `import` feature only for ingestion code paths. Add local re-exports for `dense_kernel` and `powl8_op` until those are promoted to the crate root API.

---

### wasm4pm-algos

**Purpose:** High-performance, branchless process mining algorithm implementations optimized for minimal branch misses, predictable WASM latency, and deterministic output.

**Nightly required:** Yes — workspace pins `nightly-2026-04-15` (driven by `generic_const_exprs`)
**Key blockers:**
- Nightly toolchain required for all consumers
- `wasm4pm-types` and `ocel-core` are path/workspace deps; not on crates.io
- `prefix_conformance` D1-D5 refusal codes hardcoded to CodeManufactory-specific activity names
- Alignment conformance uses linearised BFS from `source`/`start`-named places only — not full reachability
- `#![allow(clippy::all, unused_variables, unused_imports, dead_code)]` across the crate — WIP state

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| DFG discovery (single-pass columnar) | `dfg::discover_dfg(log: &EventLog, activity_key: &str) -> Result<DFG>` | `WRAP_LOCAL` | Tested; returns `DFG`; no provenance emitted — caller assembles `ProvenanceChain`; nightly workspace blocker |
| Alpha miner Petri net discovery | `alpha::discover_alpha_miner(log: &EventLog) -> Result<PetriNet>` | `PATCH_SMALL` | Alpha+ approximation; no loop handling; implicit places approximated; small patch to remove implicit place limitation before trusting Petri net soundness |
| Heuristic miner DFG discovery | `heuristic::discover_heuristic_miner_dfg(...)` | `DO_NOT_USE` | Functionally identical to basic DFG miner — no dependency thresholds, no noise filtering, no HeuristicsNet output; misleadingly named; defer until HeuristicsNet layer is implemented |
| Streaming DFG discovery (`discover_streaming_dfg`) | `streaming::discover_streaming_dfg(...)` | `DO_NOT_USE` | Same single-pass DFG as `dfg.rs`; no incremental/streaming state; no value over direct DFG call; misleadingly named |
| Token replay conformance (DFG, Rozinat–van der Aalst) | `token_replay::replay_log(log: &EventLog, dfg: &DFG) -> Result<ConformanceResult>` | `WRAP_LOCAL` | Tested; returns `ConformanceResult`; DFG-only (not Petri net); wrap to translate event types and assemble `ProvenanceChain` |
| Alignment-based conformance via Dijkstra | `alignment::check_conformance(log: &EventLog, net: &PetriNet) -> Result<Vec<AlignmentResult>>` | `DO_NOT_USE` | Linearised BFS on `source`/`start`-named places only — not full reachability; nets without those naming conventions return empty model sequences and trivially pass all traces; not usable alignment conformance |
| Streaming prefix-conformance oracle (D1–D8 refusal codes) | `prefix_conformance::PrefixOracle`, `PrefixOracle::check(prefix)` | `FEATURE_GATE` | D1–D5 hardcoded to CodeManufactory activity vocabulary (`ReceiptEmitted`, `GatePassed`, `DiagnosticRaised`, `RouteSelected`, `RepairApplied`, `RepairSuggested`, `RefusalEmitted`); only usable if caller activity vocabulary matches exactly |
| BLAKE3 receipt verification for OCEL 2.0 batch provenance | `receipt::verify_receipt(envelope: &Value) -> (VerificationResult, String, String)` | `WRAP_LOCAL` | Present and tested; concrete function, no public trait; wrap to adapt JSON envelope format to cargo-cicd receipt schema; most stable capability in this crate |

**Recommended integration path:** Thin local adapter crate (`WRAP_LOCAL`) translating cargo-cicd event types, assembling `ProvenanceChain` manually, exposing only DFG discovery, DFG token replay conformance, and BLAKE3 receipt verification. Exclude heuristic miner, streaming DFG, and prefix conformance oracle until generalised or nightly constraint resolved.

---

### wasm4pm-utils

**Purpose:** Low-level, zero-allocation utility primitives for the wasm4pm platform.

**Nightly required:** No
**Key blockers:**
- Path dependency only (`path = 'crates/wasm4pm-utils'`) — not on crates.io
- `wasm_bindgen` and `serde_wasm_bindgen` referenced under `#[cfg(target_arch = "wasm32")]` but absent from `Cargo.toml [dependencies]` — wasm32 builds fail
- `PackedKeyTable` silent correctness trap: `#[serde(skip)]` on `indices` field means deserialized table returns `None` for all `get()` until next `insert()`
- No public traits exported

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| Dense FNV-1a symbol index | `DenseIndex::compile(symbols: &[&str]) -> Result<DenseIndex>` | `WRAP_LOCAL` | Implemented, deterministic, `Serialize/Deserialize`; thin workspace re-export or path reference required |
| Const-generic bitsets | `KBitSet<WORDS>` with `and()`, `or()`, `not()`, `set()`, `contains()`, `missing_count()` | `WRAP_LOCAL` | `Copy + Serialize/Deserialize`; all bitwise ops present; no public trait — wrap concrete `KBitSet<N>` types to isolate const-generic WORDS parameter |
| Open-addressing hash table | `PackedKeyTable<K,V>`, `StaticPackedKeyTable<K,V,N>` | `PATCH_SMALL` | `PackedKeyTable`: rebuild indices post-deserialization (2-line patch: `#[serde(default)]` + manual `Deserialize` impl calling `rebuild_indices_if_needed()`). `StaticPackedKeyTable`: missing `Serialize/Deserialize` — add thin newtype wrapper with derives |
| Tarjan SCC decomposition | `compute_sccs_generic(adj: &[KBitSet<W>]) -> Vec<Vec<usize>>`, `compute_sccs_branchless(...)` | `WRAP_LOCAL` | Both implementations complete and tested for parity; adapter needed to convert cargo-cicd graph types to `KBitSet` adjacency form |
| UCT/MCTS node scoring | `monte_carlo_tree_search_mcts(wins: u64, visits: u64, parent_visits: u64, c: u64) -> u64` | `USE_AS_IS` | Pure function, no allocation, NaN-safe, fully tested; no type dependencies |
| YAWL OR-join gate | `synchronizing_merge_wcp37(token_mask: u64, join_mask: u64) -> u64` | `USE_AS_IS` | Pure branchless u64 function; no allocations; no type dependencies |
| Jaccard similarity | `jaccard_u64_slices(a: &[u64], b: &[u64]) -> f32` | `USE_AS_IS` | Pure slice function; zero allocation; no type coupling |
| Xorshift64* adversarial perturbation | `Perturbator::new(seed: u64)` implementing `Iterator<Item=u64>` | `USE_AS_IS` | Self-contained, deterministic; directly usable for adversarial test injection |
| `to_js_str` (wasm32 JS serialization) | `to_js_str(...)` under `#[cfg(target_arch = "wasm32")]` | `DO_NOT_USE` | `wasm_bindgen` and `serde_wasm_bindgen` absent from `Cargo.toml` — wasm32 builds fail; irrelevant for native cargo-cicd targets |

**Recommended integration path:** Join or mirror the wasm4pm workspace (or await crates.io publication). Call `monte_carlo_tree_search_mcts`, `synchronizing_merge_wcp37`, `jaccard_u64_slices`, and `Perturbator` directly. Wrap `KBitSet`, `DenseIndex`, and `StaticPackedKeyTable` in thin newtype adapters. Patch `PackedKeyTable`'s post-deserialization index rebuild before any read-heavy use.

---

### ocel-core

**Purpose:** Canonical OCEL 2.0 in-memory type system, OCEDO/OCPQ formal accessors, OCEL-v2 validation, object-type flattening, and streaming ND-JSON intake.

**Nightly required:** No
**Key blockers:**
- Path dependency only (`path = '/Users/sac/wasm4pm/crates/ocel-core'`) — not on crates.io
- Date-based versioning (`26.5.30`) may cause issues with cargo's semver resolver
- `OCELAttributeValue` uses `#[serde(untagged)]` — known integer/float ambiguity on deserialization (acceptable for OCEL 2.0 payloads where type context is available)

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| OCEL 2.0 in-memory type model | `OCEL`, `OCELEvent`, `OCELObject`, `OCELRelationship`, `OCELObjectAttribute`, `OCELAttributeValue`, `ObjectTypeCardinality` | `USE_AS_IS` | All types implemented, serde-derived, stable; add as path dep and import directly |
| OCEDO formal accessors | `OCEL::eval(e)`, `OCEL::oaval(o,t)`, `OCEL::e2o()`, `OCEL::o2o()` | `USE_AS_IS` | All four accessors implemented as `#[must_use]` methods; paper-grounded (Latif et al., OCEDO Fig. 1, OCPQ Def. 2) |
| OCPQ Def. 2 invariant validation | `validate::validate(ocel: &OCEL, card: &[ObjectTypeCardinality]) -> ValidationReport` | `USE_AS_IS` | Structured `ValidationReport` with machine-stable error codes: `E2O_EMPTY`, `DANGLING_E2O`, `DANGLING_O2O`, `UNDECLARED_EVENT_TYPE`, `DUPLICATE_ID`, `CARDINALITY_MIN`, `CARDINALITY_MAX`; serde-serializable |
| Object-type cardinality enforcement | `ObjectTypeCardinality` fields `min_count`/`max_count`, `created_by`/`terminated_by` | `USE_AS_IS` | Count bounds enforced; note: `validate()` does not enforce lifecycle event sequencing (only count bounds checked); `PATCH_SMALL` if full lifecycle ordering required |
| OCEL-to-XES flattening | `flatten::flatten(ocel: &OCEL, object_type: &str) -> Result<FlatLog, String>` | `USE_AS_IS` | Deterministic `(time, event_id)` ordering; `FlatCase` carries trace labels and `event_ids` for replay provenance; no stubs |
| Streaming ND-JSON intake | `intake::NDJsonStream<R: BufRead>` implementing `Iterator<Item=Result<OCELRecord, String>>` | `USE_AS_IS` | `ExtractionPlan` allowlists event types, object types, qualifiers; referential integrity enforced during parse; `#[serde(untagged)]` on `OCELRecord` is known risk, not a blocker |
| Time-varying object attribute projection | `OCEL::oaval(object_id, attr_name, timestamp)`, `OCEL::object_attr_timeline(object_id, attr_name)` | `USE_AS_IS` | Correct temporal projection: latest value with timestamp ≤ t; full timeline accessor available |
| JSON serialization/deserialization | `#[derive(Serialize, Deserialize)]` on all public types | `USE_AS_IS` | All public structs derive via stable serde 1.0 |

**Recommended integration path:** Add `ocel-core` as a direct path dependency. Call `validate::validate`, `flatten::flatten`, and `intake::NDJsonStream` directly as library functions. No shim, wrapper, or CLI intermediary needed.

---

### pm-core

**Purpose:** `no_std + alloc`, zero-algorithm type library providing formally-grounded process mining data structures derived directly from paper definitions. Intended as a shared dep for downstream algorithm crates.

**Nightly required:** No
**Key blockers:**
- Path dependency only (`path = "../pm-core"`) — not on crates.io
- Duplicate newtype definitions across modules: `ActivityName`, `Frequency`, `DurationNs`, `TimestampNs` defined independently in `primitives`, `log`, `petri_net`, `heuristics_net`, `performance`, `transition_system`
- `log` module uses `String` aliases (`ActivityName = String`) rather than `primitives` newtypes — interop requires explicit conversion
- Pre-1.0 (`0.1.0`) — API may change without semver guarantees
- No public traits — no `EventLogTrait`, no `PetriNetTrait`

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| XES IEEE 1849-2016 event log hierarchy | `XesLog`, `XesTrace`, `XesEvent`, `AttributeMap` | `WRAP_LOCAL` | `log` module uses `ActivityName = String`; adapter needed to normalize `ActivityName` before passing to `petri_net` or `dfg` types |
| OCEL 2.0 object-centric event log model | `ObjectCentricEventLog`, `E2ORelation`, `O2ORelation` | `WRAP_LOCAL` | Formally grounded (Van der Aalst spec); wrap to insulate from pre-1.0 API churn on relation types |
| Petri net structure | `PetriNet`, `Place`, `Transition`, `Arc`, `Marking` (Murata 1989) | `WRAP_LOCAL` | Duplicate `ActivityName`/`Frequency` newtypes conflict with `log` module aliases; wrapping isolates cross-module type mismatches |
| Directly-Follows Graph | `DirectlyFollowsGraph`, `DFGEdge`, frequency tracking | `PATCH_SMALL` | Small patch to unify `ActivityName` at `primitives` level (or add `From/Into` impls) eliminates conversion boilerplate for every DFG construction call |
| Block-structured process tree | `ProcessTree`, `ProcessTreeNode`, operators `{sequence, exclusive-choice, parallel, loop}` (Leemans 2013) | `USE_AS_IS` | Self-contained; no upstream coupling to log/petri_net type-alias inconsistency |
| Optimal trace alignment types | `AlignmentMove`, `AlignmentCost`, `Alignment` (Adriansyah 2014) | `WRAP_LOCAL` | Pure data structures; no public trait means downstream algorithm crates couple to concrete types — add trait facade |
| Four-dimensional quality model | `QualityModel { fitness, precision, generalization, simplicity }`, `ETConformancePrecision` (Munoz-Gama & Carmona 2010) | `USE_AS_IS` | Leaf value types; no cross-module type conflicts |
| DECLARE constraint language | `DeclareModel`, `DeclareTemplate` | `USE_AS_IS` | Self-contained; no cross-module newtype conflicts |
| Heuristics Net | `HeuristicsNet`, dependency scores, input/output bindings (Weijters & van der Aalst 2003) | `WRAP_LOCAL` | Defines its own `ActivityName` and `Frequency` newtypes shadowing `primitives` — wrap to normalize before composing with `petri_net` or `log` types |
| Social network mining types | `HandoverNetwork`, `WorkingTogetherNetwork` | `DEFER_CONTRIB` | Peripheral to core process-intelligence pipeline; structurally stable but no near-term integration value |
| Performance spectrum | `PerformanceSpectrum`, `SegmentDuration` (Denisov et al. 2018) | `WRAP_LOCAL` | `performance` module defines its own `DurationNs` newtype shadowing `primitives` — wrap to normalize duration representation |
| Log skeleton | `LogSkeleton` (Verbeek 2021) | `USE_AS_IS` | Compact constraint records; no cross-module type conflicts |
| `no_std + alloc` embeddability | crate-level attribute | `USE_AS_IS` | WASM targets depend directly without feature gating; primary design goal of the crate |
| Serde serialization | optional `serde` feature | `FEATURE_GATE` | Enable explicitly: `pm-core = { path = "../pm-core", features = ["serde"] }`; do not rely on transitive enablement |

**Recommended integration path:** Depend on `pm-core` via a `WRAP_LOCAL` adapter crate that normalizes `ActivityName`, `DurationNs`, and `Frequency` newtype variants into a single canonical representation, insulating downstream algorithm crates from pre-1.0 instability.

---

### wasm4pm-cli (wpm binary)

**Purpose:** Official `wpm` process mining CLI binary. Exposes process discovery, conformance checking, receipt auditing, OCEL validation, SPC, lean, telco, agent, oracle, and wizard subcommands.

**Nightly required:** Yes — `rust-toolchain.toml` pins `nightly-2026-04-15`
**Key blockers:**
- Binary crate only; `lib.rs` re-exports are minimal utility types (`Config`, `Io`, `Table`, `Wasm4pmError`, `Report`, `ContextExt`) — depending on it as a library couples cargo-cicd to the full CLI dependency graph including `dialoguer`, `indicatif`, `colored`
- Three unpublished path dependencies: `wasm4pm-algos`, `wasm4pm` (with `cloud` feature), `ocel-core`
- Several subcommands have placeholder logic (see notes below)

**Capability inventory:**

| Capability | Command | Leverage | Notes |
|---|---|---|---|
| Heuristic process model discovery | `wpm mining discover <log>` | `SHELL_OUT` | DFG output to stdout; only heuristic algo wired; inductive bails explicitly (`anyhow::bail!`); no `--format json` yet |
| Conformance checking against DFG/PNML models | `wpm mining conformance <log> <model>` | `PATCH_SMALL` | `PATCH_SMALL`: model file argument is ignored — loaded as `DFG::new()` (empty mock); wire actual DFG/PNML deserialization from model path before use |
| Receipt auditing against Adversarial Ingress Gates | `wpm receipt doctor --format json --strict <receipt.json>` | `USE_AS_IS` | Implemented; exits non-zero on refused receipts; `--audience producer|operator|ci` flag; direct CI gate |
| OCEL 2.0 structural validation | `wpm receipt verify-ocel2 <receipt.json>` | `SHELL_OUT` | Implemented and file-driven; exit code gates pipeline |
| OCEL 2.0 canonicalization | `wpm receipt canonicalize-ocel2 <receipt.json>` | `SHELL_OUT` | Implemented; emits JSON to stdout |
| Fixture mutation detection | `wpm receipt detect-fixture-mutation <receipt.json>` | `SHELL_OUT` | Implemented; exits non-zero on detected mutation |
| Boundary evidence verification | `wpm receipt verify-boundary-evidence <receipt.json>` | `SHELL_OUT` | Implemented; exits non-zero on failure |
| Proof-class verification | `wpm receipt verify-proof-class <receipt.json>` | `SHELL_OUT` | Implemented; exits non-zero on failure |
| Cryptographic challenge nonce verification | `wpm receipt verify-challenge <receipt.json>` | `SHELL_OUT` | Implemented; exits non-zero on failure |
| Producer-safe diagnostic report | `wpm receipt producer-safe-report <receipt.json>` | `SHELL_OUT` | Implemented; delegates to doctor with `--format json`; machine-parseable |
| Operator-private diagnostic report | `wpm receipt operator-private-report <receipt.json>` | `SHELL_OUT` | Implemented; delegates to doctor with `--format json`; machine-parseable |
| SPC status and ring buffer history | `wpm spc status`, `wpm spc history` | `FEATURE_GATE` | Gate SPC checks conditionally: parse cycle count from stdout; skip SPC rules until count threshold met; state is process-local and resets on each invocation |
| Andon oracle for online prefix conformance | `wpm oracle check --law <law> <tape.ocel>` | `DEFER_CONTRIB` | Placeholder implementation (`println!` only, no real conformance logic); `OrderingLaw` deserialization wired but no prefix conformance evaluation executed |
| Lean process waste audit | `wpm lean` | `DO_NOT_USE` | Hardcoded heuristics (checks `.wasm4pm/results` count and `/tmp/wasm-server.sock` existence); not a real lean analysis engine; not machine-parseable |
| AutoProcess four-phase pipeline | `wpm autoprocess --format json <log>` | `USE_AS_IS` | Perception → Decision → Protection → Optimization; JSON output mode available; exit code gates four-phase conformance |
| Telco router smoke check | `wpm telco status` | `SHELL_OUT` | Reports `ACTIVE`/`INACTIVE` state and nanosecond latency targets; parse `ACTIVE` from stdout as smoke-test health check |
| RL agent lifecycle management | `wpm agent list`, `wpm agent status`, `wpm agent reset` | `DO_NOT_USE` | Thread-local RL orchestrator state resets on each binary invocation; no persistence layer; not useful for CI |
| Interactive project wizard | `wpm wizard` | `DO_NOT_USE` | Interactive (`dialoguer`); cannot be driven non-interactively; defer until `--non-interactive` flag contributed upstream |
| Config management | `wpm config` | `DO_NOT_USE` | `wpm config show` exits with code 2 (subcommand does not exist); manage cargo-cicd config independently |
| Doctor system check | `wpm doctor` | `DO_NOT_USE` | Exits 0 even when checks fail — cannot gate CI without output parsing; replace with direct dependency checks |

**Recommended integration path:** Install `wpm` binary (built from source with `nightly-2026-04-15`). Shell out exclusively to the receipt subcommand family (`doctor`, `verify-ocel2`, `detect-fixture-mutation`, `verify-boundary-evidence`, `verify-proof-class`, `verify-challenge`, `producer-safe-report`, `operator-private-report`) using `--format json` and receipt JSON files as the exchange medium, treating non-zero exit codes as proof gate failures.

---

### miniml-core

**Purpose:** Minimal, WASM-targeted ML library providing classical ML algorithms, statistical methods, optimization routines, and process-sequence analytics compiled as cdylib/rlib with wasm-bindgen bindings.

**Nightly required:** No
**Note:** Package name in `Cargo.toml` is `miniml`, not `miniml-core`. Reference as `miniml` in dependency declarations.
**Key blockers:**
- `cdylib` crate-type forces `wasm-bindgen` as a hard dependency for all consumers including native
- Blanket `#![allow(clippy::all, unused_mut, unused_imports, dead_code, ...)]` — widespread dead code
- Many modules (`neural`, `causal`, `bayesian`, `gaussian_process`, `association`, `survival`, `graph`, `markov`, `distributions`, `stacking`, `advanced_cv`, `transfer`, `augmentation`) are private stubs with no public re-exports
- No feature flags to disable WASM bindings for native-only use

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| Process-sequence analytics (anomaly, drift, bandit, UCB1, beam search) | `optimization::score_sequence_anomaly()`, `optimization::build_transition_model()`, `optimization::detect_drift()`, `optimization::compute_ewma()`, `optimization::page_hinkley_test()`, `optimization::select_ucb1()` | `USE_AS_IS` | Fully native Rust; no `JsError`/`JsValue` returns; no WASM binding in these modules; highest-value integration surface for process mining pipelines |
| Metaheuristic optimization (GA, PSO, simulated annealing) | `optimization::GeneticOptimizer::optimize<T>()`, `optimization::PSOOptimizer::optimize()`, `optimization::AnnealingOptimizer::optimize<T>()` | `USE_AS_IS` | Fully generic native APIs; no `JsError`/`JsValue`; import from `miniml::optimization` |
| Streaming feature extraction | `StreamingFeatureExtractor`, `WelfordStatistics`, `IncrementalTfIdf` | `USE_AS_IS` | Zero `wasm_bindgen` annotations; `update_from_trace()` and `extract_vector()` are pure Rust with `Serialize/Deserialize`; import from `miniml::StreamingFeatureExtractor` |
| Classical regression (linear, polynomial, exponential, logarithmic, power, quantile) | `*_impl(...)` variants returning `Result<Model, MlError>` | `WRAP_LOCAL` | Dual `_impl`/WASM pattern; native consumers must call `foo_impl()` variants not the `#[wasm_bindgen]` wrappers; thin re-export adapter per module |
| Classification (logistic, naive Bayes, decision tree, random forest, gradient boosting, SVM, AdaBoost, perceptron, KNN) | `*_impl(...)` variants returning `Result<Model, MlError>` | `WRAP_LOCAL` | Same dual `_impl` pattern; `_impl` variants return `MlError`; WASM wrappers return `JsError` |
| Clustering (K-means, K-means++, DBSCAN, hierarchical, silhouette) | `kmeans_impl()`, `dbscan_impl()`, `hierarchical_impl()`, `silhouette_impl()` | `WRAP_LOCAL` | Model structs are `#[wasm_bindgen]` but native methods work on native targets; thin adapter needed |
| Dimensionality reduction (PCA) | `pca_impl()` returning `Result<PcaModel, MlError>` | `WRAP_LOCAL` | `PcaModel` uses `#[wasm_bindgen]` with native-accessible getters; same `_impl` adapter pattern |
| Feature engineering (StandardScaler, MinMaxScaler, RobustScaler, etc.) | `StandardScaler::fit_transform()`, `MinMaxScaler::transform()`, etc. | `WRAP_LOCAL` | Scaler methods return `JsError` directly on the `impl` block (no `_impl` variant); thin newtype wrapper converting `JsError` to local error type |
| Monte Carlo methods (integration, bootstrap, expected-value) | `MonteCarloResult`, `MonteCarloBootstrapResult`, top-level `monte_carlo_*` functions | `WRAP_LOCAL` | `#[wasm_bindgen]` structs; field access via getter methods works natively; `JsError` wrapping on outer functions — thin adapter converting `JsError` to `String/anyhow` |
| Stub/incomplete modules (neural, causal, bayesian, gaussian_process, etc.) | Private `mod` declarations | `DEFER_CONTRIB` | Real code behind re-exports but blanket suppressor, no feature gates; mixed completeness; none suitable for production integration |

**Recommended integration path:** Add path dependency on `miniml-core` directory as `miniml` (per package name). Import `optimization::*` and `StreamingFeatureExtractor` directly for process-sequence analytics. Introduce a single `miniml_adapter` module wrapping `_impl` variants and `JsError`-returning methods behind a clean `MlError` surface for all other capabilities.

---

### wasm4pm-cognition

**Purpose:** AutoSystems cognition kernel providing classic AI reasoning breeds, BLAKE3-linked receipt chain, adversarial false-pass detectors, and the verb8 contract entry point — all gated behind machine-evidence proof before any success verdict is allowed.

**Nightly required:** No
**Key blockers:**
- Path dependency on `../prolog8` — must resolve within workspace or publish `prolog8` to crates.io
- Ed25519 signing is default feature (`actor-ed25519`); to use MAC-only path, explicitly disable default features
- `crate-type = ["cdylib", "rlib"]` — `cdylib` is unusual for a library dep; some build systems warn or fail
- `#[deny(missing_docs)]` — downstream re-exports of undocumented items fail to compile

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| Classic AI breed dispatch (Eliza/frames, MYCIN/rules, Prolog Horn-clause, STRIPS, Hearsay-II blackboard, SOAR, CBR, Dendral, GPS) | `CognitionBreed` trait, `dispatch_breed_test(input: BreedInput) -> Result<BreedOutput>` | `WRAP_LOCAL` | Stable, documented; thin adapter constructing `BreedInput` and mapping `BreedOutput` to cargo-cicd types |
| BLAKE3-linked receipt chain with Ed25519 signing | `ReceiptChain`, `ChainLink`, `ReceiptChain::append()`, `ReceiptChain::verify_replay()` | `FEATURE_GATE` | Ed25519 signing on by default via `actor-ed25519` feature (pulls `ed25519-dalek`, `rand_core`); disable default features + enable `actor-mac-fallback` for MAC-only path; feature split is clean |
| Adversarial false-pass detectors (8 detectors) | `FindingRegistry`, `StubGateDetector`, `SelfCertifyDetector`, `ReplayBrokenDetector`, `RepairWeakensDetector`, `HumanAuthorityDetector`, `MissingEvidenceDetector`, `BenchMissingDetector`, `CentralFirehoseDetector` | `WRAP_LOCAL` | Public and documented; operate against `EvidenceSource` trait; implement `OtelEvidenceSource` or `FilesystemEvidenceSource` adapter; no feature flag for native use |
| Verb8 contract protocol | `run_contract(contract: CognitionContract) -> ContractResult`, `CognitionContract` struct, function type aliases `PreconditionFn`, `ExecutionFn`, `PostconditionFn` | `WRAP_LOCAL` | Fully public with stable generic signatures; wrap cargo-cicd pipeline stages into contract shape |
| Pareto dominance scoring | `is_dominated(candidate: &Candidate, others: &[Candidate], profile: &DomainProfile) -> bool`, `reject_dominated(candidates: Vec<Candidate>, ...) -> Vec<Candidate>` | `USE_AS_IS` | Pure functions over `Candidate`/`DimensionSpec`/`DomainProfile`; no feature gates; no path deps beyond workspace |
| Cost-law evaluation | `DimensionGroup<U>` with `Currency`, `Time`, `Probability`, `Score`, `Throughput` phantom markers | `USE_AS_IS` | Fully typed, doc-covered; unit-marker phantom types prevent cross-unit misuse at compile time |
| OTel + filesystem evidence ingestion | `EvidenceSource` trait, `OtelEvidenceSource`, `FilesystemEvidenceSource`, `CompositeEvidenceSource` | `WRAP_LOCAL` | `OtelEvidenceSource` parses JSON span arrays; `FilesystemEvidenceSource` reads `.wasm4pm/results/<target>.json`; thin adapter struct satisfying `EvidenceSource` from cargo-cicd native OTel span types |
| WASM bindings (cognition_show, cognition_run, cognition_verify, cognition_replay, system_build) | Under `features = ["wasm"]` | `FEATURE_GATE` | Never enable `wasm` feature for native cargo-cicd builds; only relevant for wasm-pack browser builds |
| AutoInstinct sub-module | `autoinstinct::{NeuroticState, SymbolicVisionSystem, SemanticParser, HeuristicPlanner}` | `DEFER_CONTRIB` | Early-stage symbolic cognition layer; no integration into main breed dispatch path; no stable API declared |

**Recommended integration path:** Add `wasm4pm-cognition` as an rlib workspace path dep (default features, no `wasm` feature). Implement a thin `EvidenceSource` adapter over cargo-cicd's OTel span model. Call `run_contract` for proof-gate enforcement and `reject_dominated` for Pareto frontier selection directly.

---

### prolog8

**Purpose:** Byte-capped Datalog/Prolog proof engine for bounded policy rules, action admission, and replayable decisions (arity ≤ 8, body atoms ≤ 8, variables ≤ 8).

**Nightly required:** No
**Key blockers:**
- `crate-type = ["cdylib", "rlib"]` — linking as library dep requires `rlib`; `cdylib` present but harmless
- Broad `#![allow(clippy::all, ...)]` — some dead or transitional code may cause confusion downstream
- No public traits — consumers must depend on concrete `Kernel` and `Catalog` types directly
- WASM ABI (`wasm` feature) must be explicitly enabled; default build omits it

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| Byte-capped rule/atom admission | `admit_atom(atom: &str) -> Result<(), RejectionCode>`, `admit_rule(rule: &Rule8) -> Result<(), RejectionCode>` | `USE_AS_IS` | Stable, well-typed; 27 `RejectionCode` variants; `Copy + PartialEq + Serialize`; directly usable as policy enforcement layer |
| Kernel query execution | `Kernel::load_facts(block: FactBlock8)`, `Kernel::query(goal: &Atom8) -> Decision` | `WRAP_LOCAL` | No public traits; construct `Kernel` directly; thin `CognitionAdapter` struct needed to own catalog construction and map domain event log records to `FactBlock8`/`FactRow8` |
| Positive/negative proof emission | `ProofNode`, `Decision`, `ProofNodeId` | `WRAP_LOCAL` | Fully serializable; no traversal helper API — small adapter flattening `ProofNode` trees into conformance evidence records needed |
| Receipt assembly and deterministic replay | `Receipt` (assembled by `Kernel::query` automatically), `replay(kernel: &Kernel, receipt: &Receipt) -> ReplayStatus` | `USE_AS_IS` | Directly usable as proof-of-execution gate in CI pipelines with no wrapping |
| BLAKE3 cryptographic hashing | `hash_bytes(data: &[u8]) -> Blake3Hash`, `combine_roots(...)`, `link_hash(...)`, domain constants `DOMAIN_PROLOG8_RECEIPT` etc. | `USE_AS_IS` | Pure functions with no side effects; domain constants exported; directly usable for receipt chaining |
| Predicate catalog construction | `Catalog::new()`, `Catalog::add_predicate(meta: PredicateMeta)`, `Catalog::intern_term(name: &str) -> TermId` | `WRAP_LOCAL` | No auto-id allocation or schema-import path; factory wrapper mapping process-mining predicate schemas to catalog entries required per deployment context |
| WASM/cdylib JSON ABI | `wasm::query_json()`, `wasm::replay_json()`, `wasm::engine_info_json()` (under `features = ["wasm"]`) | `FEATURE_GATE` | Correct for WASM deployment; irrelevant for native targets; requires explicit feature management in CI |

**Recommended integration path:** Depend on `prolog8` as a direct rlib library dependency. Build a single `CognitionAdapter` struct owning catalog construction (mapping process-mining predicate schemas to `PredicateId`/`TermId`), translating event log rows to `FactBlock8` inputs, and exposing `query`/`replay`/`receipt` methods.

---

### wasm4pm-macros

**Purpose:** Proc-macro attributes (`#[powl_test]` and `#[powl_activity]`) that instrument Rust test functions with POWL-model conformance checking and activity recording.

**Nightly required:** Yes — workspace pins `nightly-2026-04-15`
**Key blockers:**
- Not on crates.io — requires path or git dependency
- Consumer crate must have `wasm4pm` in its dep tree for generated code to compile (references `wasm4pm::testing::{PowlTestHarness, ConformanceVerdict, AndonPull, record_activity}`)
- `exact` parameter silently ignored (see below)

**Capability inventory:**

| Capability | Public API | Leverage | Notes |
|---|---|---|---|
| `#[powl_test]` conformance test expansion | `#[powl_test(route = "...", model = "...")]` | `WRAP_LOCAL` | Implemented; injects `PowlTestHarness` local variable `h` and calls `h.finish()`; requires nightly + path dep + `wasm4pm` in dep tree; feature-gated wrapper re-exporting attribute under stable alias recommended |
| POWL model path resolution via `CARGO_MANIFEST_DIR` | `concat!(env!("CARGO_MANIFEST_DIR"), "/", model_path)` at compile time | `USE_AS_IS` | Resolves correctly relative to consumer crate; supply correct relative model path string in attribute |
| `expect_refusal` negative conformance testing | `#[powl_test(route = "...", model = "...", expect_refusal = "AndonPull::Variant")]` | `USE_AS_IS` | Implemented; emits `assert_eq!` against named `AndonPull` variant; same nightly + path dep constraints |
| `#[powl_activity]` recording with production no-op | `#[powl_activity]` on any function | `USE_AS_IS` | Prepends `record_activity()` call; inlined away outside test/powl-test cfg; same nightly + path dep constraints |
| `exact` parameter | `#[powl_test(..., exact = true)]` | `DO_NOT_USE` | Parsed to avoid compile errors; has zero effect; callers must not rely on it for conformance semantics — risk of false-passing tests |

**Recommended integration path:** Declare `wasm4pm-macros` as a path (or pinned git) dev-dependency under `nightly-2026-04-15` toolchain. Wrap attribute invocations in a feature-gated helper macro enforcing `route` and `model` arguments while explicitly omitting `exact` until it is implemented upstream.

---

### ocpq

**Purpose:** Faithful, paper-grounded implementation of the OCPQ runtime from Küsters & van der Aalst (arXiv:2506.11541v1, 2025). Evaluates object-centric process queries and constraints over an OCEL log.

**Nightly required:** No
**Key blockers:**
- `ocel-core` is a workspace path dependency — consuming crates must be in the same workspace or patch the dependency
- `cdylib` crate-type causes link warnings when used as rlib transitive dependency (harmless but noisy in some CI setups)
- `wasm` feature must be explicitly enabled; default build omits WASM surface
- Not on crates.io — only available as a local workspace path

**Capability inventory:**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| BASIC E2O predicate evaluation | `BasicPredicate::E2O`, `BasicPredicate::holds(binding: &Binding, ocel: &OCEL) -> bool` | `USE_AS_IS` | Implemented, paper-grounded (Def. 5); accessible via public rlib surface; direct call within workspace |
| BASIC O2O predicate evaluation | `BasicPredicate::O2O`, `BasicPredicate::holds(...)` | `USE_AS_IS` | Identically complete; same direct-call path |
| BASIC TBE predicate evaluation (time-between-events duration constraints) | `BasicPredicate::Tbe { seconds_min, seconds_max }`, `BasicPredicate::holds(...)` | `USE_AS_IS` | Implemented with `chrono` timestamp arithmetic; directly callable as rlib dep |
| BindingBox output set computation | `BindingBox::output(ocel: &OCEL) -> Vec<Binding>` | `WRAP_LOCAL` | Naive Cartesian-product enumeration — correct but O(n^k) in log size and variable count; add a binding-count size guard / pagination layer before exposing to pipeline consumers |
| QueryTree constraint evaluation | `evaluate_constraint(tree: &QueryTree, ocel: &OCEL) -> ConstraintResult`, `evaluate_node_constraint(node: &QueryNode, ocel: &OCEL) -> ConstraintResult` | `WRAP_LOCAL` | Complete and paper-grounded; `ConstraintResult` is opaque to cargo-cicd's proof-gate surface — thin adapter mapping `satisfied`/`violated` counts to pass/fail gate verdicts needed |
| CHILD SET cardinality constraint enforcement | `ConstraintPredicate::ChildSet { n_min, n_max }`, wired into `QueryTree` evaluation | `USE_AS_IS` | Fully wired and tested; callable as-is within workspace |
| JSON entry point | `ocpq_eval_json(query_json: &str, ocel_json: &str) -> Result<String, String>` | `USE_AS_IS` | Self-contained, no feature flags; cleanest integration surface for cargo-cicd — pass serialized `QueryTree` + OCEL JSON, receive `ConstraintResult` JSON |
| WASM surface | `ocpq_eval()` under `features = ["wasm"]` | `FEATURE_GATE` | Default build omits it; use `ocpq_eval_json()` instead for native integration; identical logic |

**Recommended integration path:** Add `ocpq` as a workspace path dependency. Call `ocpq_eval_json()` directly for proof-gate constraint evaluation. Wrap `ConstraintResult` satisfied/violated counts into gate pass/fail verdicts. Add binding-count guard before invoking `BindingBox::output()` on large logs.

---

### tps-metrics

**Purpose:** CLI binary that collects TPS metrics (takt time, lead time, mura, value stream mapping, andon status) from a git repository's commit history.

**Nightly required:** No
**Key blockers:**
- No library target: only `[[bin]]` in `Cargo.toml`; all modules are `private mod` inside `main.rs` — inaccessible to external crates
- `error_rate_per_kloc` hardcoded to `0.0` (stub) — requires external backend integration before it carries signal
- Broad `#![allow(clippy::all, ...)]` in `main.rs` — rough-draft code quality
- Lead time uses commit-to-HEAD proxy for merge latency — inaccurate on non-linear histories

**Capability inventory:**

| Capability | Command / Flag | Leverage | Notes |
|---|---|---|---|
| Takt time analysis | `tps-metrics takt --json` | `SHELL_OUT` | Commits-per-day rate, consistency score, drought-day detection; structured JSON available |
| Lead time analysis | `tps-metrics lead-time --json` | `SHELL_OUT` | Average/median/P95 commit-to-merge latency; treat P95/median as directional signals (commit-to-HEAD proxy) |
| Mura (unevenness) detection | `tps-metrics mura --json` | `SHELL_OUT` | Daily/hourly variance, burst score, evenness score; structured JSON available |
| Value stream mapping | `tps-metrics value-stream --json` | `SHELL_OUT` | Value-added ratio, coding vs wait time, bottleneck identification; treat as approximate git-history estimates |
| Andon dashboard | `tps-metrics andon --json` | `DEFER_CONTRIB` | Build success rate and test pass rate available via `SHELL_OUT`; `error_rate_per_kloc` is always `0.0` (stub) — do not rely on andon signal for quality gating until backend integration contributed |
| JSON output mode | `--json` flag on all sub-commands | `SHELL_OUT` | Canonical integration surface; all sub-commands support this flag |
| Overall TPS health score | `tps-metrics health --json` | `SHELL_OUT` | Aggregate score accessible; validity constrained by andon stub and lead-time proxy inaccuracy |

**Recommended integration path:** Invoke `tps-metrics` as a subprocess with `--json` for each sub-command. Parse JSON via `FILE_EXCHANGE`. Suppress reliance on `andon error_rate_per_kloc` until a lib target and external backend integration are contributed upstream. If library access is required, contribute a `lib.rs` target with `[[lib]]` declared in `Cargo.toml`.

---

### wasm4pm (core WASM/rlib crate)

**Purpose:** Core cdylib/rlib crate compiling high-performance process mining algorithms to WebAssembly and native Rust, via a handle-based state management API.

**Nightly required:** Yes — uses `#![feature(generic_const_exprs, adt_const_params, const_trait_impl, min_specialization, portable_simd)]` with `#![allow(incomplete_features)]`
**Key blockers:**
- Path dependencies on sibling workspace crates: `wasm4pm-compat`, `wasm4pm-cognition`, `wasm4pm-macros`, `miniml` — cannot resolve outside the workspace without vendoring or publishing
- `crate-type = ["cdylib", "rlib"]` — link-time conflicts possible on some toolchain configurations when rlib is used as dep in another binary
- Broad `#![allow(dead_code, unused_variables, ...)]` — masks whether large portions of public API are actually functional
- Custom binary OCEL format (not standard OCEL 2.0 JSON/XML)

**Capability inventory (as found in sources/wasm4pm snapshot):**

| Capability | Public Types / Functions | Leverage | Notes |
|---|---|---|---|
| DFG discovery | `dfg_mining(events: &[Event]) -> Evidence<ProcessModel, Admitted, W>` | `WRAP_LOCAL` | Implemented and tested; thin adapter to construct `Event` structs and extract `ProcessModel` from `Evidence` wrapper |
| Alpha miner (Alpha+ approximation) | `alpha_miner(events: &[Event]) -> Evidence<ProcessModel, Admitted, W>` | `WRAP_LOCAL` | Implemented; simplified linear-chain Petri net rather than true Alpha+; label as heuristics-miner not ILP-miner to avoid overclaiming |
| POWL discovery with cut detection | `PowerMiner::mine() -> Evidence<TypedPowl, Admitted, PowerWitness>` | `WRAP_LOCAL` | Implemented; depends on `wasm4pm-compat::TypedPowl`, `TreeProjectable`, `OperatorKind`; XOR, sequence, partial order, parallelism cuts; loop cut is heuristic; path dep must be resolved |
| Token-based replay conformance | `TokenReplayEngine::replay_case()`, `TokenReplayEngine::replay_log() -> Evidence<TokenReplayResult, Admitted, TokenReplay>` | `WRAP_LOCAL` | Implemented; fitness in [0,1]; thin adapter needed to construct `PetriNet` from internal `BTreeSet`/`BTreeMap` API |
| Alignment-based conformance (A*) | `AlignmentEngine`, `AlignmentConformance`, `AlignmentConformance::compute() -> Evidence<AlignmentResult, Admitted, AlignmentWitness>` | `WRAP_LOCAL` | Implemented with A* search; state-space cap 5000 iterations (hardcoded); thin adapter needed for `PetriNet` construction |
| Footprints conformance checking | (no dedicated module found) | `DEFER_CONTRIB` | Not present in sources/wasm4pm snapshot; defer until footprints module contributed |
| OCEL 2.0 JSON/XML import/export | (no standard JSON/XML parser found) | `DO_NOT_USE` | `ZeroCopyOcel` and `ZeroCopyOcelV2` parse custom binary OCEL format (magic `0x4F43454C`); not standard OCEL 2.0 JSON or XML; use `ocel-core` for standard format |
| XES event log import/export | (no XES module found in sources/wasm4pm) | `DO_NOT_USE` | Not present in this snapshot despite crate description; use `wasm4pm-types` `import` feature for XES |
| Streaming real-time process mining with SIMD | (no streaming/SIMD module found) | `DO_NOT_USE` | Not present in this snapshot; no `portable_simd` feature used in sources/wasm4pm; do not use |
| ML-based remaining-time, drift, anomaly detection | (no ML module found in sources/wasm4pm) | `DO_NOT_USE` | Not present in this snapshot; use `miniml-core` (`optimization::detect_drift`, `score_sequence_anomaly`) instead |
| POWL-to-Petri-net/BPMN/YAWL model conversions | (no conversion modules found) | `DEFER_CONTRIB` | Not in this snapshot; defer to future wasm4pm release |
| Handle-based WASM state management (LRU, object pool) | `ffi.rs`: `wasm_init()`, `wasm_alloc()`, `wasm_get_last_error()` | `DO_NOT_USE` | Arena allocator, not LRU cache; no handle-based object pool found; C ABI not usable as Rust library surface |
| Graduation bridge from wasm4pm-compat | `graduation::accept_from_compat(candidate: GraduationCandidate) -> Result<EngineHandle>` | `WRAP_LOCAL` | Implemented; validates `GraduationCandidate` witnesses before engine intake; direct integration path if `wasm4pm-compat` path dep resolved |
| Temporal conformance / frequency-weighted DFG | `dfg_mining()` with `(time, event_id)` ordering | `WRAP_LOCAL` | Temporal ordering enforced during mining by sorting events by timestamp; frequency-weighted DFG available; no dedicated performance DFG (timing metrics per edge) module |

**Recommended integration path:** Vendor or join the wasm4pm workspace to resolve `wasm4pm-compat` path dependency. Depend on `wasm4pm` as an rlib. Wrap mining (`alpha_miner`, `dfg_mining`), conformance (`TokenReplayEngine`, `AlignmentConformance`), and `PowerMiner` APIs with thin adapter structs constructing `Event`/`PetriNet` inputs and extracting `Evidence` payloads. Avoid all unimplemented capabilities (XES, streaming, ML, SIMD, model conversions) not present in the sources/wasm4pm snapshot.

---

## wpm CLI Command Matrix

| Command | Works | Exit Code Reliable | Output Format | Leverage | Recommended Use in cargo-cicd |
|---|---|---|---|---|---|
| `wpm receipt doctor --format json --strict <file>` | Yes | Yes | JSON | `USE_AS_IS` | Hard CI gate; non-zero exit blocks release; parse JSON for structured diagnostics |
| `wpm oracle check --law <law> <tape>` | Yes (`--help`) | Yes | text | `USE_AS_IS` | Hard CI gate on `AndonPull`; exits non-zero on conformance failure; wire once oracle impl is confirmed non-placeholder |
| `wpm autoprocess --format json <log>` | Yes (`--help`) | Yes | JSON | `USE_AS_IS` | Four-phase conformance gate (Perception → Decision → Protection → Optimization); JSON output mode |
| `wpm audit <log.xes>` | Yes (`--help`) | Yes | text | `USE_AS_IS` | Conformance gate step for XES compliance; exit code gates pipeline |
| `wpm receipt verify-ocel2 <file>` | Yes | Yes | text | `SHELL_OUT` | Structural validation; parse stdout + exit code |
| `wpm receipt canonicalize-ocel2 <file>` | Yes | Yes | JSON | `SHELL_OUT` | OCEL canonicalization; capture JSON stdout |
| `wpm receipt detect-fixture-mutation <file>` | Yes | Yes | text | `SHELL_OUT` | Mutation detection gate; exit code suffices |
| `wpm receipt verify-boundary-evidence <file>` | Yes | Yes | text | `SHELL_OUT` | Boundary evidence gate; exit code suffices |
| `wpm receipt verify-proof-class <file>` | Yes | Yes | text | `SHELL_OUT` | Proof-class gate; exit code suffices |
| `wpm receipt verify-challenge <file>` | Yes | Yes | text | `SHELL_OUT` | Challenge nonce gate; exit code suffices |
| `wpm receipt producer-safe-report <file>` | Yes | Yes | JSON | `SHELL_OUT` | Machine-parseable producer-audience report |
| `wpm receipt operator-private-report <file>` | Yes | Yes | JSON | `SHELL_OUT` | Machine-parseable operator-audience report |
| `wpm spc status` | Yes | No (process-local state) | text | `FEATURE_GATE` | Gate SPC checks conditionally: parse cycle count, skip rules until threshold met; state resets per invocation |
| `wpm spc check` | Yes (`--help`) | Yes | JSON | `SHELL_OUT` | Western Electric rule violations; JSON output for process drift alerts |
| `wpm mining discover <log>` | Yes | Yes | text | `SHELL_OUT` | DFG output; no JSON mode yet; parse stdout or await JSON flag |
| `wpm mining conformance <log> <model>` | Partially | Yes | text | `PATCH_SMALL` | Patch: wire model file path to actual DFG/PNML deserialization; currently uses `DFG::new()` mock |
| `wpm telco status` | Yes | Yes | text | `SHELL_OUT` | Smoke-test health check; parse `ACTIVE`/`INACTIVE` from stdout before latency-sensitive stages |
| `wpm lean` | Yes (exit 0) | No | text | `DO_NOT_USE` | Hardcoded heuristics; not machine-parseable; not a real lean analysis engine |
| `wpm agent list/status/reset` | Yes (`--help`) | No (process-local state) | text | `DO_NOT_USE` | Thread-local state resets per invocation; no persistence; not useful for CI |
| `wpm wizard` | Yes (`--help`) | N/A (interactive) | interactive | `DO_NOT_USE` | Interactive only; not scriptable; defer until `--non-interactive` flag contributed |
| `wpm doctor` | Partially | No (exits 0 on failure) | text | `DO_NOT_USE` | Unreliable CI gate; exits 0 even when checks fail; replace with direct dependency checks |
| `wpm config show` | No | No (exit 2) | error | `DO_NOT_USE` | Subcommand does not exist |
| `wpm man audit` | No | No (exit 2) | error | `DO_NOT_USE` | Subcommand does not exist |

---

## Priority Integration List

Ranked by integration value for cargo-cicd, highest value first.

### 1. `wpm receipt doctor --format json --strict` (wasm4pm-cli)

**Leverage:** `USE_AS_IS`
**Integration cost:** Shell out to binary; parse JSON; check exit code.
**Value:** Covers all Adversarial Ingress Gate checks in a single invocation. Exits non-zero on refused receipts. This is the most complete, production-ready capability in the entire ecosystem and requires zero upstream changes.
**Action:** `cmd = ["wpm", "receipt", "doctor", "--format", "json", "--strict", receipt_path]; gate on exit_code != 0`

---

### 2. `ProvenanceChain`, `Blake3Hash`, `blake3_hex()`, `canonical_json()` (wasm4pm-types)

**Leverage:** `USE_AS_IS`
**Integration cost:** Add `wasm4pm-types` as workspace path dep; import from crate root.
**Value:** Highest-confidence surface in the entire crate scan. Regression-hardened (PR #54 NaN class, PR #66 injectivity). BLAKE3 provenance chains are the cryptographic backbone of all receipt verification — integrate as a direct library dep to generate and verify receipts in cargo-cicd native code without shelling out.
**Action:** `wasm4pm-types = { path = "..." }` in `Cargo.toml`; call `blake3_hex()`, `canonical_json()`, `ProvenanceChain::new()`, `ProvenanceChain::append()`

---

### 3. `ocpq_eval_json()` (ocpq)

**Leverage:** `USE_AS_IS`
**Integration cost:** Add `ocpq` as workspace path dep; call `ocpq_eval_json(query_json, ocel_json) -> Result<String, String>`; parse `ConstraintResult` JSON; map `satisfied`/`violated` counts to gate verdicts.
**Value:** Paper-grounded OCPQ query evaluation over OCEL logs. The cleanest single-function integration surface in the ecosystem — no feature flags, no WASM binding, no type complexity. Enables proof-gate constraint evaluation directly from Rust without shelling out.
**Action:** Add `ocpq = { path = "crates/ocpq" }`; write `fn check_ocpq_constraint(query: &QueryTree, ocel: &OCEL) -> GateVerdict` wrapper around `ocpq_eval_json()`

---

### 4. `ocel-core` validate / flatten / NDJsonStream

**Leverage:** `USE_AS_IS`
**Integration cost:** Add `ocel-core` as direct path dep; call three functions directly.
**Value:** Fully stable, paper-grounded, no stubs. Provides OCEL 2.0 structural validation (`ValidationReport` with machine-stable error codes), XES flattening (deterministic `FlatLog` for replay provenance), and streaming ND-JSON intake. Foundational for any OCEL 2.0 event log pipeline.
**Action:** `ocel-core = { path = "/path/to/ocel-core" }`; call `validate::validate(&ocel, &cardinality)`, `flatten::flatten(&ocel, object_type)`, `NDJsonStream::new(reader, plan)`

---

### 5. Process-sequence analytics from miniml-core (`optimization::*`)

**Leverage:** `USE_AS_IS`
**Integration cost:** Add `miniml` path dep (note: package name is `miniml` not `miniml-core`); import from `miniml::optimization`.
**Value:** Sequence anomaly scoring, drift detection (EWMA + Page-Hinkley), UCB1 bandit selection, transition model construction, and beam search are all fully native Rust APIs with no `JsError`/`JsValue`. Directly applicable to process mining telemetry pipelines.
**Action:** `miniml = { path = "crates/miniml-core" }`; call `optimization::score_sequence_anomaly()`, `optimization::detect_drift()`, `optimization::page_hinkley_test()`, `optimization::select_ucb1()`

---

### 6. Token replay conformance (wasm4pm-algos or wasm4pm)

**Leverage:** `WRAP_LOCAL`
**Integration cost:** Thin adapter translating cargo-cicd `EventLog` types to wasm4pm `Event` slices; assemble `ProvenanceChain` from result.
**Value:** DFG-based token replay returning `ConformanceResult` with Rozinat–van der Aalst fitness metric. Available in both `wasm4pm-algos` (`token_replay::replay_log`) and `wasm4pm` (`TokenReplayEngine`). Choose based on which workspace dep is already resolved.
**Action:** Write `process_conformance_adapter::replay_log(log: &CicdEventLog) -> ConformanceResult` translating types and calling the appropriate replay function

---

### 7. `wpm oracle check` (wasm4pm-cli)

**Leverage:** `USE_AS_IS` (once implementation confirmed non-placeholder)
**Integration cost:** Shell out; check exit code.
**Value:** Validates an OCEL tape against a declared process law. Exits non-zero on `AndonPull` (conformance failure). Direct CI gate step. Verify that the oracle `check` implementation is not a placeholder before promoting to hard gate.
**Action:** `cmd = ["wpm", "oracle", "check", "--law", law_path, tape_path]; gate on exit_code != 0`

---

### 8. `reject_dominated` and `is_dominated` (wasm4pm-cognition)

**Leverage:** `USE_AS_IS`
**Integration cost:** Resolve `prolog8` path dep (or add mock implementation); import Pareto functions directly.
**Value:** Pure functions for Pareto dominance scoring over multi-dimensional candidate sets. Unit-marker phantom types prevent cross-unit misuse. Directly applicable to multi-objective optimization in CI pipeline quality scoring.
**Action:** Resolve `wasm4pm-cognition` path dep; call `reject_dominated(candidates, profile)` and `is_dominated(candidate, others, profile)` directly

---

### 9. BLAKE3 receipt verification (wasm4pm-algos)

**Leverage:** `WRAP_LOCAL`
**Integration cost:** Thin JSON envelope adapter.
**Value:** `receipt::verify_receipt(envelope: &Value) -> (VerificationResult, String, String)` is the most stable capability in `wasm4pm-algos`. Verified receipt integrity without requiring the full nightly ecosystem — wrappable in a subprocess shim if nightly is a blocker.
**Action:** Write a receipt-envelope adapter struct and call `verify_receipt` after resolving nightly workspace dep

---

### 10. `wpm autoprocess --format json` (wasm4pm-cli)

**Leverage:** `USE_AS_IS`
**Integration cost:** Shell out with JSON config file and input log; parse JSON output; check exit code.
**Value:** Four-phase pipeline (Perception → Decision → Protection → Optimization) invocable as a single CLI call with structured JSON output. Highest-level conformance gate available via the CLI surface without any library coupling.
**Action:** `cmd = ["wpm", "autoprocess", "--format", "json", "--config", config_path, log_path]; gate on exit_code != 0`

---

## Blocked / Deferred Capabilities

The following capabilities cannot be integrated now and are excluded from the priority list.

| Capability | Crate | Reason | Unblock Path |
|---|---|---|---|
| Alignment-based conformance (full reachability) | wasm4pm-algos | Linearised BFS from `source`/`start`-named places only — trivially passes all traces for nets without those conventions | Contribute full reachability graph expansion to `alignment.rs` |
| Heuristic miner (true HeuristicsNet) | wasm4pm-algos | Functionally identical to basic DFG miner; no dependency thresholds, no noise filtering, no HeuristicsNet output | Implement HeuristicsNet layer with frequency thresholds and loop handling |
| Streaming/incremental DFG discovery | wasm4pm-algos | `discover_streaming_dfg` is a standard DFG pass with no incremental state | Implement true sliding-window or incremental DFG algorithm |
| `pm-core` cross-module type normalization | pm-core | Duplicate `ActivityName`/`Frequency`/`DurationNs` newtypes across modules cause type mismatches when combining types | Unify newtypes at `primitives` module level; add `From/Into` conversion impls |
| `wpm oracle check` (if placeholder) | wasm4pm-cli | `oracle/check.rs` uses `println!` only — no real conformance logic executed | Implement prefix conformance evaluation in oracle check module |
| `wpm mining conformance` | wasm4pm-cli | Model file argument ignored; loads `DFG::new()` (empty mock) | Wire DFG/PNML deserialization from model path argument |
| Andon `error_rate_per_kloc` | tps-metrics | Hardcoded to `0.0`; requires external backend (Sentry, Datadog, or build log parsing) | Integrate error tracking backend; expose as library function |
| `wasm4pm-macros` `exact` parameter | wasm4pm-macros | Parsed but silently ignored; callers believing it enables exact conformance will get false-passing tests | Implement exact conformance semantics or remove the parameter from the API |
| POWL-to-Petri/BPMN/YAWL model conversions | wasm4pm | Not present in sources/wasm4pm snapshot | Implement conversion modules in a future wasm4pm release |
| XES import in wasm4pm (native) | wasm4pm | Not present in sources/wasm4pm snapshot (despite crate description) | Implement XES parser module or use wasm4pm-types `import` feature |
| Streaming/SIMD process mining | wasm4pm | Not present in sources/wasm4pm snapshot | Implement streaming pipeline and SIMD kernels |
| Stub ML modules (neural, causal, bayesian, etc.) | miniml-core | Private, incomplete, no feature flags, blanket suppressor | Complete implementations and expose behind feature flags |
| AutoInstinct sub-module | wasm4pm-cognition | Not integrated into main breed dispatch path; early-stage | Wire into breed registry and stabilize API |
| Social network mining types | pm-core | Peripheral to core process-intelligence pipeline | Defer until social network analysis required |
| OCEL binary format (ZeroCopyOcel) | wasm4pm | Custom magic-byte binary format incompatible with standard OCEL 2.0 JSON/XML | Document and publish OCEL binary format spec; or implement standard OCEL JSON parser |

---

## Evidence Gate Integration Path

This section maps wasm4pm capabilities directly to the existing cargo-cicd evidence gate pipeline.

### Stage 1: Log Ingestion

**Recommended:** `ocel-core` `validate::validate()` + `intake::NDJsonStream`

- Validate incoming OCEL 2.0 logs before any gate evaluation: `validate::validate(&ocel, &cardinality)` returns machine-stable error codes (`E2O_EMPTY`, `DANGLING_E2O`, `CARDINALITY_MIN`, etc.)
- Stream large ND-JSON OCEL logs via `NDJsonStream<BufReader<File>>` with `ExtractionPlan` allowlists
- Flatten OCEL to XES traces via `flatten::flatten(&ocel, object_type)` for downstream DFG/conformance steps

### Stage 2: Provenance Chain Assembly

**Recommended:** `wasm4pm-types` `ProvenanceChain` + `Blake3Hash` (direct library dep)

- Assemble `ProvenanceChain` for every event log batch before conformance evaluation
- Use `blake3_hex()` and `canonical_json()` for deterministic hash inputs
- Append receipt entries via `ProvenanceChain::append(entry)` at each pipeline stage transition

### Stage 3: Conformance Evaluation

**Option A (CLI, lowest integration cost):**
- `wpm receipt doctor --format json --strict <receipt.json>` — covers all 8 Adversarial Ingress Gate checks
- `wpm oracle check --law <law> <tape.ocel>` — OCEL tape vs process law (verify implementation is non-placeholder)
- `wpm autoprocess --format json <log>` — four-phase pipeline gate

**Option B (library, maximum control):**
- `ocpq_eval_json(query_json, ocel_json)` — OCPQ constraint evaluation over OCEL log
- `token_replay::replay_log(log, dfg)` (wasm4pm-algos, wrapped) — DFG fitness metric
- `AlignmentConformance::compute()` (wasm4pm, wrapped) — A* optimal alignment fitness

### Stage 4: Adversarial Detection

**Recommended:** `wasm4pm-cognition` `FindingRegistry` + `EvidenceSource` adapter

- Implement `EvidenceSource` trait over cargo-cicd's OTel span model
- Run `FindingRegistry::run_all(evidence)` to execute all 8 detectors: `StubGate`, `SelfCertify`, `ReplayBroken`, `RepairWeakens`, `HumanAuthority`, `MissingEvidence`, `BenchMissing`, `CentralFirehose`
- Gate release on zero findings from `FindingRegistry`

### Stage 5: Receipt Verification and Release Gate

**Option A (CLI):**
- `wpm receipt doctor --format json --strict <receipt.json>` — covers all gate checks; exit code gates release

**Option B (library):**
- `receipt::verify_receipt(envelope: &Value)` (wasm4pm-algos, wrapped) — BLAKE3 envelope integrity
- `replay(kernel, receipt) -> ReplayStatus` (prolog8) — deterministic policy replay verification
- `ReceiptChain::verify_replay()` (wasm4pm-cognition) — Ed25519-signed receipt chain verification

### Minimum Viable Evidence Gate (no nightly, no workspace join required)

If cargo-cicd cannot immediately join the wasm4pm workspace or adopt nightly Rust:

1. Build and install `wpm` binary from source (nightly-2026-04-15 toolchain)
2. Add `ocel-core` as a path dep (stable Rust; self-contained)
3. Shell out to `wpm receipt doctor --format json --strict <receipt.json>` for all receipt gates
4. Shell out to `wpm autoprocess --format json <log>` for conformance gates
5. Import `ocpq_eval_json()` via `ocpq` path dep (stable Rust; self-contained given ocel-core resolved)

This covers the full evidence gate surface with two stable-Rust library deps and two CLI shell-outs.

---

*End of wasm4pm Full Capability Map*
