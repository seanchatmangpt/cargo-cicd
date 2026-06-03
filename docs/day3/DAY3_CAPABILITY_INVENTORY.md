# DAY3 CAPABILITY INVENTORY — cargo-cicd v26.6.2

Date: 2026-06-03

## Summary

| Category | Count |
|---|---|
| LIVE surfaces | 15 |
| PARTIAL surfaces | 5 |
| BLOCKED surfaces | 6 |
| STUB / UNKNOWN | 3 |
| Diagnostic codes | 15 |
| Core models | 10 |
| LSP status | PARTIAL |
| wpm integration | SHELL_OUT |
| Conformance status | VARIANCE |
| ggen surfaces | 13 |
| Spec Kit | absent |
| Public boundary | clean |
| audit_key_regression_protected | true |
| CICD-WPM-004 present | true |

---

## Capability Matrix

| Surface | Capability | Status | Evidence Source | Runtime Command | Test | Receipt | Risk | Day 3 Relevance | Recommended Action |
|---|---|---|---|---|---|---|---|---|---|
| status show | Emits start+complete events to events.jsonl with ISO timestamps and session case_id. Outputs toolchain, target size, branch, dirty count. WARN verdict when tree is dirty. | LIVE | events.jsonl | `cargo cicd status show` | yes | yes | low | baseline evidence emission | maintain |
| status audit | Invokes wpm oracle on events.xes, emits TRUTHFUL/VARIANCE verdict with fitness score. Depends on wpm binary at /Users/sac/wasm4pm/target/release/wpm. | LIVE | events.jsonl | `cargo cicd status audit` | no | yes | medium — wpm path dependency | conformance fitness | monitor wpm PATH gap |
| target show | Shows target dir size vs configured max. Emits evidence. Used as pipeline step. | LIVE | events.jsonl | `cargo cicd target show` | no | yes | low | pipeline step | maintain |
| target prune | Dry-run by default (WARN:dry_run verdict). Lists candidates for deletion. --apply flag executes. Evidence emitted for both dry and apply modes. | LIVE | events.jsonl | `cargo cicd target prune` | no | yes | low | pipeline step | maintain |
| test changed | Conservative changed-file detection against origin/main. Returns plan and recommendation. No test execution when no changed test files detected. | LIVE | events.jsonl | `cargo cicd test changed` | no | yes | low | CI scoping | maintain |
| trybuild changed | Scopes trybuild fixture runs to changed files only. No-op when no fixtures changed. Used as pipeline step. | LIVE | events.jsonl | `cargo cicd trybuild changed` | no | yes | low | CI scoping | maintain |
| git status | Shows branch, staged/dirty/untracked counts, ahead/behind, dirty file list, and next-step recommendation. | LIVE | events.jsonl | `cargo cicd git status` | no | yes | low | git hygiene | maintain |
| git close | Enforces clean-tree precondition before phase closure. Refuses to batch-commit unrelated dirty files. Returns error on dirty tree with explanation. | LIVE | events.jsonl | `cargo cicd git close` | no | yes | low | phase gate | maintain |
| publish run | Emits cicd.toml with workspace state snapshot. Calls wpm receipt doctor inline. Adjudication result is printed. Used as pipeline step. | LIVE | events.jsonl | `cargo cicd publish run` | no | yes | medium — no test file | publish gate | add receipt schema |
| workspace doctor | Checks Cargo.toml, toolchain, git repo, cicd.toml. Runs autonomic policies (target_pressure, toolchain_mismatch, trybuild_changed, git_phase_dirty). WARN on rust-toolchain file. | LIVE | events.jsonl | `cargo cicd workspace doctor` | no | yes | low | workspace health | maintain |
| pipeline run | Full pipeline orchestration. | LIVE | events.jsonl | `cargo cicd pipeline run` | no | yes | low | integration | maintain |
| lsp doctor | LSP health check. | LIVE | events.jsonl | `cargo cicd lsp doctor` | no | yes | low | LSP health | maintain |
| evidence doctor | Evidence emission health check. | LIVE | events.jsonl | `cargo cicd evidence doctor` | no | yes | low | evidence health | maintain |
| evidence audit | Evidence audit against model. | LIVE | events.jsonl | `cargo cicd evidence audit` | no | yes | low | evidence health | maintain |
| analyzers/evidence.rs | Evidence analyzer module. | LIVE | module | — | yes | — | low | evidence baseline | maintain |
| lsp explain | Run logic and CICD_CATALOG lookup implemented; positional `code` arg not wired through clap-noun-verb 26.6.2 build_command(). | PARTIAL | events.jsonl | `cargo cicd lsp explain <CODE>` | no | no | low — bounded local fix | **Day 3 primary target** | wire build_command() positional arg |
| commands/explain.rs | Only 7 of 22 catalog codes wired. | PARTIAL | module | — | no | — | low | lsp explain coverage | expand after arg wiring |
| analyzers/runtime_court.rs | WPM-004 not wired into runtime_court analyzer. | PARTIAL | module | — | no | — | low | CICD-WPM-004 emission | wire WPM-004 branch |
| analyzers/rendered_surface.rs | Fixture present; no test file. | PARTIAL | fixture | — | no | — | low | ggen surface testing | add test file |
| analyzers/publish.rs | Fixture present; no test file. | PARTIAL | fixture | — | no | — | medium | publish receipt | add test + receipt schema |
| lsp serve | cargo-cicd-lsp binary not found on PATH. | BLOCKED | — | — | no | — | medium | editor integration | resolve binary PATH |
| lsp explain CICD-WPM-004 | WPM-004 not wired in explain. | BLOCKED | — | — | no | — | low | diagnostic coverage | wire after runtime_court |
| lsp explain CICD-EVIDENCE-* | EVIDENCE codes not wired in explain. | BLOCKED | — | — | no | — | low | diagnostic coverage | expand explain wiring |
| lsp explain CICD-PUBLIC-* | PUBLIC codes not wired in explain. | BLOCKED | — | — | no | — | low | diagnostic coverage | expand explain wiring |
| lsp explain CICD-WPM-001/002/003 | WPM codes not wired in explain. | BLOCKED | — | — | no | — | low | diagnostic coverage | expand explain wiring |
| CICD-FALSECLOSE-001 | FalseCloseRisk in code enum, no fixture. | STUB | — | — | no | — | low | unknown | investigate |
| CICD-TEST-002 | TestsImpactUnknown, no fixture. | STUB | — | — | no | — | low | unknown | investigate |
| CICD-PUBLISH-003 | PublishNoCicdToml, no fixture. | STUB | — | — | no | — | low | unknown | investigate |

