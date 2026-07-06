# Chapter 6: Conclusions and Future Work

## Appendix A: Vision 2030 — Strategic Roadmap

---

> **Document status:** PhD thesis chapter draft — cargo-cicd v26.6.2  
> **Date:** 2026-06-16  
> **Scope:** Chapter 6 (Conclusions and Future Work) plus the Vision 2030 Strategic Roadmap appendix

---

## Chapter 6: Conclusions and Future Work

### 6.1 Summary of Contributions

This dissertation has presented cargo-cicd: a Level 5 process-data engine for Rust workspace CI/CD automation, operating at the boundary between conventional build tooling and formal process management. The project advances the state of the art across four interconnected dimensions.

**Contribution 1: A manufactured, ontology-grounded CLI grammar.**
The noun-verb command surface of cargo-cicd is not handwritten. It is *manufactured* from a formal RDF/Turtle ontology (`ontology/cargo-cicd-capabilities.ttl`) through a code-generation pipeline (`ggen`) that applies SPARQL inference rules and Tera templates to produce noun modules, CLI test scaffolding, and reference documentation in a single, reproducible pass. This manufacturing discipline — where code is a derivation, not a primary artifact — enforces consistency between specification and implementation in a way that conventional handwritten CLIs cannot. The ontology itself grounds each capability in PROV-O (for provenance), SKOS (for concept taxonomy), and DCTERMS (for metadata), positioning the vocabulary for future linked-data integration and semantic web querying. The clap-noun-verb crate, developed alongside cargo-cicd and published at version 26.6.2, provides a reusable Rust library for the noun-verb pattern, potentially enabling other tools to adopt the same grammar discipline.

**Contribution 2: A stratified evidence emission model.**
All verbs in cargo-cicd follow the same invariant evidence pattern: every execution unit opens a `start` event, performs work, closes with a `complete` event carrying a `verdict_claimed` field, and serializes both events to an XML Event Stream (XES) file and a JSONL companion. This design makes process conformance testable without instrumenting the binary — tests assert on the wasm4pm oracle's `Accept`/`Refuse`/`Blocked` verdict, not on internal state. Seven invariants (E1-E7) formalize the evidence contract and are enforced by the test suite. In particular, Invariant E1 — cargo-cicd never adjudicates itself — enforces a separation of concerns that mirrors formal process certification: the executing party and the judging party are structurally distinct. This pattern is applicable beyond Rust CI/CD and represents a general model for auditable automation.

**Contribution 3: A feature-gated Level 5 engine with clean isolation.**
The `EngineState` aggregate root collects eleven state dimensions (workspace, toolchain, target, changed files, test plan, trybuild, git phase, process events, artifacts, policies, projection profile). All dimensions are populated by stateless adapters that silently fail and never cross-call one another. The entire Level 5 engine is opt-in behind the `process-data` feature flag, preserving a lean default binary. The `autonomic` and `wasm4pm` flags add policy suggestion and oracle adjudication respectively, each implying `process-data`. The `advanced` flag unlocks ten production-quality capability modules (parallel scanning via `ignore`+`rayon`, BLAKE3 fingerprinting, structured tracing via `tracing`+`tracing-subscriber`, rich diagnostics via `miette`+`thiserror`, TTL-aware caching via `moka`, compact binary snapshots via `bitcode`, dependency graph analysis via `petgraph`, nanosecond timelines via `jiff`, latency histograms via `hdrhistogram`, and multi-pattern scanning via `aho-corasick`). This layered architecture demonstrates that feature-gated capability growth can be achieved without degrading the default user experience or the public API boundary.

**Contribution 4: A separation between suggestion and action in autonomic policies.**
The autonomic policy layer enforces suggest-only mode as a design invariant. Seven policies evaluate distinct dimensions of workspace health (target pressure, toolchain mismatch, trybuild change detection, branch lag, evidence staleness, publish adjudication absence, and dirty git phase) and emit structured `PolicyEntry` records. No policy takes remedial action. This design reflects a principled position on autonomous software: that systems operating in shared developer environments should earn the right to act through a staged escalation from observation to suggestion to confirmed action, not assume authority upfront. The autonomic layer thus serves as a prototype for safe, reversible automation.

---

### 6.2 Limitations of the Current Design (v26.6.2)

Six limitations of the present design deserve explicit acknowledgment.

**L1: External oracle dependency with fragile discovery.** The wasm4pm oracle (`wpm` binary) is discovered through a three-step heuristic: environment variable, a hardcoded path recorded during a capability scan, and PATH lookup. The hardcoded path (`/Users/sac/wasm4pm/target/release/wpm`) is machine-specific and is present in committed source code (`src/integrations/wasm4pm_shell.rs`), which is a portability antipattern. In CI environments without `wpm` installed, tests must declare `ExpectedWpmVerdict::Blocked`, which weakens the evidence gate from a hard quality checkpoint to a conditional one. The deferred library integration (planned for v26.6.3 via the FILE_EXCHANGE path) mitigates but does not eliminate this issue.

**L2: Non-atomic cicd.toml writes.** The `CicdTomlWriter` serializes `EngineState` to `cicd.toml` on the workspace root with a simple `std::fs::write`. There is no atomic rename or file locking. Concurrent invocations of two verbs (feasible in shell scripts or CI parallelism) can produce partial or corrupt `cicd.toml` state. The FAQ in CLAUDE.md acknowledges this but offers no mitigation beyond a note about `--confirm` flags.

**L3: Single-node, single-workspace scope.** All adapters operate on a single workspace root. The `CargoMetadataAdapter` scans Cargo.toml line-by-line for workspace members but does not support nested workspaces, virtual manifests with complex path dependencies, or workspaces spanning networked filesystems. In large monorepos with hundreds of crates and multiple nested virtual manifest layers, this design boundary becomes a hard limitation.

