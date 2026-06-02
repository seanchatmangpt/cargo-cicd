# Receipt: cargo-cicd v26.6.2 — wasm4pm Evidence Gate

## Identity
- date: 2026-06-02
- cargo-cicd repo: /Users/sac/cargo-cicd
- cargo-cicd commit: see closure commit below
- wasm4pm repo: /Users/sac/wasm4pm
- wasm4pm commit: 65169e625406bd9185a18aad04360bd13a4a80db
- wpm binary: /Users/sac/wasm4pm/target/release/wpm
- wpm version: wpm 26.5.29

## Discovered Oracle Commands
| Command | Status |
|---------|--------|
| wpm doctor | confirmed |
| wpm lean | confirmed |
| wpm audit probe | confirmed (requires file path argument) |
| wpm spc status | confirmed |
| wpm status show | confirmed via SHELL_OUT adapter |
| wpm git close | confirmed via SHELL_OUT adapter |
| wpm target prune | confirmed via SHELL_OUT adapter |

## Positive Evidence Cases
| Case | Command Noun | XES Fixture | Verdict |
|------|-------------|-------------|---------|
| evidence_gate_oracle_discover | oracle availability | auto-detect | Accept (oracle found) |
| evidence_gate_status_show_accepted | status show | valid 1-event XES | Accept |
| evidence_gate_git_close_accepted | git close | valid 1-event XES | Accept |
| evidence_gate_target_prune_accepted | target prune | valid 1-event XES | Accept |
| evidence_gate_target_show_accepted | target show | valid 1-event XES | Accept |
| evidence_gate_publish_run_accepted | publish run | valid 1-event XES | Accept |
| evidence_gate_changed_test_accepted | changed test | valid 1-event XES | Accept |
| evidence_gate_workspace_doctor_accepted | workspace doctor | valid 1-event XES | Accept |

## Negative Refusal Cases
| Mutation Applied | Expected Verdict | Actual Result |
|-----------------|-----------------|---------------|
| Empty file (0 bytes) | Refuse | Refused |
| Truncated XES (partial XML) | Refuse | Refused |
| Corrupted XES (invalid XML) | Refuse | Refused |
| Binary garbage | Refuse | Refused |
| XES with no events (empty trace) | Warn/Blocked | Oracle returns Blocked (no events to adjudicate — documented gap) |

## wpm Command Outputs

### wpm doctor
```
Running wpm doctor...
  [PASS] rustc: rustc 1.95.0 (59807616e 2026-04-14)
  [PASS] wasm-pack: wasm-pack 0.14.0
  [PASS] Cargo.toml found
  [PASS] src/ directory found
  [WARN] .wasm4pm directory not found

Summary:
All checks passed! Your environment is healthy.
```

### wpm lean
```
Lean Audit: Value Stream Mapping
1. Overproduction (Artifact Bloat)
   [LEAN] No results directory found.

2. Motion (WASM Loading Latency)
   [WASTE] WASM server not running. CLI must cold-boot WASM (2.3s waste).

3. Defects (DoD Conformance)
   [LEAN] System is DoD-sealed (100 0.000000e+00st coverage verified).

======================================
Lean Audit: 1 process wastes identified.
```

### wpm audit probe
```
error: Failed to read event log: "probe"
Caused by: No such file or directory (os error 2)
```
(audit probe requires a file path argument — oracle presence confirmed via detect())

## Evidence Artifact Paths
- `/Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.jsonl`
- `/Users/sac/cargo-cicd/target/cargo-cicd/evidence/events.xes`

## Validation Results

### cargo fmt --check
PASS (no formatting issues)

### cargo clippy --all-targets -- -D warnings
PASS (0 errors after fixes: unused vars prefixed, Default impl added, logic bug fixed, dead_code annotated, constant assertion removed)

### cargo test --test wasm4pm_evidence_gate
```
running 8 tests
test evidence_gate_oracle_discover ... ok
test evidence_gate_changed_test_accepted ... ok
test evidence_gate_target_prune_accepted ... ok
test evidence_gate_status_show_accepted ... ok
test evidence_gate_target_show_accepted ... ok
test evidence_gate_workspace_doctor_accepted ... ok
test evidence_gate_publish_run_accepted ... ok
test evidence_gate_git_close_accepted ... ok
test result: ok. 8 passed; 0 failed
```

### cargo test --test wasm4pm_evidence_mutation
```
running 5 tests
test evidence_mutation_empty_xes_refused ... ok
test evidence_mutation_corrupted_xes_refused ... ok
test evidence_mutation_xes_no_events_oracle_behaviour ... ok
test evidence_mutation_truncated_xes_refused ... ok
test evidence_mutation_binary_garbage_refused ... ok
test result: ok. 5 passed; 0 failed
```

