# Changelog

All notable changes to cargo-cicd are documented here.

Format: feat(scope): description — scope is core|cli|target|test|git|autonomic|docs|receipts

---

## [26.6.30] — 2026-06-26

### Fixed
- `cargo-cicd-lsp`: bundle `schemas/` into the crate and correct the `include_str!` path depth so `cargo publish` no longer fails on the bundled cicd.toml schema

### Changed
- Reduced the test suite from 799 to 200 tests: removed getter/round-trip/smoke tests that only re-asserted struct fields, keeping behavior-level coverage
- `chicago-tdd-tools` bumped to 26.6.30; `star-toml` bumped to 26.6.30

## [26.6.29] — 2026-06-22

### Added
- Full CLI integration of `chicago-tdd-tools` v26.6.29; all noun commands registered
- `trace`, `gate`, `verify`, `ocel`, and `hooks` nouns implemented
- Sprint 1–7 "operational physics" pass: admission control, evidence, gates, and LSP ANDON substrate hardening

### Fixed
- Wired `certification` and `sbom` CLI commands; eliminated all build warnings; closed dead-code paths

### Changed
- Removed local path parameter from the `chicago-tdd-tools` dependency (now version-only)

## [26.6.22] — 2026-06-20

### Changed
- Pre-publish `Cargo.toml` updates: added version constraints to workspace-internal dependencies, removed the `readme` field from subcrates with no local README

## [26.6.19] — 2026-06-19

### Added

**OCEL 2.0 Unification**
- `events.ocel.json` is now the primary evidence format; `events.xes` is a legacy dual-write side-channel
- `pipeline run` oracle call switched from XES to `receipt_verify_ocel2()` — no XES fallback
- `status audit` reads `events.ocel.json` (was `events.xes`) for oracle adjudication
- `evidence reset` now removes `events.ocel.json` alongside JSONL and XES
- `wasm4pm_shell.audit()` parameter renamed to `path` (accepts any evidence file, not XES-only)

**CLI Nouns**
- `certification show` — Display IEC 61508, ISO 26262, SOC2 Trust Service Criteria, and TOGAF ADM coverage
- `sbom generate` — Generate CycloneDX SBOM via `cargo cyclonedx --format json`; degrades to WARN:cyclonedx_unavailable if tool absent
- `sbom show` — Print first 20 lines of `sbom.json` with evidence emission

**Compliance & Certification**
- IEC 61508 SIL 2 coverage: 9 clauses mapped (1, 6, 7, 7.4, 7.9, 8, 9, 10, 12)
- ISO 26262 ASIL B coverage: Part 6 clauses mapped (5, 7, 7.4.11, 8, 9, 10)
- SOC2 Trust Service Criteria: 6 criteria mapped (CC6.1, CC7.2, A1.1, PI1.1, PI1.4, C1.1)
- TOGAF ADM: 6 of 9 phases covered (B, C-App, C-Data, D, G, H); A, E, F deferred
- `src/certification/soc2.rs` — `TrustCategory` enum, 6 `Soc2Criterion` structs
- `src/certification/togaf.rs` — 9-phase `TogafPhase` structs with covered/deferred classification

**CI Gates**
- `affidavit-gate` GitHub Actions job — builds `--features affidavit`, runs seal/verify (non-blocking)
- `lsp-admissibility` job — builds `--features anti-llm-cheat`, runs `lsp check` (non-blocking)
- `workspace sync` smoke step added to `check-and-test` job
- `status audit` release gate with wpm oracle in `release.yml`
- CI evidence upload switched from `*.xes` to `*.ocel.json`