**L4: Conservative trybuild mode blocks compound change sets.** The INVARIANT_NO_FULL_TRYBUILD_BY_DEFAULT rule prevents full trybuild execution when no changed fixtures are detected. While this protects against expensive full runs, it creates a blind spot: if trybuild fixtures test properties that span multiple crates and only one crate's source (but not its corresponding fixture) changes, the fixture mismatch may go undetected until a subsequent full run. The current `ChangedFileDetector` operates on file paths, not on semantic dependency boundaries.

**L5: wasm4pm library coupling deferred with incomplete seam documentation.** The `wasm4pm_current.rs` module documents a 75-capability scan of the wasm4pm library that found 22 USE_AS_IS types, 4 WRAP_LOCAL candidates, and 24 DO_NOT_USE items, with integration deferred because the nightly Rust requirement, unstable core type APIs, and unfinalized receipt ledger schema made adoption risk-prohibitive for v26.6.2. However, the seam is left as an empty placeholder (`Wasm4pmIntegrationSeam` with `_deferred: ()`). The interface contract between cargo-cicd's XES emission and wasm4pm's OCEL import surface is documented in prose but not enforced by a type-level contract or integration test.

**L6: LSP server completeness gap.** The `cargo-cicd-lsp` crate (`crates/cargo-cicd-lsp/`) has a complete architectural skeleton — server lifecycle management (`initialize`, `shutdown`), workspace state tracking, diagnostic storage, file event debouncing, protocol mapping for diagnostics and code actions, and a suite of analyzers covering changed tests, git phase, publish readiness, public boundary violations, target hygiene, close readiness, and evidence events. The `lsp explain` noun is accessible via the CLI. However, the LSP server is not wired to the full `EngineState` at runtime: it does not share the Level 5 engine's state model and runs as a structurally independent process. The diagnostic integration between LSP-emitted evidence and wasm4pm adjudication is not implemented.

---

### 6.3 Threats to Validity

The following threats apply to the claims advanced in this dissertation.

**6.3.1 Internal Validity**

*Construct threat:* The evidence pattern asserts process conformance by having the wasm4pm oracle return `Accept` or `Refuse` for an XES event log. However, the oracle's internal rules — the conditions under which a given XES log is accepted — are not formally specified in this dissertation. If those rules are changed in a future wasm4pm version, tests that currently pass may break without any change to cargo-cicd. The conformance contract is effectively informal and oracle-version-dependent.

*Measurement threat:* The test hierarchy distinguishes Tier 1 (unit/smoke) from Tier 2 (evidence gate). Tier 2 tests that run with `ExpectedWpmVerdict::Blocked` contribute to the green test count but do not exercise the oracle path. Coverage metrics that include Blocked tests may overstate the degree of oracle integration actually exercised in CI.

**6.3.2 External Validity**

*Generalizability:* All experiments and evidence were collected against a single workspace (`cargo-cicd` itself). The claim that the noun-verb grammar and evidence pattern generalize to large, heterogeneous Rust workspaces remains a research hypothesis, not a validated empirical finding. Industry-scale Rust workspaces (e.g., those with 500+ crates, mixed stable/nightly toolchains, platform-conditional compilation) present combinatorial configuration spaces not covered by the current fixture set.

*Oracle portability:* The wasm4pm oracle is a specialized binary with its own process model, version lifecycle, and runtime requirements. Systems that cannot run wasm4pm (air-gapped environments, cross-compilation targets, restricted container sandboxes) cannot exercise the full evidence gate. The `Blocked` fallback path is a safety valve, not a substitute for oracle execution.

**6.3.3 Conclusion Validity**

*Feature flag interaction surface:* The feature matrix (`default`, `process-data`, `autonomic`, `contrib`, `wasm4pm`, `advanced`) defines 2^6 = 64 possible Cargo feature combinations, of which the test suite exercises a small subset. It is possible that certain flag combinations produce unreachable code paths, dead configuration, or subtle behavioral differences not covered by the present tests.

*Ontology-code drift:* The manufacturing pipeline (ggen) produces code from the ontology but no test verifies that ggen has been run after every ontology edit. The `tests/ggen_customization_guard.rs` test guards against some forms of drift, but since ggen outputs are committed to the repository, a developer who edits a noun module directly without updating the ontology can create a discrepancy that passes CI.

---

### 6.4 Future Research Directions

The limitations and threats identified in the preceding sections motivate six research directions that constitute a natural continuation of this work.

**Direction 1: Formal verification of the manufacturing pipeline.**
The ggen pipeline transforms an RDF ontology through SPARQL inference into Rust source code. No formal correctness proof governs this transformation. Future work should investigate whether Lean 4 or Coq can be used to specify and verify the SPARQL-to-code mapping, establishing that for every valid ontology input there exists a unique, correct Rust output modulo operator-defined templates. This would upgrade the manufacturing pipeline from a well-tested heuristic to a formally verified compiler pass.

**Direction 2: A type-level contract for the wasm4pm integration seam.**
The current seam (`Wasm4pmIntegrationSeam`) is a documented placeholder. Future work should define a Rust trait capturing the oracle's adjudication contract — accepting an XES path and returning a structured verdict — so that both the SHELL_OUT adapter (v26.6.2) and the planned FILE_EXCHANGE adapter (v26.6.3) implement the same trait. This makes the integration seam testable with mock oracles and auditable against version changes in wasm4pm's API.

**Direction 3: Semantic impact analysis for test selection.**
The current `ChangedFileDetector` performs file-path-based classification. A richer approach would parse the Rust type system (via `rust-analyzer` APIs or the `rustdoc` JSON output) to build a semantic dependency graph and identify which tests are transitively affected by a change, not just which files changed. The `WorkspaceGraph` in `src/advanced/dep_graph.rs` provides the infrastructure; the missing piece is the per-crate symbol dependency layer.

