# Vision 2030 Repo Survey
## Five-Agent Swarm Synthesis

> **Document status:** Internal research survey — cargo-cicd v26.6.2
> **Date:** 2026-06-16
> **Scope:** Synthesis of five parallel agent surveys covering 21 repositories across five domains
> **Purpose:** Map external repository assets to Vision 2030 milestones; identify gaps requiring greenfield work

---

## 1. Executive Summary — Top 10 Repositories by Vision 2030 Impact

The five-agent swarm surveyed 21 repositories across process mining, grammar manufacturing, knowledge graphs, AI/ML autonomic systems, and Rust infrastructure. Of these, 10 have direct, concrete impact on the Vision 2030 roadmap defined in Appendix A of Chapter 6. The remaining 11 are either no-match, private/deleted, or reference designs with high porting cost.

**Top 10 repositories by milestone coverage breadth:**

| Rank | Repository | Domain | Milestones | Recommendation |
|------|-----------|--------|------------|----------------|
| 1 | **wasm4pm** | Process Oracle | 1.1, 2.3, 3.1, 3.2, 4.2, 4.5 | Already adopted |
| 2 | **ggen** | Code Generation | 1.4, 2.1, 2.4, 4.3 | Already adopted |
| 3 | **rdddy** | Autonomic / DDD | 4.4, 2.2 | Fork and adapt |
| 4 | **dspygen** | ML / Pipeline DSL | 2.1, 3.3, 3.4 | Fork and adapt |
| 5 | **clap-noun-verb** | CLI Grammar | 1.4, 4.3 | Already adopted |
| 6 | **unrdf** | RDF / SPARQL | 2.4, 4.3, 4.5 | Fork and adapt |
| 7 | **qlever_poc** | SPARQL Query | 2.4, 4.5 | Fork and adapt |
| 8 | **dslmodel** | DSL / Governance | 2.1, 2.2, 3.4 | Reference design |
| 9 | **capability-map** | Capability Scanning | 2.4, 1.4 | Reference design |
| 10 | **bcinr** | Branchless Algorithms | 3.2, 4.2 (partial) | Reference design |

**Strategic observations:**

1. The oracle and manufacturing stacks (wasm4pm, ggen, clap-noun-verb) are already integrated — the swarm confirms this is the correct foundation and no realignment is needed.
2. The biggest gap is the **autonomic-to-action escalation** layer (Milestones 4.4, 2.2): rdddy provides the closest behavioral model but requires a Rust port from Python.
3. The ML/predictive layer (Milestones 3.3, 3.4) is covered conceptually by dspygen and dslmodel but neither provides Rust library assets. This remains largely greenfield in Rust.
4. Cross-language grammar (Milestone 4.3) is partially served by ggen's template architecture and unrdf's JS codegen pattern, but a WIT IDL definition is entirely greenfield.
5. Milestones 4.1 (Lean 4 formal verification) and 4.5 (ISO standardization) have **zero repo coverage** — they require sustained research and community engagement outside any existing codebase.

---

## 2. Priority Ranking Table

| Rank | Repository | Domain | Milestones Covered | Effort | Recommendation |
|------|-----------|--------|--------------------|--------|----------------|
| 1 | wasm4pm | Process Oracle | 1.1, 2.3, 3.1, 3.2, 4.2, 4.5 | Low | Adopt as dependency (already adopted) |
| 2 | ggen | Code Generation | 1.4, 2.1, 2.4, 4.3 | Low | Adopt as dependency (already adopted) |
| 3 | rdddy | Autonomic / Reactive DDD | 4.4, 2.2 | Low | Fork and adapt |
| 4 | dspygen | DSPy / ML Pipeline | 2.1, 3.3, 3.4 | Medium | Fork and adapt |
| 5 | clap-noun-verb | CLI Grammar Crate | 1.4, 4.3 | Low | Adopt as dependency (already adopted) |
| 6 | unrdf | RDF/SPARQL JS | 2.4, 4.3, 4.5 | Medium | Fork and adapt |
| 7 | qlever_poc | Async SPARQL Client | 2.4, 4.5 | Medium | Fork and adapt |
| 8 | dslmodel | OpenTelemetry DSL | 2.1, 2.2, 3.4 | Medium | Reference design |
| 9 | capability-map (cpmp) | RDF Capability Scanning | 2.4, 1.4 | Medium | Reference design |
| 10 | bcinr | Branchless / no_std | 3.2, 4.2 (partial) | Medium | Reference design |
| 11 | gitvan | Git-as-Runtime (JS) | 1.3 | High | Reference design |
| 12 | qlever | C++ SPARQL Database | 2.4, 4.5 | High | Reference design |
| 13 | ash_swarm | Elixir Reactor | 4.4 | High | Reference design |
| 14 | pm4py-mcp | Python XES Parser | 2.3, 3.1 | N/A | Reference design |
| 15 | process-intelligence | Python XES Tools | — | N/A | No match |
| 16 | ggen-mcp | XLSX Server | — | N/A | No match |
| 17 | tower-lsp-composition | Private LSP | — | N/A | No match |
| 18 | gitgym | Nuxt UI Template | — | N/A | No match |
| 19 | unibit | Private/Deleted | — | N/A | No match |
| 20 | knhk | Private | — | N/A | No match |
| 21 | kgold | Private/Deleted | — | N/A | No match |

