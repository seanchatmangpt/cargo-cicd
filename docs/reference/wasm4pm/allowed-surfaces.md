# wasm4pm Allowed Surfaces — cargo-cicd v26.6.2

Surfaces classified USE_AS_IS from the wasm4pm Full Capability Map.

## Primary Gate

| Command | Usage | Exit Code |
|---------|-------|-----------|
| `wpm receipt doctor --format json --strict <receipt.json>` | Runtime receipt adjudication gate | 0 = Admitted, 1 = Refused |

This is the **primary runtime adjudication gate**. `publish_ready = true` requires this command to return exit 0.

## Secondary Check

| Command | Usage | Exit Code |
|---------|-------|-----------|
| `wpm audit <file.xes>` | XES log health check | 0 = Pass/Warn, 1 = Fail |

XES audit is evidence health, not the release court.

## Receipt Format

cargo-cicd emits OCEL 2.0 compliant receipts to `target/cargo-cicd/evidence/receipts/latest.json`.

The receipt includes:
- `algorithms[0].expected_path.expected_ocel2` — declared cargo-cicd process model
- `algorithms[0].observed_path.observed_ocel2` — actual runtime events
- `boundary_evidence.exit_code` + `boundary_evidence.command` — boundary proof
- Hash fields intentionally absent (CanonicalHashVerifier skipped; correctness via structure)
- No `alignment`, `challenge_nonce`, `runtime_observer`, or `all_real` fields

Single-pass adjudication: `ReceiptDoctor::emit_and_adjudicate()` writes the receipt and calls
`wpm receipt doctor --format json --strict` once. Exit 0 = Admitted, exit 1 = Refused.

## Evidence Files

| Path | Format | Contents |
|------|--------|----------|
| `target/cargo-cicd/evidence/events.xes` | XES | Accumulated process events |
| `target/cargo-cicd/evidence/events.jsonl` | JSONL | Machine-readable companion |
| `target/cargo-cicd/evidence/receipts/latest.json` | JSON | `Wasm4pmExecutionReceipt.v1` |