**Direction 4: Atomic evidence ledger with cryptographic lineage.**
Evidence files are written to `target/cargo-cicd/evidence/` as independent XES and JSONL files with no chain-linking between them. Future work should investigate an append-only, hash-linked evidence ledger where each entry includes a BLAKE3 hash of the preceding entry's content (using the `fingerprint` module as a foundation). This would make the evidence history tamper-evident and enable cryptographic audits of the complete process timeline without relying on filesystem metadata.

**Direction 5: Declarative autonomic policy language.**
The current policy system requires implementing a Rust function for each policy. A more expressive model would define policies as declarative rules in `cicd.toml` (or a companion `policies.toml`), evaluated by a lightweight rule engine. This would allow workspace maintainers to add, modify, or disable policies without recompiling the binary, and would make the policy set inspectable by tooling and documentation generators.

**Direction 6: Cross-language noun-verb grammar.**
The noun-verb grammar as implemented in `clap-noun-verb` is Rust-specific. Investigating whether the same ontology-to-grammar manufacturing pipeline can produce CLI scaffolding for Go, Python, and TypeScript workspaces would test the hypothesis that the design is language-agnostic at the process level even if it is language-specific at the implementation level. This direction connects to the standardization work described in the Vision 2030 roadmap.

---

## Appendix A: Vision 2030 — Strategic Roadmap

### Preamble

The Vision 2030 roadmap extends the research and engineering agenda of cargo-cicd beyond the current v26.6.2 release. It is organized into four time horizons, each with specific implementation targets, research hypotheses, and milestone acceptance criteria. The roadmap is technically grounded: every item names the relevant modules, crates, integration seams, or standards bodies involved, and distinguishes between near-certain implementation work and research-contingent exploration.

The fundamental thesis of the roadmap is that cargo-cicd is best understood not as a CI/CD helper for Rust but as a specimen of a more general class of *process-data engines* — systems that manufacture structured evidence about their own operation and subject that evidence to external adjudication. The 2030 horizon envisions this model maturing into a portable standard for software process conformance, applicable across languages, organizations, and jurisdictions.

---

### H1 — 2026 H2 (Near-Term, Six Months)

**Milestone 1.1: WebAssembly Component Model integration for portable wasm4pm oracles**

*Target:* Replace the SHELL_OUT adapter (`src/integrations/wasm4pm_shell.rs`) with a WebAssembly Component Model host that loads `wpm.wasm` as a guest component.

*Rationale:* The current SHELL_OUT path has three structural weaknesses: it depends on a platform-specific binary, it communicates through untyped string-based stdout parsing, and the hardcoded fallback path creates a portability defect. The WebAssembly Component Model (WASM-CM), as specified by the W3C WebAssembly Working Group, provides typed interface definitions (WIT files) through which a host can load a component and call exported functions with structured arguments and return types.

*Implementation strategy:* 
1. Define a WIT interface `wasm4pm-oracle.wit` capturing the `audit(xes-bytes: list<u8>) -> verdict` contract.
2. Implement a Rust host using `wasmtime` (or `wasmi` for a lighter runtime) that loads the interface.
3. Implement a thin adapter crate `cargo-cicd-wasm-oracle` that satisfies the same Rust trait as the current `Wasm4pmShell`, so the call site in evidence emission does not change.
4. Gate the component host behind a new `wasm-oracle` feature flag.
5. Retain the SHELL_OUT adapter as the default fallback for environments without WASM runtime support.

*Acceptance criteria:* `cargo test --features wasm-oracle` passes all evidence gate tests (`wasm4pm_evidence_gate`, `wasm4pm_evidence_mutation`, `wasm4pm_refusal_cases`) without invoking a native `wpm` binary. The hardcoded path in `wasm4pm_shell.rs` is removed.

*Research hypothesis H1.1:* WASM component model typed interfaces reduce oracle integration failures caused by stdout-parsing ambiguity by at least 80% in integration tests, as measured by the proportion of false-positive `Warn` verdicts from `infer_verdict()`.

---

**Milestone 1.2: Full LSP server integration with the Level 5 engine**

*Target:* Wire `cargo-cicd-lsp` to `EngineState::from_workspace()` so that IDE diagnostics reflect live Level 5 engine state, and extend the `lsp explain` verb to surface evidence summaries inline in editor hover responses.

*Rationale:* The current LSP crate (`crates/cargo-cicd-lsp/`) has a structurally sound server lifecycle, diagnostic storage, and analyzer suite (fourteen analyzer modules covering changed tests, git phase, publish readiness, public boundary, target hygiene, close readiness, runtime court, evidence, rendered surface, workspace structure, pipeline checks, remote tracking, and URI mapping). However, the LSP does not share the engine's state model. Consequently, its diagnostics are produced by independent analyses that may diverge from what `cargo cicd status show` reports.

*Implementation strategy:*
1. Extract `EngineState::from_workspace()` into a publicly accessible async function in `cargo-cicd-core`.
2. Call this from the LSP backend's `TextDocumentDidSave` and `WorkspaceDidChangeWatchedFiles` handlers.
3. Map each `PolicyEntry` to an LSP `Diagnostic` with appropriate severity (`Warn` → Warning, `FAIL` → Error).
4. Expose the most recent `ProcessEvent` as a hover response for `.rs` files that changed it.
5. Emit an LSP progress notification (`$/progress`) during `EngineState::from_workspace()` so the editor UI shows that the engine is working.

*Acceptance criteria:* A VS Code extension test (using `@vscode/extension-tester`) that opens the cargo-cicd workspace, edits `src/main.rs`, saves the file, and asserts that a `target_pressure` policy diagnostic appears in the Problems panel within five seconds.