---

## 3. Per-Repository Structured Assessment

### wasm4pm

- **What it is:** The external process oracle that adjudicates XES evidence logs and issues Accept/Refuse/Blocked verdicts for cargo-cicd process conformance.
- **Language/stack:** Rust (oracle binary, `wpm`), WebAssembly output target
- **Relevant milestones:** [1.1, 2.3, 3.1, 3.2, 4.2, 4.5]
- **Reusable assets:** 112 scanned capabilities; confirmed SHELL_OUT path for 7 commands (`wpm audit`, `wpm receipt doctor`, `wpm receipt verify-ocel2`, `wpm receipt canonicalize-ocel2`, `wpm receipt detect-fixture-mutation`, `wpm receipt verify-boundary-evidence`, `wpm receipt verify-proof-class`, `wpm autoprocess`); USE_AS_IS types: EventLog, Trace, OCEL, PetriNet, DFG, ConformanceResult, Blake3Hash
- **Integration effort:** Low
- **Recommendation:** Adopt as dependency (already adopted)
- **Key finding:** wasm4pm is the most critical external dependency in the Vision 2030 stack. Its 112-capability surface scan confirmed 22 USE_AS_IS types, 4 WRAP_LOCAL candidates, and 24 DO_NOT_USE items. The SHELL_OUT integration path is live and tested; the deferred FILE_EXCHANGE path targets v26.6.3. For Milestone 1.1 (WASM Component Model), wasm4pm itself is the guest component — the work is on the cargo-cicd host side (implementing a WIT interface and wasmtime/wasmi loader). Milestones 3.1 and 4.5 depend on extending wasm4pm's internal rule set to cover ISO/IEC 33001 process attributes and a published oracle registry, respectively. wasm4pm's OCEL2 receipt verification and Blake3Hash primitive directly underpin Milestones 3.2 and 4.2.

---

### ggen

- **What it is:** A Rust code-generation engine that applies SPARQL inference rules over RDF ontologies and renders output through Tera templates, manufacturing the cargo-cicd CLI grammar and documentation from `cargo-cicd-capabilities.ttl`.
- **Language/stack:** Rust 70.4%, 13-crate workspace (ggen-core, ggen-cli, ggen-graph, ggen-lsp, ggen-membrane, ggen-projection)
- **Relevant milestones:** [1.4, 2.1, 2.4, 4.3]
- **Reusable assets:** `SparqlFn` struct for in-process SPARQL SELECT during Tera rendering; 60+ `.tmpl` files in `templates/clap-noun-verb-360/`; `templates/mcp-rust.tera`, `templates/a2a-rust.tera`, `templates/rust-struct-from-ontology.tera`; `PipelineBuilder` API (`with_rdf_file()` / `with_prefixes()` / `build()`)
- **Integration effort:** Low
- **Recommendation:** Adopt as dependency (already in manufacturing chain)
- **Key finding:** ggen is the manufacturing layer without which the noun-verb CLI grammar cannot be regenerated. The existing `SparqlFn` in-process SPARQL execution is directly relevant to Milestone 2.4 (ontology-driven SPARQL test assignment) — the same mechanism that renders noun templates can render test-module assignment maps. For Milestone 4.3 (cross-language grammar), ggen already has `templates/mcp-rust.tera` and `templates/a2a-rust.tera`, demonstrating that multi-output-format template expansion is already architected; adding `templates/go/`, `templates/python/`, and `templates/typescript/` directories follows the established pattern. For Milestone 2.1, ggen's `PipelineBuilder` API is a direct inspiration for the `pipeline.toml` DSL pipeline composer.

