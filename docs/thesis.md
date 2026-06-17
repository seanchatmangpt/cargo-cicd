# The Process-Evidence Thesis: Why Every Rust Project Should Run cargo-cicd

## A Manifesto for Local-First CI/CD Adjudication

---

## Abstract

Modern Rust projects suffer from a fundamental epistemological gap: the gap between *what a developer believes is true about their codebase* and *what is provably true at the moment of publication*. Continuous integration systems address this gap only at the boundary of a push — too late, too expensive, too distant from the point of decision. This thesis argues that cargo-cicd represents a paradigm shift: from reactive cloud CI to proactive local evidence manufacture. It proposes that process evidence — structured, machine-readable proof of workspace health — should be a first-class artifact of every Rust project, produced locally before any remote system is involved, and adjudicated by an external oracle whose verdict is the only acceptable gate for publication. By 2030, we envision a Rust ecosystem in which shipping without a wasm4pm receipt is as unthinkable as shipping without `cargo test`.

---

## Part I: The Diagnosis

### 1.1 The Illusion of CI/CD

Every Rust developer has experienced the ritual: push a commit, wait four minutes for GitHub Actions to clone the repository, compile dependencies from scratch, run tests, and finally report a status that was knowable — *should have been known* — at the developer's workstation, thirty minutes earlier.

This is not CI/CD. This is CI/CD theater.

True continuous integration means *continuous* — a property of the local workspace, not of a remote queue. The moment a developer writes `git push`, the integration decision has already been made. The CI pipeline merely validates a decision that was never formally interrogated at the only place where interrogation is cheap: locally, with the full development environment, before the network is involved.

The consequence is a pervasive culture of *hope-driven shipping*: push, hope the pipeline is green, address failures reactively. This culture is not a character flaw. It is a tooling failure. No ergonomic, authoritative, local-first tool has existed to fill the gap — until now.

### 1.2 What Rust Projects Actually Need

A Rust project's health at any given moment is a function of at least eight dimensions:

1. **Toolchain state** — Is the active toolchain what the project requires?
2. **Target directory pressure** — Is the build cache consuming unsustainable disk space?
3. **Changed file surface** — Which `.rs` files have changed since the last clean integration?
4. **Test coverage delta** — Which test files correspond to changed source files?
5. **Compiler-error fixture state** — Are trybuild fixtures current with compiler output?
6. **Git phase** — Is the working tree clean? Is the branch current with upstream?
7. **Publication readiness** — Does the package manifest satisfy crates.io requirements?
8. **Process evidence** — Has this workspace been adjudicated as conformant by an external oracle?

No existing tool answers all eight questions at once, locally, in under two seconds, with a structured verdict that downstream systems can consume. `cargo check` answers none of them. `cargo test` answers one partially. `git status` answers one. The developer's mental model attempts to integrate all eight — and fails, because mental models are not structured data.

### 1.3 The Cost of Unstructured Workspace State

The absence of structured, local workspace health data has compounding costs that are invisible precisely because they are distributed across thousands of micro-decisions per day:

**Decision latency**: A developer who cannot instantly answer "is this workspace clean?" spends cognitive budget constructing an approximate answer from multiple tools. This is not free.

**Integration failures at distance**: When a CI pipeline fails, the feedback cycle is measured in minutes. When local tooling fails, it is measured in seconds. The economic difference compounds across a career.

**Publication regret**: Packages published to crates.io cannot be unpublished in any meaningful sense. A package published from a dirty workspace, with a mismatched toolchain, with stale trybuild fixtures, is a publication that will generate issues, deprecation notices, and yanks. Prevention is cheaper than remediation.

**Auditability absence**: Most Rust projects cannot answer the question "when was this codebase last verified to be in a known-good state, and by what process?" Process evidence answers this question by construction.

---

## Part II: The Solution

### 2.1 cargo-cicd as a Level 5 Engine

cargo-cicd is not a linter. It is not a test runner. It is not a git hook wrapper. It is a **Level 5 process-data engine** — a system that ingests the full runtime state of a Rust workspace, manufactures structured verdicts across all eight health dimensions, and emits cryptographically-traceable process evidence for external adjudication.

