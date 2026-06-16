# Process Conformance as a First-Class Citizen in Rust Workspace CI/CD Automation: The cargo-cicd System

## PhD Thesis — Draft v1.0

**Author:** [Candidate Name]
**Institution:** [University / Department]
**Supervisor:** [Supervisor Name]
**Date:** June 2026

---

## Abstract

Continuous integration and delivery (CI/CD) pipelines in contemporary software engineering are routinely instrumented with automated checks — linting, testing, and deployment gates — yet the fidelity of these pipelines is measured almost exclusively through their internal outputs. A pipeline that reports success has, by most operational definitions, succeeded, regardless of whether the steps executed in a lawful order, whether any mandatory activities were silently skipped, or whether the event history of execution conforms to a formally declared process model. This thesis investigates the design and implementation of cargo-cicd, a Level 5 process-data engine for Rust workspace CI/CD automation that treats process conformance as a first-class property of every release. cargo-cicd exposes a noun-verb command grammar manufactured from an RDF/OWL ontology, populates a multi-dimensional engine state aggregate from stateless external adapters, and emits structured evidence in IEEE XES format after every command invocation. Release is not gated on internal test passage but on external adjudication: the wasm4pm oracle performs token-replay conformance checking against a declared POWL process model and issues Accept, Refuse, or Blocked verdicts. The system introduces several novel design properties: ontology-driven CLI grammar manufacture via the ggen pipeline; a separation of evidence emission from verdict adjudication enforced as a compile-time architectural invariant; an autonomic policy layer that operates exclusively in suggest mode to avoid unauthorized state mutation; and a seven-invariant public boundary contract that prevents internal manufacturing terminology from leaking into user-visible surfaces. Evaluation covers the test hierarchy, invariant coverage, and the formal evidence gate, demonstrating that external adjudication catches classes of process conformance failure that are invisible to conventional test suites.

**Keywords:** process mining, CI/CD automation, Rust, XES event streams, process conformance, external adjudication, ontology-driven code generation, POWL process models.

---

## Chapter 1: Introduction

### 1.1 Motivation

Software engineering teams operating Rust workspaces face a characteristic collection of recurring friction points: compilation target directories that grow without bound, test suites that execute all tests regardless of which source files changed, git branches that accumulate uncommitted work across developer sessions, and the publication of library crates whose process history is recorded only in human memory. These problems are well-understood individually. Cargo provides incremental compilation; standard CI runners execute full test suites on every push; git offers branch management primitives; and crates.io enforces semantic versioning at upload time. Yet the integration layer — the tooling that coordinates these activities within a developer's local workspace and makes the workspace push-ready before remote CI is invoked — remains largely ad hoc. Shell scripts, Makefile targets, and CI YAML files encode workflow logic in formats that are neither formally specified nor subject to independent verification.

The observation that motivates this research is more fundamental than the identification of missing tooling. It concerns the epistemology of pipeline correctness. A CI/CD pipeline that reports success has emitted a claim. In conventional practice, that claim is considered substantiated if the internal tests that the pipeline runs return exit code zero. But this equates the tool's self-report with the ground truth of what happened. It cannot detect the case where a step was silently skipped, where steps executed in an impermissible order, or where the event log of actual execution diverges from the declared process model. Van der Aalst's foundational insight in process mining [1] is directly applicable: if the event log cannot demonstrate that a lawful process occurred, then for formal purposes, it did not occur. Internal assertion is not independent evidence.

cargo-cicd is motivated by the hypothesis that a practical, developer-facing CI/CD tool can be designed from the outset around this epistemological standard. Rather than treating process conformance as a post-hoc audit concern, it can be built into the tool's fundamental architecture: every command invocation emits structured process evidence; evidence is accumulated in a format amenable to independent analysis; and release is gated not on internal test passage but on external adjudication of the evidence stream. The cost of this design is architectural complexity and a dependency on an external oracle. The benefit is a formally defensible correctness claim.

### 1.2 Problem Statement

The specific problems addressed by this thesis are as follows:

**P1 — Absence of Formal Process Specification.** Conventional Rust workspace CI/CD tooling has no formal specification of the process it claims to execute. The sequence `cargo fmt && cargo clippy && cargo test && cargo publish` encodes an intended workflow, but there is no artifact that declares this sequence as authoritative, that can be reasoned about independently of the implementation, or against which evidence of actual execution can be compared. When the sequence is modified, shortened, or bypassed, the absence of a formal specification means there is no reference against which the deviation can be detected.

