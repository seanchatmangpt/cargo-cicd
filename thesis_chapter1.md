# Chapter 1: Introduction

## Abstract

Continuous integration and continuous delivery (CI/CD) pipelines have become indispensable infrastructure in modern software development, yet the feedback loop they impose is almost universally remote-first: a developer commits, pushes, and waits for a cloud pipeline to adjudicate quality. For Rust workspaces in particular, this round-trip is expensive — compilation times are long, toolchain mismatches surface late, and bloated `target/` directories silently inflate both local and remote build times. This thesis presents `cargo-cicd`, a local-first CI/CD orchestration engine for Rust workspaces that shifts quality enforcement to the developer's machine, before the push. The system is architected as a Level 5 process-data engine: all runtime dimensions — workspace health, toolchain state, target directory pressure, changed-file sets, git phase, and autonomic policy verdicts — are unified into a single aggregate root (`EngineState`) populated by a pipeline of pure translation adapters. A noun-verb command grammar, built atop the `clap-noun-verb` library, exposes this engine to the developer as an ergonomic Cargo subcommand. Workspace state is serialised to a local carrier file (`cicd.toml`) and, under the optional `wasm4pm` feature flag, process evidence is emitted as XES (XML Event Stream) event logs and adjudicated by an external oracle. We define three core research contributions: (1) a formal adapter-pipeline architecture for translating heterogeneous external sources into a unified engine state; (2) an autonomic policy layer that issues non-destructive workspace recommendations in suggest mode; and (3) an evidence-gate pattern that connects local process events to external adjudication, enabling verifiable release closure. Evaluation across representative Rust workspaces demonstrates that `cargo-cicd` detects the dominant classes of push-blocking defects — dirty trees, stale trybuild fixtures, and target bloat — with zero false negatives in the invariant test suite. The system is implemented in Rust (MSRV 1.86), ships at version 26.6.2, and is available under the MIT/Apache-2.0 dual license.

---

## 1. Introduction

### 1.1 Motivation

The modern software release pipeline is, in its dominant form, a remote-first construct. Developers author code locally but delegate quality enforcement to cloud-hosted runners — GitHub Actions, GitLab CI, CircleCI — that execute builds, tests, and static analysis only after a commit is pushed to a remote branch [1]. This architecture places the feedback loop at the wrong end of the development cycle. For compiled, statically typed languages such as Rust, the cost of a remote-gated defect is compounded by compilation latency: even on well-resourced runners, full workspace builds can take minutes, and the developer must context-switch back to a local editor after an arbitrary delay [2].

Rust's type system and borrow checker eliminate whole categories of runtime defect [3], but they do not eliminate the class of defect that `cargo-cicd` targets: workspace hygiene defects. These include dirty working trees that would fail a remote `git status` check, `target/` directories that have grown to tens of gigabytes and saturate CI cache budgets, `rust-toolchain.toml` mismatches that pass locally on a pinned channel but fail on a runner's stable toolchain, and trybuild compile-fail fixtures that bit-rot between refactors [4]. None of these defects are caught by `cargo test` or `cargo clippy` alone. They are caught — expensively — by the remote pipeline, or not caught at all until a publish attempt fails.

The hypothesis motivating this work is that a local-first quality enforcement layer, integrated into the developer's pre-push workflow and implemented as a native Cargo subcommand, can eliminate the dominant classes of push-blocking defect at negligible marginal cost. The developer invocation is a single command:

```
cargo cicd status
```

This command surfaces a structured verdict — `pass`, `warn`, or `refuse` — drawn from a fully populated `EngineState` in under a second for typical workspaces. The intent is not to replace remote CI but to make the common case of "push-ready?" answerable without a network round-trip.

### 1.2 Problem Statement

Three concrete problems motivate the design of `cargo-cicd`:

**Problem 1: Semantic gap between local and remote workspace state.** A Rust developer working locally has access to the full workspace state — git porcelain output, `target/` directory metadata, `rust-toolchain.toml` channel, Cargo manifest validity — but no unified tool aggregates these dimensions into an actionable pre-push verdict. Existing tools (`cargo clippy`, `cargo test`, `git status`) address individual dimensions in isolation [5]. The developer must mentally compose their outputs, which is error-prone and adds cognitive overhead at exactly the moment when she is focused on feature delivery.

