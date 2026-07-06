# cargo-cicd Documentation Index

cargo-cicd keeps Rust workspaces clean, fast, and push-ready. This index
organizes all documentation using the [Diátaxis](https://diataxis.fr/) framework,
which divides documentation into four distinct quadrants based on what the reader
is trying to do. Diátaxis prevents the most common documentation failure: mixing
information that serves different needs into the same document, which serves none
of them well.

Alongside the four quadrants, `docs/vision/` holds forward-looking, aspirational,
and roadmap material that does **not** describe shipped behavior — see
[Vision documentation](#vision-forward-looking-not-shipped) below for the
distinction.

---

## The Diátaxis quadrants

```
                   Practical
                       |
         TUTORIAL       |       HOW-TO GUIDE
     (learning by       |    (solving a specific
          doing)        |         problem)
                       |
  Study ———————————————+——————————————— Work
                       |
       EXPLANATION      |       REFERENCE
   (understanding       |    (looking things up
       the design)      |      while working)
                       |
                   Theoretical
```

| Quadrant | Axis | Reader's goal | Writer's role |
|----------|------|---------------|---------------|
| Tutorial | Practical + Study | Acquire new skills by doing | Teacher |
| How-to guide | Practical + Work | Solve a specific real-world problem | Guide |
| Reference | Theoretical + Work | Look up accurate information while working | Documenter |
| Explanation | Theoretical + Study | Understand why things are the way they are | Analyst |

---

## Tutorials

**Tutorials are learning-oriented.** They take a newcomer through a complete
experience from start to finish. The reader acquires skills by doing real work
in a guided environment. Tutorials assume no prior familiarity with cargo-cicd.
Use this section when you are using cargo-cicd for the first time, or when you
want to see the full workflow end-to-end.

| Document | What you will learn |
|----------|---------------------|
| [Quick Start](tutorials/quick-start.md) | Install cargo-cicd and run the basic workflow in 5 minutes |
| [Tutorial 1 — Your First Clean Workspace](tutorials/01-first-clean-workspace.md) | Install cargo-cicd, run `status`, and see a CLEAN/DIRTY verdict — complete first session |
| [Tutorial 2 — Your First OCEL Evidence Record](tutorials/02-ocel-evidence.md) | Emit an OCEL 2.0 evidence file, inspect its structure, understand the Blocked verdict |
| [Tutorial 3 — Run the Full Pipeline](tutorials/03-full-pipeline.md) | Enable all features and run `03_max_pipeline`; all 10 advanced modules in one program |
| [First Playground Run](tutorials/first-playground-run.md) | Explore commands in a scratch workspace without touching your real project |

---

## How-to guides

**How-to guides are problem-oriented.** They give a practitioner the steps to
solve a specific, well-defined real-world problem. They assume you already know
what cargo-cicd is and why you are using it. Use this section when you know what
you want to accomplish and need to know how to do it.

| Document | Problem solved |
|----------|----------------|
| [Inspect workspace status](how-to/inspect-workspace-status.md) | Read the current health of your workspace |
| [Run the playground](how-to/run-the-playground.md) | Spin up the local playground workspace for experimentation |
| [Custom ontology guide](how-to/custom-ontology-guide.md) | Write and register a custom capability in the ontology-driven manufacturing pipeline |
| [DX guide](how-to/dx-guide.md) | Improve day-to-day developer experience working in this workspace |
| [Git hooks](how-to/git-hooks.md) | Install and configure the repository's git hooks |
| [LSP editor integration](how-to/lsp-editor-integration.md) | Connect VS Code, Neovim, Helix, or Zed to `cargo-cicd-lsp` |
| [CI/CD pipeline integration](how-to/ci-cd-pipelines.md) | GitHub Actions, GitLab CI, Docker, and Makefile integration examples |
| [IDE integration](how-to/ide-integration.md) | Editor-specific setup: VS Code, JetBrains, Vim/Neovim, Emacs, Sublime |

### Contributing

| Document | Covers |
|----------|--------|
| [Contributing overview](contributing/README.md) | Entry point and reading order for the contributing guide set |
| [1. Development setup](how-to/contributing/01-development-setup.md) | Cloning, building, and running the workspace locally |
| [2. Pull request workflow](how-to/contributing/02-pull-request-workflow.md) | Branching, commit format, and review process |
| [3. Adding features](how-to/contributing/03-adding-features.md) | Patterns for adding a noun, verb, adapter, or policy |
| [4. Code style](how-to/contributing/04-code-style.md) | Formatting and idiom conventions |
| [5. Documentation standards](how-to/contributing/05-documentation-standards.md) | When and how to update CLAUDE.md and docs/ |
| [6. Release process](how-to/contributing/06-release-process.md) | Steps to cut and publish a release |
| [7. Known gotchas](how-to/contributing/07-known-gotchas.md) | Common pitfalls and how to avoid them |

---

## Reference

**Reference material is information-oriented.** It is the technical description
of the machinery — commands, flags, schemas, and formats — consulted while you
are working. Reference material does not explain why things exist; it accurately
and completely describes what they are. Use this section when you need the exact
syntax, schema, or contract for something you are already working with.

### Canonical reference: `cargo doc`

The authoritative reference for all public Rust types, functions, and traits is
generated by `cargo doc`. The files in `docs/reference/` supplement it for CLI
flags and TOML schemas that rustdoc cannot capture, but `cargo doc` is the
primary source of truth.

```sh
# Browse the full API reference locally:
cargo doc --no-deps --open

# Include the advanced modules:
cargo doc --no-deps --features advanced --open
```

Key entry points in `cargo doc`:
- `cargo_cicd::EngineState` — the aggregate root for all workspace dimensions
- `cargo_cicd::evidence` — OCEL 2.0 evidence emission and XES compatibility layer
- `cargo_cicd::ocel` — OCEL 2.0 types (OcelLog, OcelEvent, OcelObject, …)
- `cargo_cicd::advanced` — all 10 optional best-of-breed modules
- `cargo_cicd::CicdToml` — the `cicd.toml` schema

### CLI reference (start here)

| Document | Contents |
|----------|----------|
| [CLI Reference Index](reference/CLI_REFERENCE_INDEX.md) | Master navigation, document map, noun-verb grammar, learning path |
| [Complete Command Reference](reference/COMMANDS.md) | Every command, fully documented: flags, output, exit codes |
| [Troubleshooting Guide](reference/CLI_TROUBLESHOOTING.md) | Diagnostic and fix guide organized by problem category |

### Supplementary markdown reference

| Document | Contents |
|----------|----------|
| [cicd.toml schema](reference/cicd-toml.md) | Every field in `cicd.toml`: type, default, whether user-writable |
| [Configuration](reference/configuration.md) | Configuration keys recognized by cargo-cicd at startup |
| [Evidence format](reference/evidence-format.md) | OCEL/XES and JSONL event schemas, field definitions, and serialization rules |
| [XES format](reference/xes-format.md) | The full XES 2.0 attribute contract used for backwards compatibility |
| [Feature flags](reference/feature-flags.md) | `process-data`, `autonomic`, `wasm4pm`, `advanced`, `contrib` — what each enables and implies |
| [Standing schema](reference/standing-schema.md) | Schema for standing/fleet convergence state |
| [Glossary](reference/glossary.md) | Definitions of cargo-cicd-specific terms |
| [Code provenance](reference/code-provenance.md) | How generated vs. hand-written code is classified and marked |
| [Trustworthiness scoring](reference/trustworthiness-scoring.md) | How evidence and receipts are scored for trust |
| [Definition of Done (reference edition)](reference/definition-of-done.md) | Structured DoD checklist cross-referenced from CLAUDE.md and the invariants suite |

### Per-command reference

Each document covers one command: flags, arguments, output format, exit codes,
and evidence emitted.

| Command | Reference page |
|---------|----------------|
| `cargo cicd status show` | [reference/commands/status.md](reference/commands/status.md) |
| `cargo cicd target show` | [reference/commands/target-show.md](reference/commands/target-show.md) |
| `cargo cicd target prune` | [reference/commands/target-prune.md](reference/commands/target-prune.md) |
| `cargo cicd test changed` | [reference/commands/test-changed.md](reference/commands/test-changed.md) |
| `cargo cicd trybuild changed` | [reference/commands/trybuild-changed.md](reference/commands/trybuild-changed.md) |
| `cargo cicd git status` | [reference/commands/git-status.md](reference/commands/git-status.md) |
| `cargo cicd git close` | [reference/commands/git-close.md](reference/commands/git-close.md) |
| `cargo cicd publish run` | [reference/commands/publish-run.md](reference/commands/publish-run.md) |
| `cargo cicd workspace doctor` | [reference/commands/workspace-doctor.md](reference/commands/workspace-doctor.md) |
| `cargo cicd certification show` | [reference/commands/certification-show.md](reference/commands/certification-show.md) |
| `cargo cicd sbom generate` | [reference/commands/sbom-generate.md](reference/commands/sbom-generate.md) |

Noun-oriented alternates covering every verb for a noun in one page:
[git](reference/commands/git.md) · [publish](reference/commands/publish.md) ·
[status](reference/commands/status.md) · [target](reference/commands/target.md) ·
[test](reference/commands/test.md) · [trybuild](reference/commands/trybuild.md) ·
[workspace](reference/commands/workspace.md)

### LSP reference

| Document | Contents |
|----------|----------|
| [LSP overview](lsp/README.md) | What `cargo-cicd-lsp` provides and how to connect an editor |
| [Lifecycle](reference/lsp/lifecycle.md) | How the LSP server starts, handles requests, and shuts down |
| [Diagnostics](reference/lsp/diagnostics.md) | Which diagnostics the LSP server emits and their severity levels |
| [Conformance](reference/lsp/conformance.md) | Which LSP protocol capabilities are implemented and which are out of scope |

(Editor setup itself is a how-to: see [LSP editor integration](how-to/lsp-editor-integration.md).)

### Testing reference

| Document | Contents |
|----------|----------|
| [Invariants](reference/testing/invariants.md) | The non-negotiable invariants and what would happen if each were violated |
| [Capability test matrix](testing/CAPABILITY_TEST_MATRIX.md) | Which test covers which capability, and which capabilities lack test coverage |
| [Combinatorial maximalist test plan](testing/COMBINATORIAL_MAXIMALIST_TEST_PLAN.md) | The strategy for exhaustive noun-verb-flag combination testing |
| [wasm4pm evidence gate](reference/testing/wasm4pm-evidence-gate.md) | How the wasm4pm evidence gate tests are structured and what they validate |
| [wasm4pm evidence case matrix](reference/testing/wasm4pm-evidence-case-matrix.md) | All evidence test cases: happy path, mutation, and refusal |
| [wasm4pm refusal ledger](reference/testing/wasm4pm-refusal-ledger.md) | Catalogued refusal cases and the expected oracle response for each |
| [Negative fixture ledger](testing/NEGATIVE_FIXTURE_LEDGER.md) | Fixture workspaces used by negative-path tests, and what each exercises |
| [wasm4pm oracle discovery](reference/testing/wasm4pm-oracle-discovery.md) | How the test harness locates the `wpm` binary at runtime |

### wasm4pm reference

| Document | Contents |
|----------|----------|
| [Allowed surfaces](reference/wasm4pm/allowed-surfaces.md) | Which cargo-cicd surfaces may call the wasm4pm oracle |
| [Excluded surfaces](reference/wasm4pm/excluded-surfaces.md) | Surfaces explicitly prohibited from calling the oracle, and why |
| [Capability inventory](reference/wasm4pm/capability-inventory.md) | Full inventory of wasm4pm capabilities used by cargo-cicd |
| [Capability map](wasm4pm/WASM4PM_CAPABILITY_MAP.md) | cargo-cicd's own evidence emission flow, receipt schema, and wpm verdict semantics |
| [Full capability map](reference/wasm4pm/full-capability-map.md) | Crate-by-crate leverage classification of the upstream wasm4pm ecosystem |

---

## Explanation

**Explanation is understanding-oriented.** It discusses context, background, and
design decisions. Explanation answers the question "why?" rather than "what?" or
"how?". Use this section when you want to understand the rationale behind a
design choice, not when you need to accomplish a task.

| Document | What it explains |
|----------|-----------------|
| [Why local-first CI/CD](explanation/why-local-first-cicd.md) | Why cargo-cicd runs on your machine before you push, rather than on a remote server after |
| [Why cicd.toml](explanation/why-cicd-toml.md) | Why workspace state is persisted in a TOML file rather than kept in memory or a database |
| [Evidence emission](explanation/evidence-emission.md) | How and why cargo-cicd emits process evidence, and the role of the wasm4pm oracle |
| [Why wasm4pm evidence validation](explanation/why-wasm4pm-evidence-validation.md) | The reasoning behind external adjudication of process evidence rather than self-certification |
| [Why changed test planning](explanation/why-changed-test-planning.md) | Why `test changed` runs only a subset of tests, and why this is safe rather than risky |
| [Autonomic policies](explanation/autonomic-policies.md) | What the autonomic policy layer is and how suggest mode works |
| [Combinatorial maximalism rationale](explanation/combinatorial-maximalism.md) | The philosophy behind exhaustive combinatorial testing of noun-verb combinations |
| [Fleet standing convergence](explanation/fleet-standing-convergence.md) | How standing state converges across a fleet of workspaces |

### Policy

| Document | What it explains |
|----------|-------------------|
| [Claim rules](explanation/policy/claim-rules.md) | What cargo-cicd is and is not allowed to claim about a workspace's state |
| [Claude Code standing policy](explanation/policy/claude-code-standing-policy.md) | How Claude Code agents are expected to interact with standing state |
| [External operator side effects](explanation/policy/external-operator-side-effects.md) | Boundaries on side effects an external operator may trigger |
| [No dashboard fiction](explanation/policy/no-dashboard-fiction.md) | Why cargo-cicd does not present dashboards or metrics that aren't backed by real data |

---

## Vision (forward-looking, not shipped)

**`docs/vision/` is explicitly out-of-band from the four Diátaxis quadrants.**
Everything here describes a proposal, roadmap, thesis, or compliance mapping for
a **future or aspirational** state of cargo-cicd — not current, shipped behavior.
Do not treat anything in this directory as a description of what the CLI does
today; cross-check against `--help` output and `src/nouns/` before relying on it.

| Document | Nature |
|----------|--------|
| [ROADMAP-2030](vision/ROADMAP-2030.md) | Long-range roadmap |
| [Vision 2030 Index](vision/VISION-2030-INDEX.md) | Index into the Vision 2030 document set |
| [Vision 2030 PRD](vision/vision-2030-prd.md) | Product requirements for the Vision 2030 initiative |
| [PRD: Vision 2030](vision/prd-vision-2030.md) | Companion PRD document |
| [Thesis (top-level)](vision/thesis.md) | Entry point for the PhD-thesis-style writeup |
| [ERRC review](vision/ERRC_REVIEW.md) | Eliminate-Reduce-Raise-Create strategic review, including the unwired-legacy-noun pattern |
| [TOGAF ADM coverage](vision/TOGAF-ADM-COVERAGE.md) | Mapping to TOGAF Architecture Development Method phases |
| [Cargo Evidence RFC](vision/CARGO-EVIDENCE-RFC.md) | RFC proposal for evidence support upstream in Cargo |
| [Certification body integration](vision/CERT-BODY-INTEGRATION.md) | Proposal for third-party certification body integration |
| [Distributed oracle design](vision/distributed-oracle-design.md) | Design for a multi-oracle, threshold-signed wpm deployment |
| [MCP integration strategy](vision/MCP_INTEGRATION_STRATEGY.md) | Proposal for Model Context Protocol integration |
| [Phase 1 plan](vision/PHASE-1-PLAN.md) / [Phase 2 design](vision/PHASE-2-DESIGN.md) / [Phase 3 design](vision/PHASE-3-DESIGN.md) | Multi-phase roadmap design documents |
| [Process mining architecture](vision/process-mining-architecture.md) | Future process-mining dashboard and analytics architecture |
| [wasm4pm contrib extraction roadmap](vision/wasm4pm-contrib-extraction-roadmap.md) | Plan for extracting cargo-cicd contributions upstream into wasm4pm |
| [wasm4pm integration recommendation](vision/wasm4pm-integration-recommendation.md) | Recommendation that preceded ADR-010 |
| [wasm4pm leverage matrix](vision/wasm4pm-leverage-matrix.md) | Which oracle capabilities provide the highest value per integration cost |

### Compliance mappings (aspirational)

| Document | Standard |
|----------|----------|
| [IEC 61508 mapping](vision/compliance/IEC-61508-MAPPING.md) | Functional safety |
| [ISO 26262 mapping](vision/compliance/ISO-26262-MAPPING.md) | Automotive functional safety |
| [SOC 2 mapping](vision/compliance/SOC2-MAPPING.md) | Trust services criteria |

### Thesis set

`docs/vision/thesis/` contains a multi-chapter PhD-thesis-style treatment of
cargo-cicd, including a rendered PDF, a Rust-ecosystem survey, and a
Vision-2030 repository survey. Start at
[docs/vision/thesis/README.md](vision/thesis/README.md).

---

## Architecture decisions

Architecture Decision Records (ADRs) document the significant technical choices
made in cargo-cicd and the reasoning behind them. Each ADR is immutable once
accepted — it records what was decided and why, not what is currently preferred.
ADRs are not how-to guides or explanations; they are historical records of
decisions that shaped the system. **Because ADRs are immutable, some may
reference filenames (e.g. `SOLUTION_ARCHITECTURE.md`) that have since been
removed from `docs/`; treat such references as historical, not navigational.**

| ADR | Decision |
|-----|----------|
| [ADR-001](adr/ADR-001-three-crate-separation.md) | CLI, integration, and domain logic separation with enforced import rules |
| [ADR-002](adr/ADR-002-evidence-gate-invariants.md) | Non-negotiable evidence gate invariants that all process evidence must satisfy |
| [ADR-003](adr/ADR-003-receipt-doctor-primary-gate.md) | `wpm receipt doctor` is the primary release gate, not `wpm audit` |
| [ADR-004](adr/ADR-004-lsp-observer-not-actor.md) | The LSP integration is a read-only observer; it never mutates workspace state |
| [ADR-005](adr/ADR-005-keyed-subtraction-lifecycle.md) | Evidence events use a keyed subtraction lifecycle rather than append-only accumulation |
| [ADR-006](adr/ADR-006-trailing-var-arg-pattern.md) | Trailing var-arg pattern for forwarding unknown flags to underlying cargo commands |
| [ADR-007](adr/ADR-007-no-silent-fallback-on-verdict-keys.md) | Verdict key mismatches must panic rather than fall back silently to a default |
| [ADR-008](adr/ADR-008-pipeline-vs-ambient-trace.md) | Pipeline runs and ambient single-command runs produce distinct trace classes in XES |
| [ADR-009](adr/ADR-009-forbidden-terms-public-boundary.md) | Certain internal terms are forbidden from all public-facing output and enforced by an invariant test |
| [ADR-010](adr/ADR-010-publish-gate-adjudicated-receipt.md) | Publish requires an adjudicated wasm4pm receipt, not just a clean status check |
| [ADR-011](adr/ADR-011-xes-v2-format.md) | XES v2 format adoption |
| [ADR-012](adr/ADR-012-wasm4pm-oracle-architecture.md) | wasm4pm oracle architecture |
| [ADR-013](adr/ADR-013-oracle-public-key-embedding.md) | Oracle public key embedding |
| [ADR-014](adr/ADR-014-cargo-toml-evidence-section.md) | `Cargo.toml` evidence section |
| [ADR-015](adr/ADR-015-jsonl-companion-format.md) | JSONL companion format |
| [ADR-016](adr/ADR-016-distributed-oracle-consensus.md) | Distributed oracle consensus |
| [ADR-017](adr/ADR-017-code-provenance-classification.md) | Code provenance classification |
| [ADR-018](adr/ADR-018-ontology-driven-manufacturing.md) | Ontology-driven manufacturing |
| [ADR-019](adr/ADR-019-feature-flag-strategy.md) | Feature flag strategy |
| [ADR-020](adr/ADR-020-phase2-pluggable-process-models.md) | Phase 2 pluggable process models |

---

## Supplementary documentation

These documents exist outside the four Diátaxis quadrants and outside
`docs/vision/`, but are part of the cargo-cicd documentation corpus.

| Document | Contents |
|----------|----------|
| [Workflow checklists](CHECKLIST.md) | Step-by-step checklists for recurring workflows |
| [Definition of Done (top-level)](DEFINITION_OF_DONE.md) | Per-category "done" criteria; see also [reference/definition-of-done.md](reference/definition-of-done.md) |
| [MCP plugin guide](MCP_PLUGIN_GUIDE.md) | The `cargo-advanced-tools` MCP plugin definition used by Claude Code |
| [Claude Code hooks settings](claude-code-hooks-settings.md) | Hook configuration for Claude Code sessions in this repo |
| `claude-code-example-configs/` | Example Claude Code permission/setup JSON files |
| `ontology-registry-schema.ttl` | TTL schema for the ontology registry |

### Deferred / speculative

| Document | Contents |
|----------|----------|
| [Deferred wasm4pm contrib extraction](deferred/WASM4PM_CONTRIB_EXTRACTION.md) | Specific deferred candidates and extraction criteria (distinct from the roadmap doc in `docs/vision/`) |
| `design/MCP_*.md` | A 4-part never-shipped MCP design suite (strategy, schema reference, implementation guide, summary); start at [design/MCP_STRATEGY_SUMMARY.md](design/MCP_STRATEGY_SUMMARY.md) |
| `star-toml-refactor/` | ARD, PRD, and refactor spec for the (now-shipped) extraction of `star-toml` into its own published crate |

### Release history

| Document | Contents |
|----------|----------|
| [v26.6.2 release notes](release/v26.6.2.md) | Point-in-time release record |
| [Crates.io release checklist](release/CRATES_IO_RELEASE_CHECKLIST.md) | Steps for publishing to crates.io |
| [Package contents audit](release/PACKAGE_CONTENTS_AUDIT.md) | What ships in the published crate |
| [Public boundary audit](release/PUBLIC_BOUNDARY_AUDIT.md) | Forbidden-term and public-surface audit record |
| [Runtime evidence reconciliation](release/RUNTIME_EVIDENCE_RECONCILIATION.md) | Reconciling emitted evidence against runtime behavior |

---

## Quick navigation by goal

Use the table below when you know what you are trying to accomplish and want
to go directly to the right document.

| I want to... | Go here |
|--------------|---------|
| Install cargo-cicd for the first time | [Quick Start](tutorials/quick-start.md) |
| See what commands are available | [Command reference](reference/COMMANDS.md) |
| Check if my workspace is ready to push | [Inspect workspace status](how-to/inspect-workspace-status.md) |
| Understand `git close` behavior | [git-close reference](reference/commands/git-close.md) |
| Look up a specific flag or exit code | [Per-command reference](reference/commands/) |
| Understand what cicd.toml fields mean | [cicd.toml schema](reference/cicd-toml.md) |
| Enable optional features (autonomic, wasm4pm) | [Feature flags](reference/feature-flags.md) |
| Understand XES evidence format | [Evidence format](reference/evidence-format.md) |
| Understand why changed tests are safe | [Why changed test planning](explanation/why-changed-test-planning.md) |
| Understand why cicd.toml exists | [Why cicd.toml](explanation/why-cicd-toml.md) |
| Understand how evidence adjudication works | [Why wasm4pm evidence validation](explanation/why-wasm4pm-evidence-validation.md) |
| Understand the overall design philosophy | [Why local-first CI/CD](explanation/why-local-first-cicd.md) |
| Find out why a technical decision was made | [Architecture decisions](adr/) |
| Connect an IDE to the cargo-cicd language server | [LSP editor integration](how-to/lsp-editor-integration.md) |
| Debug a failing evidence gate test | [wasm4pm evidence gate](reference/testing/wasm4pm-evidence-gate.md) |
| See the full test coverage picture | [Capability test matrix](testing/CAPABILITY_TEST_MATRIX.md) |
| See cargo-cicd's roadmap or long-range plans | [Vision documentation](#vision-forward-looking-not-shipped) |
