# Chapter 5: Conclusion and Future Work

## 5.1 Summary of Contributions

This dissertation set out to answer three research questions that motivated the design and implementation of cargo-cicd:

**RQ1:** Can a structured process-data engine be embedded within a conventional developer CLI tool without exposing internal machinery to end users?

**RQ2:** Does a noun-verb command grammar, derived from an ontological manufacturing pipeline, produce a more consistent and extensible DevOps CLI surface than ad-hoc subcommand hierarchies?

**RQ3:** Can local-first CI/CD orchestration — running entirely on the developer's machine, prior to any remote pipeline invocation — reduce integration failures and build cache pressure in Rust workspaces at a rate comparable to dedicated cloud CI systems?

The thesis answers all three questions affirmatively. cargo-cicd demonstrates that a Level 5 process-data engine — one whose internal state, evidence emission, and policy evaluation are architecturally distinct from its public interface — can be manufactured and deployed as an ordinary Cargo subcommand. End users interact with a simple noun-verb CLI (`cargo cicd status`, `cargo cicd test changed`) while the engine silently accumulates workspace state across eleven structured dimensions, emits XES-formatted process evidence, and optionally submits that evidence to an external adjudicator (the wasm4pm oracle) for independent verdict.

The key achievement is the clean layering: the public boundary presents no forbidden terms, no engine internals, and no process-mining vocabulary. The private boundary, gated behind the `process-data` feature flag, exposes the full Level 5 engine to contributors, integration tests, and the release-gate evidence pipeline. This separation was enforced not by convention but by the Rust feature flag system and validated at every commit by the `invariants` integration test suite.

## 5.2 Theoretical Contributions

### 5.2.1 The Level 5 Engine Taxonomy

Prior work in DevOps tooling has characterised CI/CD systems at four levels of maturity: scripted pipelines (Level 1), parameterised pipelines (Level 2), reusable pipeline libraries (Level 3), and policy-driven pipelines (Level 4) [1, 2]. This dissertation proposes Level 5 as a distinct category: a *process-data engine* that treats every developer action as a structured event in a process trace, models workspace state as an aggregate root, and enables post-hoc adjudication of process conformance by an external oracle.

The distinction between Level 4 and Level 5 is not merely one of sophistication. Level 4 systems enforce policy at invocation time — a rule fires, a build is blocked. Level 5 systems accumulate evidence of *process intent* alongside outcomes, enabling retrospective conformance analysis, cross-run comparison, and formal acceptance verdicts. This is the core theoretical novelty of cargo-cicd: the tool does not merely enforce rules; it manufactures evidence that rules were followed.

### 5.2.2 Adapter Pattern Formalisation for DevOps State

The adapter pipeline introduced in Chapter 3 formalises a pattern that is common in practice but rarely stated precisely. Each adapter in cargo-cicd satisfies three invariants: (i) it reads exactly one external source, (ii) it performs no business logic — only translation — and (iii) it is stateless across invocations. This triple invariant ensures that `EngineState` is the single source of truth and that adapter correctness can be tested in isolation using fixture workspaces.

This formalisation extends the classical Adapter pattern [3] by adding the *single-source* constraint, which is non-trivial in DevOps contexts where external sources (git, cargo, rustup, the filesystem) frequently overlap in the information they expose. For example, both `git status --porcelain` and `cargo metadata` can reveal the presence of uncommitted changes to `Cargo.lock`; the single-source invariant requires one adapter to own that fact, eliminating the risk of contradictory state.

### 5.2.3 Noun-Verb Grammar for DevOps CLIs

Chapter 2 introduced the `clap-noun-verb` grammar, derived from an OWL ontology (`ontology/cargo-cicd.ttl`) via SPARQL queries and Tera templates. The grammar encodes DevOps commands as *noun* (the resource: `target`, `test`, `git`, `workspace`) and *verb* (the action: `show`, `prune`, `changed`, `close`). Default-verb injection allows bare nouns to work without subcommand disambiguation, preserving ergonomics while enforcing structural consistency.

