# wasm4pm Full Capability Map — cargo-cicd v26.6.2

Architecture of wasm4pm integration based on capability map analysis.

## Evidence Flow

```
cargo-cicd command
  → emits ProcessEvent (start + complete)
  → appends to target/cargo-cicd/evidence/events.jsonl
  → rebuilds target/cargo-cicd/evidence/events.xes

cargo cicd evidence doctor
  → assembles Wasm4pmExecutionReceipt.v1
  → two-pass BLAKE3 bootstrap via wpm
  → writes target/cargo-cicd/evidence/receipts/latest.json
  → wpm receipt doctor --format json --strict latest.json
  → Admitted / Refused

cargo cicd publish run
  → calls ReceiptDoctor::emit_and_adjudicate()
  → gates publish_ready = true on Admitted verdict
```

## Capability Classification

### USE_AS_IS

| Surface | Role |
|---------|------|
| `wpm receipt doctor --format json --strict` | Primary runtime adjudication gate |
| `wpm audit <file.xes>` | Secondary XES log health check |
| `wpm --version` | Binary presence probe |

### DO_NOT_USE

See `WASM4PM_EXCLUDED_SURFACES.md`.

## Receipt Schema

cargo-cicd produces `Wasm4pmExecutionReceipt.v1` with these fields:

```json
{
  "receipt_type": "Wasm4pmExecutionReceipt",
  "receipt_schema": "Wasm4pmExecutionReceipt.v1",
  "package": "cargo-cicd",
  "version": "26.6.2",
  "commit": "<git short head>",
  "hash_algorithm": "BLAKE3",
  "time_basis": "LogicalMonotonicClock",
  "canonicalization": {"name": "CanonicalOCEL2ForWasm4pm", "version": 1, ...},
  "example_id": "cargo-cicd-evidence",
  "input": {"event_log_hash": "<xes hash>", "event_log_format": "xes", ...},
  "algorithms": [],
  "algorithm_count": 0,
  "commands_observed": ["status:show", "target:show", ...],
  "created_at": "<ISO-8601>",
  "previous_receipt_hash": null,
  "receipt_hash": "<BLAKE3 bootstrapped via wpm>"
}
```

The `receipt_hash` is bootstrapped via wpm's own BLAKE3 computation, not an external library.

## Verdict Semantics

| wpm exit code | Meaning |
|---------------|---------|
| 0 | Admitted (no Deny findings) |
| 1 | Refused (Deny findings present or state=Refused) |
| absent | Blocked (binary not found) |

Blocked is a first-class expectation, not an error. Tests that run without wpm installed declare `Blocked` as their expected verdict (invariant E7).