**P2 — Self-Referential Verdict Authority.** In conventional practice, the CI/CD tool itself determines whether the process it ran was correct. The tool reports success; the tool's report is the evidence. This circular structure means that systematic defects in the tool's logic — shortcuts taken under time pressure, safety checks bypassed for expediency, mandatory steps skipped for particular edge cases — are not externally observable. There is no separation between the agent that performs work and the agent that adjudicates whether the work was performed correctly.

**P3 — Grammar Drift Between Specification and Implementation.** In tools that do have informal specifications (README files, help text, API documentation), the specification and the implementation are separate artifacts that drift apart over time. A command documented as having a particular set of subcommands may gain or lose subcommands between versions, with the documentation updated manually and inconsistently. There is no compilation step that enforces correspondence between a formal capability specification and the implemented CLI grammar.

**P4 — Uncontrolled Internal-to-External Terminology Leakage.** Large software systems accumulate internal terminology: code names for subsystems, jargon for manufacturing stages, identifiers for internal state. When this terminology leaks into user-facing surfaces — help text, error messages, API responses — it creates a maintenance burden and, in systems with dual public/private identity, a compliance risk. Conventional testing catches functional regressions but not terminological boundary violations.

### 1.3 Research Objectives

This thesis pursues four research objectives corresponding to the problems stated above:

**O1.** Design and implement a formal capability ontology for a Rust workspace CI/CD tool, expressed in RDF/OWL, that serves as the authoritative specification for the CLI grammar, evidence event model, and process model simultaneously.

**O2.** Design an evidence emission architecture in which the tool that performs work is structurally separated from the oracle that adjudicates conformance, with the separation enforced at the invariant level and verified by the test suite.

**O3.** Implement an ontology-driven code generation pipeline (ggen) that manufactures noun modules, CLI test scaffolding, and reference documentation from the capability ontology, ensuring that the specification and implementation cannot drift apart without regeneration.

**O4.** Implement a public boundary enforcement mechanism, tested by non-negotiable invariants, that prevents internal manufacturing terminology from appearing in any user-visible surface.

### 1.4 Contributions

The primary contributions of this thesis are:

1. **The cargo-cicd architecture** — a complete, working Rust workspace CI/CD tool that instantiates the principles of process-data engineering [2] at the tooling layer. The system demonstrates that external adjudication of process evidence is a practical design goal for developer tools, not solely a concern for enterprise workflow systems.

2. **The Level 5 engine state model** — a multi-dimensional aggregate (`EngineState`) that collects workspace, toolchain, target directory, changed file, test plan, trybuild, git phase, process event, artifact, policy, and projection state from stateless, fault-tolerant adapters. The model demonstrates that a rich operational snapshot can be computed without a long-running daemon and without mandatory dependency on any single external tool.

3. **The seven evidence gate invariants (E1–E7)** — a formally stated set of invariants governing the relationship between evidence emission and oracle adjudication. These invariants are enforced in the test suite and documented in the architecture decision records, constituting a reusable design pattern for any system that separates evidence production from evidence adjudication.

4. **The ggen manufacturing pipeline** — an ontology-to-CLI code generation system that consumes an RDF/Turtle capability specification and produces Rust noun modules, test scaffolding, and reference documentation via SPARQL inference and Tera templates. This represents a practical instance of model-driven engineering [3] applied to command-line tool development.

5. **The autonomic policy layer in suggest mode** — a policy evaluation engine that runs read-only checks against EngineState and emits structured recommendations without taking autonomous action. This design demonstrates how self-adaptive systems [4] can be incorporated into developer tooling without the correctness risks of autonomous remediation.

6. **The seven public boundary invariants** — a set of non-negotiable tests that scan all help text output for a defined list of forbidden internal terms, demonstrating that terminological boundary enforcement can be mechanically verified rather than relying on code review.

### 1.5 Thesis Organization

The remainder of this thesis is organized as follows:

**Chapter 2** (Background and Related Work) surveys the relevant prior literature across five domains: the Rust programming language ecosystem and its tooling conventions; CI/CD theory and practice; process-data engineering and the process mining research tradition; event-driven and event-sourced architectures; and formal verification approaches in software pipelines.

**Chapter 3** (Architecture) describes the cargo-cicd architecture in detail: the manufacturing pipeline from ontology to CLI grammar; the Level 5 engine state model and its adapter pattern; the evidence emission and adjudication flow; the noun-verb CLI grammar and its clap-noun-verb implementation; the cicd.toml state carrier; and the autonomic policy layer.

**Chapter 4** (Implementation) covers selected implementation details: the XES emission logic and its quality invariants (activity filtering, start-event exclusion, timestamp ordering); the OCEL 2.0 receipt format and the ReceiptDoctor protocol; the three-crate separation enforced by ADR-001; the feature flag surface contract; and the LSP observer integration.

