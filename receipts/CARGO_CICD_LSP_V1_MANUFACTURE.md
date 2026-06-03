# Manufacturing Receipt — cargo-cicd-lsp v1

**Date:** 2026-06-02
**Gate:** Dung Gate (output/artifact manufacture)
**Manufactured by:** cargo-cicd LSP documentation pass

---

## Artifact

`cargo-cicd-lsp` — LSP server that surfaces local Rust workspace readiness problems in the editor before CI fails.

---

## Crates Added

| Crate | Role | Feature Flag |
|-------|------|--------------|
| `tower-lsp` | LSP server framework (stdio + TCP transport) | `lsp` |
| `serde_json` | JSON-RPC message serialisation | `lsp` |
| `tokio` | Async runtime for the LSP event loop | `lsp` |

All additions are gated behind the `lsp` feature flag. The `lsp` feature does not imply `process-data` or `autonomic`.

---

## Diagnostic Codes Implemented

### CFG — Configuration (8 codes)
CICD-CFG-001 through CICD-CFG-008

### TGT — Targets (5 codes)
CICD-TGT-001 through CICD-TGT-005

### TCH — Toolchain (4 codes)
CICD-TCH-001 through CICD-TCH-004

### CHG — Changed Files (3 codes)
CICD-CHG-001 through CICD-CHG-003

### GIT — Git Phase (5 codes)
CICD-GIT-001 through CICD-GIT-005

### WRK — Workspace (4 codes)
CICD-WRK-001 through CICD-WRK-004

**Total: 29 diagnostic codes across 6 domains**

---

## Tests Added

| Test file | Coverage |
|-----------|----------|
| `tests/lsp_diagnostics.rs` | CICD-CFG-001..008 raise and clear |
| `tests/lsp_diagnostics.rs` | CICD-TGT-001..005 raise and clear |
| `tests/lsp_diagnostics.rs` | CICD-TCH-001..004 raise and clear |
| `tests/lsp_diagnostics.rs` | CICD-CHG-001..003 raise and clear |
| `tests/lsp_diagnostics.rs` | CICD-GIT-001..005 raise and clear |
| `tests/lsp_diagnostics.rs` | CICD-WRK-001..004 raise and clear |
| `tests/lsp_lifecycle.rs` | Keyed-subtraction law: cleared codes absent from published set |
| `tests/lsp_lifecycle.rs` | Suppression chain: CFG error silences downstream evaluators |
| `tests/lsp_lifecycle.rs` | Routing: each domain routes to correct URI |
| `tests/lsp_doctor.rs` | `doctor` exit code 0 for clean workspace |
| `tests/lsp_doctor.rs` | `doctor` exit code 1 when diagnostics present |
| `tests/lsp_explain.rs` | `explain` returns prose for every defined code |
| `tests/lsp_explain.rs` | `explain` returns non-zero exit for unknown code |

---

## Docs Created

| File | Purpose |
|------|---------|
| `docs/lsp/README.md` | What it does, what it does NOT do, installation, commands |
| `docs/lsp/DIAGNOSTICS.md` | Full table of 29 CICD-XXX-NNN codes with severities and clearing conditions |
| `docs/lsp/LIFECYCLE.md` | raised → routed → pending repair → cleared by evidence; keyed-subtraction law |
| `docs/lsp/EDITOR_INTEGRATION.md` | VS Code, Neovim, Helix configuration stubs |
| `receipts/CARGO_CICD_LSP_V1_MANUFACTURE.md` | This receipt |

---

## Quality Gates

| Gate | Condition | Status |
|------|-----------|--------|
| No forbidden terms | ALIVE, Inspection Gate, wall, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 absent from all docs | Pass |
| Public/private boundary | LSP docs contain no private architecture terms | Pass |
| Clearing law stated | Keyed-subtraction law documented in LIFECYCLE.md | Pass |
| Severity coverage | All 4 LSP severity levels used across the code table | Pass |
| Domain coverage | All 6 domains (CFG, TGT, TCH, CHG, GIT, WRK) defined and documented | Pass |
| Command surface complete | `serve`, `doctor`, `explain` all documented in README.md | Pass |
| Editor stubs present | VS Code, Neovim, Helix stubs present in EDITOR_INTEGRATION.md | Pass |

---

## Keyed-Subtraction Law (Summary)

Diagnostic clearing is by evidence, not by timer or user gesture.

The server publishes the complete current diagnostic set for each URI on every evaluation. Any code previously published for a URI that does not appear in the new evaluation is treated as cleared by the client. This is the keyed-subtraction law.

Full specification: [docs/lsp/LIFECYCLE.md](../docs/lsp/LIFECYCLE.md)
