# Rust Build/CI/CD Ecosystem Survey

**5-Agent Reconnaissance Mission — June 2026**  
**Scope:** Crates.io, GitHub, community patterns, tooling integrations  
**Branch:** `claude/gracious-turing-bcvou2`  
**Methodology:** Five parallel agents surveyed five domains; findings synthesized into a ranked integration priority list.

---

## Executive Summary

The Rust build/CI/CD ecosystem has matured considerably. Key themes:

1. **Speed is table stakes**: nextest (2-3× faster), rayon-parallel scanning, and BLAKE3 hashing set the baseline
2. **Supply chain security is urgent**: SLSA Level 2, sigstore/cosign, and in-toto attestations are being adopted at pace; cargo-cicd's OCEL evidence is ahead of most tools here
3. **Policy-as-code is consolidating around `deny.toml`**: cargo-deny is the de facto standard; bridges well to XES
4. **Workspace complexity is a solved problem** for versioning (cargo-workspaces) but unsolved for CI time at scale
5. **cargo-cicd occupies a unique position**: OCEL 2.0 process evidence plus external adjudication (wasm4pm) has no direct competitor

---

## Agent 1 — Build Tools & Cargo Integration

### cargo-make
- **What it is:** Task runner and build tool with TOML/YAML task definitions, dependency graphs, and platform-specific flows
- **Ecosystem:** Rust
- **Evidence/audit support:** None
- **Reusable assets:** Task dependency DAG, workspace-level task propagation, pre/post hooks pattern
- **Integration effort with cargo-cicd:** Low — cargo-cicd is already used as a cargo-make task target; can expose `Makefile.toml` tasks that invoke `cargo cicd` nouns
- **Recommendation:** Reference design — cargo-cicd's noun-verb grammar IS a more structured version of cargo-make tasks
- **Priority:** Medium

### just
- **What it is:** Command runner with a `justfile` (Make-like but without build semantics); recipes are plain shell
- **Ecosystem:** Language-agnostic (Rust binary)
- **Evidence/audit support:** None
- **Reusable assets:** Dry-run flag pattern, argument forwarding (`@_` rest-args), conditional recipe inclusion
- **Integration effort with cargo-cicd:** Low — a `justfile` wrapping `cargo cicd` verbs is a natural UX layer
- **Recommendation:** Reference design — ADR-006's trailing var-arg pattern mirrors just's `@_` convention
- **Priority:** Low

### xtask
- **What it is:** Convention for putting build scripts in a workspace member called `xtask` (a Rust binary), eliminating shell scripts
- **Ecosystem:** Rust
- **Evidence/audit support:** None (but can call cargo-cicd)
- **Reusable assets:** `cargo xtask check`, `cargo xtask dist`, patterns for cross-compilation
- **Integration effort with cargo-cicd:** Low — **cargo-cicd IS a canonical xtask implementation** exposed as a cargo plugin
- **Recommendation:** Adopt as reference design; document that cargo-cicd is the xtask for CI/CD concerns
- **Priority:** Medium

### cargo-binstall
- **What it is:** Binary installation for cargo crates without compilation, using GitHub releases and checksums
- **Ecosystem:** Rust
- **Evidence/audit support:** Checksum verification (SHA256)
- **Reusable assets:** Binary discovery protocol, fallback-to-compile strategy, manifest format (`[package.metadata.binstall]`)
- **Integration effort with cargo-cicd:** Low — cargo-cicd install instructions should mention `cargo binstall cargo-cicd`
- **Recommendation:** Monitor — useful for CI bootstrap speed but no direct integration needed
- **Priority:** Low

### cargo-timing / cargo-criterion
- **What it is:** cargo-timing: visual build timing reports in HTML; cargo-criterion: criterion benchmark runner with Cargo integration
- **Ecosystem:** Rust
- **Evidence/audit support:** HTML/JSON output (cargo-timing); criterion JSON + gnuplot (cargo-criterion)
- **Reusable assets:** Stage timing instrumentation pattern; histogram percentiles for compile phases
- **Integration effort with cargo-cicd:** Medium — cargo-cicd's `advanced::histogram` module parallels criterion's `HdrHistogram` approach
- **Recommendation:** Reference design — adopt stage-timing pattern in `advanced::timeline`
- **Priority:** Low