---

## Diagnostic Codes

| Code | Status |
|---|---|
| CICD-EVIDENCE-001 | catalog entry present |
| CICD-EVIDENCE-002 | catalog entry present |
| CICD-EVIDENCE-003 | catalog entry present |
| CICD-EVIDENCE-004 | catalog entry present |
| CICD-GIT-001 | catalog entry present; reachable via lsp explain after Day 3 fix |
| CICD-GIT-002 | catalog entry present |
| CICD-PUBLIC-001 | catalog entry present |
| CICD-PUBLIC-002 | catalog entry present |
| CICD-WPM-001 | catalog entry present |
| CICD-WPM-002 | catalog entry present |
| CICD-WPM-003 | catalog entry present |
| CICD-WPM-004 | catalog entry present; runtime_court wiring absent |
| CICD-GGEN-001 | catalog entry present |
| CICD-GGEN-002 | catalog entry present |
| CICD-GGEN-003 | catalog entry present |

---

## Core Models

| Model | Fields |
|---|---|
| Evidence | case_id, event, freshness, receipt_ref, timestamp |
| Diagnostics | code, finding, lifecycle, route, severity |
| Git | head, phase, status |
| Ggen | custom_region, drift, rendered_surface |
| PublicBoundary | scan, terms |
| Publish | dry_run, readiness |
| Target | snapshot, threshold |
| TestsChanged | impact, mapper, stale |
| Workspace | manifest, paths, root, snapshot |
| Wpm | capability, court, verdict |

---

## wpm Integration

- Strategy: SHELL_OUT — Wasm4pmShell shells out via std::process::Command
- Detection order: WPM_PATH env var → known scan path → PATH lookup
- Binary present: /Users/sac/wasm4pm/target/release/wpm (version 26.5.29)
- Binary in PATH: no — WPM_PATH or known scan path required

---

## Conformance Status

- Status: VARIANCE
- Pipeline internal oracle: TRUTHFUL at fitness 0.9636 (1 deviating trace, M:1 R:1)
- External wpm audit on events.xes: VARIANCE at fitness 0.8194 (M:2 R:1)
- Discrepancy: oracle and external wpm audit disagree on the same XES file
- Root cause: uninvestigated

---

## ggen Surfaces (13)

- docs/reference/commands.md
- README.md
- docs/tutorials/getting-started.md
- docs/tutorials/first-playground-run.md
- docs/reference/commands/workspace-doctor.md
- docs/reference/commands/trybuild-changed.md
- docs/reference/commands/target-prune.md
- docs/reference/commands/git-close.md
- docs/reference/commands/test-changed.md
- docs/reference/commands/publish-run.md
- docs/reference/commands/git-status.md
- docs/reference/commands/status.md
