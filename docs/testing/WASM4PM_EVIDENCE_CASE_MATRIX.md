# wasm4pm Evidence Gate — Case Matrix

**cargo-cicd v26.6.2**

All evidence cases from the three gate test files.

- Positive acceptance cases: `tests/wasm4pm_evidence_gate.rs`
- Mutation (negative) cases: `tests/wasm4pm_evidence_mutation.rs`
- Refusal ledger cases: `tests/wasm4pm_refusal_cases.rs`

---

## Positive Acceptance Cases

| Test | Command | Evidence Fields | Mutation | Expected Verdict | Actual Status |
|---|---|---|---|---|---|
| `evidence_gate_status_show_accepted` | `status show` | command=`status show`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_target_show_accepted` | `target show` | command=`target show`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_target_prune_accepted` | `target prune plan` | command=`target prune plan`, verdict=`DRY-RUN` | none | Accept (or Blocked) | PASS |
| `evidence_gate_changed_test_accepted` | `test changed` | command=`test changed`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_git_close_accepted` | `git close` | command=`git close`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_publish_run_accepted` | `publish run` | command=`publish run`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_workspace_doctor_accepted` | `workspace doctor` | command=`workspace doctor`, verdict=`PASS` | none | Accept (or Blocked) | PASS |
| `evidence_gate_oracle_discover` | n/a (oracle health) | none | none | No panic (E7 compliance) | PASS |

---

## Mutation (Negative) Cases

Each test emits valid XES then corrupts it before oracle adjudication.

| Test | Command | Evidence Fields | Mutation | Expected Verdict | Actual Status |
|---|---|---|---|---|---|
| `evidence_mutation_corrupted_xes_refused` | n/a | Raw content: `"NOT VALID XML AT ALL"` | Bypasses emission; writes invalid XML directly | Refuse (or Blocked) | PASS |
| `evidence_mutation_empty_xes_refused` | n/a | Raw content: empty bytes | Bypasses emission; writes zero bytes | Refuse (or Blocked) | PASS |
| `evidence_mutation_xes_no_events_oracle_behaviour` | n/a | Valid XES structure, no events in trace | Bypasses `emit_xes`; writes well-formed XES with empty trace | Accept (or Blocked or Refuse) — oracle behaviour documented | PASS |
| `evidence_mutation_binary_garbage_refused` | n/a | Raw content: `\x00\x01\x02\xff\xfe NOT XML` | Bypasses emission; writes binary garbage | Refuse (or Blocked) | PASS |
| `evidence_mutation_truncated_xes_refused` | `status show` | command=`status show`, verdict=`PASS` | Truncate file to first 20 bytes after `emit_xes` | Refuse (or Blocked) | PASS |

---

## Refusal Ledger Cases

| Test | Command | Evidence Fields | Mutation | Expected Verdict | Actual Status |
|---|---|---|---|---|---|
| `refusal_corrupted_xml_refused` | n/a | Raw content: `"THIS IS NOT XML"` | Writes invalid XML directly | Refuse (or Blocked) | PASS |
| `refusal_empty_xes_refused` | n/a | Raw content: empty bytes | Writes zero bytes directly | Refuse (or Blocked) | PASS |
| `refusal_missing_file_returns_refuse` | n/a | Path: `/nonexistent/path/missing.xes` | File does not exist | Refuse (or Blocked) | PASS |
| `refusal_no_events_trace_behaviour` | n/a | Valid XES, empty trace | Bypasses `emit_xes`; no events in trace | Accept or Refuse or Blocked — behaviour documented, not asserted | PASS |

---

## Invariant Structural Tests

| Test | Invariant | What it proves | Actual Status |
|---|---|---|---|
| `evidence_invariant_e1_no_self_certification` | E1 | `emit_xes` returns `Result<()>` — no verdict path exists without oracle | PASS |
| `evidence_invariant_e2_evidence_required_before_adjudication` | E2 | XES file does not exist before `emit_xes`; exists after | PASS |
| `evidence_invariant_e3_blocked_is_first_class` | E3 | `assert_wpm_verdict` with `Blocked` does not panic when oracle unavailable | PASS |

---

## Notes

- "Accept (or Blocked)" means: when wpm binary is present, verdict is Accept; when absent, verdict is Blocked (invariant E7).
- "Refuse (or Blocked)" means: when wpm binary is present, verdict is Refuse; when absent, verdict is Blocked (invariant E7).
- `evidence_mutation_xes_no_events_oracle_behaviour` and `refusal_no_events_trace_behaviour` document observed oracle behaviour — wpm accepts well-formed XES even with an empty trace (exit 0). No hard assertion is made; this is exploratory.
- The `evidence_gate_oracle_discover` test calls `oracle.is_available()` and optionally `audit_xes` on a nonexistent path. Neither must panic.
