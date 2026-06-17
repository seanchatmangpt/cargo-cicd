# Chapter 2: Background and Literature Review

## 2.1 Introduction

The design of `cargo-cicd` sits at the intersection of several mature and emerging research threads: continuous integration and delivery (CI/CD) systems, local-first software architectures, process mining and event-stream analysis, autonomic computing, and language-specific toolchain orchestration. This chapter surveys each of these threads in turn, identifies the gaps that motivate the present work, and establishes the theoretical vocabulary used throughout the remainder of the thesis.

---

## 2.2 Continuous Integration and Delivery Systems

### 2.2.1 Historical Foundations

The practice of continuous integration was systematized by Fowler and Foemmel [1] in the early 2000s as a discipline requiring that developers integrate their work into a shared mainline frequently—at minimum daily—and that each integration be verified by an automated build. The original formulation was developer-centric and tool-agnostic: the obligation was to the practice, not to any particular server. However, the rapid industrialization of CI/CD over the subsequent decade led to a strong association between the discipline and server-side orchestration platforms. CruiseControl [2], Hudson, Jenkins [3], and eventually cloud-native offerings such as Travis CI, CircleCI, and GitHub Actions [4] all share the architectural assumption that a remote agent with privileged access to the repository and deployment environment is the primary locus of process control.

This server-centric model confers genuine advantages: isolation of build environments, parallelism across heterogeneous hardware, and centralized audit logs. However, it also introduces a structural latency that is rarely discussed in the literature. The feedback loop from a local code change to a CI verdict is bounded below by network round-trip time, queue wait time, and cold-start overhead for containerized runners. Empirical studies of CI queue latency in large organizations have reported median wait times exceeding ten minutes [5], which is well above the threshold at which developers context-switch away from the originating task [6]. The consequence is that developers operating against server-side CI pipelines routinely commit work that they have not locally verified, relying on the remote system to surface defects after the fact.

### 2.2.2 GitHub Actions and the Declarative Pipeline Model

GitHub Actions [4] popularized the YAML-based declarative pipeline as the dominant paradigm for CI/CD configuration. In this model, pipeline behavior is specified as a directed acyclic graph of jobs and steps, triggered by repository events (push, pull request, tag). Each job executes in an isolated runner environment; steps invoke either shell commands or pre-packaged action modules published to the GitHub Marketplace.

The declarative model offers composability and version-controlled pipeline definitions, but it encodes an impedance mismatch between the local developer environment and the remote execution context. Environment variables, secrets, installed toolchains, and filesystem state in a GitHub Actions runner differ systematically from a developer's workstation. This means that a pipeline definition that passes on the runner may behave differently locally, and vice versa. Earthly [7] and similar tools (discussed in Section 2.5) attempt to bridge this gap by containerizing the entire build context, but at the cost of requiring Docker and introducing container image management as a new operational concern.

### 2.2.3 Local-First CI/CD

The local-first software movement, articulated by Kleppmann et al. [8] in the context of collaborative applications, argues that data and computation should reside primarily on the user's device, with network synchronization as a secondary concern. Transposed to the CI/CD domain, a local-first philosophy holds that the primary CI/CD feedback loop should be executable on the developer's workstation without network access, with remote systems serving as secondary validators rather than primary arbiters.

This position is not merely theoretical. Tools such as `act` [9], which replays GitHub Actions workflows locally using Docker, and `cargo-make` [10], which provides a Makefile-like task runner for Rust workspaces, embody local-first principles in practice. However, both tools treat the local environment as a simulation of the remote environment rather than as a first-class computational substrate. `cargo-cicd` departs from this framing: it treats the local Rust workspace as the authoritative source of truth and positions the CI/CD process as a data-producing activity whose outputs—state files, event streams, process evidence—are the primary deliverables.

---

## 2.3 The Rust Ecosystem and Toolchain Orchestration

### 2.3.1 Cargo as a Build System and Package Manager

Rust's package manager and build system, Cargo [11], occupies an unusually central position in the Rust ecosystem relative to the role played by build tools in other languages. Unlike Maven in Java or pip in Python, Cargo is the canonical interface for dependency resolution, compilation, testing, documentation generation, and package publication. This centrality means that a Rust CI/CD tool that integrates deeply with Cargo can provide a coherent orchestration surface without requiring the developer to learn a separate tool grammar.