The contribution is twofold. First, the grammar is *manufactured*: nouns and verbs are generated from the ontology, not written by hand, which means the CLI surface and its formal specification remain in sync. Second, the grammar is *extensible*: adding a noun requires registering it in the ontology, regenerating, and implementing the generated scaffolding — a process that enforces interface consistency without requiring contributors to audit existing code. This approach builds on prior work in domain-specific language design [4, 5] and ontology-driven code generation [6], applying those ideas specifically to the DevOps CLI domain.

## 5.3 Practical Contributions

### 5.3.1 The cargo-cicd Tool

cargo-cicd v26.6.2 is a production-quality Cargo subcommand distributed on crates.io under the MIT/Apache-2.0 dual license. It provides nine commands across seven nouns, covering the complete local CI/CD lifecycle: workspace health diagnosis, git state inspection, changed-test selection, trybuild fixture management, target directory maintenance, and crate publishing. The tool requires no network access for its core operations, runs in under two seconds on representative workspaces, and integrates directly into GitHub Actions pipelines.

### 5.3.2 The cicd.toml Schema

The `cicd.toml` file is a local state carrier whose schema is defined in `src/cicd_toml.rs`. It persists cross-run state (last-known target size, last HEAD hash, last test verdicts) and configuration (`[target]`, `[test.changed]`, `[autonomic]`), enabling subsequent invocations to skip expensive recomputation. The schema is versioned and validated at load time; a corrupted or stale `cicd.toml` is treated as a `refuse` verdict by the `WorkspaceDoctorEvent`, not as a silent error. The schema design draws on prior work in local configuration file standards [7] and workspace state management [8].

### 5.3.3 The wasm4pm Integration Protocol

The wasm4pm integration protocol, formalised in Chapter 4, defines the contract between cargo-cicd (the evidence emitter) and the wasm4pm oracle (the adjudicator). Evidence is emitted in XES format [9] to `target/cargo-cicd/evidence/`, with one XES file per invocation. The `wpm receipt doctor --format json --strict` command validates structured receipts; `wpm audit <file.xes>` validates raw XES traces. Release closure requires both commands to return an `Accept` verdict. This protocol instantiates the broader process-mining conformance checking paradigm [10, 11] in a local-first DevOps context, demonstrating that formal process conformance need not require a cloud infrastructure or a process-mining server.

## 5.4 Lessons Learned

### 5.4.1 Rust for DevOps Tooling

Rust proved well-suited to DevOps tooling for reasons beyond performance. The ownership model naturally enforces the single-source invariant: only one adapter can hold a mutable reference to a state dimension at a time, making race conditions in the adapter pipeline a compile-time error rather than a test-time discovery. The `cfg(feature = ...)` attribute provided a clean mechanism for the two-tier public/private boundary, enforced by the type system at every call site. The `anyhow` crate's error propagation idioms kept adapter code concise without sacrificing error context.

The primary challenge was the Minimum Supported Rust Version (MSRV) constraint. cargo-cicd requires Rust 1.85 or later, driven by dependencies that use stabilised features from the 2024 edition. Managing MSRV in a tool that targets developers running older stable channels required explicit `rust-version` declarations in `Cargo.toml` and a `rust-toolchain.toml` fixture in the test suite. Future tooling in this space should establish MSRV policy before selecting dependencies, not after.

### 5.4.2 Test Isolation via Fixture Workspaces

The `FixtureWorkspace` pattern — an ephemeral git repository created in a `TempDir` for each integration test, with a known initial state — proved essential for test reliability. Without fixture isolation, integration tests would depend on the developer's local git state, Cargo cache, and toolchain configuration, making them flaky on CI and impossible to reproduce. The eight fixture archetypes (`clean`, `dirty`, `missing_manifest`, `with_toolchain_mismatch`, `with_target_over_limit`, `with_corrupted_cicd_toml`, `with_stale_cicd_toml`, `with_changed_trybuild_fixture`) cover the seven invariant verdicts and provide a vocabulary for describing workspace health states.

A key lesson is that fixture workspaces must be valid git repositories from their first commit. Several early test failures were caused by adapters that invoked `git status --porcelain` in a directory that had never been committed to, producing unexpected output. The solution — initialising and committing the fixture in its constructor — is now encoded in the `FixtureWorkspace::clean()` implementation and serves as the base for all derived fixtures.

### 5.4.3 The Cost of Deferred Observability