---

**Milestone 1.3: Distributed workspace support for monorepos with 100+ crates**

*Target:* Extend `CargoMetadataAdapter` and `ChangedFileDetector` to handle virtual manifest workspaces, nested workspace members, and monorepos where the Cargo workspace root is not the repository root.

*Rationale:* The current `CargoMetadataAdapter` performs a line-by-line scan of `Cargo.toml` looking for `members = [...]`. This approach fails for: (a) virtual manifests where no `[package]` section exists; (b) glob patterns in `members` (e.g., `members = ["crates/*"]`); (c) workspace members that are themselves workspace roots (nested workspaces). At the 100-crate scale, the `TargetScannerAdapter`'s sequential `walkdir` traversal also becomes prohibitively slow.

*Implementation strategy:*
1. Replace the line-by-line scan with a proper TOML parse of `Cargo.toml` using the existing `toml` crate dependency, resolving glob patterns in `members` via the `glob` crate.
2. Integrate `parallel_scan::scan()` (already implemented in `src/advanced/parallel_scan.rs`) as the `TargetScannerAdapter` backend when the `advanced` feature is enabled, enabling multi-core workspace traversal.
3. Add a `DistributedWorkspaceAdapter` that accepts a workspace root prefix and scans multiple `Cargo.toml` files across a repository tree, merging results into a unified `WorkspaceState`.
4. Benchmark against a synthetic 100-crate workspace generated by a fixture builder in `tests/fixture_workspaces.rs`, targeting sub-second full-workspace scan.

*Acceptance criteria:* `cargo cicd status show` on a 100-crate workspace with a virtual manifest root completes in under 1 second on a 4-core laptop. All 100 crate names appear in the emitted `cicd.toml` `[workspace]` members list.

---

**Milestone 1.4: Advanced feature set completion**

All ten modules in `src/advanced/` are implemented. The remaining integration work is to connect them to the live engine.

| Module | Current state | Remaining work |
|---|---|---|
| `parallel_scan` | Complete, unit-tested | Wire to `TargetScannerAdapter` behind `advanced` flag |
| `fingerprint` | Complete, unit-tested | Wire to `CicdTomlWriter` for content-addressed evidence filenames |
| `observability` | Complete, unit-tested | Call `init_tracing()` in `main()` behind `advanced` flag; wrap all adapter calls in `PipelineStage` |
| `diagnostics` | Complete (miette+thiserror) | Replace `anyhow` error propagation in noun handlers with `miette` report types |
| `cache` | Complete, unit-tested | Wire `EngineCache` to `CargoMetadataAdapter` and `ToolchainDetector` with 5-minute TTL |
| `snapshot` | Complete, unit-tested | Call `encode()` after `EngineState::from_workspace()` to write `target/cargo-cicd/snapshots/latest.bin` |
| `dep_graph` | Complete, unit-tested | Wire `WorkspaceGraph` to `ChangedFileDetector::dependents_of()` for semantic test selection |
| `timeline` | Complete, unit-tested | Replace `evidence::now_iso8601()` with `ProcessTimeline::record()` for nanosecond precision |
| `histogram` | Complete, unit-tested | Add `StageLatencies` tracking to each adapter call; surface percentiles in `status show` output |
| `pattern` | Complete | Wire `PatternScanner` to `workspace doctor` for multi-pattern governance checks |

*Acceptance criteria:* `cargo test --features advanced` passes all tests. `cargo cicd status show --features advanced` outputs a stage latency table (p50/p90/p99 in milliseconds) for each adapter invocation.

---

### H2 — 2027 (Medium-Term)

| # | Item | Key files/crates | Research hypothesis |
|---|---|---|---|
| 2.1 | Declarative pipeline composition | New: `pipeline.toml` DSL | Declarative pipelines reduce accidental CI ordering bugs by 60% vs. imperative scripts |
| 2.2 | Multi-oracle adjudication consensus | `src/integrations/` | Consensus across N>1 oracle instances improves verdict stability for borderline XES logs |
| 2.3 | Real-time evidence streaming | New: WebSocket/SSE transport | Sub-100ms latency for evidence delivery to monitoring dashboards |
| 2.4 | Formal ontology expansion | `ontology/cargo-cicd-capabilities.ttl` | SPARQL inference over the ontology can automate test-module assignment |
| 2.5 | External tool integration | New adapters in `src/adapters/` | Integration with cargo-dist/cargo-release/cargo-nextest reduces release ceremony time by 40% |

**Milestone 2.1: Declarative pipeline composition (pipeline.toml DSL)**

The `pipeline run` verb currently executes a hardcoded sequence of CI/CD activities. Milestone 2.1 replaces this with a declarative DSL in which users specify the pipeline as an ordered list of steps with conditional expressions.

```toml
# Example pipeline.toml DSL (v27 target syntax)
[pipeline]
name = "release"
on = ["push", "tag"]

[[pipeline.step]]
name = "status"
command = "cargo cicd status show"
pass_if = "verdict == PASS"

[[pipeline.step]]
name = "test"
command = "cargo cicd test changed"
pass_if = "verdict != FAIL"
skip_if = "changed_files == []"

[[pipeline.step]]
name = "publish"
command = "cargo cicd publish run"
depends_on = ["status", "test"]
requires_oracle = true
```

*Implementation strategy:*
1. Define the `PipelineToml` struct in `src/pipeline.rs` and add serde deserialization from `pipeline.toml`.
2. Add a `PipelineParser` adapter in `src/adapters/pipeline_parser.rs` that validates the DSL and returns a `Vec<PipelineStep>`.
3. Modify `src/nouns/pipeline.rs` to read from `pipeline.toml` when present, falling back to the hardcoded sequence.
4. Implement conditional expression evaluation using a minimal expression parser (no external dependency needed for the initial scope).
5. Emit one `ProcessEvent` per step, grouped into a single `<trace>` in the XES output with `case_id = pipeline.<pipeline.name>`.