The Cargo workspace model—in which a single `Cargo.toml` at the workspace root coordinates multiple member crates—is particularly relevant to `cargo-cicd`. Workspace-level commands such as `cargo test --workspace` and `cargo clippy --workspace` operate across all member crates, and the `cargo metadata` subcommand provides a machine-readable JSON representation of the full workspace dependency graph. This metadata interface is the primary data source for the `CargoMetadataAdapter` in `cargo-cicd`, which translates external Cargo representations into the internal `WorkspaceState` and `ToolchainState` dimensions of the engine.

### 2.3.2 Toolchain Management with rustup

Rust's official toolchain manager, `rustup` [12], provides a stable interface for installing, switching, and querying Rust compiler versions. The `rust-toolchain.toml` file at a workspace root pins the workspace to a specific Rust release channel (stable, beta, nightly) and version. This pinning mechanism is the basis for Minimum Supported Rust Version (MSRV) enforcement, a common requirement in library crates that must support downstream users on older compilers.

MSRV mismatches—where the workspace's pinned toolchain differs from the developer's active toolchain, or where dependencies have silently raised their own MSRV—are a significant source of CI failures that are not surfaced by local builds. The `ToolchainDetector` adapter in `cargo-cicd` explicitly models this mismatch as a first-class state dimension (`ToolchainState`), enabling policy evaluation against it before a push.

### 2.3.3 Static Analysis: clippy and rustfmt

The Rust toolchain ships two canonical static analysis tools: `clippy` [13], a collection of lints for catching common mistakes and enforcing idiomatic Rust, and `rustfmt` [14], a code formatter. Both tools are invoked via Cargo subcommands (`cargo clippy`, `cargo fmt --check`) and produce machine-readable output suitable for programmatic consumption. Their integration into CI/CD pipelines is standard practice in the Rust community, but the invocation order, flag configuration, and treatment of warnings versus errors varies widely across projects. `cargo-cicd` aims to provide an opinionated, workspace-aware default configuration for both tools, reducing the per-project configuration burden.

---

## 2.4 Process Mining and Event Streams

### 2.4.1 Process Mining as a Discipline

Process mining, as surveyed by van der Aalst [15], is a discipline concerned with extracting knowledge about business and computational processes from event logs produced by information systems. The central insight of process mining is that event logs—timestamped records of what happened, when, and in what case context—contain latent information about the actual process, which may deviate substantially from the intended or documented process. Process mining techniques include process discovery (inferring a process model from an event log), conformance checking (verifying that observed behavior conforms to a reference model), and enhancement (improving a reference model using event log evidence).

The application of process mining to software engineering processes is an active research area. Rubin et al. [16] demonstrated that process mining can be applied to version control histories to discover software development workflows, and Leemans et al. [17] showed that CI/CD pipeline logs can be mined to detect anomalies and predict build failures. `cargo-cicd` applies these insights in the other direction: rather than mining logs produced by an existing system, it actively instruments its own execution to produce process evidence in a standard format.

### 2.4.2 The XES Standard

The eXtensible Event Stream (XES) format [18] is the IEEE standard (IEEE Std 1849-2016) for event log interchange in process mining. An XES document is an XML file organized as a collection of traces, where each trace corresponds to a case (e.g., a single CI/CD run, a single build) and contains an ordered sequence of events. Each event carries a set of attributes: a timestamp, an activity name, and arbitrary domain-specific key-value pairs.

The choice of XES as the evidence format for `cargo-cicd` is deliberate. XES is both human-readable and machine-processable; it is supported by the ProM process mining framework [19] and by the `wasm4pm` oracle used as the release gate in this work. The XES trace structure maps naturally onto the concept of a CI/CD session: a trace is a single invocation of `cargo-cicd`, events within the trace correspond to individual adapter queries and policy evaluations, and trace-level attributes carry workspace metadata (commit hash, branch name, workspace root). This mapping allows the `wasm4pm` oracle to apply conformance-checking techniques to `cargo-cicd` execution logs, verifying that the tool's behavior conforms to its declared process model.

### 2.4.3 wasm4pm: A WebAssembly-Native Process Mining Oracle

The `wasm4pm` system represents a novel approach to process evidence adjudication: a process mining oracle compiled to WebAssembly, enabling portable, reproducible conformance checking that can be embedded in test harnesses without external service dependencies. The `wpm` binary exposes two primary interfaces relevant to `cargo-cicd`: the `wpm audit` command, which checks an XES file for structural validity and conformance against a registered process model, and the `wpm receipt doctor` command, which validates a JSON receipt document against a schema and semantic constraints.