cargo-cicd v26.6.2 does not use a standard logging framework. Instrumentation is via `ProcessEvent` emission to XES and direct `println!`/`eprintln!` calls. This was a deliberate early decision to avoid the `tracing`/`env_logger` dependency weight while the architecture was still evolving. In retrospect, the absence of structured logging added significant friction during adapter debugging: reproducing a failure required adding temporary `eprintln!` statements, rebuilding, and re-running the test. A structured logging framework, initialised early, would have reduced this friction substantially without adding meaningful binary size for release builds (which strip debug symbols).

## 5.5 Future Work

The following directions are identified as high-priority extensions for the cargo-cicd research programme. They are ordered from most architecturally constrained (requiring core changes) to most independent (additive features).

### 5.5.1 Remote State Synchronisation for cicd.toml

The current `cicd.toml` is intentionally local-only: it is added to `.gitignore` and is not pushed to remote. This means that a developer switching machines, or a CI system cloning the repository fresh, starts with no cached state and incurs the full cost of cold-start workspace analysis on every run. Remote state synchronisation would allow `cicd.toml` to be stored in a lightweight remote store (a git-notes ref, a repository-scoped artifact store, or a dedicated key-value backend) and fetched on clone. The primary challenge is conflict resolution: two developers modifying the same workspace concurrently may produce divergent `cicd.toml` states. A merge strategy based on last-write-wins per state dimension, with conflict detection for configuration keys, is the most promising approach.

### 5.5.2 Workspace Federation for Monorepos

cargo-cicd currently recognises a single workspace root, identified by the presence of a `[workspace]` section in `Cargo.toml`. Monorepos that span multiple Cargo workspaces — a common pattern in organisations that maintain both a core library workspace and a tools workspace in the same repository — are not supported. Workspace federation would allow a single `cicd.toml` at the repository root to aggregate state from multiple workspace roots, with per-workspace configuration sections and a unified policy evaluation pass. This requires extending the `EngineState` aggregate to carry a vector of `WorkspaceState` instances and modifying all adapters to accept a workspace selector.

### 5.5.3 Policy Apply Mode

The autonomic policy engine currently operates in `suggest` mode: policies evaluate workspace state and emit recommendations, but no action is taken without explicit user intervention. The `--apply` flag is parsed but not implemented. Full policy apply mode would allow policies to take direct action — pruning stale build artifacts, running the formatter, staging changed fixture files — subject to a confirmation prompt and an audit trail in the XES evidence log. The primary safety requirement is idempotency: a policy applied twice must produce the same result as applied once. This is already satisfied for `TargetPressurePolicy` (pruning is idempotent) but requires careful specification for `GitPhaseDirtyPolicy` (which may need to stage and commit files).

### 5.5.4 Parallel Test Execution with State Locking

The current test runner serialises test execution to avoid concurrent writes to `cicd.toml` and the XES evidence directory. For large workspaces with many independent crates, this serialisation is the dominant source of latency. Parallel test execution would run tests for independent crates concurrently, with a file-level lock on `cicd.toml` and a per-session evidence directory (using the session ID as a path component). The `--jobs N` flag is already recognised by the CLI; the implementation requires a work-stealing scheduler and a lock-free merge of per-job `TestPlanState` results into the aggregate `EngineState`.

### 5.5.5 Custom Policy Plugin System

Built-in policies cover the invariant workspace health conditions: git dirty state, target directory pressure, toolchain mismatch, and manifest validity. Organisation-specific policies — enforcing internal dependency version alignment, checking for approved license identifiers, or requiring specific commit message formats — cannot currently be expressed. A plugin system would allow policies to be defined as dynamic libraries (`.so`/`.dylib`/`.dll`) or as WebAssembly modules, loaded at runtime from a `[plugins]` section in `cicd.toml`. The WebAssembly approach is architecturally preferred: it is sandboxed, cross-platform, and composable with the existing wasm4pm integration.

### 5.5.6 Windows Cross-Compilation Support

cargo-cicd is tested on Linux and macOS. Windows support is blocked by two issues: path separator handling in `TargetScannerAdapter` (which uses Unix-style path separators in several string operations) and the absence of a Windows CI runner in the release pipeline. Resolving these issues would expand the addressable developer population significantly, as a substantial fraction of Rust developers work on Windows. The changes are largely mechanical — replacing string-based path operations with `std::path::PathBuf` throughout — but require a Windows test runner and fixture workspace validation on NTFS.