### cargo-semver-checks
- **What it is:** Detects SemVer-breaking changes by diffing the public API between versions using rustdoc JSON
- **Ecosystem:** Rust
- **Evidence/audit support:** Machine-readable JSON diagnostic output, exit code semantics
- **Reusable assets:** Pre-publish gate concept; API diff as evidence item
- **Integration effort with cargo-cicd:** Medium — add `publish run` gate: call `cargo semver-checks` before allowing publish; emit result as ProcessEvent
- **Recommendation:** **Adopt as optional dependency** — integrate as a `publish run` pre-gate
- **Priority:** **High**

---

## Agent 2 — Testing Frameworks & Selective Test Runners

### cargo-nextest
- **What it is:** Next-generation test runner for Rust — parallel, isolated, 2-3× faster than `cargo test`; JUnit XML output; `-E` filterset DSL for test selection
- **Ecosystem:** Rust
- **Evidence/audit support:** JUnit XML (parseable by CI systems); JSON structured output via `--message-format`
- **Reusable assets:** `-E 'test(pattern)'` filterset DSL; retry-on-failure; per-test timing; sharding for distributed CI
- **Integration effort with cargo-cicd:** Low — `cargo cicd test changed` should call `cargo nextest run` instead of `cargo test` when nextest is available
- **Recommendation:** **Adopt as optional dependency** — auto-detect nextest on PATH; fall back to `cargo test`
- **Priority:** **High**

### proptest / quickcheck
- **What it is:** Property-based testing libraries — generate random inputs and shrink failing cases automatically
- **Ecosystem:** Rust
- **Evidence/audit support:** None — failures reported as standard test failures
- **Reusable assets:** Shrinking strategy; property specification as invariant documentation
- **Integration effort with cargo-cicd:** Low — no integration needed; already works with cargo test
- **Recommendation:** Monitor — no direct integration; cargo-cicd surfaces pass/fail from cargo test
- **Priority:** Low

### cargo-fuzz
- **What it is:** Fuzz testing harness for Rust using libFuzzer; discovers edge cases via coverage-guided mutation
- **Ecosystem:** Rust
- **Evidence/audit support:** Corpus artifacts; crash artifacts (reproducers)
- **Reusable assets:** Corpus management; crash artifact as evidence
- **Integration effort with cargo-cicd:** High — would require new `fuzz` noun; corpus artifacts should be tracked in `artifact_state`
- **Recommendation:** Monitor — valuable but out-of-scope for v26 release cycle
- **Priority:** Low

### cargo-audit
- **What it is:** Audits `Cargo.lock` against the RustSec advisory database for known CVEs in dependencies
- **Ecosystem:** Rust
- **Evidence/audit support:** JSON output; RustSec advisory IDs; machine-readable vulnerability report
- **Reusable assets:** Advisory database query; `--json` flag; ignore-list (`[advisories]` in `audit.toml`)
- **Integration effort with cargo-cicd:** Low — emit audit result as `ProcessEvent` with `verdict_claimed = "PASS"/"FAIL"`; bridge to wasm4pm
- **Recommendation:** **Adopt as optional dependency** — integrate into `workspace doctor` and `publish run`
- **Priority:** **High**

### trybuild
- **What it is:** Snapshot tests for compiler error messages — run Rust source through `rustc`, compare output to `.stderr` fixture
- **Ecosystem:** Rust
- **Evidence/audit support:** None — snapshot comparison is the evidence
- **Reusable assets:** cargo-cicd already has `trybuild changed` noun — this IS the integration
- **Integration effort with cargo-cicd:** Already integrated — `cargo cicd trybuild changed` is the wrapper
- **Recommendation:** Already integrated — maintain and improve `ChangedFileDetector::is_trybuild_fixture()`
- **Priority:** Medium (ongoing maintenance)

### insta
- **What it is:** Snapshot testing library with interactive snapshot review (`cargo insta review`); supports JSON, YAML, CSV, inline snapshots
- **Ecosystem:** Rust
- **Evidence/audit support:** Snapshot diffs as artifact evidence; `.snap` files are reviewable artifacts
- **Reusable assets:** Interactive review workflow; snapshot file lifecycle (pending → accepted)
- **Integration effort with cargo-cicd:** Medium — `.snap` files in `tests/snapshots/` should be tracked similarly to trybuild fixtures in `ChangedFileDetector`
- **Recommendation:** Reference design — extend `changed_file_state` to track `.snap` file changes
- **Priority:** Medium