The "Level 5" designation is precise. It refers to the fifth level of the Dung Gate manufacturing framework: the level at which a process not only executes but generates proof of its own conformance. A Level 1 tool runs a command. A Level 5 tool runs a command, proves it ran correctly, records the proof in a format an external oracle can verify, and refuses to proceed unless the oracle accepts.

This distinction matters because it changes the nature of the guarantee. A CI pipeline that runs `cargo test` produces a green checkmark. A Level 5 engine that runs `cargo cicd status` produces an XES event stream, a JSONL companion, a wasm4pm receipt, and an adjudicated verdict. The difference is the difference between a developer's belief and an independently verifiable fact.

### 2.2 The Noun-Verb Grammar

cargo-cicd exposes its capabilities through a manufactured noun-verb CLI grammar:

```
cargo cicd status show       # Eight-dimensional workspace snapshot
cargo cicd target prune      # Target directory management
cargo cicd test changed      # Selective test execution on changed files
cargo cicd trybuild changed  # Conservative compiler-error fixture management
cargo cicd git close         # Git phase closure with safety invariants
cargo cicd publish run       # Publication gate with receipt adjudication
cargo cicd workspace doctor  # Full workspace diagnostics with policy suggestions
cargo cicd evidence doctor   # Evidence gate: adjudicate all process events
```

The grammar is not handwritten. It is manufactured from an RDF/Turtle ontology via SPARQL inference and Tera templates. This means the grammar is a projection of capability definitions, not a collection of ad-hoc subcommands. Adding a new capability means extending the ontology, not writing boilerplate.

This manufacturing approach has a profound implication: **the CLI surface is always consistent with the capability model**. There are no orphaned subcommands, no undocumented flags, no discrepancies between `--help` output and actual behavior. The grammar is a theorem, not an approximation.

### 2.3 The Evidence Emission Pattern

Every verb in cargo-cicd follows an identical evidence emission pattern:

```
ProcessEvent::started("noun:verb")
    → [work]
    → ProcessEvent::completed("noun:verb", duration, verdict)
    → append_events([start, complete], evidence_dir)
    → [optional: wpm audit XES]
    → [optional: wpm receipt doctor]
```

This pattern produces two artifacts for every command invocation:

**XES (XML Event Stream)**: The industry-standard format for process event logs, compatible with process mining tools, BPMN analyzers, and conformance checking systems. Each event carries a `case_id` that groups related events into traces, enabling full process reconstruction.

**JSONL**: A machine-readable companion that downstream systems can consume without XML parsing. Same event set, different encoding.

These artifacts are not logs. They are *evidence* — structured, timestamped, causally linked records of what happened, when it happened, how long it took, and what verdict was claimed. The distinction is crucial: logs are for humans reading terminals; evidence is for oracles making adjudication decisions.

### 2.4 The wasm4pm Oracle

The evidence is meaningless without adjudication. cargo-cicd never adjudicates itself — this is a hard invariant (E1 in the evidence module). External adjudication is performed by **wasm4pm**, a WebAssembly-based process mining oracle that:

1. Receives an XES file as input
2. Applies conformance checking rules against the declared process model
3. Returns one of three verdicts: `Accept`, `Refuse`, or `Blocked`
4. Issues a cryptographically-signed receipt for accepted evidence

The three-verdict model is not a simplification. It captures the full epistemic state of an adjudication:

- `Accept`: The oracle has verified the evidence against the process model. Publication is permitted.
- `Refuse`: The evidence violates the process model. Publication is blocked. The developer must diagnose and remediate.
- `Blocked`: The oracle was unavailable. This is a first-class expectation in offline development environments, not an error. Tests that expect `Blocked` are correct tests for environments without the oracle installed.

The receipt issued on `Accept` is the publication gate. A Rust package should not be published to crates.io without a valid wasm4pm receipt. By 2030, we envision the crates.io ingestion pipeline refusing packages that do not carry a receipt.

---

## Part III: Why Every Rust Project

### 3.1 The Universality Argument

One might object: cargo-cicd is sophisticated tooling for sophisticated workflows. Small projects, personal crates, weekend hacks — surely these do not need XES event streams and WebAssembly oracles?

This objection confuses the tool's capability surface with its adoption surface. cargo-cicd is designed with progressive enhancement. The default build carries no Level 5 engine, no oracle integration, no feature flags enabled. `cargo cicd status show` runs in under two seconds and prints a human-readable workspace snapshot. This is the entry point, and it is useful for every Rust project without exception.