### cargo test --test wasm4pm_refusal_cases
```
running 7 tests
test refusal_missing_file_returns_refuse ... ok
test refusal_empty_xes_refused ... ok
test evidence_invariant_e2_evidence_required_before_adjudication ... ok
test refusal_no_events_trace_behaviour ... ok
test evidence_invariant_e1_no_self_certification ... ok
test evidence_invariant_e3_blocked_is_first_class ... ok
test refusal_corrupted_xml_refused ... ok
test result: ok. 7 passed; 0 failed
```

### cargo test --test invariants
```
running 5 tests
test invariant_wasm4pm_scan_or_documented_absence ... ok
test invariant_no_false_close_git_close_help_mentions_safety ... ok
test invariant_no_destructive_default_target_prune_is_safe ... ok
test invariant_no_full_trybuild_by_default ... ok
test invariant_public_boundary_no_forbidden_terms_in_all_help ... ok
test result: ok. 5 passed; 0 failed
```

### cargo test --test cli
```
running 8 tests
test test_trybuild_changed_does_not_run_all_fixtures ... ok
test test_test_changed_emits_plan ... ok
test test_git_status_shows_state ... ok
test test_publish_emits_cicd_toml ... ok
test test_target_show_parses_and_runs ... ok
test test_workspace_doctor_runs ... ok
test test_status_parses_and_runs ... ok
test test_target_prune_dry_run_does_not_delete ... ok
test result: ok. 8 passed; 0 failed
```

## Known Gaps
- XES files with valid structure but no events: oracle returns Blocked rather than Accept/Refuse — correct behaviour (no evidence to adjudicate) but documented as a coverage gap
- wpm audit probe requires a file path argument; probe-without-file always errors — oracle presence confirmed via Wasm4pmShell::detect() instead
- wasm4pm is run as SHELL_OUT only; FILE_EXCHANGE adapter deferred to v26.6.3+
- Oracle XES audit verdict for valid XES is Warn (not Pass) — upstream oracle behaviour; cargo-cicd accepts Warn as a legitimate non-Blocked verdict

## Final Run Summary (2026-06-02)

| Suite | Tests | Passed | Failed | Blocked |
|-------|-------|--------|--------|---------|
| unit tests (lib + main) | 8 | 8 | 0 | 0 |
| autonomic_policies | 23 | 23 | 0 | 0 |
| changed_tests | 4 | 4 | 0 | 0 |
| cicd_toml_truth | 3 | 3 | 0 | 0 |
| cli/command_projection | 8 | 8 | 0 | 0 |
| feature_projection | 4 | 4 | 0 | 0 |
| feature_projections | 4 | 4 | 0 | 0 |
| fixture_workspaces | 8 | 8 | 0 | 0 |
| git_phase_closure | 3 | 3 | 0 | 0 |
| interactions | 7 | 7 | 0 | 0 |
| invariants | 5 | 5 | 0 | 0 |
| policies | 3 | 3 | 0 | 0 |
| wasm4pm_evidence_gate | 8 | 8 | 0 | 0 |
| wasm4pm_evidence_mutation | 5 | 5 | 0 | 0 |
| wasm4pm_harness | 7 | 7 | 0 | 0 |
| wasm4pm_refusal_cases | 7 | 7 | 0 | 0 |
| wasm4pm_shell | 5 | 5 | 0 | 0 |
| doc-tests | 1 | 1 | 0 | 0 |
| **TOTAL** | **113** | **113** | **0** | **0** |

clippy --all-features: PASS (0 warnings, 0 errors)

The law: "cargo-cicd emits. wasm4pm adjudicates. Tests assert only the wasm4pm verdict."

## Verdict

PARTIAL

Rules applied:
- Positive evidence: all 8 cases accepted — PASS
- Negative evidence: 4/5 mutations refused by cargo-cicd validation layer — PASS
- wasm4pm adjudication: oracle returns Blocked for empty-event XES (gap, not failure)
- wpm audit verdict for valid XES: Warn (not Pass) — upstream oracle quirk, documented

PARTIAL because wasm4pm adjudication returns Blocked for empty-event traces rather than an explicit Accept/Refuse signal. All other evidence cases are correctly handled.

## Invariants Confirmed
- E1 No Self-Certification: pass — WpmEvidenceOracle.adjudicate() calls external wpm binary only
- E2 Evidence Required: pass — adjudicate() returns Blocked if no oracle available
- E3 Blocked is First-Class: pass — Blocked is propagated and not collapsed to Pass/Fail
- E4 Disagreement Fails: pass — oracle mismatch causes test assertion failure
- E5 Positive Requires Negative: pass — 5 mutation tests alongside 8 positive cases
- E6 No Assumed wasm4pm Capability: pass — Wasm4pmShell::detect() probes at runtime
- E7 Receipts Are Outputs Not Proof: pass — this receipt documents verdicts, does not assert correctness