*Research hypothesis H2.1:* Projects using declarative `pipeline.toml` exhibit 60% fewer accidental step-ordering bugs (defined as a publish step executing before its test precondition passes) compared to equivalent imperative shell scripts, measured across a synthetic benchmark of 20 pipeline configurations.

---

**Milestone 2.2: Multi-oracle adjudication with consensus**

A single oracle instance may be temporarily unavailable, may have a stale rule set, or may produce inconsistent verdicts across versions. Milestone 2.2 introduces a `MultiOracleConsensus` adapter that queries N oracle instances and applies a configurable consensus rule (majority vote, unanimous, or first-responder).

*Implementation strategy:*
1. Introduce an `Oracle` trait in `src/integrations/oracle.rs`:
   ```rust
   pub trait Oracle: Send + Sync {
       fn audit(&self, xes_path: &str) -> Result<WpmVerdict>;
       fn name(&self) -> &str;
   }
   ```
2. Implement `ShellOracle` (wrapping `Wasm4pmShell`) and `WasmOracle` (wrapping the WASM component from Milestone 1.1) as concrete types.
3. Implement `ConsensusOracle { oracles: Vec<Box<dyn Oracle>>, policy: ConsensusPolicy }` with policies `Majority`, `Unanimous`, and `FirstResponder`.
4. Surface the consensus result in the `ProcessEvent.verdict_adjudicated` field with an annotation identifying which oracles agreed.

*Research hypothesis H2.2:* Majority consensus across three oracle instances reduces the false-positive `Refuse` rate (verdicts that disagree with human expert review) by at least 35% compared to a single-oracle configuration, measured across the `wasm4pm_refusal_cases` test corpus expanded to 50 cases.

---

**Milestone 2.3: Real-time evidence streaming via WebSocket/SSE**

The current evidence model is file-based: XES and JSONL files are written to `target/cargo-cicd/evidence/` and consumed by the oracle after the command completes. Milestone 2.3 adds a streaming mode in which process events are emitted over a WebSocket or SSE connection in real time, enabling live monitoring dashboards.

*Implementation strategy:*
1. Add a `cargo cicd evidence stream` verb that starts a local HTTP server (using `axum`) on a configurable port.
2. Subscribe to `ProcessEvent` emissions via an `mpsc` channel wired to the `ProcessEventState` population path.
3. Serialize each event to JSONL and push it to all connected SSE clients.
4. Provide a companion HTML dashboard (`target/cargo-cicd/dashboard.html`) generated by the `evidence stream` verb that visualizes the live event feed.

*Acceptance criteria:* A `cargo cicd pipeline run` in another terminal produces live event updates in the dashboard within 50ms of each adapter completing.

---

**Milestone 2.4: Formal ontology expansion — SPARQL inference for automated test selection**

The current ontology defines nouns, verbs, evidence events, feature projections, and policies. Milestone 2.4 extends it with a test-assignment subontology: for each capability (`cc:TestChanged`, `cc:StatusShow`, etc.), a SPARQL CONSTRUCT rule infers which test modules are normatively required.

```turtle
# Example inference rule (SPARQL CONSTRUCT)
CONSTRUCT {
    ?capability cc:requiresTest ?testModule .
}
WHERE {
    ?capability cc:noun ?noun ;
                cc:verb ?verb .
    ?testModule a cc:TestModule ;
                cc:covers ?noun .
}
```

This enables `ggen` to generate not only the noun module scaffolding but also a test coverage map that can be consumed by `cargo cicd test changed` to select the normatively required test subset for each changed capability.

---

**Milestone 2.5: External tool integration — cargo-dist, cargo-release, cargo-nextest**

Three popular Cargo ecosystem tools have natural integration points with cargo-cicd:

- **cargo-dist**: The `publish run` verb can call `cargo-dist` to produce release artifacts and include the resulting manifest in the `PublishRunEvent`.
- **cargo-release**: The `git close` verb can delegate version bumping and changelog management to `cargo-release`, capturing its exit status as part of the `GitCloseEvent`.
- **cargo-nextest**: The `test changed` verb can replace its current `cargo test` invocation with `cargo nextest run --filter-expr` to gain per-test timing and retry semantics.

*Implementation strategy for cargo-nextest:*
1. Add a `NextestAdapter` in `src/adapters/nextest.rs` that detects `cargo-nextest` on PATH.
2. Modify `src/nouns/test.rs` to prefer `NextestAdapter::run_changed()` when available.
3. Parse nextest's JSON output (`--message-format json`) to populate `TestPlanState` with per-test durations.
4. Surface the p99 per-test duration in `status show` output when `advanced` is enabled (using `StageLatencies`).

---

### H3 — 2028 (Growth)

| # | Item | Standard/Reference | Key Implementation Risk |
|---|---|---|---|
| 3.1 | ISO/IEC 33001 process conformance checking | ISO/IEC 33001:2015 | Mapping cargo-cicd process models to SPA/SPE framework is non-trivial |
| 3.2 | Federated evidence ledger | XES + BLAKE3 Merkle chain | Cross-workspace aggregation requires stable IRI-based workspace identity |
| 3.3 | Predictive CI/CD via ML build time estimation | `ChangedFileState`, `EngineSnapshot` | Feature engineering from file-level change data is research-grade |
| 3.4 | Declarative autonomic policy language | `policies.toml` DSL | Policy evaluation semantics must be formally specified to avoid conflicts |

**Milestone 3.1: ISO/IEC 33001 process conformance checking**