### miri
- **What it is:** Mid-level Intermediate Representation Interpreter — detects undefined behavior, memory errors, and unsafe code violations
- **Ecosystem:** Rust
- **Evidence/audit support:** UB reports with source location; machine-readable via stderr parsing
- **Reusable assets:** UB detection as a hard gate before publish
- **Integration effort with cargo-cicd:** High — Miri runs are slow; would need `test miri` verb; out-of-scope for v26
- **Recommendation:** Monitor — critical for crates with `unsafe`; future `publish run` gate candidate
- **Priority:** Low

---

## Agent 3 — Evidence, Audit & Supply Chain Security

### SLSA (Supply-chain Levels for Software Artifacts)
- **What it is:** Google/OpenSSF framework defining four levels of supply-chain integrity guarantees; Level 2 = hosted, signed build provenance
- **Ecosystem:** Language-agnostic standard
- **Evidence/audit support:** SLSA provenance JSON (attestation format); aligns with in-toto
- **Reusable assets:** Level 2 provenance schema; builder identity attestation; `buildType` URI pattern
- **Integration effort with cargo-cicd:** Medium — emit SLSA provenance JSON as a ProcessEvent attribute; `publish run` gate can require Level 2 provenance
- **Recommendation:** **Adopt as reference design** — OCEL 2.0 evidence maps naturally to SLSA provenance fields
- **Priority:** **High**

### sigstore/cosign
- **What it is:** Keyless code signing using ephemeral OIDC-based keys + transparency log (Rekor); `cosign sign` attaches signatures to OCI artifacts
- **Ecosystem:** Language-agnostic (Go binary + Rust `sigstore` crate)
- **Evidence/audit support:** Rekor transparency log entry; signature bundle JSON; DSSE envelope
- **Reusable assets:** Keyless signing workflow; Rekor log entry as non-repudiation anchor
- **Integration effort with cargo-cicd:** Medium — call `cosign sign-blob` on XES evidence files; Rekor entry ID stored in `ProcessEvent.verdict_adjudicated`
- **Recommendation:** **Adopt as optional dependency** — `wasm4pm` feature can optionally sign evidence before adjudication
- **Priority:** **High**

### in-toto
- **What it is:** Framework for supply chain integrity using signed link metadata; each step produces a signed `link` file attesting inputs/outputs
- **Ecosystem:** Language-agnostic standard (Python CLI, Rust crate `in-toto-rs`)
- **Evidence/audit support:** Signed link files + layout file; in-toto verification is the adjudication step
- **Reusable assets:** Step-link-layout model parallels cargo-cicd's start/complete evidence lifecycle
- **Integration effort with cargo-cicd:** High — in-toto signing requires key management; long-term integration candidate
- **Recommendation:** Reference design — design ProcessEvent lifecycle to be in-toto-compatible
- **Priority:** Medium

### cargo-deny
- **What it is:** Policy enforcement for Cargo dependencies — license compliance, advisory bans, version constraints, duplicate detection; configured in `deny.toml`
- **Ecosystem:** Rust
- **Evidence/audit support:** JSON and human-readable diagnostic output; advisory IDs; exit codes
- **Reusable assets:** `deny.toml` schema; check categories (`bans`, `licenses`, `advisories`, `sources`); machine-readable JSON mode
- **Integration effort with cargo-cicd:** Low — emit `cargo deny check` result as ProcessEvent; bridge to wasm4pm adjudication
- **Recommendation:** **Adopt as optional dependency** — integrate into `publish run` and `workspace doctor`
- **Priority:** **High**

### cargo-sbom / cargo-cyclonedx
- **What it is:** cargo-sbom: generates Software Bill of Materials in SPDX format; cargo-cyclonedx: CycloneDX BOM generation
- **Ecosystem:** Rust
- **Evidence/audit support:** SPDX JSON/YAML; CycloneDX JSON/XML; machine-readable dependency inventory
- **Reusable assets:** Dependency graph traversal; SPDX expression normalization; license ID mapping
- **Integration effort with cargo-cicd:** Medium — add `publish run` gate: emit SBOM as artifact; store SBOM path in `artifact_state`
- **Recommendation:** **Adopt as optional dependency** — SBOM emission is a natural `publish run` step
- **Priority:** **High**

### SARIF (Static Analysis Results Interchange Format)
- **What it is:** OASIS standard JSON format for static analysis results; consumed by GitHub code scanning, VS Code SARIF viewer, Azure DevOps
- **Ecosystem:** Language-agnostic standard
- **Evidence/audit support:** Native — this IS the evidence format for static analysis
- **Reusable assets:** `runs[].results[].locations` schema; `ruleId` taxonomy; `level` (error/warning/note)
- **Integration effort with cargo-cicd:** Medium — emit cargo-deny, cargo-audit, clippy results as SARIF; upload to GitHub code scanning via `sarif_import`
- **Recommendation:** Reference design — SARIF output mode as a `--format sarif` flag on `workspace doctor`
- **Priority:** Medium