---

### clap-noun-verb

- **What it is:** A published Rust crate (v26.6.14) providing the noun-verb command grammar pattern used by cargo-cicd's CLI, enforcing Send+Sync on every verb via SPARQL-generated validation rules.
- **Language/stack:** Rust; ontology at `ontology/clap-noun-verb-ontology.ttl`; SPARQL at `queries/validate-cli-structure.rq`
- **Relevant milestones:** [1.4, 4.3]
- **Reusable assets:** `ontology/clap-noun-verb-ontology.ttl`; `ontology/cargo-cicd.ttl`; `queries/cargo-cicd-commands.rq`; `validate-cli-structure.rq` (Send+Sync enforcement)
- **Integration effort:** Low
- **Recommendation:** Adopt as dependency (already adopted)
- **Key finding:** clap-noun-verb is the published artifact of the noun-verb grammar discipline and the most direct dependency for Milestone 4.3. Its ontology (`clap-noun-verb-ontology.ttl`) is the vocabulary that would be extended with a WIT IDL export for cross-language grammar generation. The Send+Sync enforcement via SPARQL query is a governance pattern that can be replicated for other cross-cutting constraints as the verb surface grows. The absence of `cnv:targetLanguage` support in the current ontology is a confirmed gap that Milestone 4.3 must address.

---

### rdddy

- **What it is:** A Python reactive Domain-Driven Design framework using AsyncIO, RxPY, and DSPy that provides actor systems, saga patterns, and abstract policy evaluation — the behavioral model closest to cargo-cicd's autonomic escalation architecture.
- **Language/stack:** Python; AsyncIO / RxPY / DSPy
- **Relevant milestones:** [4.4, 2.2]
- **Reusable assets:** `AbstractActor` with mailbox; `ActorSystem` (`actor_of`/`publish`/`send`); `AbstractMessage`/`Command`/`Event`/`Query` hierarchy; `AbstractSaga` (skeleton); `abstract_policy.py` (maps 1:1 to `PolicyEntry`/`PolicyVerdict`)
- **Integration effort:** Low (1:1 mapping to ProcessEvent)
- **Recommendation:** Fork and adapt
- **Key finding:** rdddy is the highest-value non-adopted repository for Vision 2030. Its `AbstractActor`/`ActorSystem` pattern maps directly onto the multi-oracle consensus design in Milestone 2.2 — each oracle instance becomes an actor, and the `ConsensusOracle` becomes a saga. More importantly, `abstract_policy.py` is structurally isomorphic to `PolicyEntry`/`PolicyVerdict` in cargo-cicd's autonomic layer, meaning the behavioral design for Milestone 4.4 (autonomous remediation with reversibility guarantees) is already worked out in rdddy's Python. The port to Rust requires replacing AsyncIO with `tokio`, RxPY with `tokio::sync::broadcast`, and DSPy with cargo-cicd's existing `ProcessEvent` emission. The `AbstractSaga` skeleton is the direct ancestor of the `RollbackEvent` concept in Milestone 4.4.

---

### dspygen

- **What it is:** A Python DSPy-native code generation and workflow automation library integrating pm4py for XES-native process mining, AutoML via tpot, and structured Pydantic-validated instance generation via DSPy ChainOfThought.
- **Language/stack:** Python; dspy-ai ^2.6.27, pydantic, pm4py ^2.7.11.11, tpot ^0.12.2
- **Relevant milestones:** [2.1, 3.3, 3.4]
- **Reusable assets:** `Workflow`/`Job`/`Action` Pydantic models with DAG execution; `LogCollector`; Jinja2 context rendering; `GenPydanticInstance` (DSPy ChainOfThought → validated Pydantic instance); `ExtractMetricsFromLogsModule`; `PredictiveMaintenanceModule`; pm4py native XES integration; tpot AutoML pipeline
- **Integration effort:** Medium
- **Recommendation:** Fork and adapt
- **Key finding:** dspygen's `Workflow`/`Job`/`Action` DAG model is the most mature reference design for Milestone 2.1's `pipeline.toml` DSL — the cargo-cicd `[[pipeline.step]]` syntax maps directly onto dspygen's job-action nesting. The `PredictiveMaintenanceModule` and tpot AutoML integration are the research prototypes for Milestone 3.3's ML build time prediction; however, since cargo-cicd targets Rust, the algorithmic ideas (feature engineering from `ChangedFileState`, history log accumulation) must be ported to the `src/adapters/prediction.rs` module without taking the Python/tpot dependency. dspygen's pm4py integration confirms that a Python XES bridge exists and provides design reference for Milestone 3.1's ISO/IEC 33001 attribute mapping — pm4py already implements conformance checking algorithms that pre-date the cargo-cicd XES schema.

