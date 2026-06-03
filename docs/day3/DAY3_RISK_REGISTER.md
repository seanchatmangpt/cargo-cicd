# DAY3_RISK_REGISTER — cargo-cicd v26.6.2

**Date:** 2026-06-03
**Scope:** Risks relevant to Day 3 work and current system state.

---

## Risk Table

| ID | Risk | Probability | Severity | Mitigation | Status |
|---|---|---|---|---|---|
| RISK-D3-001 | LSP server does not start in editor | Medium | High — blocks all editor diagnostic features | Add initialize test; verify LSP server binary is on PATH before shipping | Open |
| RISK-D3-002 | wpm binary path changes | Low | Medium — breaks status audit, publish run, any wpm shell-out | `WPM_PATH` env var takes priority over known scan path; document in workspace doctor output | Open |
| RISK-D3-003 | Conformance regresses if DFG model changes | Medium | Medium — fitness drop may go undetected until wpm audit | `cargo cicd pipeline run` produces a baseline receipt; gate on fitness delta exceeding threshold | Open |
| RISK-D3-004 | Publish without adjudication | Low | High — release could occur without wpm verdict on record | Receipt doctor gate blocks publish run when verdict is absent; document the gate in publish run help text | Open |
| RISK-D3-005 | Private term leak in public docs | Low | Medium — exposes internal terminology through ggen-rendered surfaces | Public boundary scan runs on every ggen render; current status is clean | Open |

---

## Risk Detail

### RISK-D3-001: LSP server does not start in editor

**Scenario:** `lsp serve` is invoked by an editor (VS Code, Neovim, Helix) but the `cargo-cicd-lsp` binary is not found on PATH. The editor silently falls back to no diagnostics.

**Current state:** `cargo-cicd-lsp` binary not found on PATH. `lsp serve` is BLOCKED.

**Mitigation:**
1. Write an initialize test that invokes the LSP server and confirms a valid `initialize` response.
2. Ensure `cargo-cicd-lsp` is installed to a PATH-visible location during `cargo install`.
3. `lsp doctor` surfaces the binary-not-found condition with a repair hint.

---

### RISK-D3-002: wpm binary path changes

**Scenario:** The wpm binary moves from `/Users/sac/wasm4pm/target/release/wpm` (e.g., after a clean build, relocation, or version upgrade). All features that shell out to wpm stop working silently.

**Current state:** wpm binary at known scan path (version 26.5.29). Not in PATH. `WPM_PATH` env var not set.

**Mitigation:**
1. `WPM_PATH` env var takes priority in `Wasm4pmShell` detection order — set it in CI and local shell profile.
2. `workspace doctor` emits a WARN when wpm is detected via scan path only (not PATH or WPM_PATH).
3. Document the detection order in `cargo cicd lsp explain CICD-WPM-001`.

---

### RISK-D3-003: Conformance regresses if DFG model changes

**Scenario:** A change to event emission, activity naming, or pipeline structure causes the wpm-derived DFG model to drift. Fitness drops below 0.9636 (current oracle baseline) or below 0.8194 (current ambient VARIANCE) without detection.

**Current state:** VARIANCE between oracle (0.9636) and ambient wpm audit (0.8194). Root cause uninvestigated.

**Mitigation:**
1. `cargo cicd pipeline run` emits a receipt with `pipeline_trace_fitness` — use this as a regression baseline.
2. Add a test that asserts fitness does not drop below a documented threshold on the canonical event log.
3. Document the current VARIANCE and its known bounds so any further regression is immediately visible.

---

### RISK-D3-004: Publish without adjudication

**Scenario:** `cargo publish` is invoked outside of `cargo cicd publish run`, bypassing the wpm receipt doctor gate. A release ships without a wpm verdict on record.

**Current state:** `publish run` calls wpm receipt doctor inline. Adjudication result is printed. No receipt struct persists the verdict with commit hash and timestamp.

**Mitigation:**
1. Define a publish receipt struct (`wpm_verdict`, `commit_hash`, `timestamp`, `fitness_at_adjudication`) and require it before publish proceeds.
2. Receipt doctor gate refuses publish when verdict is absent.
3. Document the gate in `cargo cicd lsp explain CICD-PUBLISH-003`.

---

### RISK-D3-005: Private term leak in public docs

**Scenario:** ggen renders a documentation surface that contains internal terminology not suitable for public consumption. The term passes through undetected because the public boundary scan was not run after a ggen render.

**Current state:** Public boundary scan status is clean. 12 ggen-rendered surfaces are present.

**Mitigation:**
1. Public boundary scan runs on every ggen render cycle.
2. CICD-PUBLIC-001 and CICD-PUBLIC-002 diagnostic codes cover boundary violations.
3. Current clean status should be preserved — any new ggen surface must pass the scan before staging.