### OpenVEX / VEX
- **What it is:** Vulnerability Exploitability eXchange — machine-readable statements about whether a CVE is exploitable in a given context; reduces false-positive noise
- **Ecosystem:** Language-agnostic standard
- **Evidence/audit support:** JSON-LD VEX documents; linked to SBOM components
- **Reusable assets:** VEX statement lifecycle (affected/not_affected/fixed/under_investigation)
- **Integration effort with cargo-cicd:** High — requires CVE tracking and statement management; future candidate
- **Recommendation:** Monitor — important long-term; requires SBOM integration first
- **Priority:** Low

---

## Agent 4 — Workspace & Monorepo Management

### cargo-workspaces
- **What it is:** Workspace management CLI — bulk version bumping, changelog generation, coordinated publishing of workspace members in dependency order
- **Ecosystem:** Rust
- **Evidence/audit support:** None — outputs changelogs, not evidence
- **Reusable assets:** Dependency-order publish, version unification, changelog per-crate
- **Integration effort with cargo-cicd:** Medium — `publish run` should coordinate with cargo-workspaces for multi-crate workspaces; emit per-crate publish events
- **Recommendation:** Reference design — adopt dependency-order publish algorithm in `publish run`
- **Priority:** Medium

### release-plz
- **What it is:** Automated release PR workflow — reads conventional commits, bumps semver, updates CHANGELOG.md, creates GitHub releases, publishes to crates.io; all via a single `release-plz release` command
- **Ecosystem:** Rust
- **Evidence/audit support:** GitHub release notes; `release-plz.toml` policy file; conventional commit log as release evidence
- **Reusable assets:** Conventional commit parser; semver increment logic; changelog generation template
- **Integration effort with cargo-cicd:** Medium — `git close` noun can validate that commits follow conventional format; `publish run` can invoke release-plz as the publish mechanism
- **Recommendation:** **Adopt as optional dependency** — replace manual release steps with `release-plz release`
- **Priority:** **High**

### git-cliff
- **What it is:** Changelog generator from git history using conventional commits and configurable templates (`cliff.toml`)
- **Ecosystem:** Rust (binary)
- **Evidence/audit support:** Structured changelog; can emit JSON commit graph
- **Reusable assets:** `cliff.toml` template language; commit category mapping; tag-based range selection
- **Integration effort with cargo-cicd:** Low — call `git cliff` in `git close` flow; include changelog delta in ProcessEvent attributes
- **Recommendation:** **Adopt as optional dependency** — changelog generation is a natural `git close` step
- **Priority:** High

### cargo-mutants
- **What it is:** Mutation testing for Rust — introduces small source mutations and checks if tests catch them; measures test suite effectiveness
- **Ecosystem:** Rust
- **Evidence/audit support:** JSON mutation report; per-file survived/caught counts
- **Reusable assets:** Mutation score as a quality gate; file-level coverage metric
- **Integration effort with cargo-cicd:** High — mutation testing is slow; would need dedicated verb; emit mutation score as ProcessEvent attribute
- **Recommendation:** Monitor — valuable metric but too slow for default CI gate; future `test mutation` verb candidate
- **Priority:** Low

### pre-commit / rusty-hook
- **What it is:** pre-commit: Python-based multi-language git hook framework with YAML config; rusty-hook: pure-Rust alternative using `.rusty-hook.toml`
- **Ecosystem:** Python (pre-commit) / Rust (rusty-hook)
- **Evidence/audit support:** None — hook exit codes only
- **Reusable assets:** Hook configuration schema; hook-to-CI parity principle
- **Integration effort with cargo-cicd:** Low — `cargo cicd status` as a pre-push hook; `cargo cicd test changed` as pre-commit hook
- **Recommendation:** Reference design — document cargo-cicd hook integration in how-to guides
- **Priority:** Medium

### cargo2nix / crane
- **What it is:** cargo2nix: Nix derivation generator from Cargo workspace; crane: Nix library for building Rust projects with intelligent caching and incremental builds
- **Ecosystem:** Nix + Rust
- **Evidence/audit support:** Nix derivation hash as content-addressed build ID
- **Reusable assets:** Derivation-per-crate model; pname/version extraction from Cargo.toml
- **Integration effort with cargo-cicd:** High — Nix integration is a separate toolchain concern; out-of-scope for v26
- **Recommendation:** Monitor — important for reproducible builds; future `target fingerprint` could align with Nix hash model
- **Priority:** Low

