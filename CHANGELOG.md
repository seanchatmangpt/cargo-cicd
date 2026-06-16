# Changelog

All notable changes to cargo-cicd are documented here.

Format: feat(scope): description — scope is core|cli|target|test|git|autonomic|docs|receipts

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
