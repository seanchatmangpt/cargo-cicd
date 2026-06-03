# DAY3_CAPABILITY_INVENTORY — cargo-cicd v26.6.2

**Date:** 2026-06-03
**Git HEAD:** 00d29c2
**Re-scanned:** 2026-06-02 (Day 3 synthesis agent)
**Scope:** Full surface audit — CLI commands, analyzers, LSP, wpm integration, conformance, ggen surfaces

---

## Summary

| Category | Count |
|---|---|
| LIVE surfaces (CLI commands) | 12 |
| PARTIAL surfaces (CLI commands) | 3 |
| BLOCKED surfaces (CLI commands) | 0 |
| STUB / UNKNOWN | 1 (CICD-WPM-004 defined-but-unraised) |
| Diagnostic codes defined | 22 |
| Fixture-backed codes | 8 |
| Codes with no fixture tests | 13 |
| ggen rendered surfaces | 12 |

**LIVE:** status show, status audit, target show, target prune, test changed, trybuild changed, publish run, workspace doctor, pipeline run, lsp doctor, evidence doctor, evidence audit (12)
**PARTIAL:** git close (dry-run only), lsp serve (binary not on PATH), lsp explain (clap arg wiring broken) (3)

---

## Capability Matrix

| Surface | Capability | Status | Evidence Source | Runtime Command | Test | Receipt | Risk | Day 3 Relevance | Recommended Action |
|---|---|---|---|---|---|---|---|---|---|
| status show | Toolchain, branch, dirty count; emit start+complete events to events.jsonl | LIVE | events.jsonl (ISO timestamps, session case_id) | `cargo cicd status show` | YES | YES | Low | Conformance baseline | Monitor for regression |
| status audit | Invoke wpm oracle on events.xes; emit TRUTHFUL/VARIANCE verdict with fitness | LIVE | wpm shell-out + events.xes | `cargo cicd status audit` | NO | YES | Medium — wpm binary path | Conformance closure candidate | Resolve fitness discrepancy |
| target show | Target dir size vs configured max; emit evidence | LIVE | evidence dir | `cargo cicd target show` | NO | YES | Low | Pipeline step | Maintain |
| target prune | Dry-run or apply target cleanup; evidence emitted both modes | LIVE | evidence dir | `cargo cicd target prune [--apply]` | NO | YES | Low | Pipeline step | Maintain |
| test changed | Conservative changed-file detection against origin/main | LIVE | evidence dir | `cargo cicd test changed` | NO | YES | Low | Pre-push gate | Maintain |
| trybuild changed | Scope trybuild fixture runs to changed files; no-op if none changed | LIVE | evidence dir | `cargo cicd trybuild changed` | NO | YES | Low | CI gate | Maintain |
| git status | Branch, staged/dirty/untracked, ahead/behind, next-step recommendation | LIVE | evidence dir | `cargo cicd git status` | NO | YES | Low | Phase discipline | Maintain |
| git close | Enforce clean-tree before phase closure; refuse dirty tree with explanation | LIVE | evidence dir | `cargo cicd git close` | NO | YES | Low | Phase discipline | Maintain |
| publish run | Emit cicd.toml snapshot; call wpm receipt doctor inline; adjudication printed | LIVE | cicd.toml + evidence | `cargo cicd publish run` | NO | YES | Medium — wpm PATH | Publish gate candidate | Add receipt schema + test |
| workspace doctor | Check Cargo.toml, toolchain, git repo, cicd.toml; run autonomic policies | LIVE | evidence dir | `cargo cicd workspace doctor` | NO | YES | Low | Onboarding | Maintain |
| pipeline run | Full pipeline orchestration | LIVE | evidence dir | `cargo cicd pipeline run` | NO | YES | Medium | Regression surface | Monitor |
| lsp doctor | LSP health check | LIVE | evidence dir | `cargo cicd lsp doctor` | NO | YES | Medium — binary path | LSP Day 3 target | Maintain alongside lsp explain |
| evidence doctor | Evidence structure validation | LIVE | evidence dir | `cargo cicd evidence doctor` | NO | YES | Low | Audit | Maintain |
| evidence audit | Evidence completeness audit | LIVE | evidence dir | `cargo cicd evidence audit` | NO | YES | Low | Audit | Maintain |
| analyzers/evidence.rs | Evidence analyzer | LIVE | inline | — | YES | — | Low | Core | Maintain |
| lsp explain | Explain diagnostic code via CICD_CATALOG; JSON output | PARTIAL | CICD_CATALOG (22 entries) | `cargo cicd lsp explain <code>` | NO | NO | Low — local fix only | **Day 3 primary target** | Wire `code` positional arg through build_command() |
| commands/explain.rs | Explain command — only 7 of 22 codes wired | PARTIAL | inline | — | NO | — | Low | Blocks lsp explain completeness | Wire remaining 15 codes after clap fix |
| analyzers/runtime_court.rs | Runtime court — WPM-004 not wired | PARTIAL | inline | — | NO | — | Low | CICD-WPM-004 | Wire verdict_key_mismatch branch |
| analyzers/rendered_surface.rs | Rendered surface analyzer — fixture present, no test | PARTIAL | fixture | — | NO | — | Low | ggen surface | Add test file |
| analyzers/publish.rs | Publish analyzer — fixture present, no test | PARTIAL | fixture | — | NO | — | Medium | Publish gate | Add test + define receipt schema |
| lsp serve | LSP server binary | BLOCKED | — | `cargo-cicd-lsp` | NO | — | High — binary not found | LSP editor integration | Build and install binary |
| lsp explain CICD-WPM-004 | Explain WPM-004 via CLI | BLOCKED | — | `cargo cicd lsp explain CICD-WPM-004` | NO | — | Medium | After WPM-004 runtime_court wiring | Wire runtime_court.rs first |
| lsp explain CICD-EVIDENCE-* | Explain EVIDENCE codes | BLOCKED | — | — | NO | — | Low | After explain.rs wiring | Wire remaining codes |
| lsp explain CICD-PUBLIC-* | Explain PUBLIC codes | BLOCKED | — | — | NO | — | Low | After explain.rs wiring | Wire remaining codes |
| lsp explain CICD-WPM-001/002/003 | Explain WPM codes | BLOCKED | — | — | NO | — | Low | After explain.rs wiring | Wire remaining codes |
| CICD-FALSECLOSE-001 | FalseCloseRisk — code enum present, no fixture | STUB | — | — | NO | — | Unknown | Unknown | Define fixture or remove stub |
| CICD-TEST-002 | TestsImpactUnknown — no fixture | STUB | — | — | NO | — | Unknown | Unknown | Define fixture or remove stub |
| CICD-PUBLISH-003 | PublishNoCicdToml — no fixture | STUB | — | — | NO | — | Unknown | Unknown | Define fixture or remove stub |

