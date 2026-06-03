# cargo-cicd-lsp Delivery Receipt

**Date:** 2026-06-02
**Version:** 26.6.2

## What Was Added

### Crates

- **crates/cargo-cicd-core/** — shared domain crate used by both the main CLI and the LSP server. Provides workspace snapshots, diagnostic models, git phase/status/head models, evidence (case ID, event, freshness, receipt ref, timestamp), publish readiness, public boundary scanning, ggen drift detection, target threshold/snapshot, tests-changed impact/mapper/stale, and wpm capability/verdict models.

- **crates/cargo-cicd-lsp/** — LSP server (tower-lsp). Contains analyzers, lifecycle model, protocol mapping, server backend, state management, and file watcher with debounce.

### CLI Commands Added

- `cargo cicd lsp serve` — start the LSP server on stdio or a TCP socket
- `cargo cicd lsp doctor` — validate LSP configuration and connectivity without starting the server
- `cargo cicd lsp explain <CODE>` — print a human-readable explanation and repair guide for a diagnostic code

### Diagnostic Codes Implemented

27 stable codes across 9 families:

| Family | Codes |
|---|---|
| GIT | CICD-GIT-001, CICD-GIT-002, CICD-GIT-003 |
| EVIDENCE | CICD-EVIDENCE-001, CICD-EVIDENCE-002, CICD-EVIDENCE-003, CICD-EVIDENCE-004 |
| WPM | CICD-WPM-001, CICD-WPM-002, CICD-WPM-003 |
| TEST | CICD-TEST-001, CICD-TEST-002, CICD-TEST-003 |
| TARGET | CICD-TARGET-001, CICD-TARGET-002, CICD-TARGET-003, CICD-TARGET-004 |
| PUBLISH | CICD-PUBLISH-001, CICD-PUBLISH-002, CICD-PUBLISH-003, CICD-PUBLISH-004 |
| PUBLIC | CICD-PUBLIC-001, CICD-PUBLIC-002, CICD-PUBLIC-003 |
| GGEN | CICD-GGEN-001, CICD-GGEN-002, CICD-GGEN-003 |
| CLOSE | CICD-CLOSE-001, CICD-CLOSE-002, CICD-CLOSE-003, CICD-CLOSE-004 |

### Analyzers Implemented

9 analyzers in `crates/cargo-cicd-lsp/src/analyzers/`:

1. `git_phase` — GIT family diagnostics from working tree state
2. `evidence` — EVIDENCE family diagnostics from evidence log
3. `runtime_court` — WPM family diagnostics from wpm binary presence and version
4. `changed_tests` — TEST family diagnostics from changed file/coverage mapping
5. `target_hygiene` — TARGET family diagnostics from workspace build targets
6. `publish` — PUBLISH family diagnostics from cicd.toml and Cargo.toml version
7. `public_boundary` — PUBLIC family diagnostics from public API surface scan
8. `rendered_surface` — rendered surface drift detection
9. `close_readiness` — CLOSE family diagnostics aggregating all other families

### Tests Added

4 integration test files in `crates/cargo-cicd-lsp/tests/`:

- `diagnostics_evidence.rs` — evidence analyzer fixture tests (2 tests)
- `diagnostics_git.rs` — git phase analyzer fixture tests (2 tests)
- `diagnostics_public_boundary.rs` — public boundary analyzer fixture tests (2 tests)
- `lifecycle_clear.rs` — lifecycle clear/raise/route tests (3 tests)

1 fixture workspace: `crates/cargo-cicd-lsp/fixtures/workspaces/dirty-tree/`

### Docs Added

4 files in `docs/lsp/`:

- `DIAGNOSTICS.md` — full diagnostic code reference with severity, observed surface, repair surface, and clear condition for all 27 codes
- `EDITOR_INTEGRATION.md` — editor setup guides (Neovim/nvim-lspconfig, VS Code, Helix, Zed)
- `LIFECYCLE.md` — diagnostic lifecycle model (pending, raise, route, clear, residual)
- `README.md` — LSP overview, architecture, and quick-start

### Source Files

238 total `.rs` files across the workspace.

## Quality Gates

| Gate | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --workspace --all-features -- -D warnings` | pass |
| `cargo test --workspace --all-features` | 30 test suites, 0 failed |
| Total tests passing | 157 (sum across all suites) |

## Partial Items

None. All planned diagnostic families, analyzers, CLI commands, tests, and docs are present.

## Blocked Items

None.