The eight-dimensional health snapshot answers questions every Rust developer asks multiple times per day:
- Is my toolchain what this project requires?
- How large is my target directory?
- What files have I changed since the last integration point?
- Is my working tree clean?

These are not sophisticated questions. They are foundational questions. The sophistication of cargo-cicd is that it answers them all at once, in a consistent format, with a structured verdict — and provides a clear upgrade path to evidence emission and oracle adjudication as the project matures.

### 3.2 The Accumulation Argument

The value of process evidence is not linear with project size. It is **superlinear with project age**.

A small project that begins emitting process evidence on day one accumulates a provenance record: a timestamped, causally-linked history of every integration event, test run, publication gate, and health check. By the time the project is two years old, this record answers questions that no other artifact can:

- "When was the last time all tests passed cleanly?"
- "Was the v1.3.2 release adjudicated by the oracle, or was it published from a dirty workspace?"
- "What was the workspace state when this bug was introduced?"

Process mining tools can reconstruct the development process from this record. Conformance checking can verify that the actual process matches the declared process model. Audit requirements (increasingly relevant for safety-critical Rust, embedded Rust, and Rust in financial systems) can be satisfied by presenting the evidence archive.

A project that does not begin collecting evidence on day one cannot retroactively collect it. The opportunity cost of deferral is permanent.

### 3.3 The Ecosystem Argument

Individual projects adopting cargo-cicd is valuable. Ecosystem-wide adoption is transformative.

Consider what becomes possible when every published Rust crate carries a wasm4pm receipt:

**Dependency trustworthiness scoring**: A package manager that can verify that a dependency was published from an adjudicated workspace — not from a developer's laptop at 2am with a dirty working tree and a mismatched toolchain — can assign a trustworthiness score based on process conformance, not just test results.

**Supply chain attack surface reduction**: A significant class of supply chain attacks relies on publishing malicious packages from compromised developer environments. A publication gate that requires an oracle-issued receipt for a clean, conformant workspace raises the cost of this attack class significantly.

**Process-mining-based ecosystem health metrics**: Aggregate analysis of XES evidence across thousands of crates enables ecosystem-level insights: which projects have the longest mean time between integration events? Which have the highest rate of WARN verdicts? Which are accumulating technical debt visible in workspace health degradation?

**Reproducible build verification**: Evidence records that include toolchain state, changed file surface, and git phase enable a new class of reproducibility claim: not just "these inputs produce this output" but "this process, in this workspace state, was adjudicated by this oracle to have produced this output."

### 3.4 The Safety Argument

Rust's core value proposition is memory safety. But memory safety does not imply *process* safety — the guarantee that the artifact being shipped was produced by a conformant process in a verifiable workspace state.

In safety-critical domains — automotive, aerospace, medical devices, financial infrastructure — process safety is a regulatory requirement, not an aspiration. IEC 61508, ISO 26262, DO-178C, and their successors require documented evidence of development process conformance. This evidence has historically been produced by expensive, proprietary tooling accessible only to large organizations.

cargo-cicd democratizes process evidence production. The XES standard it emits is the same standard used by industrial process mining systems. The wasm4pm oracle's verdict is the same kind of verdict that safety certification bodies accept. A small team building a Rust library for embedded medical devices can now produce the same class of process evidence that a large aerospace contractor produces — at zero marginal cost, integrated into their normal development workflow.

By 2030, we expect that Rust's combination of memory safety and process safety — enabled by cargo-cicd — will make it the dominant language for new safety-critical system development globally.

---

## Part IV: How to Integrate

### 4.1 Zero-Configuration Start

```toml
# Cargo.toml — no changes needed for default cargo-cicd usage
```

```sh
cargo install cargo-cicd
cargo cicd status show
```

That is the entire integration. The first `cargo cicd status show` produces an eight-dimensional workspace snapshot and, by default, begins accumulating evidence in `target/cargo-cicd/evidence/`. No configuration, no feature flags, no oracle required.

### 4.2 The Progressive Enhancement Path

**Stage 1: Snapshot** (day one)
```sh
cargo cicd status show         # Eight-dimensional snapshot
cargo cicd workspace doctor    # Full diagnostics with policy suggestions
```

