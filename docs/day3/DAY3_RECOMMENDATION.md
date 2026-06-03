# Day 3 Recommendation — cargo-cicd v26.6.2

**Generated:** 2026-06-02
**Branch:** main (ec59465)

---

## Recommended First Target

**Candidate A: Extend CICD-WPM-004 regression protection to the publish path**

Specifically: confirm and test that `publish.rs` reads the correct JSON key (`overall_fitness`, per `wpm-verdict-v1.json`) from `wpm receipt doctor` output, and cannot produce a false Admitted verdict from a wrong-key JSON response.

---

## Rationale

1. **Highest FruitScore (15.0) of all candidates.** All infrastructure is already passing: `WpmVerdict::authoritative_fitness()`, CICD-WPM-004 diagnostic code, `wpm-verdict-v1.json` schema contract, and 5 unit tests in `diagnostics_verdict_key.rs`. Day3 work extends this protection one hop — from the schema parser to the publish verb path.

2. **No prerequisites.** Unlike Candidate B (requires Law 5 fix) or Candidate D (requires Admitted path stability), Candidate A can begin immediately from a clean working tree.

3. **Protects the release gate before touching it.** Candidate D (publish gate dry-run) is the higher-impact target but must not be attempted until the key contract in `publish.rs` is confirmed. Candidate A is the required safety step before Candidate D.

4. **Narrowly scoped.** The entire Day3 execution for Candidate A fits in one `src/` read, one potential one-line fix, and one new fixture test file.

---

## Bounded Scope

This recommendation is scoped to:
- `src/nouns/publish.rs` — read and verify the JSON key used in ReceiptDoctorVerdict extraction
- `schemas/wpm-verdict-v1.json` — the authoritative key contract
- One new test file: `tests/receipt_doctor_key_contract.rs` — schema fixture test

Out of scope for this target:
- `cargo publish --dry-run` invocation (Candidate D)
- LSP `backend.rs` capability fix (Candidate B prerequisite)
- Any changes to evidence emission or XES format

---

## Safe Execution Steps

### Step 1: Confirm oracle is present

```sh
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate 2>&1 | tail -5
```

If this fails (wpm absent or returns non-zero for valid XES), stop and rebuild wpm before proceeding.

### Step 2: Read the publish path key usage

Read `src/nouns/publish.rs` and locate the `serde_json::Value::get(...)` call that extracts the ReceiptDoctor verdict state. Confirm the key name matches `schemas/wpm-verdict-v1.json`.

### Step 3: Fix or confirm

- If the key matches: proceed to Step 4.
- If the key does not match: apply a one-line fix to `publish.rs`, run `cargo make check`, confirm no compilation errors.

### Step 4: Write schema fixture test

Create `tests/receipt_doctor_key_contract.rs` with:
- A JSON blob using the wrong key (e.g. `"fitness": 0.9` instead of `"overall_fitness": 0.9`) — assert verdict is NOT Admitted.
- A JSON blob using the correct key (`"overall_fitness": 0.9`) — assert verdict is Admitted.
- A JSON blob with both keys present — assert `overall_fitness` wins.

### Step 5: Run the new test

```sh
cargo test --test receipt_doctor_key_contract
```

All assertions must pass.

### Step 6: Emit evidence

```sh
cargo cicd publish run  # in a fixture workspace — confirm XES emitted
```

Verify `target/cargo-cicd/evidence/events.xes` exists and is well-formed.

### Step 7: Oracle adjudication (optional, local only)

```sh
REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate
```

Confirm Accept verdict for valid XES.

---

## Expected Receipts

After completing the above steps, the following receipts should exist:

| Receipt | Location | Assertion |
|---|---|---|
| Test passage | `tests/receipt_doctor_key_contract.rs` | 3/3 pass |
| XES emission | `target/cargo-cicd/evidence/events.xes` | File exists, non-empty, well-formed XML |
| Compilation | `cargo make check` | Zero errors, zero warnings |
| Oracle verdict (if REQUIRE_WPM_ORACLE=1) | stdout of wasm4pm_evidence_gate | Accept |

---

## After Candidate A: Next Step

Once Candidate A receipts are confirmed:

1. Fix Law 5 (`backend.rs` capability function — one line).
2. Proceed to Candidate B (LSP editor proof — `tests/lsp_initialize_fixture.rs`).
3. Candidate D (publish gate dry-run) is the final Day3 target if time permits.

---

## What Success Looks Like

Day3 is complete when:
- `tests/receipt_doctor_key_contract.rs` — 3/3 pass
- `cargo make check` — clean
- `cargo make test` — all prior tests still pass (no regression)
- `target/cargo-cicd/evidence/events.xes` — emitted and well-formed
- `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate` — Accept verdict confirmed