### 5.5.7 Tracing and Observability Integration

Structured logging via the `tracing` crate [12] would provide three benefits: (i) runtime-configurable verbosity via `RUST_LOG`, eliminating the need for recompilation during debugging; (ii) span-level timing for adapter calls, enabling performance regression detection; and (iii) integration with `tracing-opentelemetry` for export to observability backends. The integration path is well-defined: add `tracing` and `tracing-subscriber` to `[dependencies]`, initialise a subscriber in `main()`, and replace `eprintln!` calls in adapters with `tracing::debug!` and `tracing::info!`. The XES evidence pipeline is orthogonal to the tracing infrastructure and would not be replaced; tracing serves developer debugging, while XES evidence serves process conformance.

## 5.6 Closing Remarks

cargo-cicd began as a response to a practical observation: Rust workspace developers routinely push code that fails CI for reasons that are locally detectable — dirty git trees, bloated `target/` directories, broken `trybuild` fixtures, stale toolchain pins. A tool that checks these conditions in two seconds, before any network round-trip, eliminates an entire class of CI failures and recovers developer time that would otherwise be spent waiting for remote pipelines.

The contribution of this dissertation is to show that such a tool need not be a collection of shell scripts. By modelling the workspace as an aggregate of structured state dimensions, populating that state through a pipeline of single-source adapters, and emitting every action as process evidence in a standard format, cargo-cicd becomes something more than a linter: it becomes a process-data engine whose behaviour can be formally verified by an external oracle. The wasm4pm integration protocol demonstrates that local-first CI/CD and formal process conformance are not in tension — they are complementary.

The noun-verb grammar, manufactured from an ontological specification, enforces a discipline that CLI tools rarely achieve: the public interface is a formal projection of a formal model, not an accretion of ad-hoc subcommands. This discipline pays dividends as the tool grows: adding a noun to the ontology propagates a consistent interface through code generation, documentation, and test scaffolding without requiring contributors to audit or imitate existing code.

The limitations documented in Section 5.5 are real constraints, not theoretical gaps. Remote state synchronisation, workspace federation, and the custom policy plugin system each require non-trivial architectural work. But the foundation laid by cargo-cicd v26.6.2 — the `EngineState` aggregate, the adapter pipeline, the feature-gated two-tier boundary, the wasm4pm evidence protocol — is designed to support these extensions without requiring a rewrite. The Level 5 engine taxonomy is not a ceiling; it is a floor.

---

## References

[1] Humble, J. and Farley, D. (2010). *Continuous Delivery: Reliable Software Releases through Build, Test, and Deployment Automation*. Addison-Wesley Professional.

[2] Forsgren, N., Humble, J. and Kim, G. (2018). *Accelerate: The Science of Lean Software and DevOps*. IT Revolution Press.