The introduction of `wasm4pm` as a release gate—rather than as an optional integration—reflects a principled stance on the epistemology of process correctness. Internal unit tests can verify that individual components behave as specified, but they cannot verify that the emergent process—the sequence of operations actually performed during a real CI/CD run—conforms to the declared model. Only an external oracle, operating on evidence produced by the running system, can make this determination. The evidence-gate pattern instantiated by `wasm4pm` is thus a form of operational conformance checking, distinct from both static analysis and behavioral testing.

---

## 2.5 Related Work

### 2.5.1 cargo-make

`cargo-make` [10], developed by Sagiv and contributors, is a task runner for Rust workspaces that extends the `cargo` subcommand namespace with a Makefile-inspired task definition syntax. Tasks are defined in a `Makefile.toml` file and can depend on other tasks, enabling composition of multi-step workflows. `cargo-make` supports conditional execution (platform-specific tasks, environment-variable guards) and provides a library of predefined tasks for common Rust operations.

`cargo-cicd` uses `cargo-make` as its preferred build frontend (the `cargo make build`, `cargo make check`, and `cargo make test` commands are wrappers around `cargo-make` task definitions), but the relationship is one of composition rather than competition. `cargo-make` excels at task orchestration but does not model workspace state, emit process evidence, or apply policy evaluation. It is a task runner; `cargo-cicd` is a process-data engine.

### 2.5.2 cargo-nextest

`cargo-nextest` [20], the next-generation test runner for Rust, provides significant improvements over the built-in `cargo test` in terms of test isolation, parallelism, and output formatting. It introduces the concept of test partitioning (running a subset of tests per invocation) and provides machine-readable JSON output for test results.

`cargo-cicd` integrates with `cargo-nextest` through its `TestPlanState` dimension: when `cargo-nextest` is detected in the workspace, it is preferred as the test executor. However, `cargo-cicd`'s primary contribution relative to `cargo-nextest` is not test execution but test selection: the `ChangedFileDetector` adapter identifies which crates have changed since the last CI run, and the `TestPlanState` restricts test execution to those crates. This changed-test selection logic reduces CI feedback latency for large workspaces without requiring any modification to the test runner itself.

### 2.5.3 Earthly

Earthly [7] is a build system that uses a Dockerfile-like syntax (`Earthfile`) to define reproducible builds. It combines the familiarity of Makefiles with the reproducibility guarantees of container images, targeting the gap between local development and CI/CD pipelines. Earthly targets are cache-aware and composable, and Earthly supports both local and remote execution.

The principal difference between Earthly and `cargo-cicd` is the level of abstraction. Earthly operates at the level of filesystem images and shell commands; it is language-agnostic and general-purpose. `cargo-cicd` operates at the level of Rust workspace semantics: it knows about `Cargo.toml` manifests, toolchain pinning, `clippy` lints, and `trybuild` UI test fixtures. This domain specificity allows `cargo-cicd` to provide richer, more actionable output at the cost of portability to non-Rust contexts.

### 2.5.4 trunk

`trunk` (for Rust) is a build tool targeting WebAssembly applications, providing asset bundling, `wasm-pack` integration, and development server functionality. While sharing a name with other tools, `trunk` occupies a narrow niche—WebAssembly-targeting Rust projects—and does not address the general CI/CD orchestration problem that `cargo-cicd` targets.

### 2.5.5 Autonomic Computing and the MAPE-K Loop

The term "autonomic computing" was introduced by IBM in 2001 [21] as a research agenda for self-managing systems, drawing an analogy to the human autonomic nervous system's capacity to regulate physiological processes without conscious intervention. The MAPE-K reference architecture—Monitor, Analyze, Plan, Execute, over a shared Knowledge base—provides a four-phase feedback loop for autonomic control. In the MAPE-K model, a managed system exposes sensors and effectors; a control loop reads sensor data (Monitor), interprets it against a model (Analyze), determines corrective actions (Plan), and applies them (Execute), with the Knowledge base providing shared state across all phases.

