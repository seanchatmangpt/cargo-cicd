# Receipt: wasm4pm Full Capability Map Adoption — cargo-cicd v26.6.2

## Verdict: ACCEPTED

## Key Changes

1. **Primary gate corrected**: `wpm receipt doctor --format json --strict` replaces `wpm audit` as the primary runtime adjudication gate.
2. **DO_NOT_USE surfaces excluded**: `wpm doctor`, `wpm lean`, stub oracles removed from all CI gates.
3. **New `evidence doctor` command**: `cargo cicd evidence doctor` adjudicates the latest receipt.
4. **Publish gate upgraded**: `cargo cicd publish run` now gates on `wpm receipt doctor` acceptance.
5. **Capability map documented**: `docs/wasm4pm/WASM4PM_CAPABILITY_MAP.md`, `WASM4PM_ALLOWED_SURFACES.md`, `WASM4PM_EXCLUDED_SURFACES.md`.

## Evidence

| Gate | Result |
|------|--------|
| `cargo cicd evidence doctor` | ACCEPTED (Admitted) |
| `cargo cicd publish run` | RECEIPT_DOCTOR:accepted |
| `wpm receipt doctor --strict latest.json` | `state: Admitted` |
| All tests | 121 passed, 0 failed |
| clippy | CLEAN |
| playground 8/8 | PASS |

## wpm Binary

Path: `/Users/sac/wasm4pm/target/release/wpm`

## Excluded Surfaces (DO_NOT_USE)

- `wpm doctor` as CI gate (exits 0 even when checks fail)
- `wpm lean` as validation gate (not machine-parseable)
- `wpm mining conformance` (confirmed stub)
- `wpm oracle check` / `wpm oracle watch` (confirmed stubs)