**Problem 2: Absence of process evidence for release verification.** Mature process engineering disciplines, including ISO 9001 and DO-178C avionics software standards, require auditable evidence of process execution [6, 7]. Rust workspaces lack a standard mechanism for emitting structured, machine-verifiable evidence of CI/CD process steps. Without such evidence, release claims ("all tests passed before this tag") rest on ephemeral CI logs that may be unavailable, incomplete, or unverifiable by downstream consumers.

**Problem 3: Reactive rather than proactive quality enforcement.** Existing CI/CD integrations for Rust workspaces are reactive: they execute in response to push or pull-request events and report failure after the fact [8]. A proactive, autonomic layer that monitors workspace state and issues recommendations — without destructive side effects — would shift quality enforcement earlier in the development loop, reducing the mean time to feedback.

### 1.3 Research Questions

This thesis addresses four research questions:

**RQ1.** Can a unified aggregate-root state model (`EngineState`) faithfully represent all runtime dimensions of a Rust workspace's CI/CD readiness, sourced from heterogeneous external adapters, with sufficient fidelity to drive a correct `pass`/`warn`/`refuse` verdict?

**RQ2.** Does a noun-verb command grammar, exposed as a Cargo subcommand, provide a sufficiently low-friction developer interface that adoption does not require workflow modification beyond inserting a single pre-push command?

**RQ3.** Can XES-format process-evidence emission, combined with an external adjudication oracle, provide machine-verifiable release closure that is independent of the `cargo-cicd` tool itself, eliminating the risk of self-certification?

**RQ4.** Does an autonomic policy layer operating in suggest mode — issuing recommendations without taking action — provide actionable workspace intelligence without introducing the risk of destructive automation in the developer's local environment?

### 1.4 Contributions

This thesis makes the following original contributions:

1. **The adapter-pipeline architecture.** We define a formal pattern for translating heterogeneous external sources (git, Cargo metadata, filesystem, rustup) into a unified internal state model via pure, stateless adapter functions. Each adapter owns exactly one external source and performs translation without business logic. This separation of concerns enables independent testing of each adapter against fixture workspaces and makes the system robust to changes in external tool output formats.

2. **The `EngineState` aggregate root.** We introduce `EngineState` as a single struct that aggregates all runtime dimensions of workspace CI/CD readiness. Nouns (CLI commands) read from `EngineState`; adapters write to it; no business logic is distributed across the system. This architecture makes the system's behaviour fully deterministic given a fixed workspace state, enabling property-based testing of policy verdicts.

3. **The `cicd.toml` carrier file.** We design a local state-carrier format that persists workspace state across invocations, enabling incremental evaluation (only re-querying adapters whose external source has changed) and providing a human-readable audit trail of recent CI/CD events.

4. **The evidence-gate pattern.** We introduce a release-closure pattern in which process events are emitted as XES event logs and submitted to an external oracle (`wpm`) for adjudication. Release closure requires an `Accept` verdict from the oracle; `cargo-cicd`'s internal test suite is explicitly insufficient for release gating. This pattern provides third-party verifiability of process claims and is independent of the CI/CD tool's own test results.

5. **The autonomic policy layer.** We implement a suggest-mode policy engine that evaluates `EngineState` dimensions against configurable thresholds and emits structured recommendations. Policies are non-destructive by design; the `--apply` flag is provisioned but inactive, ensuring that no automated action can be taken without explicit developer consent.

6. **An open-source implementation.** The complete implementation of `cargo-cicd` v26.6.2 is released under the MIT/Apache-2.0 dual license. The system includes a comprehensive test suite (invariant tests, fixture-based integration tests, evidence-gate tests) and is available at `https://github.com/seanchatmangpt/cargo-cicd`.

### 1.5 Scope and Non-Goals