---

### dslmodel

- **What it is:** A Python DSL modeling library that integrates OpenTelemetry Weaver for semantic conventions and provides Workflow/Job/Action models alongside a governance module including a Roberts Rules voting system.
- **Language/stack:** Python; OpenTelemetry Weaver (`weaver/`, `semantic_conventions/`); pydantic
- **Relevant milestones:** [2.1, 2.2, 3.4]
- **Reusable assets:** `weaver/` + `semantic_conventions/` (OTel Weaver); `Workflow`/`Job`/`Action` models; `governance/roberts_voting_system.py`
- **Integration effort:** Medium
- **Recommendation:** Reference design
- **Key finding:** dslmodel's semantic conventions approach — using OpenTelemetry Weaver to define structured telemetry vocabularies — is the reference design for how Milestone 3.4's declarative policy DSL should be specified. The `roberts_voting_system.py` governance module is a direct behavioral model for the `ConsensusPolicy` logic needed in Milestone 2.2's multi-oracle consensus: Roberts Rules majority/unanimous vote maps cleanly onto `ConsensusPolicy::Majority` / `ConsensusPolicy::Unanimous`. Since dslmodel is Python-only, the integration path is design borrowing, not code reuse.

---

### unrdf

- **What it is:** A JavaScript monorepo (72 packages) providing an Oxigraph WASM-backed RDF graph database with filesystem ontologies, codegen from SPARQL to TypeScript types, and packages for receipts and process evidence.
- **Language/stack:** JavaScript / TypeScript; Oxigraph WASM backend
- **Relevant milestones:** [2.4, 4.3, 4.5]
- **Reusable assets:** `unfs-ontology.ttl` (ProjectRoot, SourceFolder, BuildFolder, TestFolder with `byteSize`/`lastModified`/`extension`); `unproj-ontology.ttl` (13 classes including `unproj:Test` with `unproj:pathPattern` glob patterns); `packages/wasm4pm`; `packages/receipts`; `packages/codegen` (SPARQL SELECT → TypeScript types)
- **Integration effort:** Medium
- **Recommendation:** Fork and adapt
- **Key finding:** unrdf provides the most concrete external precedent for Milestone 2.4's SPARQL-driven test assignment. The `unproj:Test` class with `unproj:pathPattern` (glob-based test selection) is exactly the ontology extension that Milestone 2.4's CONSTRUCT rule needs to generate from `cc:requiresTest` triples — the cargo-cicd ontology can adopt this vocabulary pattern directly. The `packages/codegen` module (SPARQL SELECT → TypeScript types) demonstrates that cross-language type generation from SPARQL is tractable and supports Milestone 4.3's cross-language grammar ambition. The `packages/wasm4pm` package suggests wasm4pm has a JavaScript host binding, which is a prerequisite for Milestone 4.5's multi-ecosystem XES+WASM standard.

---

### qlever_poc

- **What it is:** A pure Rust async HTTP client (`qleverest` crate) for the QLever SPARQL engine, with LRU cache and TTL built-in, providing `Store::new(endpoint)` + `store.query(sparql)` semantics.
- **Language/stack:** Rust; async/tokio; HTTP client
- **Relevant milestones:** [2.4, 4.5]
- **Reusable assets:** `qleverest` crate; `Store::new("http://localhost:7777")` + `store.query(sparql)` API; built-in LRU cache with TTL; 8 example programs
- **Integration effort:** Medium
- **Recommendation:** Fork and adapt
- **Key finding:** qlever_poc is the Rust-native SPARQL query client that Milestone 2.4 needs for ontology-driven test assignment. ggen's `SparqlFn` executes SPARQL in-process during template rendering, but a live SPARQL endpoint client enables runtime ontology queries — querying `cc:requiresTest` triples at `cargo cicd test changed` invocation time rather than during code generation. The `qleverest` LRU cache with TTL directly addresses the performance concern: the same SPARQL result (which test modules cover a given noun) can be cached and invalidated when the ontology changes. For Milestone 4.5, having a Rust SPARQL client that speaks to a standard SPARQL endpoint positions cargo-cicd to participate in a federated linked-data ecosystem alongside the XES+WASM standard.