ISO/IEC 33001:2015 (Software and systems engineering — Process assessment) specifies a framework for assessing the capability and maturity of software processes. The XES evidence emitted by cargo-cicd is structurally compatible with process assessment records: each `<trace>` in the XES output corresponds to a process instance, and the `verdict_claimed`/`verdict_adjudicated` fields map to process performance indicators.

*Research direction:* Map the nine cargo-cicd capability types (status show, target show, target prune, test changed, trybuild changed, git status, git close, publish run, workspace doctor) to ISO/IEC 33001 process attributes (PA 1.1 Process Performance, PA 2.1 Performance Management, PA 2.2 Work Product Management). Extend the XES schema with attribute keys that carry the ISO process capability dimension so that wasm4pm can evaluate conformance against normative process attribute ratings.

*Implementation strategy:*
1. Add `pa_11_performance`, `pa_21_management`, `pa_22_work_product` fields to `ProcessEvent`.
2. Populate these fields from the evidence emission pattern based on the presence of start/complete event pairs and oracle verdicts.
3. Extend the XES emission in `src/evidence.rs` to include these as `<string>` attributes on the `<event>` element.
4. Publish a `wasm4pm-33001` oracle rule set (separate repository) that adjudicates XES logs against the ISO framework.

*Research hypothesis H3.1:* Cargo workspaces that achieve PA 1.1 rating via cargo-cicd evidence demonstrate statistically significant reduction in regression defect rate (measured as issues reopened within 30 days), compared to workspaces using only conventional CI without structured evidence emission.

---

**Milestone 3.2: Federated evidence ledger — cross-workspace XES aggregation**

Individual cargo-cicd workspaces emit evidence to their local `target/cargo-cicd/evidence/` directory. In an organization with multiple Rust workspaces, there is no mechanism for aggregating evidence across workspaces for portfolio-level process assessment.

*Implementation strategy:*
1. Assign each workspace a globally unique IRI (the repository URL suffices: `https://github.com/seanchatmangpt/cargo-cicd`), stored in `cicd.toml` under `[workspace]`.
2. Extend `CicdTomlWriter` to include this IRI in every `ProcessEvent`.
3. Implement a `cargo cicd evidence aggregate` verb that reads evidence from multiple workspace paths (specified in a `[federation]` section of `cicd.toml`) and produces a merged XES log with cross-workspace trace grouping.
4. Chain evidence files using BLAKE3 hashes: each `ProcessEvent` includes a `predecessor_hash` field set to the BLAKE3 hash of the previous event's XES serialization, using `src/advanced/fingerprint.rs`.

*Research hypothesis H3.2:* Hash-chained cross-workspace evidence provides tamper-detection sensitivity of 100% for single-event mutations (as verified by the `wasm4pm_evidence_mutation` test pattern extended to federated logs), with less than 5% overhead in evidence emission time.

---

**Milestone 3.3: Predictive CI/CD — ML-based build time estimation**

The `EngineSnapshot` structure in `src/advanced/snapshot.rs` captures `changed_files`, `target_bytes`, and `stages` (per-stage timing records). Across many snapshots, this data constitutes a training corpus for a model that predicts build time given a change set.

*Implementation strategy:*
1. Accumulate `EngineSnapshot` records in `target/cargo-cicd/snapshots/history.bin` using an append-only bitcode log.
2. Implement a `PredictionAdapter` in `src/adapters/prediction.rs` that reads the history log and applies a linear regression model (no ML framework dependency needed for the initial model) to predict `elapsed_ms` for the current `ChangedFileState`.
3. Surface the prediction in `status show` output: "Estimated build time: 47s (±12s based on 83 prior runs)".
4. Validate the prediction against actual timing recorded in `StageLatencies`.

*Research hypothesis H3.3:* A linear regression model trained on 100 prior `EngineSnapshot` records achieves a mean absolute percentage error (MAPE) below 25% for predicting `test changed` execution time, using `changed_files.len()` and `target_bytes` as primary predictors.

---

**Milestone 3.4: Declarative autonomic policy language**

The current policy system (seven hard-coded Rust functions in `src/policies/`) requires a code change and recompile to add, modify, or disable a policy. Milestone 3.4 introduces a declarative policy language in `cicd.toml`.

```toml
# Example declarative policy section (v28 target syntax)
[[policy]]
name = "target_pressure"
enabled = true
condition = "target.total_size_bytes > 5368709120"  # 5 GiB
recommendation = "Run `cargo cicd target prune` to reclaim disk space."
verdict = "WARN"

[[policy]]
name = "custom_branch_lag"
enabled = true
condition = "git.behind_count > 10"
recommendation = "Your branch is significantly behind main. Pull or rebase."
verdict = "WARN"
```

*Implementation strategy:*
1. Add a `PolicyDsl` struct deserializable from `[[policy]]` sections in `cicd.toml`.
2. Implement a `DslPolicyEvaluator` in `src/autonomic/policy_engine.rs` that evaluates condition expressions against `EngineState` fields using a simple arithmetic expression parser.
3. Merge DSL policies with built-in policies in `policies::run_all_policies()`, giving user-defined policies higher precedence on name collision.
4. Test with the existing `tests/autonomic_policies.rs` test suite extended to cover DSL policies.

---

### H4 — 2029–2030 (Visionary)

This horizon contains the highest-risk, highest-impact items. Each requires sustained research effort, community building, and coordination with external standards bodies.