`cargo-cicd`'s policy evaluation loop is a direct implementation of the MAPE-K pattern. The adapters correspond to the Monitor phase (reading from git, filesystem, Cargo); the `EngineState` aggregate corresponds to the Knowledge base; the autonomic policies (e.g., `GitPhaseDirtyPolicy`, `TargetPressurePolicy`) correspond to the Analyze and Plan phases; and the `--apply` flag (reserved for future work) corresponds to the Execute phase. The deliberate restriction to `suggest` mode by default reflects the autonomic computing literature's recognition that automated execution in the absence of sufficient operational experience carries unacceptable risk [22].

### 2.5.6 Noun-Verb CLI Grammar

The design of command-line interfaces has received systematic treatment in the context of POSIX utility conventions [23] and more recently in the context of modern CLI design guidelines. The noun-verb grammar—in which the first positional argument names a resource (noun) and the second names an operation (verb)—has been popularized by tools such as `kubectl` (`kubectl get pods`, `kubectl apply -f`), the AWS CLI (`aws s3 ls`), and the Azure CLI. This grammar scales well to multi-domain tools, providing a discoverable namespace structure without requiring the user to memorize a flat list of subcommands.

`cargo-cicd` implements a noun-verb grammar through the `clap-noun-verb` library, a local crate developed in concert with `cargo-cicd`. Each noun is a module implementing the `NounCommand` trait; each verb within a noun implements the `VerbCommand` trait. The default-verb injection mechanism—which maps bare noun invocations (e.g., `cargo cicd status`) to a designated default verb (e.g., `status show`)—follows the convention established by `kubectl`, where `kubectl get` without a resource type produces a usage message rather than an error.

---

## 2.6 Gaps in Existing Work

The survey above reveals several gaps that collectively motivate the design of `cargo-cicd`.

**Gap 1: Local state is not modeled.** Existing CI/CD tools either operate entirely on the remote server (GitHub Actions, Jenkins) or treat the local environment as a simulation substrate (Earthly, act). None of the surveyed tools maintain a persistent, structured record of local workspace state across invocations. The `cicd.toml` carrier introduced by `cargo-cicd` fills this gap, providing a TOML-serialized state file that persists workspace configuration, last-known adapter readings, and emitted process events across tool invocations.

**Gap 2: Process evidence is not emitted.** Existing Rust CI/CD tools produce human-readable output (terminal text) and machine-readable test results (JUnit XML, nextest JSON), but they do not emit structured process evidence that can be submitted to an external oracle for conformance checking. The XES evidence emission in `cargo-cicd` fills this gap, enabling the `wasm4pm` evidence-gate pattern as a release closure mechanism.

**Gap 3: Policy evaluation is not integrated.** The surveyed tools execute tasks but do not reason about whether the workspace's current state satisfies the preconditions for a safe push. `clippy` checks code style; `cargo test` runs tests; `git status` reports dirty files. But no existing tool integrates these signals into a unified policy evaluation that produces a workspace-level verdict (pass, warn, refuse). `cargo-cicd`'s autonomic policy layer fills this gap.

**Gap 4: Adapter isolation is not enforced.** In tools such as `cargo-make`, task definitions intermingle external system queries (git, cargo, filesystem) with business logic and output formatting. This coupling makes the tools difficult to test in isolation and difficult to extend. `cargo-cicd` enforces a strict adapter contract: each adapter owns exactly one external source, performs no business logic, and is independently testable against fixture workspaces.

**Gap 5: Changed-test selection is not workspace-aware.** `cargo-nextest` supports test partitioning but does not natively integrate with git to determine which partitions are relevant given the set of changed files. `cargo-cicd`'s `ChangedFileDetector` and `TestPlanState` provide this integration, reducing test execution time for large workspaces without requiring changes to the test runner.

Taken together, these gaps define the problem space that `cargo-cicd` addresses: a local-first, process-evidence-emitting, policy-evaluating CI/CD engine for Rust workspaces, grounded in autonomic computing principles and validated by external process mining oracles.

---

## References

[1] M. Fowler and M. Foemmel, "Continuous Integration," ThoughtWorks technical report, 2006. Available: https://martinfowler.com/articles/continuousIntegration.html

[2] P. Duvall, S. Matyas, and A. Glover, *Continuous Integration: Improving Software Quality and Reducing Risk*. Addison-Wesley Professional, 2007.

