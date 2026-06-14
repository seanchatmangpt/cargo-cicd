# Changelog

All notable changes to cargo-cicd are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Version numbers follow the project's manufacturing versioning scheme.

---

## [Unreleased]

### Added
- Placeholder for work not yet released.

### Changed
- Nothing yet.

### Fixed
- Nothing yet.

---

## [26.6.2] — 2026-06-14

### Added

#### CLI Nouns and Verbs
- `status show` — workspace health report: dirty file count, target size, git phase, publish readiness.
- `status audit` — deep workspace audit with structured output.
- `target show` — display target directory size, age of oldest artefact, and prune candidates.
- `target prune` — remove artefacts older than the configured threshold (`--confirm` required).
- `test changed` — detect and run only the tests affected by changes since the last commit.
- `trybuild changed` — detect and run only the compile-fail fixtures affected by changes.
- `git status` — report branch, ahead/behind counts, and working tree state.
- `git close` — finalize a branch: verify green state, merge to trunk, emit close evidence.
- `publish run` — gate and emit `cicd.toml`; shells out to `wpm receipt doctor` before proceeding.
- `workspace doctor` — structural health check: member crates, toolchain, edition, resolver.
- `evidence doctor` — build and submit an OCEL 2.0 receipt to the wasm4pm oracle.
- `evidence audit` — run `wpm audit` against the current XES evidence file.
- `pipeline run` — sequential command runner: executes the full declared manufacturing pipeline in one invocation.
- `lsp serve` — start the LSP server for editor integration.
- `lsp doctor` — diagnose LSP server availability and configuration.
- `lsp explain <CODE>` — explain a diagnostic code from the 28-code CICD catalog (GIT/EVIDENCE/WPM/TARGET/PUBLISH/PUBLIC/GGEN/CLOSE/SPEC families).
- Default verb injection: bare nouns resolve to their most-used verb (`status` → `show`, `publish` → `run`, `workspace` → `doctor`, `evidence` → `doctor`).

#### wasm4pm Evidence Gate
- XES evidence emission: every command emits a `ProcessEvent` with a real UTC timestamp to `target/cargo-cicd/evidence/events.xes`.
- JSONL companion emission: parallel JSONL log written to `target/cargo-cicd/evidence/events.jsonl`.
- Session tracing: events are grouped into XES `<trace>` elements by `case_id` (session identifier).
- `ReceiptDoctor::emit_and_adjudicate()`: builds an OCEL 2.0 receipt and submits it to `wpm receipt doctor --format json --strict`.
- Pipeline trace enforcement: only traces following the declared activity sequence achieve conformance fitness >= 1.0; ambient single-command traces are classified DECEPTIVE.
- `trace_class` attribute on XES traces: distinguishes `pipeline_run` from `live_workspace` conformance targets.
- Keyed subtraction receipt lifecycle: each receipt key maps to exactly one live record; `latest.json` always reflects the most recent adjudication.
- `WpmVerdict` enum with exactly three variants: `Accept`, `Refuse`, `NotAvailable` — no silent fallback.
- Blocked verdict is first-class: tests running without wpm declare `ExpectedWpmVerdict::Blocked` and assert `WpmVerdict::Partial`.
- Oracle unavailability emits `WARN:oracle_unavailable` and proceeds with warning rather than silently passing.

#### cicd.toml Carrier
- `[workspace]` section: name, toolchain, target directory, detected automatically from `Cargo.toml`.
- `[state]` section: dirty flag, target size, changed file counts, changed test counts.
- `[target]` section: configurable `max_size_gb` and `prune_after_days` thresholds.
- `[autonomic]` section: policy mode and per-policy verdict records.
- `[[events]]` array: append-only process event log with timestamp, kind, case_id, command, duration, and verdict.
- `CicdTomlWriter` performs atomic TOML writes: `cicd.toml` is either valid or unchanged on failure.
- `publish_ready` field set to `true` only when the oracle returns `Admitted`.

#### Autonomic Policies (suggest mode)
- `StaleTargetPolicy`: triggers when `target/` artefacts exceed age or size thresholds; suggests `cargo cicd target prune`.
- `UncommittedEvidencePolicy`: triggers when `cicd.toml` has uncommitted changes across multiple lifecycle commands; suggests committing.
- `DivergentBranchPolicy`: triggers when branch is ahead of trunk by more than `max_commits_ahead` (default 10); suggests `cargo cicd git close`.
- `PublishReadinessPolicy`: triggers when a workspace member has been publish-ready for more than `publish_readiness_stale_days` (default 7); suggests `cargo cicd publish run`.
- All policies run in `suggest` mode by default: they emit structured recommendations and never modify workspace state.
- Policy results persisted in `cicd.toml [autonomic] [[policies]]` for audit trails.
- Per-policy and global suppression via `cicd.toml [policy]`.

#### LSP Server (cargo-cicd-lsp)
- Read-only LSP server manufactured as a separate workspace crate (`crates/cargo-cicd-lsp`).
- 28-code diagnostic catalog across families: GIT, EVIDENCE, WPM, TARGET, PUBLISH, PUBLIC, GGEN, CLOSE, SPEC.
- Diagnostics published for: git working tree state, evidence freshness, publish readiness, public boundary safety, wasm4pm availability, target directory growth, changed test coverage.
- Observer-only constraint: LSP adapter reads workspace state but never mutates files, runs code actions, or spawns processes.
- Editor integration documented for VS Code, Neovim, Helix, and Zed.