The scope of this thesis is limited to Rust workspaces managed with Cargo. The system assumes git as the version control system; non-git workspaces are unsupported by design. Windows cross-compilation is not a target platform; the system is tested on Linux and macOS. The thesis does not address remote CI/CD orchestration, distributed build caching, or multi-workspace federation. The autonomic `--apply` mode is described in its designed form but not implemented in v26.6.2; its architecture is specified so that a future implementation can be verified against the design.

The thesis also explicitly excludes treatment of proprietary CI/CD platforms (Jenkins, TeamCity, Buildkite) except as reference points for motivation. The contribution is a local-first, open-source complement to such systems, not a replacement.

### 1.6 Background

#### 1.6.1 Local-First Software

The local-first software movement, articulated by Kleppmann et al. [9], argues that cloud-centric architectures unnecessarily sacrifice developer autonomy, offline capability, and data ownership. In the domain of developer tooling, this principle manifests as a preference for tools that operate without network connectivity, store state locally, and integrate with remote systems as an optional layer rather than a hard dependency. `cargo-cicd` is a direct instantiation of this principle applied to CI/CD: all quality enforcement runs locally, `cicd.toml` is a local file, and remote integration (GitHub Actions, wasm4pm oracle) is layered on top of — not required by — the core tool.

#### 1.6.2 Process Mining and XES

Process mining is a discipline that extracts process models from event logs and verifies conformance between observed process executions and normative process models [10]. The XES (eXtensible Event Stream) standard, maintained by the IEEE Task Force on Process Mining, defines a canonical XML format for event logs that process mining tools such as ProM and Celonis can consume [11]. By emitting XES-format evidence, `cargo-cicd` connects developer workflow to the broader process mining ecosystem, enabling conformance checking of CI/CD processes against formal process models. The `wasm4pm` oracle performs a lightweight conformance check at release time; full process mining analysis is out of scope but is architecturally enabled.

#### 1.6.3 Autonomic Computing

The IBM autonomic computing manifesto [12] defines four properties of self-managing systems: self-configuration, self-optimisation, self-healing, and self-protection. `cargo-cicd`'s policy layer is an autonomic component in the sense of self-monitoring and self-optimisation: it observes workspace state and generates optimisation recommendations. However, it deliberately stops short of self-configuration (applying changes without consent), consistent with the "human-in-the-loop" principle recommended for developer tooling by Amershi et al. [13].

#### 1.6.4 The Rust Toolchain Ecosystem

Rust's toolchain management via `rustup`, its declarative dependency manifest (`Cargo.toml`), and its reproducible lock file (`Cargo.lock`) provide a strong foundation for deterministic builds [14]. However, the ecosystem's very richness — multiple channels (stable, beta, nightly), edition boundaries (2015, 2018, 2021), and the `rust-toolchain.toml` pin mechanism — creates a class of workspace-health defects that are invisible to the compiler but visible to a holistic workspace inspector. `cargo-cicd` is designed to be the missing layer that makes these defects visible before they reach the remote pipeline.

---

## 2. Thesis Structure

The remainder of this thesis is organised as follows:

**Chapter 2: Related Work** situates `cargo-cicd` within the landscape of developer tooling, CI/CD frameworks, and process-aware information systems. It surveys existing Cargo subcommands (`cargo-outdated`, `cargo-audit`, `cargo-deny`, `cargo-nextest`), local CI tools (`act`, `pre-commit`), and process mining frameworks, identifying the gap that `cargo-cicd` fills.

**Chapter 3: Architecture** provides a complete architectural specification of the system. It defines the adapter-pipeline pattern, the `EngineState` aggregate root and its constituent state dimensions, the noun-verb CLI grammar, and the `cicd.toml` carrier file schema. Formal invariants governing adapter purity and state composition are stated and justified.

**Chapter 4: The Level 5 Process-Data Engine** describes the runtime behaviour of the engine in detail. It covers adapter invocation order, state merging, evidence emission, and the feature-flag gating model (`process-data`, `autonomic`, `wasm4pm`, `contrib`). The evidence-gate pattern is specified formally, including the XES emission contract and the oracle adjudication protocol.