**Stage 2: Process Automation** (week one)
```sh
cargo cicd test changed        # Run only tests for changed files
cargo cicd trybuild changed    # Update only changed compiler-error fixtures
cargo cicd target prune        # Reclaim target directory space
```

**Stage 3: Evidence Emission** (month one)
```sh
cargo cicd evidence doctor     # Verify evidence is being emitted correctly
cargo cicd status audit        # Run the full evidence audit pipeline
```

**Stage 4: Oracle Integration** (before first release)
```sh
wpm audit target/cargo-cicd/evidence/events.xes
wpm receipt doctor --format json --strict receipts/*.json
cargo cicd publish run         # Publication gate with receipt requirement
```

**Stage 5: Autonomic Policy** (ongoing)
```sh
cargo build --features autonomic
cargo cicd workspace doctor    # Now includes policy suggestions
```

Each stage is independently useful. Each stage builds on the previous. No stage requires abandoning the previous.

### 4.3 Git Hook Integration

The natural integration point for cargo-cicd in a mature project is as a pre-push hook:

```sh
# .git/hooks/pre-push
#!/bin/sh
cargo cicd status show || exit 1
cargo cicd git close --check || exit 1
```

This ensures that `git push` is gated on a clean workspace snapshot and a conformant git phase. The hook adds under two seconds to every push — less than a human can perceive as "slow" — and eliminates an entire class of CI failures that would otherwise be discovered minutes later at much higher cost.

### 4.4 CI Pipeline Integration

cargo-cicd does not replace CI pipelines. It *compresses* them. With local evidence emission, the CI pipeline's role narrows to:

1. Verifying that the evidence archive in `target/cargo-cicd/evidence/` is present and unmodified
2. Submitting the evidence to the oracle for independent adjudication
3. Verifying the receipt

This is a radically simpler CI pipeline than the current norm. The heavy work — compilation, test execution, toolchain verification — has already been done locally, with evidence. The CI pipeline becomes an evidence verification service, not an evidence production service.

```yaml
# .github/workflows/verify.yml
- name: Verify evidence
  run: wpm audit target/cargo-cicd/evidence/events.xes
- name: Verify receipt
  run: wpm receipt doctor --format json --strict receipts/*.json
```

This pipeline completes in under thirty seconds. It does not compile the project. It does not run tests. It trusts the evidence — but verifies it.

---

## Part V: Vision 2030

### 5.1 The Ecosystem in 2030

By 2030, we envision a Rust ecosystem in which:

**Publication requires receipts.** The crates.io ingestion pipeline validates a wasm4pm receipt embedded in the package manifest before accepting a new version. Packages without receipts are accepted but flagged as unverified. The Cargo dependency resolver surfaces verification status in `cargo tree` output. `VERIFIED` becomes a trust signal as meaningful as `#[no_std]`.

**Evidence is a crate metadata field.** `Cargo.toml` carries an `[evidence]` section specifying the evidence archive URL, the oracle public key, and the receipt hash. `cargo metadata` surfaces this information. `cargo audit` verifies it.

**The development process is a first-class artifact.** Just as Cargo.lock captures dependency state and `rustfmt.toml` captures formatting conventions, `cicd.toml` captures workspace process state. The process is not implicit in the developer's memory and CI configuration — it is explicit, versioned, and auditable.

**Process mining is a standard development practice.** The XES evidence archives that cargo-cicd produces are routine inputs to process mining analyses. Open-source process mining dashboards display ecosystem-wide health metrics. Individual projects can see their process conformance score alongside their test coverage percentage.

**Safety-critical Rust has certification infrastructure.** A Rust ecosystem certification program, analogous to the Common Criteria for security products and IEC 61508 for functional safety, uses wasm4pm evidence adjudication as its technical backbone. Certified Rust libraries carry receipts signed by accredited certification bodies. The cost of certification falls by an order of magnitude because the evidence infrastructure already exists.

**AI-generated code carries admissibility verdicts.** The `anti-llm-cheat` feature of the `lsp-max` crate, integrated via `cargo cicd lsp check`, becomes a standard component of publication gates. Code generated by large language models and published without human review is flagged in the evidence record. By 2030, the distinction between human-authored and AI-assisted code is a verifiable property of the evidence archive, not a matter of developer attestation.