#### Feature Flags
- `process-data` — enables `EngineState` population for `ProcessEventState` and `ArtifactState`; XES/JSONL evidence emission.
- `autonomic` — implies `process-data`; enables `PolicyState`, policy evaluation, and suggest mode.
- `wasm4pm` — implies `process-data`; enables `Wasm4pmShell`, oracle adjudication, and richer runtime integration.
- `contrib` — implies `process-data`; enables contributor-facing commands and internal diagnostics.
- All features are opt-in; the default feature set is empty (full public CLI remains available without features).

#### EngineState Aggregate Root
- `EngineState` as single aggregate root with 11 dimensions: `WorkspaceState`, `ToolchainState`, `TargetState`, `ChangedFileState`, `TestPlanState`, `TrybuildState`, `GitPhaseState`, `ProcessEventState`, `ArtifactState`, `PolicyState`, `ProjectionProfile`.
- Adapter layer: `GitStatusAdapter`, `TargetScannerAdapter`, `ToolchainDetector`, `CargoMetadataAdapter`, `ChangedFileDetector`, `TrybuildDetector`, `CicdTomlWriter` — each adapter owns one external source with no business logic.
- Five EngineState invariants enforced in tests: workspace immutability, git phase alignment, target size accuracy, no circular dependencies, feature flag containment.

#### ggen Ontology Pipeline
- `ggen.toml` + `ontology/cargo-cicd.ttl` + SPARQL queries + Tera templates manufacturing pipeline.
- Generates CLI documentation, command reference pages, tutorial scaffolding, and test stubs from the ontology.
- Customization guard: `tests/ggen_customization_guard.rs` enforces that hand-edited `mode = "Preserve"` files are never overwritten.

#### Three-Crate Workspace
- `cargo-cicd` — CLI binary: NounCommand + VerbCommand implementations.
- `crates/cargo-cicd-core` — domain crate: pure functions, no clap dependency.
- `crates/cargo-cicd-lsp` — LSP server crate: observer-only workspace diagnostics.
- Enforced dependency direction: CLI → Integration → Domain; Domain never imports CLI.

#### Test Suite (118+ tests)
- `tests/invariants.rs` — 7 non-negotiable public boundary invariants, including forbidden terms scan.
- `tests/cli/command_projection.rs` — CLI parsing and noun-verb routing tests.
- `tests/cicd_toml_truth.rs` — cicd.toml schema validation.
- `tests/autonomic_policies.rs` — policy evaluation and suggest mode.
- `tests/changed_tests.rs` — changed test detection logic.
- `tests/git_phase_closure.rs` — git phase state and close semantics.
- `tests/feature_projection.rs` — feature flag capability matrix contract.
- `tests/wasm4pm_evidence_gate.rs` — closing evidence gate tests (require oracle).
- `tests/wasm4pm_evidence_mutation.rs` — oracle sensitivity to evidence mutations.
- `tests/wasm4pm_refusal_cases.rs` — wasm4pm refusal scenarios.
- `tests/wasm4pm_harness.rs` — shared wasm4pm test harness.
- `tests/lsp_explain.rs` — LSP diagnostic code explanation routing.
- `tests/refusal_calibration.rs` — calibration tests for refusal cases.
- `tests/ggen_customization_guard.rs` — ggen preservation mode guard.
- `tests/wpm_verdict_key_contract.rs` — no silent fallback on absent verdict keys.

### Changed
- `publish run` now gates on `wpm receipt doctor --format json --strict` before writing `cicd.toml`; prior behavior was to write unconditionally.
- Evidence directory changed from `.cicd/` to `target/cargo-cicd/evidence/` to align with Cargo convention and ensure exclusion from crates.io publish.
- `wasm4pm` feature flag now implies `process-data` rather than standing alone.

### Fixed
- LSP `explain` routing: `lsp explain <CODE>` now correctly uses trailing var-arg and routes through clap-noun-verb without a main.rs interception hack.
- Evidence mutation tests: declared-activity filter now applied before fitness scoring to prevent ambient traces from inflating conformance scores.
- Session-aware evidence emission applied to all nouns: `status`, `publish`, `workspace`, `trybuild`, and `test` now correctly group events by session `case_id`.
- `clap-noun-verb` pinned to `26.6.2` (crates.io indexed version) after a revert from a development build.
- `needless_late_init` clippy warning in target adapter resolved.

---

## [26.6.1] — 2026-05-01

### Added
- Initial noun-verb CLI grammar: `status`, `target`, `test`, `trybuild`, `git`, `publish`, `workspace`.
- `cicd.toml` carrier for workspace state.
- Basic `GitStatusAdapter` and `TargetScannerAdapter`.
- Foundational `EngineState` with core dimensions.
- Integration test scaffolding using `assert_cmd` and `tempfile`.

### Changed
- Nothing tracked; this is a placeholder for the prior release.

### Fixed
- Nothing tracked; this is a placeholder for the prior release.

---

[Unreleased]: https://github.com/seanchatmangpt/cargo-cicd/compare/v26.6.2...HEAD
[26.6.2]: https://github.com/seanchatmangpt/cargo-cicd/compare/v26.6.1...v26.6.2
[26.6.1]: https://github.com/seanchatmangpt/cargo-cicd/releases/tag/v26.6.1
