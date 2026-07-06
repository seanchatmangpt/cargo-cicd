# wasm4pm Refusal Ledger

**cargo-cicd v26.6.2**

This ledger records all proven refusal patterns. Each entry documents the corruption applied to
a valid (or invalid) evidence artifact, the expected wasm4pm verdict, and what the refusal proves.

Source files: `tests/wasm4pm_evidence_mutation.rs`, `tests/wasm4pm_refusal_cases.rs`

---

## Refusal Proofs

| Case | Corruption Type | Evidence State | wpm Command | Expected: REFUSE | Actual |
|---|---|---|---|---|---|
| `evidence_mutation_corrupted_xes_refused` | Not-XML text | `"NOT VALID XML AT ALL"` written directly | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `evidence_mutation_empty_xes_refused` | Empty file | Zero bytes written directly | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `evidence_mutation_binary_garbage_refused` | Binary garbage | `\x00\x01\x02\xff\xfe NOT XML` written directly | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `evidence_mutation_truncated_xes_refused` | Truncated mid-element | Valid XES emitted, then truncated to 20 bytes | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `refusal_corrupted_xml_refused` | Not-XML text | `"THIS IS NOT XML"` written directly | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `refusal_empty_xes_refused` | Empty file | Zero bytes written directly | `wpm audit` | REFUSE (or BLOCKED) | PASS |
| `refusal_missing_file_returns_refuse` | Missing file | Path `/nonexistent/path/missing.xes` does not exist | `wpm audit` | REFUSE (or BLOCKED) | PASS |

---

## Mutation Helper Inventory

The following helper functions are exported `pub` from `tests/wasm4pm_evidence_mutation.rs` for
use in composed refusal tests:

| Helper | Corruption Type | Mechanism | Expected wpm Response |
|---|---|---|---|
| `corrupt_xes_contradictory_verdict` | Semantic contradiction | Replace `PASS`/`pass` with `FAIL` in attribute values | Refuse (verdict inconsistent with model) |
| `corrupt_xes_missing_trace` | Structural removal | Strip `<trace>…</trace>` element entirely | Refuse (no process evidence) |
| `corrupt_xes_no_closing_tag` | Malformed XML | Remove `</log>` closing tag | Refuse (XML not well-formed) |
| `corrupt_xes_empty_file` | Empty evidence | Overwrite with zero bytes | Refuse (no evidence) |
| `corrupt_xes_binary_garbage` | Non-parseable | Overwrite with `\x00\x01\x02\xff\xfe \xde\xad\xbe\xef NOT XML` | Refuse (not parseable as XES) |
| `corrupt_xes_truncated` | Truncated mid-element | Truncate to 20 bytes | Refuse (XML incomplete) |
| `corrupt_xes_invalid_attribute` | XML invalidity | Inject unescaped `<` inside attribute value | Refuse (XML not well-formed) |
| `corrupt_xes_wrong_encoding_declaration` | Encoding conflict | Declare `EBCDIC-US` encoding on UTF-8 content | Refuse (encoding mismatch) |

---

## Documented Behaviour Cases (not hard-asserted refusals)

| Case | Evidence State | Oracle Response | Notes |
|---|---|---|---|
| `evidence_mutation_xes_no_events_oracle_behaviour` | Valid XES structure, empty trace, no events | Accept (exit 0) | wpm accepts well-formed XES regardless of event count. Documented, not asserted as Refuse. |
| `refusal_no_events_trace_behaviour` | Valid XES structure, empty trace, no events | Accept or Refuse or Blocked | Same observed behaviour. No verdict asserted — test documents oracle response without mandating it. |

---

## What Each Refusal Proves

**Not-XML / binary garbage:** wasm4pm parses the XES file before adjudication. Non-parseable
content fails at the parse stage. Acceptance is not vacuous.

**Empty file:** Zero bytes produce no parseable XES structure. No evidence is not acceptance.

**Truncated mid-element:** An XML document truncated mid-element is not well-formed. wasm4pm
rejects structurally incomplete evidence.

**Missing file:** An invocation error (file not found) maps to `Refuse` via `audit_xes`. The
absence of evidence is treated as a refusal signal, not a pass.

**`corrupt_xes_missing_trace`:** Removing the `<trace>` element produces an XES log with no
process trace. There is no activity sequence to adjudicate.

**`corrupt_xes_no_closing_tag`:** XML without a closing `</log>` tag is not well-formed XML
and cannot be parsed by any conformant XES parser.

**`corrupt_xes_invalid_attribute`:** An unescaped `<` inside an attribute value is a fatal XML
error. The document is not well-formed.

**`corrupt_xes_wrong_encoding_declaration`:** Declaring `EBCDIC-US` encoding on a UTF-8 file
creates an irreconcilable conflict at the XML parser level.

---

## BLOCKED Note

All refusal cases that depend on `oracle.is_available()` report `Blocked` when the wpm binary
is absent (invariant E7). `Blocked` is not a bypass — it is a first-class gate state.
The refusal proof is only confirmed when wpm is present and returns `Fail`.