| # | Item | Technical Prerequisite | Standards Body | Risk Level |
|---|---|---|---|---|
| 4.1 | Formal verification of the manufacturing pipeline | Lean 4 bindings for SPARQL algebra | None (internal) | High (research) |
| 4.2 | Zero-trust supply chain via cryptographically signed evidence chains | Milestone 3.2 (federated ledger) + COSE signatures | IETF (RFC 8152) | Medium |
| 4.3 | Cross-language noun-verb grammar | Stable WIT IDL for `clap-noun-verb` | W3C WASM WG | Medium |
| 4.4 | Autonomous remediation with reversibility guarantees | Milestone 3.4 (policy DSL) | None (internal) | High |
| 4.5 | XES+WASM as W3C/ISO standard | Milestones 1.1, 2.2, 3.1 | W3C, ISO/IEC JTC 1/SC 7 | Very high (political) |

**Milestone 4.1: Formal verification of the manufacturing pipeline**

*Technical vision:* Prove in Lean 4 that the SPARQL-to-Rust transformation performed by `ggen` is correct with respect to a formal specification of the noun-verb grammar. Specifically, prove that for every valid ontology triple `(capability, cc:noun, noun) ∧ (capability, cc:verb, verb)`, the generated Rust code produces a clap `Command` with a subcommand named `noun` containing a subcommand named `verb`.

*Implementation roadmap:*
1. Formalize the ontology as a Lean 4 inductive type `Capability`.
2. Specify the SPARQL inference rules as Lean 4 functions over `Capability`.
3. Define the target Rust AST fragment as a Lean 4 inductive type `ClapCommand`.
4. Prove the translation function `ggen: Capability → ClapCommand` satisfies the grammar invariant.
5. Extract a verified Lean implementation as a companion to the existing ggen Tera-template implementation, running both in CI and asserting output equivalence.

*Research hypothesis H4.1:* A Lean 4 proof of the manufacturing pipeline transformation eliminates the entire class of ontology-code drift defects (currently guarded by `tests/ggen_customization_guard.rs`) without requiring runtime test execution.

---

**Milestone 4.2: Zero-trust supply chain — cryptographically signed evidence chains**

Building on the hash-chained federated ledger from Milestone 3.2, this milestone adds cryptographic signatures to evidence events so that each event can be independently verified as having been produced by a specific key holder.

*Implementation strategy:*
1. Generate an Ed25519 signing key per workspace, stored in `~/.config/cargo-cicd/signing.key` (never committed to the repository).
2. Sign each `ProcessEvent` with the workspace signing key using the `ed25519-dalek` crate, storing the signature as a base64 field in the XES `<event>` element.
3. Publish the corresponding verification key to a well-known URL (`https://<repo-host>/.well-known/cargo-cicd-signing-key.json`) or embed it in the `cicd.toml` under `[workspace.signing]`.
4. Extend the wasm4pm oracle to verify signatures before adjudicating XES logs, returning `Refuse` for unsigned events when signing is configured.
5. Use CBOR Object Signing and Encryption (COSE, IETF RFC 8152) as the signature envelope format to align with emerging supply chain standards (SLSA, SCITT).

*Research hypothesis H4.2:* Ed25519-signed evidence chains detect 100% of post-emission tampering attempts (single-event insertion, deletion, field modification) with less than 2% overhead on evidence emission throughput.

---

**Milestone 4.3: Cross-language noun-verb grammar**

*Vision:* The ontology-to-CLI manufacturing pipeline is language-agnostic at the semantic level. The noun-verb grammar, default verb injection, evidence emission pattern, and cicd.toml state carrier can be implemented in any language. Milestone 4.3 produces reference implementations for Go, Python, and TypeScript, all manufactured from the same `ontology/cargo-cicd-capabilities.ttl` source.

*Implementation strategy:*
1. Abstract the `ggen.toml` template system to support multiple output targets, each with its own set of Tera templates: `templates/go/`, `templates/python/`, `templates/typescript/`.
2. For each language, implement the evidence emission pattern (start event, work, complete event, XES serialization) as a library: `cargo-cicd-go`, `cargo-cicd-python`, `cargo-cicd-ts`.
3. Define the oracle adjudication interface as a WIT (WebAssembly Interface Type) file so that the same wasm4pm oracle can be called from any language's WASM runtime.
4. Publish the WIT definition as part of the `clap-noun-verb` crate's public interface, renamed `process-data-grammar.wit`.

*Research hypothesis H4.3:* Cross-language noun-verb implementations manufactured from the same ontology source produce XES logs that are adjudicated identically by the wasm4pm oracle for equivalent process traces, as verified by a cross-language oracle conformance test suite.

---

**Milestone 4.4: Autonomous remediation with reversibility guarantees**

The autonomic policy layer in v26.6.2 is enforced to suggest-only mode. This is a principled position, but it creates friction when the recommended action is unambiguous, safe, and frequently accepted. Milestone 4.4 introduces a first class of *autonomic actions* that can be executed automatically, subject to reversibility and safety constraints.

*Design principles for safe autonomous action:*
1. **Reversibility:** Every autonomous action must be undoable. The action must record a `RollbackEvent` with sufficient information to restore the prior state (e.g., for `target prune`, record the sizes and mtimes of deleted artifacts before deletion).
2. **Confirmation threshold:** An action is only executed autonomously if it has been manually confirmed at least N times (configurable, default N=3) in the session history stored in `cicd.toml`.
3. **Scope limitation:** Autonomous actions are limited to read-only filesystem operations plus the pre-approved destructive set (`target prune`, `cargo update`, `git stash`). They may never modify committed history or publish to external registries.
4. **Audit trail:** Every autonomous action emits a `ProcessEvent` with `lifecycle_transition = "autonomous"` so the action is distinguishable from user-initiated actions in the XES log.

*Candidate first-class autonomous actions:*
- `target prune --confirm` when `target_pressure` policy fires and workspace has been accumulating build artifacts for more than 7 days.
- `cargo update --workspace` when `cargo_lock_age` policy fires and the branch is clean.
- `git stash` before `git close` when `git_phase_dirty` policy fires and there are unstaged changes that do not touch tracked files.