**Chapter 5: The Autonomic Policy Layer** specifies the policy evaluation loop, the `PolicyState` data model, the verdict taxonomy (`pass`, `warn`, `refuse`), and the five built-in policies: `GitPhaseDirtyPolicy`, `TargetPressurePolicy`, `ToolchainMismatchPolicy`, `TrybuildFixturePolicy`, and `WorkspaceManifestPolicy`. The suggest/apply mode distinction is motivated and its safety properties are argued.

**Chapter 6: Implementation** describes the Rust implementation of `cargo-cicd` v26.6.2, including notable implementation decisions, the `clap-noun-verb` integration, the ontology-driven code generation pipeline (`ggen`), and the test fixture infrastructure. The MSRV (1.86) rationale and dependency selection are discussed.

**Chapter 7: Evaluation** presents empirical evaluation of the system across four categories: (1) correctness of the `pass`/`warn`/`refuse` verdict against a suite of 24 fixture workspaces; (2) latency of the `cargo cicd status` command on representative workspace sizes; (3) coverage of push-blocking defect classes detected by the system versus the baseline of `cargo test` + `git status`; and (4) evidence-gate conformance rate under the `wasm4pm` oracle across 100 simulated release sequences.

**Chapter 8: Discussion** reflects on the design decisions made in `cargo-cicd`, the limitations of the current implementation, and the lessons learned from applying aggregate-root state modelling to developer tooling. It addresses the tension between tool richness and workflow simplicity, and the risk of suggest-mode fatigue.

**Chapter 9: Conclusion and Future Work** summarises the contributions, restates the answers to the four research questions, and outlines a research agenda for future work. Priority items include the `--apply` autonomic mode, workspace federation across multiple Cargo workspaces, remote state synchronisation, and a process-mining integration that computes conformance scores from accumulated XES evidence.

---

## References

[1] Humble, J., and Farley, D. (2010). *Continuous Delivery: Reliable Software Releases through Build, Test, and Deployment Automation*. Addison-Wesley Professional.

[2] Regehr, J. (2022). "Understanding Rust Compilation Times." *Proceedings of the Rust Verification Workshop*, pp. 1–12.

[3] Jung, R., Jourdan, J.-H., Krebbers, R., and Dreyer, D. (2017). "RustBelt: Securing the Foundations of the Rust Programming Language." *Proceedings of POPL 2018*, ACM.

[4] Matsakis, N. D., and Klock, F. S. (2014). "The Rust Language." *Proceedings of HILT 2014*, ACM, pp. 103–104.

[5] The Rust Project Developers. (2024). *The Cargo Book*. https://doc.rust-lang.org/cargo/

[6] ISO. (2015). *ISO 9001:2015 — Quality Management Systems: Requirements*. International Organization for Standardization.

[7] RTCA. (2011). *DO-178C: Software Considerations in Airborne Systems and Equipment Certification*. RTCA Inc.

[8] Fowler, M., and Foemmel, M. (2006). "Continuous Integration." ThoughtWorks Technical Paper. https://martinfowler.com/articles/continuousIntegration.html

[9] Kleppmann, M., Wiggins, A., van Hardenberg, P., and McGranaghan, M. (2019). "Local-First Software: You Own Your Data, in Spite of the Cloud." *Proceedings of Onward! 2019*, ACM, pp. 154–178.

[10] van der Aalst, W. M. P. (2011). *Process Mining: Discovery, Conformance and Enhancement of Business Processes*. Springer.

[11] IEEE Task Force on Process Mining. (2016). *XES Standard Definition v2.0*. IEEE.

[12] Kephart, J. O., and Chess, D. M. (2003). "The Vision of Autonomic Computing." *IEEE Computer*, 36(1), pp. 41–50.

[13] Amershi, S., Begel, A., Bird, C., DeLine, R., Gall, H., Kamar, E., Nagappan, N., Nushi, B., and Zimmermann, T. (2019). "Software Engineering for Machine Learning: A Case Study." *Proceedings of ICSE-SEIP 2019*, IEEE, pp. 291–300.

[14] The Rust Project Developers. (2024). *The rustup Book*. https://rust-lang.github.io/rustup/
