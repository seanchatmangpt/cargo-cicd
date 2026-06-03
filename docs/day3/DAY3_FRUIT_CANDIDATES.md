# Day 3 Fruit Candidates — cargo-cicd v26.6.2

**Generated:** 2026-06-02
**Branch:** main (ec59465)
**Baseline:** 155+ tests passing, zero failures

---

## Scoring Formula

```
FruitScore = (Impact × ProofReadiness × UserVisibility) / (Risk × Scope)
```

Scale 1–5 each axis. Higher score = lower-hanging fruit.

---

## Top 3 Candidates

### Rank 1 — Candidate A: CICD-WPM-004 verdict key regression (publish path extension)

**FruitScore: 15.0**

| Axis | Score | Rationale |
|---|---|---|
| Impact | 3 | Prevents silent 0.0 fitness masking non-conformance in the publish path |
| ProofReadiness | 5 | 5 passing tests already in `diagnostics_verdict_key.rs`; schema struct in `cargo-cicd-core`; CICD-WPM-004 registered |
| UserVisibility | 2 | Schema-level protection; invisible to CLI users directly |
| Risk | 1 | No new code needed for LSP side; extend only to `publish.rs` key read |
| Scope | 2 | Narrow: confirm `ReceiptDoctorVerdict` in `publish.rs` reads `overall_fitness`, not `fitness` |

**What exists:** `WpmVerdict::authoritative_fitness()`, 5 unit tests, CICD-WPM-004 diagnostic code, `wpm-verdict-v1.json` schema contract.

**What is missing:** Explicit test asserting `publish.rs` ReceiptDoctor path cannot produce a false Admitted verdict from wrong-key JSON.

**Day3 execution steps:**
1. Read `src/nouns/publish.rs` lines handling ReceiptDoctorVerdict.
2. Confirm `serde_json::Value::get` key matches `wpm-verdict-v1.json` contract.
3. If mismatched: one-line fix. If matched: add a schema-fixture test asserting the contract.
4. Emit ProcessEvent, verify with XES emit → wpm audit pattern.

---

### Rank 2 — Candidate B: LSP editor proof — diagnostic JSON from real workspace fixture

**FruitScore: 6.67**

| Axis | Score | Rationale |
|---|---|---|
| Impact | 4 | Closes gap between "unit tests pass" and "editor receives diagnostics" |
| ProofReadiness | 3 | `backend.rs` complete; `run_all(WorkspaceSnapshot)` tested; gap is wire-level JSON-RPC proof |
| UserVisibility | 5 | Direct editor integration proof |
| Risk | 3 | tower-lsp JSON-RPC framing is non-trivial; Law 5 capability defect must be fixed first |
| Scope | 3 | Medium: fixture workspace + Content-Length framing wrapper + initialize + didOpen flow |

**Prerequisite (Law 5 fix):** `backend.rs` uses `server_capabilities()` (no `diagnosticProvider`) instead of `build_server_capabilities()` (declares `DiagnosticServerCapabilities`). One-line fix in `backend.rs` before any test is written.

**Day3 execution steps:**
1. Fix `backend.rs`: change `server_capabilities()` → `build_server_capabilities()`.
2. `tests/lsp_initialize_fixture.rs`: spawn binary, send `initialize` JSON-RPC, assert `capabilities.diagnosticProvider` present.
3. `tests/lsp_did_open_fixture.rs`: send `textDocument/didOpen`, assert `publishDiagnostics` notification or clean shutdown.
4. Emit XES evidence, run `wpm audit` under `REQUIRE_WPM_ORACLE=1`.

---

### Rank 3 — Candidate D: Publish gate dry-run — invoke `cargo publish --dry-run` after Admitted verdict

**FruitScore: 4.69**

| Axis | Score | Rationale |
|---|---|---|
| Impact | 5 | Closes actual release gate: Admitted receipt → dry-run proceeds; Refused/Blocked → dry-run skipped |
| ProofReadiness | 3 | `ReceiptDoctor` exists; `publish.rs` calls it; CICD-PUBLISH-002 defined; dry-run invocation absent |
| UserVisibility | 5 | Directly visible: publish halts loudly on non-Admitted verdict |
| Risk | 4 | Touching the publish gate; Refused path must not break Admitted path |
| Scope | 4 | Broad: dry-run invocation + error handling + Admitted vs Refused fixture tests |

**What is missing:** `cargo publish --dry-run` is never invoked by the publish verb. The Admitted path proceeds to emit a receipt but does not verify the crate would actually publish.

**Day3 execution steps (scoped minimum):**
1. Add `cargo publish --dry-run` invocation in `publish.rs` after `ReceiptDoctorVerdict::Admitted`.
2. Add test: Blocked verdict → dry-run not invoked.
3. Add test: Admitted verdict (mocked or real wpm) → dry-run invoked and result reported.
4. Emit XES for both paths.

---

## Summary Table

| Rank | Candidate | FruitScore | Status | Prerequisite |
|---|---|---|---|---|
| 1 | A: CICD-WPM-004 publish path | 15.0 | LIVE — extend to publish path | None |
| 2 | B: LSP editor proof | 6.67 | PARTIAL — Law 5 fix required | Fix `backend.rs` capability function |
| 3 | D: Publish gate dry-run | 4.69 | PARTIAL — dry-run invocation missing | Admitted path must be stable first |

Candidates C (conformance precision UNSUPPORTED declaration) and E (Spec Kit integration) are excluded from top 3. C is a one-liner with no standalone Day3 value; E has no seam and is deferred.

---

## Recommended First Target

**Candidate A: Extend CICD-WPM-004 regression protection to the publish path.**

Rationale: highest FruitScore, no prerequisites, all infrastructure already passing, narrowly scoped to one confirmation plus one new fixture test. Candidate B is higher user-visible value but carries a mandatory prerequisite and non-trivial JSON-RPC framing work. Day3 begins with A to secure the publish path key contract, then proceeds to B once Law 5 is fixed.

See `DAY3_RECOMMENDATION.md` for bounded execution steps.