**Documentation**
- `docs/SOC2-MAPPING.md` — SOC2 Trust Service Criteria full mapping
- `docs/TOGAF-ADM-COVERAGE.md` — TOGAF ADM phase coverage with artifact mapping
- `docs/IEC-61508-MAPPING.md`, `docs/ISO-26262-MAPPING.md` — updated XES → OCEL 2.0 throughout
- `docs/XES-2.0-SPECIFICATION.md` — legacy notice added at top of file
- `docs/reference/evidence-format.md` — rewritten to lead with OCEL 2.0 as primary format
- `docs/explanation/evidence-emission.md` — dual-write architecture documented
- `docs/integration-examples/CI_CD_PIPELINES.md` — Full CI Gate Stack subsection added
- `docs/reference/commands/certification-show.md`, `sbom-generate.md` — new command reference pages

### Changed
- Evidence primary format: OCEL 2.0 (`events.ocel.json`) — XES retained as legacy side-channel only
- `ProjectionProfile::v26_6_2()` renamed to `v26_6_19()`
- `producer_version` in receipt JSON updated to `26.6.19`

### Fixed
- `pipeline run` no longer emits or reads XES in the oracle hot path

---

## [26.6.2] — 2026-06-14

### Added

**Evidence System**
- ProcessEvent emission pattern standardized across all verbs
- case_id propagation for XES trace grouping
- events.jsonl and events.xes persistence to target/cargo-cicd/evidence/
- Evidence gate tests: wasm4pm_evidence_gate.rs, wasm4pm_evidence_mutation.rs, wasm4pm_refusal_cases.rs

**CLI Verbs (git noun)**
- git status — show branch, dirty files, staged, untracked, ahead/behind
- git close — enforce phase closure (refuses dirty trees)
- git diff — show staged and unstaged changes
- git stage — stage modified tracked files (git add -u)
- git commit — create commit with auto-generated message
- git fetch — fetch from remote origin

**CLI Verbs (workspace noun)**
- workspace doctor — run autonomic policy checks + workspace health
- workspace validate — validate Cargo.toml structure and members
- workspace sync — sync via ggen if ggen.toml present
- workspace list — list all workspace member crates

**CLI Verbs (test noun)**
- test changed — run only tests affected by changed files

**LSP Integration (crates/cargo-cicd-lsp)**
- 13 analyzers registered: workspace-structure, pipeline-check, remote-tracking, changed-tests, git-phase, target-hygiene, public-boundary, publish, ggen-customization, rendered-surface, close-readiness, and more
- CicdFinding lifecycle: Raised → PendingRepair → Routed → ResidualPreserved → Cleared
- DiagnosticStore for finding persistence
- 10 new CicdCode variants added

**Core Diagnostics (crates/cargo-cicd-core)**
- CicdCode::category() — group codes by category
- CicdCode::codes_by_category() — list codes per category
- CicdCode::all_variants() — enumerate all codes
- Severity::is_blocking() — determine if severity blocks release
- DiagnosticLifecycle enum with 5 states

**Autonomic Policies**
- 7 policies with full EngineState integration
- WorkspaceInfo, GitState, EvidenceState inputs to run_all_policies()
- Policies: git_phase_dirty, target_pressure, toolchain_mismatch, trybuild_changed, branch_behind, evidence_stale, publish_not_adjudicated

**EngineState**
- from_workspace() maximized — all 11 dimensions populated from real adapters
- ManifestParser added for lightweight Cargo.toml parsing

### Fixed
- git commit false dirty detection (uses --uno flag to ignore untracked dirs)
- Policy engine now passes real EngineState instead of Default::default()
- LSP backend calling finding_to_actions with correct argument count
- workspace.rs run_all_policies now passes EvidenceState as third argument

### Changed
- Test count increased from 206 to 277 tests

---

## [26.6.1] — 2026-06-01

### Added
- Initial evidence emission infrastructure
- ProcessEvent struct with XES serialization
- session.rs for case_id lifecycle management

---

## [26.6.0] — 2026-05-15

### Added
- Initial noun-verb CLI grammar via clap-noun-verb
- status, target, test, trybuild, publish, workspace nouns
- EngineState aggregate root with adapter pattern
- cicd.toml state carrier

---