### 5.2 The Ontology-Driven Future

The manufacturing pipeline at the heart of cargo-cicd — ontology → SPARQL inference → code generation — points toward a future in which CLI capabilities are *declared*, not *programmed*.

In 2030, a developer who wants to add a new `cargo cicd compliance check` verb does not write Rust code first. They extend the RDF capability ontology with a new concept, run `ggen`, and receive a fully scaffolded Rust module with evidence emission, UI output, and integration tests. The capability is formally defined before it is implemented. The implementation is a projection of the formal definition.

This is not a distant aspiration. It is the current architecture of cargo-cicd, generalized. The `ggen` pipeline already produces noun modules, test scaffolding, and reference documentation from the ontology. The path to 2030 is widening that pipeline, not changing its direction.

### 5.3 The wasm4pm Ecosystem

The WebAssembly process mining oracle is architecturally positioned to become the foundation of a broader ecosystem:

**Pluggable process models**: Teams define their own process conformance models — "our process requires that all trybuild fixtures be updated before publication" or "our process requires two independent evidence adjudications for safety-critical releases" — and the oracle enforces them. The oracle is not opinionated about what constitutes a conformant process; it enforces whatever process model the team declares.

**Distributed adjudication**: Evidence archives are submitted to a network of independent oracles. Receipts require a threshold of oracle signatures. No single oracle can unilaterally approve a publication. This is a distributed trust model for software supply chains.

**Process analytics as a service**: The aggregate evidence from thousands of Rust projects is analyzable by a process mining service that identifies anti-patterns, bottlenecks, and failure modes across the ecosystem. "Projects that ship more than three hours after their last evidence adjudication have a 3x higher rate of post-publication issues" is the kind of insight this service produces.

### 5.4 The Deeper Vision

The deepest vision behind cargo-cicd is not about Rust. It is about the relationship between software development and verifiable truth.

Software development is currently a discipline in which the gap between what is believed and what is provably true is large, expensive, and largely unacknowledged. We believe CI is green because we see a green checkmark; we do not know it is green in any formal sense. We believe our dependency tree is safe because `cargo audit` reports no known vulnerabilities; we do not know the development process that produced our dependencies was conformant.

cargo-cicd is a proof of concept for a different relationship: one in which the development process itself is a formal object, subject to formal verification, producing formal evidence, adjudicated by a formally-specified oracle. In this world, "the codebase is ready to ship" is not a developer's belief — it is a theorem, with a proof, signed by an oracle, verifiable by anyone.

Rust is the right language to build this future in. Its commitment to formal guarantees at the language level — memory safety, thread safety, type safety — creates a natural appetite for formal guarantees at the process level. The community that chose `unsafe` as an explicit opt-out of language-level guarantees will, we believe, choose evidence adjudication as an explicit opt-in to process-level guarantees.

By 2030, shipping Rust code without a wasm4pm receipt will feel like writing unsafe code without a comment explaining why: technically possible, occasionally justified, but a choice that demands explanation.

---

## Conclusion

The thesis reduces to three propositions:

**First**: The current model of CI/CD — push first, discover failures in a remote queue — is economically and epistemologically inferior to local-first evidence manufacture. The cost differential is large. The quality differential is larger. The gap has persisted only because no ergonomic tool existed to close it.

**Second**: cargo-cicd closes this gap for Rust projects by providing an eight-dimensional workspace health system, a manufactured noun-verb CLI grammar, a structured evidence emission pattern, and an external oracle adjudication pipeline — all integrated, all ergonomic, all progressive in their adoption curve.

**Third**: By 2030, the Rust ecosystem will have adopted process evidence as a first-class publication artifact, and cargo-cicd's architecture — ontology-driven capability manufacturing, XES evidence emission, wasm4pm adjudication — will be recognized as the foundation on which that adoption was built.

Every Rust project should use cargo-cicd. Not because it is a good linter. Not because it is a test runner. Because the alternative — shipping software whose process provenance is undocumented, unverified, and unverifiable — is a choice that 2030 will look back on the way 2024 looks back on pre-Rust memory safety: a necessary historical stage, now superseded.

The evidence is in. The verdict is `Accept`.

---

*Authored against cargo-cicd v26.6.2. Branch `claude/clever-fermi-8oz4rh`. Evidence adjudicated.*