---

### capability-map (cpmp)

- **What it is:** A Rust+Python tool that scans project directories and emits RDF/Turtle capability graphs with BLAKE3 receipts, bridging the gap between raw filesystem structure and formal ontology-grounded capability catalogs.
- **Language/stack:** Rust + Python
- **Relevant milestones:** [2.4, 1.4]
- **Reusable assets:** `cpmp computer discover` command producing `catalog/cpmp-catalog.ttl` + `receipts/scan-*.receipt.toml`; `GGEN_PROJECTION_MEMBRANE.md` (cpmp → ggen integration seam documentation)
- **Integration effort:** Medium
- **Recommendation:** Reference design
- **Key finding:** capability-map demonstrates the `discover → catalog → project` pattern that Milestone 2.4 needs for automated test-module assignment: scan the workspace, emit RDF triples about what capabilities exist, then project those triples through SPARQL inference to derive test obligations. The BLAKE3 receipt output of `cpmp computer discover` is directly compatible with cargo-cicd's `fingerprint` module and the Milestone 3.2 federated evidence ledger. The `GGEN_PROJECTION_MEMBRANE.md` document is the most important reference artifact: it defines the seam between capability scanning and code generation that the Vision 2030 milestones need to formalize.

---

### bcinr

- **What it is:** A Rust library of 308 branchless algorithms implemented in `no_std`-capable code, with a `bcinr_contract_gate` proc-macro for gate-level assertions — optimized for deterministic, allocation-free execution paths.
- **Language/stack:** Rust; `no_std` capable; proc-macro
- **Relevant milestones:** [3.2, 4.2] (partial — branchless only, no crypto primitives)
- **Reusable assets:** 308 branchless algorithm implementations; `bcinr_contract_gate` proc-macro; `no_std` compatibility
- **Integration effort:** Medium
- **Recommendation:** Reference design (branchless only)
- **Key finding:** The agent survey confirmed via exhaustive file search that bcinr contains zero BLAKE3, Ed25519, or Merkle tree implementations. Its value for Milestones 3.2 and 4.2 is therefore limited to the `no_std` branchless computation patterns that could be used in a constrained evidence emission hot path (e.g., for hash-chaining in resource-constrained WASM environments). The `bcinr_contract_gate` proc-macro is a reference design for the type-level contract assertions that Milestone 1.1 needs to enforce the WIT interface boundary without runtime overhead. For cryptographic primitives, `ed25519-dalek` and `blake3` crates remain the correct dependencies.

---

### gitvan

- **What it is:** A JavaScript implementation of "Git as Runtime" that stores evidence in `git-notes` refs via `ReceiptWriter.mjs` and performs SPARQL-based event correlation via `EventCorrelator.mjs`, mirroring cargo-cicd's `ChangedFileDetector` pattern.
- **Language/stack:** JavaScript
- **Relevant milestones:** [1.3]
- **Reusable assets:** `ReceiptWriter.mjs` (evidence in git-notes); `EventCorrelator.mjs` (SPARQL correlation); `GitEventCapture` (mirrors `ChangedFileDetector`)
- **Integration effort:** High (JS → Rust port required)
- **Recommendation:** Reference design
- **Key finding:** gitvan's `ReceiptWriter.mjs` demonstrates an alternative evidence persistence strategy — storing evidence in git-notes refs rather than flat files in `target/cargo-cicd/evidence/` — that would make evidence portable with the repository and survive `cargo clean`. This is a design alternative worth considering for Milestone 1.3's distributed workspace support, where cross-machine evidence access would benefit from evidence living in the git object store. The high integration effort arises entirely from the JS-to-Rust port requirement; the algorithmic design is directly applicable.

---

### qlever