[3] K. Kawaguchi, "Hudson: An Extensible Continuous Integration Server," in *Proc. ICSE Workshop on Emerging Trends in Free/Libre/Open Source Software Research and Development*, 2009, pp. 31–32.

[4] GitHub, Inc., "GitHub Actions Documentation," GitHub, 2019. Available: https://docs.github.com/en/actions

[5] H. Hilton, T. Tunnell, K. Chang, D. Dig, and D. Grossman, "Usage, Costs, and Benefits of Continuous Integration in Open-Source Projects," in *Proc. 31st IEEE/ACM International Conference on Automated Software Engineering (ASE)*, 2016, pp. 426–437.

[6] G. Parnin and A. Rugaber, "Resumption Strategies for Interrupted Programming Tasks," *Software Quality Journal*, vol. 19, no. 1, pp. 5–34, 2011.

[7] A. Ballantyne, "Earthly: A Build Automation Tool for the Container Era," Earthly Technologies, 2020. Available: https://earthly.dev

[8] M. Kleppmann, A. Wiggins, P. van Hardenberg, and M. McGranaghan, "Local-First Software: You Own Your Data, in Spite of the Cloud," in *Proc. ACM SIGPLAN International Symposium on New Ideas, New Paradigms, and Reflections on Programming and Software (Onward!)*, 2019, pp. 154–178.

[9] N. Kaluza, "act: Run Your GitHub Actions Locally," GitHub repository, 2019. Available: https://github.com/nektos/act

[10] E. Sagiv, "cargo-make: Rust Task Runner and Build Tool," crates.io, 2017. Available: https://crates.io/crates/cargo-make

[11] The Rust Project Developers, "The Cargo Book," The Rust Programming Language, 2015. Available: https://doc.rust-lang.org/cargo/

[12] The Rust Project Developers, "rustup: The Rust Toolchain Installer," The Rust Programming Language, 2016. Available: https://rustup.rs

[13] The Rust Project Developers, "clippy: A Collection of Lints to Catch Common Mistakes and Improve Your Rust Code," GitHub repository, 2014. Available: https://github.com/rust-lang/rust-clippy

[14] The Rust Project Developers, "rustfmt: A Tool for Formatting Rust Code According to Style Guidelines," GitHub repository, 2015. Available: https://github.com/rust-lang/rustfmt

[15] W. M. P. van der Aalst, *Process Mining: Data Science in Action*, 2nd ed. Springer, 2016.

[16] V. A. Rubin, C. W. Günther, W. M. P. van der Aalst, E. Kindler, B. F. van Dongen, and W. Schäfer, "Process Mining Framework for Software Processes," in *Proc. International Conference on Software Process (ICSP)*, Lecture Notes in Computer Science, vol. 5007, pp. 169–181, Springer, 2007.

[17] S. J. J. Leemans, D. Fahland, and W. M. P. van der Aalst, "Discovering Block-Structured Process Models from Event Logs: A Constructive Approach," in *Proc. 4th International Workshop on Business Process Intelligence (BPI)*, 2013, pp. 311–329.

[18] IEEE, "IEEE Standard for eXtensible Event Stream (XES) for Achieving Interoperability in Event Logs and Event Streams," IEEE Std 1849-2016, 2016.

[19] B. F. van Dongen, A. K. A. de Medeiros, H. M. W. Verbeek, A. J. M. M. Weijters, and W. M. P. van der Aalst, "The ProM Framework: A New Era in Process Mining Tool Support," in *Proc. 26th International Conference on Application and Theory of Petri Nets (ICATPN)*, Lecture Notes in Computer Science, vol. 3536, pp. 444–454, Springer, 2005.

[20] cargo-nextest Contributors, "cargo-nextest: A Next-Generation Test Runner for Rust," GitHub repository, 2021. Available: https://github.com/nextest-rs/nextest

[21] J. O. Kephart and D. M. Chess, "The Vision of Autonomic Computing," *IEEE Computer*, vol. 36, no. 1, pp. 41–50, Jan. 2003.

[22] M. C. Huebscher and J. A. McCann, "A Survey of Autonomic Computing—Degrees, Models, and Applications," *ACM Computing Surveys*, vol. 40, no. 3, article 7, Aug. 2008.

[23] The Open Group, "POSIX.1-2017: IEEE Std 1003.1-2017 — Base Definitions, Utility Conventions," The Open Group, 2017. Available: https://pubs.opengroup.org/onlinepubs/9699919799/