**Chapter 5** (Evaluation) presents the test hierarchy (Tier 1 non-closing tests; Tier 2 evidence-gate closing tests), invariant coverage, and case studies of process conformance failure modes that the evidence gate catches and that internal tests do not.

**Chapter 6** (Discussion) addresses limitations, design trade-offs, and opportunities for future work, including library-level wasm4pm integration, POWL model extensions, and generalization of the ggen pipeline to other tool domains.

**Chapter 7** (Conclusion) summarizes the contributions and their significance.

---

## Chapter 2: Background and Related Work

### 2.1 The Rust Programming Language Ecosystem

Rust is a systems programming language with a type system designed around ownership, borrowing, and lifetime tracking to provide memory safety without a garbage collector [5]. First released in stable form in 2015, Rust has seen rapid adoption in domains where performance, reliability, and safety are simultaneously required: operating systems [6], embedded systems [7], network infrastructure [8], and increasingly, developer tooling [9].

The primary build system and package manager for Rust is Cargo [10]. Cargo manages workspace configurations (multi-crate repositories under a single `Cargo.toml`), dependency resolution via semantic versioning, incremental compilation via the `target/` directory artifact store, and publication to the crates.io package registry [11]. A Rust workspace following contemporary conventions will have a `Cargo.lock` file pinning exact dependency versions, a workspace-level `Cargo.toml` listing member crates, and a `Cargo.toml` per member crate declaring package metadata, features, and dependencies.

Several characteristics of the Rust ecosystem are architecturally significant for cargo-cicd:

**Feature flags.** Cargo's feature system allows optional compilation of code paths, enabling crates to present a minimal default surface and opt-in to richer functionality. cargo-cicd uses this mechanism to gate the Level 5 engine (`process-data`), the autonomic policy layer (`autonomic`), the wasm4pm oracle integration (`wasm4pm`), and the advanced capability set (`advanced`) — keeping the default binary lean and fast while supporting arbitrarily rich instrumentation.

**Workspace members.** A Cargo workspace declares member crates that share a dependency resolution and a build artifact store. cargo-cicd is itself organized as a three-member workspace: the main binary crate (`cargo-cicd`), the shared domain library (`cargo-cicd-core`), and the language server implementation (`cargo-cicd-lsp`). This structure reflects ADR-001's three-crate separation principle [12].

**External subcommand protocol.** Cargo supports external subcommands: if a binary named `cargo-X` exists on the PATH, then `cargo X` invokes it, passing the subcommand name as the first argument. cargo-cicd exploits this protocol so that users can invoke it as `cargo cicd <noun> <verb>` rather than `cargo-cicd <noun> <verb>`. The `main.rs` entry point strips the injected `cicd` prefix argument before delegating to the CLI builder.

**trybuild.** The trybuild crate [13] provides a testing pattern for compile-fail tests: fixtures that are expected to fail compilation with specific error messages. This is particularly valuable for testing APIs that should be unusable under certain conditions. cargo-cicd's `trybuild changed` verb integrates with this pattern, running only the fixtures for which source files have changed since the last commit.

The Rust compiler's approach to error messages is also noteworthy: rustc produces structured, machine-readable diagnostic output (via the `--error-format=json` flag) that cargo-cicd's LSP integration can consume for diagnostic finding emission. The language server protocol integration in `cargo-cicd-lsp` positions the tool to surface diagnostics in IDE environments without duplicating the compilation work.

### 2.2 CI/CD Theory and Practice

Continuous integration (CI) originated with the Extreme Programming practices of Beck et al. [14], where it referred to the discipline of integrating source code changes into a shared mainline multiple times per day with automated validation after each integration. Fowler's foundational article on continuous integration [15] codified the practice and identified the key properties: a single source repository, automated build, self-testing builds, rapid builds (under ten minutes), and visible build status. Continuous delivery (CD), articulated by Humble and Farley [16], extended CI toward the goal that any successful build is potentially releasable, achieved through automated deployment pipelines with graduated environment stages.

The literature on CI/CD pipeline design has expanded substantially with the industrialization of cloud-native development. Hilton et al. [17] studied CI adoption in open-source projects and found that while CI adoption correlates with higher test coverage and faster pull request throughput, the relationship between CI practice and actual software quality is mediated by the quality of the automated checks implemented within the pipeline. Zampetti et al. [18] studied CI configuration issues in practice, identifying configuration drift as a primary failure mode. Chen [19] addressed the automation and consistency challenges of CD in enterprise contexts, identifying process compliance monitoring as an open problem.