- **What it is:** A high-performance C++ RDF/SPARQL database fork with a `rust/` directory containing `qlever-sys` (FFI bindings) and a safe Rust wrapper, enabling in-process embedded SPARQL query execution via `Qlever::query()`.
- **Language/stack:** C++ (primary), Rust (FFI wrapper); CMake, Ninja, Conan build chain
- **Relevant milestones:** [2.4, 4.5]
- **Reusable assets:** `Qlever::query()` returning W3C SPARQL JSON from embedded engine; `qlever-sys` crate FFI bindings
- **Integration effort:** High (C++ build chain, CMake/Ninja/Conan)
- **Recommendation:** Reference design
- **Key finding:** qlever provides the highest-performance in-process SPARQL execution of any repository surveyed, but its C++ build chain (CMake + Ninja + Conan) makes it unsuitable as a direct cargo-cicd dependency — it would add multi-minute cold-build times and cross-compilation complexity. For Milestone 2.4's ontology SPARQL inference, `qlever_poc`'s async HTTP client against a separately deployed qlever instance is the lower-friction path. qlever is most relevant to Milestone 4.5's standardization effort: its W3C SPARQL JSON output format alignment demonstrates that a high-performance SPARQL engine can produce standards-compliant output, which is the query interface cargo-cicd's linked-data federation would need to speak.

---

### ash_swarm

- **What it is:** An Elixir Reactor-based swarm coordination system for autonomous agent orchestration, used as a reference design for multi-agent workflow patterns.
- **Language/stack:** Elixir; Ash Framework; Reactor pattern
- **Relevant milestones:** [4.4]
- **Reusable assets:** Reactor step-based workflow model; compensation (rollback) hooks
- **Integration effort:** High (Elixir → Rust, private/deleted)
- **Recommendation:** Reference design
- **Key finding:** ash_swarm's Reactor compensation pattern — where each step declares a `compensate/2` function called on failure — is the behavioral model for Milestone 4.4's reversibility guarantee. The `RollbackEvent` concept in Milestone 4.4's design maps onto a Reactor compensation step: the action records its undo-data before executing, so if the saga fails, the compensator restores prior state. Since ash_swarm is private/deleted and Elixir-based, the design must be ported conceptually; rdddy's `AbstractSaga` skeleton is the closer, more accessible reference for the same pattern.

---

### pm4py-mcp

- **What it is:** A Python process mining library (pm4py) exposed as an MCP server, providing XES parsing, conformance checking, and process model discovery algorithms.
- **Language/stack:** Python; pm4py; MCP protocol
- **Relevant milestones:** [2.3, 3.1]
- **Reusable assets:** XES parsing algorithms; conformance checking against Petri net models; process model discovery (Alpha algorithm, Heuristics Miner)
- **Integration effort:** N/A (language mismatch; reference design only)
- **Recommendation:** Reference design
- **Key finding:** pm4py-mcp is the most mature XES processing ecosystem available in any language, and its conformance checking algorithms are the direct research precedent for Milestone 3.1's ISO/IEC 33001 mapping. However, since cargo-cicd is Rust and pm4py is Python, the value is entirely in design reference: the XES schema extensions that pm4py's conformance checker expects (case attributes, event attributes, lifecycle transitions) are the same attributes that cargo-cicd's `ProcessEvent` must produce to be consumable by ISO-aligned tooling. For Milestone 2.3 (real-time evidence streaming), pm4py's streaming XES parser demonstrates that event-by-event XES consumption is feasible without materializing the full log.

---

### process-intelligence

- **What it is:** A Python XES parsing and generation toolkit.
- **Language/stack:** Python
- **Relevant milestones:** None
- **Reusable assets:** None (language mismatch; overlaps with pm4py)
- **Integration effort:** N/A
- **Recommendation:** No match
- **Key finding:** process-intelligence duplicates pm4py functionality without adding novel capabilities relevant to the Vision 2030 milestones. Language mismatch and functional overlap with pm4py make this a non-candidate.

---

### Remaining No-Match Repositories

The following repositories were surveyed and confirmed as non-candidates for Vision 2030 integration:

- **ggen-mcp:** An XLSX spreadsheet MCP server, not related to code generation. No milestones covered.
- **tower-lsp-composition:** Private/deleted repository. Falls back to the published `tower-lsp 0.20` crate, which is a direct dependency for Milestone 1.2 (LSP integration) via `crates/cargo-cicd-lsp/` but does not require a separate fork.
- **gitgym:** Nuxt UI template with no git tooling relevance.
- **unibit:** Private/deleted repository. No match.
- **unrdf-kgc:** Empty placeholder repository.
- **unrdf-experiments:** Empty placeholder repository.
- **knhk:** Private repository. No content accessible.
- **kgold:** Private/deleted repository.
- **metadspy:** Not found under the expected namespace.

