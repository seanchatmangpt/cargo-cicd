# Receipt: Runtime Receipt Doctor — cargo-cicd v26.6.2

## Verdict: ACCEPTED

## Architecture

```
cargo-cicd command
→ emits ProcessEvent (start + complete, real UTC timestamp)
→ appends to target/cargo-cicd/evidence/events.jsonl
→ rebuilds target/cargo-cicd/evidence/events.xes

cargo cicd evidence doctor
→ build_receipt_json(events, command, 0)  — Wasm4pmExecutionReceipt.v1
→ wpm receipt doctor --format json --strict latest.json
→ state: Admitted

cargo cicd publish run
→ ReceiptDoctor::emit_and_adjudicate()
→ wpm receipt doctor --format json --strict
→ RECEIPT_DOCTOR:accepted → publish proceeds
```

## Receipt Format

`target/cargo-cicd/evidence/receipts/latest.json` — `algorithms`-based OCEL2 receipt.
`CanonicalHashVerifier` is skipped (no `receipt_hash` field) — structural correctness only.

## Gate Rules

- `publish_ready = true` only if `wpm receipt doctor --strict` returns Admitted
- Blocked (wpm unavailable) → proceed with warning
- Refused → publish is blocked (AndonPull)

## wpm Command Used

```bash
wpm receipt doctor --format json --strict target/cargo-cicd/evidence/receipts/latest.json
```

## Sample Output

```json
{
  "state": "Admitted",
  "findings": [],
  "denied_paths": [],
  "doctor_report_hash": "d53d18c23212ea7b6300594bb89bce60218f6eff2b9d628b8cc42d3e79bbd5ab"
}
```

## Test Coverage

- All existing 119+ tests pass
- `cargo cicd evidence doctor` ACCEPTED
- `cargo cicd publish run` RECEIPT_DOCTOR:accepted