*Research hypothesis H4.4:* Limiting autonomous action eligibility to the reversibility-qualified set and requiring N-confirmation reduces unintended data loss incidents (measured across a simulated user study of 50 developers using cargo-cicd for 30 days) to zero, while reducing the number of manual confirmations required per working session by 40%.

---

**Milestone 4.5: XES+WASM as a W3C/ISO standard for process evidence**

The most ambitious item on the roadmap is the proposal of the core cargo-cicd evidence model as a public standard. The model consists of:
- **XES** (IEEE Std 1849-2016) as the process event log format, extended with cargo-cicd's `verdict_claimed`, `verdict_adjudicated`, and `lifecycle_transition` attributes.
- **WebAssembly** as the portable oracle runtime, with a defined WIT interface for process adjudication.
- **BLAKE3 hash chains** as the tamper-evident ledger mechanism.

Together these three components define a *software process evidence standard* that is language-agnostic, oracle-portable, and cryptographically auditable.

*Path to standardization:*
1. Submit the XES extension vocabulary (`verdict_claimed`, `verdict_adjudicated`, `lifecycle_transition`, `trace_class`) as a proposed extension to the IEEE 1849-2016 working group.
2. Publish the WIT interface for process oracle adjudication as a W3C WebAssembly Community Group note, aligning with the Component Model specification track.
3. Propose the full XES+WASM+BLAKE3 stack to ISO/IEC JTC 1/SC 7 (Software and Systems Engineering) as a technical report under the family of ISO/IEC 330xx process assessment standards.
4. Build community by publishing the wasm4pm oracle's rule set as a public, versioned registry (analogous to crates.io but for process conformance rules), enabling any tool to submit XES evidence for adjudication by a published, auditable rule set.

*Research hypothesis H4.5:* A publicly available XES+WASM process evidence standard, adopted by at least three CI/CD tools from three different language ecosystems, enables cross-ecosystem process benchmarking studies that are currently not possible due to incommensurable evidence formats.

---

### Summary Milestone Table

| Milestone | Horizon | Primary module(s) affected | Key dependency | Risk |
|---|---|---|---|---|
| 1.1 WASM Component Model oracle | 2026 H2 | `src/integrations/` | wasmtime crate, WIT spec | Low |
| 1.2 LSP + Level 5 engine integration | 2026 H2 | `crates/cargo-cicd-lsp/` | EngineState async refactor | Medium |
| 1.3 Distributed workspace support | 2026 H2 | `src/adapters/` | Virtual manifest TOML parse | Low |
| 1.4 Advanced feature set wiring | 2026 H2 | `src/advanced/`, all nouns | None (all modules exist) | Low |
| 2.1 Declarative pipeline DSL | 2027 | `src/nouns/pipeline.rs` | New `pipeline.toml` schema | Low |
| 2.2 Multi-oracle consensus | 2027 | `src/integrations/` | Oracle trait abstraction | Medium |
| 2.3 Real-time evidence streaming | 2027 | `src/nouns/evidence.rs` | axum HTTP server dep | Low |
| 2.4 Ontology SPARQL test assignment | 2027 | `ontology/`, `ggen.toml` | SPARQL reasoning completeness | Medium |
| 2.5 cargo-dist/release/nextest integration | 2027 | `src/adapters/` | External tool API stability | Low |
| 3.1 ISO/IEC 33001 conformance checking | 2028 | `src/evidence.rs`, XES schema | ISO mapping research | High |
| 3.2 Federated evidence ledger | 2028 | `src/adapters/`, `src/evidence.rs` | Workspace IRI stability | Medium |
| 3.3 ML build time prediction | 2028 | `src/adapters/prediction.rs` | Feature engineering quality | High |
| 3.4 Declarative policy language | 2028 | `src/autonomic/`, `cicd.toml` | DSL semantics formalization | Medium |
| 4.1 Lean 4 formal verification | 2029 | ggen, ontology | Lean 4 SPARQL formalization | Very high |
| 4.2 Zero-trust supply chain | 2029 | `src/evidence.rs`, COSE | Ed25519 key management UX | Medium |
| 4.3 Cross-language grammar | 2029 | ggen templates, WIT | Multi-language template maintenance | High |
| 4.4 Autonomous remediation | 2030 | `src/autonomic/` | Reversibility guarantee proofs | High |
| 4.5 XES+WASM ISO standard | 2030 | All | Standards body engagement | Very high |

---

### Closing Observations

The cargo-cicd project, in its v26.6.2 form, represents a proof of concept for a broader proposition: that CI/CD tooling should be as rigorous about its own process as it is about the processes it orchestrates. The noun-verb grammar manufactured from a formal ontology, the evidence emission pattern with external oracle adjudication, and the suggest-only autonomic policy layer are three expressions of this proposition at different levels of the stack.

The Vision 2030 roadmap extends this proposition toward its natural conclusion. If process evidence is to have genuine value — if an `Accept` verdict from the wasm4pm oracle is to carry the same epistemic weight as a code review approval or a test suite green — then the evidence model must be formally specified, cryptographically anchored, and recognized by standards bodies. The path from cargo-cicd's current SHELL_OUT oracle integration to an ISO-standardized XES+WASM process evidence stack is long but traceable: each milestone in this roadmap is a waypoint on that path, not an isolated technical exercise.

The limiting factor is not technical. The hardware, language tooling, and algorithmic foundations required for every milestone in this roadmap exist today or are in active development. The limiting factor is the community of practice: developers who believe that process evidence matters, organizations willing to invest in oracle infrastructure, and standards bodies willing to extend existing frameworks to accommodate software-native process models. Building that community is the work of the decade.

---

*End of Chapter 6 and Appendix A.*