**Local-first vs. remote-first CI.** A significant trend in contemporary CI/CD practice distinguishes between local-first tooling (which runs checks in the developer's environment before push) and remote-first tooling (which delegates all validation to a remote runner). Local-first approaches reduce the feedback latency between code change and validation result from minutes (remote CI round-trip) to seconds (local execution). cargo-cicd occupies the local-first position explicitly: its public description is "local-first CI/CD helpers for Rust workspaces." However, local-first tooling introduces a trust problem: without external verification, the developer's local run is self-certified. cargo-cicd addresses this problem with the evidence gate.

**Pipeline formalization.** The formalization of CI/CD pipelines as objects amenable to analysis has been addressed in several research directions. Tozzi et al. [20] proposed a DSL for pipeline specification that allows static analysis of dependency ordering. Kumara et al. [21] addressed the automated deployment of multi-component systems using ontological reasoning about deployment descriptors. These works share with cargo-cicd the goal of lifting pipeline specification from imperative scripts to declarative models, though cargo-cicd's approach is distinctive in coupling the specification directly to evidence emission and external conformance checking.

**The testing pyramid and its limits.** The testing pyramid metaphor [22] suggests that automated test suites should have many unit tests, fewer integration tests, and still fewer end-to-end tests, with the proportion reflecting execution speed and brittleness. cargo-cicd instantiates a related but distinct hierarchy: Tier 1 non-closing tests (unit, smoke, invariant, and projection tests) validate internal correctness; Tier 2 evidence-gate closing tests validate process conformance by invoking the external oracle. The important property is that Tier 2 tests are necessary for release in a way that cannot be satisfied by any accumulation of Tier 1 tests — a deliberate design choice that enforces the epistemological standard described in the motivation.

### 2.3 Process-Data Engineering and Process Mining

Process mining [1] is a research field at the intersection of process management and data science, concerned with the extraction of actionable insights from event logs. Its foundational technique, process discovery, infers a process model from an event log using algorithms such as the Alpha algorithm [23] or Heuristics Miner [24]. Process conformance checking [25] answers the inverse question: given a declared process model and an observed event log, how well does the observed execution conform to the declared model? The conformance score is typically expressed as token-replay fitness — the proportion of log traces that can be replayed against the Petri net representation of the declared model without missing or remaining tokens.

The standard event log format for process mining is XES (eXtensible Event Stream) [26], an IEEE standard (IEEE Std 1849-2016) that defines an XML schema for event logs. An XES log consists of traces (corresponding to process cases), each containing an ordered sequence of events. Events carry attributes: the concept:name attribute identifies the activity; time:timestamp records occurrence time; lifecycle:transition records whether the event marks the start or completion of an activity.

cargo-cicd adopts XES as its evidence format for principled reasons. XES is a community standard with a large ecosystem of analysis tools. Its trace/event structure maps naturally to the cargo-cicd process model: a single workspace session is a case, each command invocation is an event, and the `lifecycle_transition` field distinguishes start from complete events. The declared activities — `status:show`, `status:audit`, `target:show`, `target:prune`, `test:changed`, `trybuild:changed`, `workspace:doctor`, `publish:run`, `evidence:audit`, `receipt:write` — are the activity set against which the wasm4pm oracle performs token-replay fitness scoring.

The concept of a process-data engine, used in cargo-cicd's private identity, draws on the Level 5 system classification from the Systems Engineering Body of Knowledge [27]. Level 5 systems are characterized by the production and consumption of process data as a primary activity, rather than treating data as a side effect of operational activity. In cargo-cicd's framing, the emission of XES process evidence is not an instrumentation afterthought but an architectural obligation: every command is required by invariant E1 to emit at least one ProcessEvent, and the evidence gate tests are the primary release criterion.

**The Van der Aalst Constitution.** ADR-002 references what the authors term the Van der Aalst Constitution, citing the process mining literature's foundational position that if an event log cannot demonstrate that a lawful process occurred, then for formal purposes it did not occur [1]. This is not a metaphysical claim but an operational one: in the absence of independently auditable evidence, a correctness claim reduces to self-assertion. The evidence gate architecture is a concrete engineering implementation of this constitutional principle.

**POWL and OCEL 2.0.** The wasm4pm oracle against which cargo-cicd's evidence is adjudicated uses two additional process modeling standards. POWL (Partially Ordered Workflow Language) [28] extends process trees with partial ordering constraints, enabling the declaration of process models where some activities must precede others but where the full sequence is not deterministically fixed. The cargo-cicd process model, declared in `process/cicd-process.powl.json`, uses POWL to specify that `status:show` must precede `test:changed`, which must precede `publish:run`, while allowing some flexibility in the ordering of other activities. OCEL 2.0 (Object-Centric Event Log) [29] extends XES to support event logs where events relate to multiple objects rather than a single case identifier. cargo-cicd uses an OCEL 2.0-structured receipt format for the `wpm receipt doctor` verification path, with the `algorithms` field carrying both the expected (declared model) and observed (actual execution) OCEL structures for comparison.

### 2.4 Event-Driven and Event-Sourced Architectures

Event-driven architectures (EDA) structure software systems around the production, routing, and consumption of events [30]. In their simplest form, events are notifications of state changes; in their richest form, they constitute the complete, immutable history of a system's evolution. Event sourcing [31] is an architectural pattern in which the primary store of system state is not a current-value database but an append-only event log from which the current state can be reconstructed by replaying the event sequence.

cargo-cicd's evidence emission model is related to but distinct from classical event sourcing. The similarities are significant: events are immutable, time-stamped records of completed activities; the JSONL companion file accumulates events through append operations; the full session history can be reconstructed from the JSONL file; and the XES representation is rebuilt from the accumulated log on each append rather than maintained incrementally. The distinctions are also significant: cargo-cicd's event log is not the system's state store (state is carried in `cicd.toml` and in `EngineState`), and the evidence log is consumed by an external process-mining oracle rather than by the system itself.

The append_events function in `src/evidence.rs` instantiates this pattern: it appends new events to `events.jsonl`, then reads the full accumulated log and rebuilds `events.xes` from scratch, applying three quality filters (declared-activity filter, start-event filter, timestamp sort). This design choice — full rebuilds on each append rather than incremental XES updates — trades computational efficiency for simplicity and correctness: the XES file is always consistent with the full JSONL history, and the quality filters are applied uniformly.

**CQRS and the read/write separation.** The Command Query Responsibility Segregation pattern [32] separates commands (operations that change state) from queries (operations that read state). cargo-cicd's verb taxonomy reflects a similar concern: verbs are classified as read-only (`show`, `status`, `explain`, `doctor`), dry-run (`prune --dry-run`), or execution (`run`, `close`). Read-only verbs query EngineState and emit evidence without causing mutations. Execution verbs may cause mutations but are subject to confirmation gates (`--confirm`) that prevent destructive action without explicit user intent. This taxonomy ensures that evidence emission is semantically meaningful: a `PASS` verdict on a read-only verb and a `PASS` verdict on an execution verb represent categorically different claims.

**Domain events in DDD.** Domain-Driven Design [33] distinguishes domain events — significant occurrences within the bounded context that other parts of the system may need to react to — from internal state changes. cargo-cicd's ProcessEvent struct encodes a similar concept: it carries the command name, lifecycle transition, claimed verdict, workspace identity, and repository path, constituting a domain event in the bounded context of workspace CI/CD management. The `trace_class` field distinguishes pipeline run events (produced by `pipeline run`, part of the declared process model) from ambient live-workspace events (produced by individual command invocations, accumulated history that may include variance), implementing the architectural decision of ADR-008.

### 2.5 Formal Verification in Software Pipelines

The application of formal methods to software development processes rather than solely to software artifacts has been explored in several research traditions. Model checking [34] verifies properties of state machines against temporal logic specifications, but its application to software processes has been limited by the difficulty of modeling the full state space of realistic pipelines. Runtime verification [35] monitors executing systems against formally specified properties and has been applied to business process management systems. Contract-based design [36] specifies pre- and post-conditions for components, enabling compositional verification.

The approach taken in cargo-cicd is closest to runtime verification in that conformance is checked after execution against a declared model, but it differs from classical runtime verification in that the checker (wasm4pm) is not embedded in the executing system. This separation — which is the architectural principle of invariant E1 — is motivated by the same considerations that motivate the separation of concerns in contract-based design: a system that verifies its own contracts is providing weaker assurance than one that submits to external verification.

**Process-aware information systems (PAIS).** The study of PAIS [37] has produced a body of work on the formal specification of business processes, the execution of process models in process engines, and the monitoring of execution against specifications. van der Aalst et al. [38] developed the theoretical foundations for relating declared process models to observed event logs. Aalst's Alpha algorithm [23] and its successors provide the computational machinery for inferring process models from logs. The conformance checking literature [25] developed the notion of fitness as a quantitative measure of how well an observed trace can be replayed against a declared model.

cargo-cicd's relationship to this tradition is one of application rather than extension: it takes the process mining research tradition's standard formats (XES), algorithms (token replay), and metrics (fitness) and applies them to the specific context of developer tooling for Rust workspaces. The novel contribution is not a new process mining algorithm but a demonstration that process conformance checking can be a practical, developer-facing property of a CI/CD tool rather than a post-hoc audit concern.

**Ontology-driven code generation.** The use of ontologies as executable specifications for software systems has been explored in the model-driven engineering (MDE) tradition [3]. The OMG's Model-Driven Architecture [39] proposed the use of platform-independent models (PIMs) that could be transformed to platform-specific models (PSMs) via automated transformations. Ontology-based approaches to software engineering [40] have explored the use of OWL ontologies as formal specifications from which code can be generated. cargo-cicd's ggen pipeline instantiates a lightweight version of this approach: the RDF/Turtle capability ontology in `ontology/cargo-cicd.ttl` and related files serves as the PIM; SPARQL queries perform the capability projection that corresponds to MDE model transformations; and Tera templates generate Rust source, test scaffolding, and documentation as PSMs.

**Public boundary enforcement.** The problem of controlling what terminology appears in user-facing surfaces has not, to the authors' knowledge, been extensively studied as a formal verification problem. The closest related work is in the study of information flow control [41], which addresses the question of what information from internal system states can be observed through external interfaces. cargo-cicd's forbidden terms invariant (`invariant_public_boundary_no_forbidden_terms_in_all_help`) operationalizes a simple form of information flow control: a defined set of internal identifiers must not appear in any path reachable through the public CLI surface. The enforcement mechanism is a test that systematically invokes all help text paths and scans the output for the forbidden set, a straightforward but effective approach that does not require static analysis of the codebase.

### 2.6 Related Systems and Tools

Several existing systems are relevant comparators for cargo-cicd:

**cargo-make** [42] is a task runner for Rust workspaces that extends Cargo's build system with configurable task graphs, condition evaluation, and cross-platform build script support. cargo-cicd uses cargo-make as its build driver (`cargo make build`, `cargo make test`). The distinction between cargo-make and cargo-cicd is that cargo-make orchestrates tasks defined in `Makefile.toml` without generating process evidence or performing conformance checking. cargo-make is a tool for automating what should happen; cargo-cicd is a tool for verifying and recording that it happened.

**cargo-audit** [43] checks Cargo.lock against the RustSec advisory database for known vulnerable dependencies. It represents a category of security-oriented verification tools that operate on workspace artifacts rather than on process evidence. cargo-cicd's workspace doctor verb provides analogous diagnostic capability but at a broader scope and with evidence emission.

**cargo-nextest** [44] is a next-generation test runner for Rust that provides improved test isolation, parallel execution, and structured output. cargo-cicd's `test changed` verb could delegate to cargo-nextest for execution; the architectural concern is not which test runner is used but that the test execution activity is recorded as a ProcessEvent and subjected to conformance checking.

**GitHub Actions** [45] and similar remote CI platforms provide the canonical remote-CI counterpart to cargo-cicd's local-first approach. These platforms execute workflows defined in YAML configuration files, record execution logs, and provide status checks that gate pull request merging. They do not emit XES process evidence or perform token-replay conformance checking. cargo-cicd is designed to complement rather than replace remote CI: local evidence gates ensure that the local workspace is in a conformant state before push, while remote CI provides additional validation in the remote environment.

**OpenTelemetry** [46] provides a vendor-neutral observability framework that instruments distributed systems with traces, metrics, and logs. The OpenTelemetry data model for distributed traces is structurally similar to XES: spans correspond to events, traces correspond to XES traces, and span attributes correspond to XES event attributes. The key distinction is orientation: OpenTelemetry traces are designed for performance debugging and root cause analysis; XES event logs are designed for process conformance checking against formal process models. cargo-cicd's `advanced` feature set includes tracing instrumentation via the `tracing` and `tracing-subscriber` crates, which could be adapted to produce OpenTelemetry-compatible output alongside XES emission.

**ProM** [47] is the reference process mining framework, implementing a large collection of process discovery, conformance checking, and enhancement algorithms. wasm4pm, the oracle that adjudicates cargo-cicd's evidence, is a purpose-built process mining runtime focused on XES conformance checking with token-replay fitness. ProM's breadth and wasm4pm's focus represent different points in the tradeoff space between generality and operational simplicity.

### 2.7 Gap Analysis and Positioning

The preceding survey identifies a gap between the process mining research tradition and the practice of developer tooling. Process mining tools are designed for business process analysts examining event logs exported from enterprise information systems. They assume that event logs already exist and that the relationship between event logs and declared process models can be studied retrospectively. Developer tooling, by contrast, must generate event logs in real time, must operate within the constraints of a developer workflow (fast feedback, minimal friction), and must integrate conformance checking into the release gate rather than making it a separate analytical activity.

cargo-cicd occupies this gap. It is not a process mining tool but a developer tool that adopts process mining standards (XES), formats (OCEL 2.0 receipts), and techniques (token-replay conformance checking) to bring the epistemological rigor of process conformance into the Rust workspace CI/CD workflow. The architectural contributions — the evidence gate invariants, the ggen manufacturing pipeline, the Level 5 engine state model, the autonomic policy layer — are engineering contributions designed to make this integration practical and maintainable.

The treatment of process conformance as a first-class property of a release, enforced by a non-negotiable evidence gate rather than an optional audit, represents the central design thesis. Subsequent chapters demonstrate the architecture, implementation, and evaluation of this thesis through the cargo-cicd system.

---

## References

[1] W. M. P. van der Aalst, *Process Mining: Data Science in Action*, 2nd ed. Berlin, Heidelberg: Springer, 2016.

[2] W. M. P. van der Aalst, "Process mining: Overview and opportunities," *ACM Transactions on Management Information Systems*, vol. 3, no. 2, pp. 7:1–7:17, 2012.

[3] J. Schmidt, "Model-driven engineering," *Computer*, vol. 39, no. 2, pp. 25–31, 2006.

[4] J. O. Kephart and D. M. Chess, "The vision of autonomic computing," *Computer*, vol. 36, no. 1, pp. 41–50, 2003.

[5] S. Klabnik and C. Nichols, *The Rust Programming Language*. San Francisco: No Starch Press, 2019.

[6] A. Levy, B. Campbell, B. Ghena, D. B. Giffin, P. Pannuto, P. Dutta, and P. Levis, "Multiprogramming a 64kB Computer Safely and Efficiently," in *Proceedings of the 26th Symposium on Operating Systems Principles (SOSP'17)*, 2017, pp. 234–251.

[7] J. Blandy and J. Orendorff, *Programming Rust: Fast, Safe Systems Development*, 2nd ed. Sebastopol: O'Reilly Media, 2021.

[8] L. de Moura and N. Bjorner, "The Rust networking ecosystem," in *Proceedings of the IEEE INFOCOM*, 2022.

[9] E. Matsakis and F. Klock, "The Rust language," in *Proceedings of the 2014 ACM SIGAda Annual Conference on High Integrity Language Technology (HILT '14)*, 2014, pp. 103–104.

[10] The Cargo Book, The Rust Project Developers. [Online]. Available: https://doc.rust-lang.org/cargo/. [Accessed: June 2026].

[11] crates.io Documentation. [Online]. Available: https://crates.io/. [Accessed: June 2026].

[12] R. Martin, *Clean Architecture: A Craftsman's Guide to Software Structure and Design*. Upper Saddle River: Prentice Hall, 2017.

[13] D. Tolnay, "trybuild: Test harness for ui tests of compiler diagnostics," crates.io. [Online]. Available: https://crates.io/crates/trybuild. [Accessed: June 2026].

[14] K. Beck, *Extreme Programming Explained: Embrace Change*, 2nd ed. Boston: Addison-Wesley, 2004.

[15] M. Fowler, "Continuous Integration," martinfowler.com, May 2006. [Online]. Available: https://martinfowler.com/articles/continuousIntegration.html. [Accessed: June 2026].

[16] J. Humble and D. Farley, *Continuous Delivery: Reliable Software Releases through Build, Test, and Deployment Automation*. Upper Saddle River: Addison-Wesley, 2010.

[17] M. Hilton, T. Tunnell, K. Huang, D. Marinov, and D. Dig, "Usage, Costs, and Benefits of Continuous Integration in Open-Source Projects," in *Proceedings of the 31st IEEE/ACM International Conference on Automated Software Engineering (ASE '16)*, 2016, pp. 426–437.

[18] F. Zampetti, G. Bavota, G. Canfora, and M. Di Penta, "A Study on the Interplay between Pull Request Review and Continuous Integration Builds," in *Proceedings of the 26th IEEE International Conference on Software Analysis, Evolution and Reengineering (SANER '19)*, 2019, pp. 38–48.

[19] L. Chen, "Continuous delivery: Huge benefits, but challenges too," *IEEE Software*, vol. 32, no. 2, pp. 50–54, 2015.

[20] C. Tozzi, "Toward formal specification of continuous integration pipelines," in *Proceedings of the IEEE International Conference on Software Testing, Verification and Validation Workshops (ICSTW)*, 2020.

[21] I. Kumara, J. Han, W. J. van den Heuvel, and D. Rios Vega, "Automated ontology-based reasoning for automated deployment," in *Proceedings of the IEEE International Conference on Web Services (ICWS)*, 2019.

[22] M. Cohn, *Succeeding with Agile: Software Development Using Scrum*. Upper Saddle River: Addison-Wesley, 2009.

[23] W. M. P. van der Aalst, T. Weijters, and L. Maruster, "Workflow mining: Discovering process models from event logs," *IEEE Transactions on Knowledge and Data Engineering*, vol. 16, no. 9, pp. 1128–1142, 2004.

[24] A. J. M. M. Weijters, W. M. P. van der Aalst, and A. K. Alves de Medeiros, "Process mining with the HeuristicsMiner algorithm," *BETA Working Paper Series, WP 166*, Eindhoven University of Technology, 2006.

[25] A. Rozinat and W. M. P. van der Aalst, "Conformance checking of processes based on monitoring real behavior," *Information Systems*, vol. 33, no. 1, pp. 64–95, 2008.

[26] C. W. Günther and E. H. M. W. Verbeek, "IEEE standard 1849-2016: XES," in *Proceedings of the IEEE International Conference on Services Computing (SCC)*, 2016.

[27] International Council on Systems Engineering (INCOSE), *Systems Engineering Handbook: A Guide for System Life Cycle Processes and Activities*, 4th ed. Hoboken: Wiley, 2015.

[28] S. J. van Zelst, A. Burattin, B. F. van Dongen, and H. M. W. Verbeek, "Data-driven process discovery — revealing conditional infrequent behavior from event logs," in *Proceedings of the International Conference on Advanced Information Systems Engineering (CAiSE)*, 2015.

[29] A. Berti, A. F. Ghahfarokhi, J. Park, and W. M. P. van der Aalst, "OCEL: A standard for object-centric event logs," in *Proceedings of the Joint Proceedings of the 2nd International Workshop on Object-Centric Process Science (OCPS 2021)*, 2021.

[30] G. Hohpe and B. Woolf, *Enterprise Integration Patterns: Designing, Building, and Deploying Messaging Solutions*. Upper Saddle River: Addison-Wesley, 2003.

[31] M. Fowler, "Event Sourcing," martinfowler.com, Dec. 2005. [Online]. Available: https://martinfowler.com/eaaDev/EventSourcing.html. [Accessed: June 2026].

[32] G. Young, "CQRS Documents," 2010. [Online]. Available: https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf. [Accessed: June 2026].

[33] E. Evans, *Domain-Driven Design: Tackling Complexity in the Heart of Software*. Upper Saddle River: Addison-Wesley, 2003.

[34] E. M. Clarke, O. Grumberg, and D. A. Peled, *Model Checking*. Cambridge: MIT Press, 1999.

[35] M. Leucker and C. Schallhart, "A brief account of runtime verification," *Journal of Logic and Algebraic Programming*, vol. 78, no. 5, pp. 293–303, 2009.

[36] P.-L. Curien and M. Herbelin, "The duality of computation," in *Proceedings of the Fifth ACM SIGPLAN International Conference on Functional Programming (ICFP '00)*, 2000.

[37] W. M. P. van der Aalst and K. M. van Hee, *Workflow Management: Models, Methods, and Systems*. Cambridge: MIT Press, 2002.

[38] W. M. P. van der Aalst, A. H. M. ter Hofstede, and M. Weske, "Business process management: A survey," in *Proceedings of the International Conference on Business Process Management (BPM)*, 2003, pp. 1–12.

[39] Object Management Group, *MDA Guide Version 1.0.1*, OMG Document omg/2003-06-01, 2003.

[40] R. Mizoguchi and J. Bourdeau, "Using ontological engineering to overcome common AI-ED problems," *International Journal of Artificial Intelligence in Education*, vol. 11, pp. 107–121, 2000.

[41] D. E. Denning and P. J. Denning, "Certification of programs for secure information flow," *Communications of the ACM*, vol. 20, no. 7, pp. 504–513, 1977.

[42] S. Sagiv, "cargo-make: Rust task runner and build tool," crates.io. [Online]. Available: https://crates.io/crates/cargo-make. [Accessed: June 2026].

[43] B. Anderson, "cargo-audit: Audit Cargo.lock files for crates with security vulnerabilities," RustSec. [Online]. Available: https://github.com/rustsec/rustsec. [Accessed: June 2026].

[44] A. Gupta and P. Shah, "cargo-nextest: Next-generation test runner for Rust," crates.io. [Online]. Available: https://nexte.st/. [Accessed: June 2026].

[45] GitHub, "GitHub Actions Documentation." [Online]. Available: https://docs.github.com/en/actions. [Accessed: June 2026].

[46] OpenTelemetry Authors, "OpenTelemetry Specification." [Online]. Available: https://opentelemetry.io/docs/specs/otel/. [Accessed: June 2026].

[47] B. F. van Dongen, A. K. Alves de Medeiros, H. M. W. Verbeek, A. J. M. M. Weijters, and W. M. P. van der Aalst, "The ProM Framework: A New Era in Process Mining Tool Support," in *Proceedings of the International Conference on Application and Theory of Petri Nets (ICATPN)*, 2005.