[3] Gamma, E., Helm, R., Johnson, R. and Vlissides, J. (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley.

[4] Fowler, M. (2010). *Domain-Specific Languages*. Addison-Wesley Professional.

[5] Mernik, M., Heering, J. and Sloane, A. M. (2005). When and how to develop domain-specific languages. *ACM Computing Surveys*, 37(4), pp. 316–344.

[6] Volter, M. and Stahl, T. (2006). *Model-Driven Software Development: Technology, Engineering, Management*. John Wiley & Sons.

[7] Preston-Werner, T. (2013). TOML: Tom's Obvious, Minimal Language. GitHub repository. https://github.com/toml-lang/toml

[8] Beller, M., Gousios, G. and Zaidman, A. (2017). Oops, my tests broke the build: an explorative analysis of Travis CI with GitHub. In *Proceedings of the 14th International Conference on Mining Software Repositories (MSR)*, pp. 356–367. IEEE.

[9] Verbeek, H. M. W., Buijs, J. C. A. M., van Dongen, B. F. and van der Aalst, W. M. P. (2010). XES, XESame, and ProM 6. In *Proceedings of the CAiSE Forum*, Lecture Notes in Business Information Processing, vol. 72, pp. 60–75. Springer.

[10] van der Aalst, W. M. P. (2016). *Process Mining: Data Science in Action*. 2nd ed. Springer.

[11] Carmona, J., van Dongen, B., Solti, A. and Weidlich, M. (2018). *Conformance Checking: Relating Processes and Models*. Springer.

[12] Tokio Contributors (2019). tracing: Application-level tracing for Rust. https://docs.rs/tracing

[13] Matsakis, N. D. and Klock, F. S. (2014). The Rust programming language. In *Proceedings of the 2014 ACM SIGAda Annual Conference on High Integrity Language Technology (HILT)*, pp. 103–104. ACM.

[14] Jung, R., Jourdan, J.-H., Krebbers, R. and Dreyer, D. (2021). Safe systems programming in Rust. *Communications of the ACM*, 64(4), pp. 144–152.

[15] Klabnik, S. and Nichols, C. (2019). *The Rust Programming Language*. No Starch Press.

[16] van der Aalst, W. M. P., Weijters, A. J. M. M. and Maruster, L. (2004). Workflow mining: discovering process models from event logs. *IEEE Transactions on Knowledge and Data Engineering*, 16(9), pp. 1128–1142.

[17] Leemans, S. J. J., Fahland, D. and van der Aalst, W. M. P. (2013). Discovering block-structured process models from event logs — a constructive approach. In *Proceedings of the 4th International Conference on Application and Theory of Petri Nets and Concurrency (PETRI NETS)*, Lecture Notes in Computer Science, vol. 7927, pp. 311–329. Springer.

[18] Kephart, J. O. and Chess, D. M. (2003). The vision of autonomic computing. *IEEE Computer*, 36(1), pp. 41–50.

[19] Huebscher, M. C. and McCann, J. A. (2008). A survey of autonomic computing — degrees, models, and applications. *ACM Computing Surveys*, 40(3), pp. 1–28.

[20] McIlroy, M. D., Pinson, E. N. and Tague, B. A. (1978). UNIX time-sharing system: Forward. *Bell System Technical Journal*, 57(6), pp. 1899–1904.

[21] Spinellis, D. (2017). State of the art: a taxonomy of software tools. *IEEE Software*, 34(5), pp. 10–11.

[22] Kochhar, P. S., Thung, F. and Lo, D. (2016). Code coverage and test suite effectiveness: empirical study with real bugs in large systems. In *Proceedings of the 22nd International Conference on Software Analysis, Evolution and Reengineering (SANER)*, pp. 560–564. IEEE.

[23] Hilton, M., Tunnell, T., Huang, K., Marinov, D. and Dig, D. (2016). Usage, costs, and benefits of continuous integration in open-source projects. In *Proceedings of the 31st IEEE/ACM International Conference on Automated Software Engineering (ASE)*, pp. 426–437. IEEE.

[24] Beller, M., Gousios, G. and Zaidman, A. (2019). Travistorrent: Synthesizing Travis CI and GitHub for full-stack research on continuous integration. In *Proceedings of the 14th International Conference on Mining Software Repositories (MSR)*, pp. 447–450. IEEE.

[25] Zhao, Y., Serebrenik, A., Zhou, Y., Filkov, V. and Vasilescu, B. (2017). The impact of continuous integration on other software development practices: a large-scale empirical study. In *Proceedings of the 32nd IEEE/ACM International Conference on Automated Software Engineering (ASE)*, pp. 60–71. IEEE.

[26] Hoare, C. A. R. (1978). Communicating sequential processes. *Communications of the ACM*, 21(8), pp. 666–677.

[27] Milner, R. (1989). *Communication and Concurrency*. Prentice Hall.

[28] Gruber, T. R. (1993). A translation approach to portable ontology specifications. *Knowledge Acquisition*, 5(2), pp. 199–220.

[29] Noy, N. F. and McGuinness, D. L. (2001). Ontology development 101: a guide to creating your first ontology. Stanford Knowledge Systems Laboratory Technical Report KSL-01-05.

[30] Dekel, U. and Herbsleb, J. D. (2009). Improving API documentation usability with knowledge pushing. In *Proceedings of the 31st International Conference on Software Engineering (ICSE)*, pp. 320–330. IEEE.
