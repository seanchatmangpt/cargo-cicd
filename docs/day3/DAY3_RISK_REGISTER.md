# Day 3 Risk Register — cargo-cicd v26.6.2

**Generated:** 2026-06-02
**Branch:** main (ec59465)

---

## Risk Scoring

- **Likelihood:** 1 (rare) – 5 (near-certain)
- **Impact:** 1 (cosmetic) – 5 (release blocked / data loss)
- **Priority = Likelihood × Impact**

---

## Risk 1: Oracle absent in clean environment — Accept path never exercised

| Field | Value |
|---|---|
| Risk | All 9 evidence gate tests pass via the Blocked path. The Accept branch (wpm returns exit 0 for valid XES) is never exercised in default `cargo test`. |
| Likelihood | 3 — wpm binary is present locally but may be absent in CI or after a repo clean |
| Impact | 4 — Release closure cannot be claimed without an Accept verdict from the oracle |
| Priority | 12 |
| Mitigation | At Day3 start, run `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate` to confirm `/Users/sac/wasm4pm/target/release/wpm` is present and returns a valid verdict for conforming XES. If it fails, rebuild wpm before proceeding with any gate work. |

---

## Risk 2: Capability duplicate (Law 5) causes LSP fixture test failure for wrong reason

| Field | Value |
|---|---|
| Risk | `backend.rs` uses `server_capabilities()` (no `diagnosticProvider`) instead of `build_server_capabilities()`. If Day3 LSP work proceeds without fixing this, the fixture test will assert a capability that is not advertised and fail — but the failure message will not clearly identify the root cause. |
| Likelihood | 5 — The defect is confirmed present; will reproduce every time |
| Impact | 3 — LSP Candidate B blocked until fixed; LSP unit tests still pass (they do not test InitializeResult) |
| Priority | 15 |
| Mitigation | Fix `backend.rs` as the first commit of Day3 before writing any fixture test. The fix is one line: change `server_capabilities()` to `build_server_capabilities()`. Verified by reading `backend.rs` line that builds `InitializeResult`. |

---

## Risk 3: `ReceiptDoctor` path in `publish.rs` reads wrong JSON key

| Field | Value |
|---|---|
| Risk | `publish.rs` extracts the verdict state from wpm JSON output via a raw `serde_json::Value::get("state")` call. If the `wpm receipt doctor` output schema uses a different key (e.g. `status` or `verdict`), this silently falls back to a Blocked/Admitted mismatch. This is the same key-mismatch pattern CICD-WPM-004 was designed to catch, but in a location not covered by the existing LSP tests. |
| Likelihood | 2 — wpm schema has been stable; mismatch would require a wpm update |
| Impact | 4 — Silent incorrect Admitted verdict means a crate with non-conforming evidence can proceed to publish |
| Priority | 8 |
| Mitigation | Add a `ReceiptDoctorVerdict` schema fixture test analogous to `diagnostics_verdict_key.rs`: provide a JSON blob with the wrong key and assert verdict is NOT Admitted. Candidate A Day3 work covers this directly. |

---

## Risk 4: JSON-RPC framing complexity causes flaky LSP fixture tests

| Field | Value |
|---|---|
| Risk | tower-lsp communicates over stdio using Content-Length framing. A naive `write_all + read_to_string` in a Rust test process will hang indefinitely waiting for EOF. Without correct framing, the LSP fixture test for Candidate B will block the test runner. |
| Likelihood | 4 — This is a known tower-lsp integration trap; no existing framing wrapper exists in the repo |
| Impact | 2 — Test suite hangs (not a data loss); fixable by adding timeout or correct framing |
| Priority | 8 |
| Mitigation | Use a minimal Content-Length framing wrapper function in the fixture test (< 20 lines). Write the `initialize` request, close stdin immediately after, read one framed response. Alternatively, use `BufReader` with a line-delimited JSON-RPC mode if tower-lsp is configured for it. Scope the first fixture test to `InitializeResult` only — do not attempt `didOpen` until framing is confirmed working. |

---

## Risk 5: `wpm receipt doctor` Blocked path proceeds silently to publish

| Field | Value |
|---|---|
| Risk | When `ReceiptDoctor::discover()` returns None (oracle absent), `publish.rs` logs a warning to stderr and continues. This means `cargo cicd publish run` can complete without any adjudicated receipt. In a CI environment without wpm, every publish is effectively unadjudicated. |
| Likelihood | 3 — Any CI environment without wpm binary triggers this path |
| Impact | 3 — Release is not oracle-adjudicated; the system degrades silently rather than failing loudly |
| Priority | 9 |
| Mitigation | Emit CICD-WPM-001 as an active Error diagnostic when the oracle is unavailable during publish. This surfaces the gap to the LSP and makes it non-silent. The `REQUIRE_WPM_ORACLE=1` environment variable can be used to make the publish verb hard-fail when oracle is absent in environments where its presence is guaranteed. |

---

## Priority Summary

| Risk | Priority | Status |
|---|---|---|
| R2: Capability duplicate (Law 5) | 15 | Active defect — fix first |
| R1: Oracle Accept path unexercised | 12 | Mitigation: REQUIRE_WPM_ORACLE=1 |
| R5: Silent unadjudicated publish | 9 | Mitigation: CICD-WPM-001 emission |
| R3: ReceiptDoctor wrong key | 8 | Mitigation: Candidate A schema fixture |
| R4: JSON-RPC framing flakiness | 8 | Mitigation: Content-Length wrapper |