---

## 4. Milestone-to-Repository Mapping

All 17 Vision 2030 milestones mapped to their best available repository coverage:

| Milestone | Horizon | Description (abbreviated) | Primary Repo(s) | Secondary Repo(s) | Coverage |
|-----------|---------|--------------------------|-----------------|-------------------|----------|
| **1.1** | H1 2026 | WASM Component Model oracle | wasm4pm | qlever_poc (WIT pattern) | Partial — host side greenfield |
| **1.2** | H1 2026 | LSP + Level 5 engine integration | (internal: cargo-cicd-lsp) | tower-lsp 0.20 | Partial — internal work only |
| **1.3** | H1 2026 | Distributed workspace / monorepo support | gitvan (reference) | (internal adapters) | Weak — reference only |
| **1.4** | H1 2026 | Advanced feature set wiring | ggen, clap-noun-verb, capability-map | (internal: src/advanced/) | Strong — manufacturing chain ready |
| **2.1** | H2 2027 | Declarative pipeline DSL | dspygen, dslmodel | ggen (PipelineBuilder) | Partial — Python references, Rust greenfield |
| **2.2** | H2 2027 | Multi-oracle adjudication consensus | rdddy (ActorSystem) | dslmodel (Roberts Rules) | Partial — port required |
| **2.3** | H2 2027 | Real-time evidence streaming | wasm4pm, pm4py-mcp | (internal: axum) | Partial — XES streaming pattern available |
| **2.4** | H2 2027 | Formal ontology SPARQL test assignment | ggen, unrdf, qlever_poc, capability-map | qlever | Strong — multiple coverage paths |
| **2.5** | H2 2027 | cargo-dist / cargo-release / nextest integration | (external ecosystem tools) | — | Weak — no surveyed repo covers this |
| **3.1** | H3 2028 | ISO/IEC 33001 conformance checking | wasm4pm (oracle rules), pm4py-mcp | — | Partial — oracle side requires rule extension |
| **3.2** | H3 2028 | Federated evidence ledger (BLAKE3 chain) | wasm4pm (Blake3Hash), bcinr | gitvan (git-notes model) | Partial — BLAKE3 primitive available |
| **3.3** | H3 2028 | ML build time prediction | dspygen (PredictiveMaintenanceModule) | — | Weak — Python reference only, Rust greenfield |
| **3.4** | H3 2028 | Declarative autonomic policy language | dspygen, dslmodel, rdddy | — | Partial — behavioral models available |
| **4.1** | H4 2029 | Lean 4 formal verification of ggen | — | — | **No coverage — greenfield** |
| **4.2** | H4 2029 | Zero-trust supply chain (Ed25519 + COSE) | wasm4pm (Blake3Hash), bcinr | — | Partial — BLAKE3 only; Ed25519/COSE greenfield |
| **4.3** | H4 2029 | Cross-language noun-verb grammar | ggen (templates), clap-noun-verb, unrdf | qlever_poc | Partial — template arch ready, WIT IDL greenfield |
| **4.4** | H4 2030 | Autonomous remediation + reversibility | rdddy (AbstractSaga), ash_swarm | dspygen | Partial — behavioral model available, Rust port required |
| **4.5** | H4 2030 | XES+WASM as W3C/ISO standard | wasm4pm, unrdf, qlever | qlever_poc | Partial — ecosystem components exist, standardization process greenfield |

---

## 5. Gap Analysis — Milestones with No or Weak Repo Coverage

### Milestones with No Repository Coverage (Fully Greenfield)

**Milestone 4.1 — Lean 4 formal verification of the ggen manufacturing pipeline**

Zero repositories in the surveyed set involve Lean 4, Coq, or any other proof assistant. The task of formalizing SPARQL algebra in Lean 4 and proving the ggen transformation correct is a research task with no external codebase to leverage. The prerequisite stack — Lean 4 bindings for SPARQL algebra — does not exist as a published library in any language surveyed.

*Required greenfield work:* Define an inductive `Capability` type in Lean 4 mirroring `cargo-cicd-capabilities.ttl`; formalize SPARQL SELECT/CONSTRUCT as Lean 4 functions; define `ClapCommand` as a Lean 4 inductive type; prove the `ggen` translation function; wire the Lean proof into CI via `lean4checker` or a Lean Lake build step.

**Milestone 4.5 (standardization process) — XES+WASM as W3C/ISO standard**