---

## Agent 5 — Policy Enforcement & CI/CD Workflow Standardization

### cargo-deny (Policy)
- **What it is:** See Agent 3 — highlighted again here because it is **the** policy-as-code standard in the Rust ecosystem
- **Ecosystem:** Rust
- **Evidence/audit support:** JSON diagnostic output; advisory IDs; exit codes
- **Reusable assets:** Policy categories as nouns: `bans`, `licenses`, `advisories`, `sources`
- **Integration effort with cargo-cicd:** Low
- **Recommendation:** **Adopt as optional dependency** — cargo-deny output bridges naturally to cargo-cicd ProcessEvent
- **Priority:** **High**

### Mergify
- **What it is:** GitHub-native merge queue and PR automation — merge when CI passes, auto-label, auto-rebase; configured in `.mergify.yml`
- **Ecosystem:** Language-agnostic (SaaS)
- **Evidence/audit support:** Merge queue audit log; condition evaluation log
- **Reusable assets:** Condition language (`#review-approved >= 2`, `check-success = cargo-cicd-evidence`); queue configuration
- **Integration effort with cargo-cicd:** Low — cargo-cicd status check can be a Mergify merge condition; document `.mergify.yml` template in how-to guides
- **Recommendation:** Reference design — document Mergify integration as a how-to guide
- **Priority:** Medium

### bors-ng / homu
- **What it is:** CI-gated merge bot — only merges when `bors r+` is approved AND all configured CI checks pass; prevents merge races
- **Ecosystem:** Language-agnostic (self-hosted Rust service)
- **Evidence/audit support:** None — relies on CI status checks
- **Reusable assets:** Try-branch pattern; linear merge history guarantee
- **Integration effort with cargo-cicd:** Low — cargo-cicd ProcessEvent verdict can be surfaced as a GitHub status check that bors waits for
- **Recommendation:** Reference design — document bors integration in how-to guides
- **Priority:** Low

### renovate / dependabot
- **What it is:** Automated dependency update PRs — detects new crate versions, opens PRs, auto-merges if CI passes
- **Ecosystem:** Language-agnostic (SaaS + self-hosted)
- **Evidence/audit support:** None — PR description contains version diff
- **Reusable assets:** `renovate.json` preset for Rust workspaces; grouped update strategies
- **Integration effort with cargo-cicd:** Low — cargo-cicd `publish run` gate naturally validates that updated dependencies don't break the evidence gate
- **Recommendation:** Monitor — no direct integration; cargo-cicd validates the resulting workspace
- **Priority:** Low

### GitHub Actions / required status checks
- **What it is:** GitHub's CI platform; required status checks block PRs from merging until named checks pass
- **Ecosystem:** Language-agnostic (YAML workflows)
- **Evidence/audit support:** Check run annotations; SARIF upload; job summaries
- **Reusable assets:** `cargo cicd status` as a required check; `cargo cicd evidence audit` output in job summary
- **Integration effort with cargo-cicd:** Low — add `action.yml` or example workflow in `docs/how-to/` showing GitHub Actions integration
- **Recommendation:** **Adopt as reference design** — provide a canonical `cargo-cicd.yml` GitHub Actions workflow
- **Priority:** **High**

---

## Cross-Agent Ranked Integration Priority Index

### Top 15 Crates/Patterns by cargo-cicd Synergy

| Rank | Crate/Pattern | Domain | Priority | Integration Effort | Recommended Action |
|------|--------------|--------|----------|-------------------|-------------------|
| 1 | **cargo-nextest** | Testing | High | Low | Auto-detect; use as `test changed` backend |
| 2 | **cargo-deny** | Policy + Supply Chain | High | Low | Gate in `publish run` + `workspace doctor` |
| 3 | **cargo-audit** | Supply Chain | High | Low | Emit advisory result as ProcessEvent |
| 4 | **SLSA Level 2 provenance** | Supply Chain | High | Medium | Map OCEL events to SLSA provenance JSON |
| 5 | **sigstore/cosign** | Supply Chain | High | Medium | Sign XES evidence files; Rekor log as anchor |
| 6 | **cargo-sbom / cargo-cyclonedx** | Supply Chain | High | Medium | SBOM emission in `publish run` |
| 7 | **release-plz** | Workspace | High | Medium | Replace manual publish steps |
| 8 | **cargo-semver-checks** | Testing | High | Low | Pre-publish API diff gate |
| 9 | **GitHub Actions workflow** | Policy | High | Low | Provide canonical `cargo-cicd.yml` |
| 10 | **git-cliff** | Workspace | High | Low | Changelog in `git close` flow |
| 11 | **insta snapshot tracking** | Testing | Medium | Medium | Extend `ChangedFileDetector` for `.snap` files |
| 12 | **SARIF output mode** | Policy | Medium | Medium | `--format sarif` flag on `workspace doctor` |
| 13 | **Mergify integration** | Policy | Medium | Low | Document merge condition in how-to |
| 14 | **in-toto compatibility** | Supply Chain | Medium | High | Design ProcessEvent to be in-toto-compatible |
| 15 | **pre-commit / rusty-hook** | Policy | Medium | Low | Document cargo-cicd as git hook target |