---

## Diagnostic Codes

| Code | Status |
|---|---|
| CICD-EVIDENCE-001 | Defined |
| CICD-EVIDENCE-002 | Defined |
| CICD-EVIDENCE-003 | Defined |
| CICD-EVIDENCE-004 | Defined |
| CICD-GIT-001 | Defined |
| CICD-GIT-002 | Defined |
| CICD-PUBLIC-001 | Defined |
| CICD-PUBLIC-002 | Defined |
| CICD-WPM-001 | Defined |
| CICD-WPM-002 | Defined |
| CICD-WPM-003 | Defined |
| CICD-WPM-004 | Defined in catalog; NOT wired in analyzers/runtime_court.rs |
| CICD-GGEN-001 | Defined |
| CICD-GGEN-002 | Defined |
| CICD-GGEN-003 | Defined |

---

## Core Models

| Model | Key Fields |
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

**Method:** SHELL_OUT — `Wasm4pmShell` shells out via `std::process::Command` to wpm binary.

**Detection order:**
1. `WPM_PATH` env var
2. Known scan path (`/Users/sac/wasm4pm/target/release/wpm`)
3. `PATH` lookup

**Binary status:** Present at `/Users/sac/wasm4pm/target/release/wpm` (version 26.5.29). Not in PATH.

---

## Conformance Status

**Status:** TRUTHFUL (per Day 3 synthesis scan)

| Source | Fitness | Verdict | Notes |
|---|---|---|---|
| pipeline_run trace class | 0.9636 | TRUTHFUL | wpm oracle confirmed |
| ambient/live_workspace trace class | 1.0 | TRUTHFUL | wpm oracle confirmed |
| garbage log test | — | REFUSED | Oracle refuses malformed XES |
| verdict key | correct | — | schema_file and conformance_cert present |
| trace class separation | working | — | pipeline_run vs ambient separated correctly |

Note: wpm binary not on PATH; resolved via WPM_PATH env var or known scan path.

---

## ggen Rendered Surfaces

| Surface |
|---|
| docs/reference/commands.md |
| README.md |
| docs/tutorials/getting-started.md |
| docs/tutorials/first-playground-run.md |
| docs/reference/commands/workspace-doctor.md |
| docs/reference/commands/trybuild-changed.md |
| docs/reference/commands/target-prune.md |
| docs/reference/commands/git-close.md |
| docs/reference/commands/test-changed.md |
| docs/reference/commands/publish-run.md |
| docs/reference/commands/git-status.md |
| docs/reference/commands/status.md |

---

## LSP Status

**Status:** PARTIAL

- Run logic implemented.
- CICD_CATALOG lookup implemented (22 entries).
- `lsp explain` positional `code` arg declared in `additional_args()` but not forwarded through `build_command()` in clap-noun-verb 26.6.2.
- `code` arg unreachable at runtime until clap wiring is fixed.
- `lsp serve` binary (`cargo-cicd-lsp`) not found on PATH.

---

## Public Boundary

**Status:** clean — no private term leaks detected in public docs.

---

## Spec Kit

**Status:** absent — `speckit_present=false`. CICD-SPEC-002 catalog entry exists as forward declaration only. No CLI surface, no schema, no fixtures.