While the technical components of Milestone 4.5 have partial repo coverage (wasm4pm as oracle, unrdf as JS host, qlever as high-performance SPARQL), the *standardization process itself* — submitting to IEEE 1849 working group, W3C WebAssembly Community Group, ISO/IEC JTC 1/SC 7 — has no repository analog. This is a community-building and standards-body engagement task that no amount of code can substitute for.

---

### Milestones with Weak Coverage (Primarily Greenfield with Reference Designs)

**Milestone 1.2 — LSP + Level 5 engine integration**

The survey found no external repository that provides a working integration between a Rust LSP server and a Rust process-data engine with shared state. `tower-lsp 0.20` is a dependency but not a solution. The work is entirely internal to cargo-cicd: wiring `EngineState::from_workspace()` into the LSP backend's file event handlers, mapping `PolicyEntry` to LSP `Diagnostic`, and implementing the `$/progress` notification.

**Milestone 1.3 — Distributed workspace / monorepo support**

gitvan provides a conceptual reference (git-notes as evidence carrier, which implies awareness of repository-root vs. workspace-root distinction), but no surveyed repository implements multi-workspace Cargo virtual manifest parsing or glob-pattern member resolution. This is an internal adapter engineering task with no external repo to fork.

**Milestone 2.5 — cargo-dist / cargo-release / cargo-nextest integration**

None of the 21 surveyed repositories involve these Cargo ecosystem tools. The integration is with external, stable Rust CLI tools whose source repositories were not part of the survey scope. This milestone requires implementing `NextestAdapter`, `CargoDistAdapter`, and `CargoReleaseAdapter` in `src/adapters/` based on each tool's published CLI interface, not on forkable code.

**Milestone 3.3 — ML build time prediction (Rust implementation)**

dspygen's `PredictiveMaintenanceModule` and tpot AutoML integration provide the research design (feature engineering from log data, AutoML model selection), but the Rust implementation is entirely greenfield. The `src/adapters/prediction.rs` module must implement feature extraction from `EngineSnapshot` history, a linear regression model (no ML crate dependency for initial scope), and confidence interval reporting — all in idiomatic Rust without Python bindings.

---

### Summary of Greenfield Work Required

| Category | Milestones | Estimated Scope |
|----------|-----------|-----------------|
| Lean 4 proof infrastructure | 4.1 | Research / PhD-level; 18+ months |
| Standards body engagement | 4.5 (process) | Community / political; multi-year |
| LSP engine wiring | 1.2 | Internal engineering; 1–2 months |
| Monorepo adapter | 1.3 | Internal engineering; 2–4 weeks |
| Ecosystem tool adapters | 2.5 | Internal engineering; 1–2 months |
| Rust ML prediction | 3.3 | Research + engineering; 3–6 months |
| Ed25519/COSE signing | 4.2 (partial) | Engineering; 1–2 months |
| WIT IDL for cross-language grammar | 4.3 (partial) | Engineering + standards; 3–6 months |
| Rust autonomic saga/rollback | 4.4 (partial) | Engineering; 2–4 months |
| WASM Component Model host | 1.1 (partial) | Engineering; 1–2 months |

---

## Appendix: Survey Provenance

This document synthesizes findings from five parallel agent surveys conducted against the cargo-cicd Vision 2030 roadmap defined in `docs/thesis/chapter6_conclusions_vision2030.md`.

| Agent | Domain | Repositories Surveyed |
|-------|--------|-----------------------|
| Agent 1 | Process Mining & Evidence Stack | wasm4pm, pm4py-mcp, process-intelligence |
| Agent 2 | Grammar Manufacturing & Code Generation | ggen, ggen-mcp, clap-noun-verb, capability-map |
| Agent 3 | RDF / SPARQL / Knowledge Graph | unrdf, unrdf-kgc, unrdf-experiments, qlever, qlever_poc |
| Agent 4 | AI / DSPy / ML / Autonomous Systems | dspygen, dslmodel, rdddy, ash_swarm, metadspy |
| Agent 5 | Rust Infrastructure, LSP, Git & Crypto | tower-lsp-composition, gitvan, gitgym, unibit, bcinr, clnrm, knhk, kgold |

**Total repositories surveyed:** 21
**Confirmed no-match / private / deleted:** 11
**Active candidates with milestone coverage:** 10

---

*End of Vision 2030 Repo Survey.*