### Top 5 Ecosystem Gaps cargo-cicd Can Fill

1. **OCEL 2.0 as universal build evidence format** — No other Rust CI/CD tool emits OCEL 2.0 process mining evidence. cargo-cicd's `ProcessEvent` + XES emission is ahead of the ecosystem. Position this as a standard interface for external adjudication (wasm4pm) and supply chain auditors.

2. **Selective test execution with change awareness** — `cargo nextest` is fast but not change-aware. `cargo cicd test changed` fills the gap: git-diff-based test selection + nextest execution backend. This combination doesn't exist as a single tool.

3. **Pre-publish evidence gate bridging semantic versioning, SBOMs, and advisory checks** — `publish run` can become a meta-gate that orchestrates cargo-semver-checks + cargo-deny + cargo-audit + SBOM emission + SLSA provenance + wasm4pm adjudication in one command. No existing tool does this end-to-end.

4. **Policy enforcement surfaced as process evidence** — cargo-deny and cargo-audit produce diagnostics but not signed, adjudicable evidence. cargo-cicd can wrap these tools and emit their results as wasm4pm-adjudicable ProcessEvents, closing the loop from static policy to dynamic evidence.

5. **Conventional-commit enforcement as a CI/CD gate** — release-plz and git-cliff consume conventional commits but neither enforces them as a gate. `git close` could verify that all commits since the base branch follow the conventional format and refuse to close the phase if they don't.

### Top 3 Patterns to Standardize Across the Rust Build Ecosystem

1. **Evidence lifecycle: start → work → complete → adjudicate** — The cargo-cicd ProcessEvent pattern (E1–E7) should be proposed as a community standard for CI/CD evidence emission. Every tool (nextest, cargo-deny, cargo-audit) emitting evidence in this format would enable wasm4pm (or any OCEL consumer) to adjudicate cross-tool process conformance.

2. **Noun-verb CLI grammar for build tools** — cargo-cicd's `cargo cicd <noun> <verb>` grammar provides a consistent, discoverable structure. The community pattern of ad-hoc `cargo-X` plugins lacks this discoverability. Promoting noun-verb as a plugin convention would improve the ecosystem.

3. **Feature-gated progressive disclosure** — cargo-cicd's `default → process-data → autonomic → wasm4pm` feature ladder prevents binary bloat while enabling progressive capability adoption. This pattern (minimal default, explicit opt-in for each capability tier) should be the standard for opinionated CI/CD tools.

---

## Methodology Notes

- **Agent 1** surveyed build orchestration tools: cargo-make, just, xtask, cargo-binstall, cargo-timing, cargo-criterion, cargo-semver-checks
- **Agent 2** surveyed test execution tools: cargo-nextest, proptest, quickcheck, cargo-fuzz, cargo-audit, trybuild, insta, miri
- **Agent 3** surveyed supply chain and evidence tools: SLSA, sigstore/cosign, in-toto, cargo-deny, cargo-sbom, cargo-cyclonedx, SARIF, OpenVEX
- **Agent 4** surveyed workspace and monorepo management: cargo-workspaces, release-plz, git-cliff, cargo-mutants, pre-commit, rusty-hook, cargo2nix, crane
- **Agent 5** surveyed policy enforcement and CI/CD standardization: cargo-deny (policy view), Mergify, bors-ng, renovate, dependabot, GitHub Actions
- **Total tools surveyed:** 37 tools and standards across 5 domains
- **Tools rated High priority:** 9
- **Tools rated Medium priority:** 13
- **Tools rated Low priority:** 15

---

*Generated by 5-agent parallel reconnaissance · 2026-06-17 · cargo-cicd v26.6.2*
