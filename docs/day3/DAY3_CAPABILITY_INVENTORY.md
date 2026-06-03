# Day 3 Capability Inventory — cargo-cicd v26.6.2

**Generated:** 2026-06-02  
**Branch:** main (6aaed05)  
**Test baseline:** all 155+ tests passing, zero failures

---

## 1. CAPABILITY MATRIX

> Status legend: **LIVE** = verified by passing test | **PARTIAL** = some path works, some blocked | **BLOCKED** = code present but execution fails | **STUB** = structure only, no logic | **DORMANT** = feature-flagged, compilable, no runtime exercise | **UNKNOWN** = not verified by any command | **REMOVE** = confirmed dead | **DEFER** = explicitly deferred by code comment

| Surface | Capability | Status | Evidence | Runtime Command | Test | Receipt | Risk | Day3 Relevance | Action |
|---|---|---|---|---|---|---|---|---|---|
| `cargo cicd status show` | Workspace state emission | LIVE | `tests/cli/test_status.rs` passes | `cargo cicd status show` | `cli::test_status` | ProcessEvent emitted to events.jsonl | Low | Low — already solid | Maintain |
| `cargo cicd target show` | Target dir size scan | LIVE | `tests/cli/test_target.rs` passes | `cargo cicd target show` | `cli::test_target` | ProcessEvent emitted | Low | Low | Maintain |
| `cargo cicd target prune` | Dry-run-safe prune | LIVE | invariant_no_destructive_default_target_prune_is_safe passes | `cargo cicd target prune` | `tests/invariants.rs` | ProcessEvent emitted | Low | Low | Maintain |
| `cargo cicd publish run` | cicd.toml emission | LIVE | `tests/cli/test_publish.rs` passes | `cargo cicd publish run` | `cli::test_publish` | ProcessEvent emitted | Medium | High — receipt adjudication gate active | Monitor adjudication path |
| `cargo cicd publish run` | wpm receipt doctor gate | PARTIAL | publish.rs L70-120: oracle discovery + adjudication implemented; passes with `BLOCKED:oracle_unavailable` when wpm absent | `cargo cicd publish run` | `tests/cli/test_publish.rs` | Receipt doctor verdict written | High | High — oracle may be absent in clean envs | Verify oracle path with wpm present |
| `cargo cicd git close` | Phase closure check | LIVE | `tests/git_phase_closure.rs` passes (10 tests) | `cargo cicd git close` | `git_phase_closure` | ProcessEvent emitted | Medium | Medium | Maintain |
| `cargo cicd test changed` | Changed-file test plan | LIVE | `tests/changed_tests.rs` passes | `cargo cicd test changed` | `changed_tests` | ProcessEvent emitted | Low | Low | Maintain |
| `cargo cicd workspace doctor` | Workspace structural check | LIVE | `tests/cicd_toml_truth.rs` passes | `cargo cicd workspace doctor` | `cicd_toml_truth` | ProcessEvent emitted | Low | Low | Maintain |
| `cargo cicd lsp serve` | LSP binary launch | BLOCKED | `src/nouns/lsp.rs` LspServeVerb: checks for `cargo-cicd-lsp` on PATH; not on PATH (`which cargo-cicd-lsp` returns nothing); binary only in `target/debug/` | `cargo cicd lsp serve` | No passing test for serve path | None | Medium | High — serve verb is BLOCKED without install | Add to PATH or add fixture test |
| `cargo cicd lsp doctor` | LSP health check (binary + wpm + Cargo.toml) | PARTIAL | Doctor verb implemented and compiles; cargo-cicd-lsp binary not on PATH so check 1 fails; wpm present at known path so check 2 passes | `cargo cicd lsp doctor` | No dedicated integration test | ProcessEvent emitted | Low | High | Add integration test |
| `cargo cicd lsp explain` | Diagnostic code lookup | LIVE | `src/nouns/lsp.rs` full CICD_CATALOG (30 codes); compiles and runs | `cargo cicd lsp explain CICD-WPM-004` | No CLI-level test (only LSP crate tests) | ProcessEvent emitted | Low | High | Add CLI test for explain |
| `cargo-cicd-lsp` binary | tower-lsp server (initialize) | PARTIAL | `crates/cargo-cicd-lsp/src/server/backend.rs`: Backend implements LanguageServer; `initialize` handler stores root, returns ServerCapabilities; builds cleanly; NOT launched as real LSP process in any test | `cargo-cicd-lsp` (stdio) | LSP crate unit tests pass (5+2+2+2); no end-to-end JSON-RPC test | None | Medium | High | Day3 LSP proof target |
| `cargo-cicd-lsp` binary | `textDocument/publishDiagnostics` | PARTIAL | `analyze_and_publish` implemented in backend.rs; calls `run_all(WorkspaceSnapshot)`; no integration test fires it end-to-end | Requires editor or JSON-RPC client | No end-to-end test | None | Medium | High | Fixture test needed |
| `cargo-cicd-lsp` | Analyzer: git phase | LIVE (unit) | `crates/cargo-cicd-lsp/tests/diagnostics_git.rs` 2/2 pass | — | `diagnostics_git` | None | Low | Medium | Maintain |
| `cargo-cicd-lsp` | Analyzer: evidence | LIVE (unit) | `crates/cargo-cicd-lsp/tests/diagnostics_evidence.rs` 3/3 pass | — | `diagnostics_evidence` | None | Low | Medium | Maintain |
| `cargo-cicd-lsp` | Analyzer: public boundary | LIVE (unit) | `crates/cargo-cicd-lsp/tests/diagnostics_public_boundary.rs` 2/2 pass | — | `diagnostics_public_boundary` | None | Low | Medium | Maintain |
| `cargo-cicd-lsp` | CICD-WPM-004 verdict key protection | LIVE | `crates/cargo-cicd-lsp/tests/diagnostics_verdict_key.rs` 5/5 pass; `WpmVerdict.overall_fitness` is authoritative; `has_precision()` distinguishes null from 0.0 | — | `diagnostics_verdict_key` | None | Low | High | Candidate A — regression now baked in |
| `cargo-cicd-lsp` | Diagnostic lifecycle (raise/clear/residual) | LIVE (unit) | `crates/cargo-cicd-lsp/tests/lifecycle_clear.rs` 2/2 pass | — | `lifecycle_clear` | None | Low | Medium | Maintain |
| `cargo-cicd-lsp` | Duplicate capability function (`initialize.rs` vs `capabilities.rs`) | BLOCKED | Two files both define server capabilities: `initialize.rs::build_server_capabilities()` (declares DiagnosticServerCapabilities) and `capabilities.rs::server_capabilities()` (no diagnostic). Backend uses `capabilities.rs` only — diagnostic capability NOT advertised | — | No test catches this divergence | None | Medium | High | Fix: backend.rs should use `build_server_capabilities()` |
| Evidence emission (XES) | XES file generation | LIVE | `tests/wasm4pm_evidence_gate.rs` 9/9 pass; `emit_xes` called, file asserted to exist | `cargo cicd <any verb>` → events.xes | `wasm4pm_evidence_gate` | XES file in `target/cargo-cicd/evidence/` | Low | High | Maintain |
| Evidence emission (JSONL) | JSONL companion format | LIVE | `src/evidence.rs` append_events; session_id present | — | `refusal_calibration` | events.jsonl | Low | Medium | Maintain |
| wasm4pm oracle | `wpm audit <xes>` | PARTIAL | `Wasm4pmShell::detect()` finds `/Users/sac/wasm4pm/target/release/wpm`; `audit` method implemented; evidence gate tests pass with Blocked when oracle absent; `infer_verdict` maps exit code to WpmVerdict | `wpm audit <file.xes>` | `wasm4pm_evidence_gate` (Blocked path exercised) | WpmResult | High | High | Run `REQUIRE_WPM_ORACLE=1` to force Accept path |
| wasm4pm oracle | `wpm receipt doctor` | PARTIAL | `ReceiptDoctor` in evidence.rs; `emit_and_adjudicate` implemented; called from publish.rs; ReceiptDoctorVerdict::Accepted/Refused/Blocked defined | `cargo cicd publish run` | No dedicated receipt doctor test | Receipt JSON | High | High | Day3 candidate D |
| wasm4pm oracle | `wpm mining conformance` | REMOVE | `wasm4pm_shell.rs` comment: "stubs model loading to DFG::new() — always meaningless" | — | None | None | None | None | Do not use |
| wasm4pm oracle | `wpm oracle check` | REMOVE | `wasm4pm_shell.rs` comment: "confirmed stub — AndonPull detection not implemented" | — | None | None | None | None | Do not use |
| Conformance precision | Precision score reporting | PARTIAL | `WpmVerdict.precision` is `Option<f64>` with explicit null semantics; `has_precision()` exists; no code path actually computes precision — always null | — | `diagnostics_verdict_key::precision_null_is_explicit_not_silent_zero` | None | Low | Medium | Declare UNSUPPORTED or implement |
| Publish gate | Dry-run prerequisite | STUB | CICD-PUBLISH-001 and CICD-PUBLISH-002 defined in diagnostic catalog; `cargo publish --dry-run` not invoked by any publish verb code | — | None | None | Medium | High | Candidate D prerequisite |
| Spec Kit | Task → evidence → receipt trace | UNKNOWN | CICD-SPEC-001 and CICD-SPEC-002 defined in lsp.rs catalog; no `specs/` or `.specify/` directory found; no Spec Kit integration code exists | — | None | None | Low | Medium | Candidate E — currently no seam |
| ggen pipeline | Ontology → template → source generation | DORMANT | `ggen.toml`, `ontology/cargo-cicd.ttl`, `queries/`, `templates/` exist; `tests/ggen_customization_guard.rs` passes; ggen binary not in repo | `ggen` | `ggen_customization_guard` | None | Low | Low | No Day3 action needed |
| `process-data` feature | Level 5 engine internals | DORMANT | Feature flag exists; `EngineState` aggregate in `src/engine/`; not exercised without `--features process-data` | `cargo test --features process-data` | `feature_projection` tests surface contract | None | Low | Low | No Day3 action needed |
| `autonomic` feature | Policy suggest mode | DORMANT | `src/autonomic/` compiles; `tests/autonomic_policies.rs` passes under feature flag | `cargo test --features autonomic` | `autonomic_policies` | None | Low | Low | No Day3 action needed |
| Public boundary invariant | No forbidden terms in CLI output | LIVE | `tests/invariants.rs::invariant_public_boundary_no_forbidden_terms_in_all_help` passes | All `--help` outputs | `invariants` | None | Low | Medium | Maintain |

---

## 2. ARCHITECTURE LAWS

**Law 1: The Oracle Separation Invariant (E1)**  
- **Observation:** `src/evidence.rs` invariant comment: "cargo-cicd NEVER adjudicates its own process conformance. All verdicts are issued by the external wasm4pm oracle."  
- **Evidence:** `tests/wasm4pm_refusal_cases.rs::evidence_invariant_e1_no_self_certification` passes. `WpmEvidenceOracle::audit_xes` delegates exclusively to `Wasm4pmShell::audit`; no internal fitness computation exists in any `src/` file.  
- **Consequence:** Internal test passage does not constitute release closure. A test that does not invoke `wpm audit` cannot be a closing test.  
- **Day3 Implication:** All Day3 work that touches release-gate logic must route through `WpmEvidenceOracle::audit_xes`. Skipping the oracle is an architecture violation, not a shortcut.

**Law 2: The `overall_fitness` Key Contract**  
- **Observation:** `crates/cargo-cicd-core/src/wpm/verdict.rs` doc comment: "Consumers MUST read `overall_fitness`, never `fitness`. If `overall_fitness` is absent, treat as BLOCKED, not 0.0."  
- **Evidence:** `diagnostics_verdict_key.rs` tests `overall_fitness_key_is_read_correctly`, `wrong_key_does_not_produce_truthful_fitness`, `when_both_keys_present_overall_fitness_wins` — all pass. `WpmVerdict::authoritative_fitness()` never falls back to a different key.  
- **Consequence:** Any code that reads `fitness` instead of `overall_fitness` will silently produce 0.0 (CICD-WPM-004). This bug must be caught by regression fixture before it can reach a release.  
- **Day3 Implication:** CICD-WPM-004 regression protection is already baked into the LSP crate. Candidate A's value is now confirming the contract holds across the full publish path, not just the schema parser.

**Law 3: Evidence Must Precede Adjudication (E2)**  
- **Observation:** `src/evidence.rs` invariant E2: "The XES file must exist on disk before `audit_xes` is called."  
- **Evidence:** Every test in `tests/wasm4pm_evidence_gate.rs` calls `emit_xes` and asserts `xes_path.exists()` before calling `assert_wpm_verdict`. `tests/wasm4pm_refusal_cases.rs::evidence_invariant_e2_evidence_required_before_adjudication` passes.  
- **Consequence:** Any test that skips XES emission and calls the oracle directly will fail by design. This prevents phantom adjudication.  
- **Day3 Implication:** Any new evidence gate test must follow the emit → assert-exists → oracle pattern. Shortcutting will hit E2 enforcement.

**Law 4: `Blocked` Is a First-Class Verdict, Not an Error**  
- **Observation:** `src/evidence.rs` invariant E7: "`ExpectedWpmVerdict::Blocked` is a first-class expectation, not an error state."  
- **Evidence:** `evidence_gate_oracle_discover` and all oracle-absent paths in `wasm4pm_evidence_gate.rs` use `absent_oracle_verdict()` which returns `Blocked`. `wasm4pm_refusal_cases.rs::evidence_invariant_e3_blocked_is_first_class` passes.  
- **Consequence:** A CI environment without the wpm binary is a valid execution context; tests must not panic. Setting `REQUIRE_WPM_ORACLE=1` is the mechanism to force the Accept path in environments where wpm is known to be present.  
- **Day3 Implication:** Day3 tests running locally where `/Users/sac/wasm4pm/target/release/wpm` exists should set `REQUIRE_WPM_ORACLE=1` to exercise the Accept branch and confirm the oracle path is actually exercised.

**Law 5: Duplicate Server Capability Functions Are a Latent Defect**  
- **Observation:** `crates/cargo-cicd-lsp/src/server/` contains two capability builders: `initialize.rs::build_server_capabilities()` (declares `DiagnosticServerCapabilities`) and `capabilities.rs::server_capabilities()` (does not). The backend uses `capabilities.rs` only.  
- **Evidence:** `backend.rs` line: `capabilities: server_capabilities()`. `initialize.rs` is imported but `build_server_capabilities` is never called. The `diagnostic_provider` capability declared in `initialize.rs` is therefore never advertised to editors.  
- **Consequence:** Editors relying on `textDocument/diagnostic` push protocol will not receive diagnostics. The `analyze_and_publish` path via `did_open`/`did_save` can still push diagnostics proactively, but pull-mode diagnostics are silently disabled.  
- **Day3 Implication:** The LSP proof (Candidate B) must verify which capabilities are actually advertised in the `InitializeResult`. The fix is one line in `backend.rs` — but it must be tested, not assumed.

**Law 6: The wpm Binary Has Confirmed Stubs That Must Not Be Invoked**  
- **Observation:** `src/integrations/wasm4pm_shell.rs` documents two confirmed stubs: `wpm oracle check` ("AndonPull detection not implemented") and `wpm mining conformance` ("stubs model loading to DFG::new() — always meaningless").  
- **Evidence:** These are documented in the module-level comment from the 2026-06-02 capability scan (wasm4pm commit 65169e62). The module exposes only 5 methods corresponding to the 7 confirmed working commands (audit, lean, receipt_doctor, spc_status, doctor); oracle_check and mining_conformance are deliberately absent.  
- **Consequence:** Any code that calls `wpm oracle check` will receive a meaningless result without error. Since the method is not exposed, this can only happen through raw `Command::new("wpm")` bypass.  
- **Day3 Implication:** Do not add any new shell-out to wpm commands not in the confirmed list. The seam is intentionally narrow.

---

## 3. DAY 3 FRUIT CANDIDATES

### Scoring formula
`FruitScore = (Impact × ProofReadiness × UserVisibility) / (Risk × Scope)`  
Scale 1–5 for each variable. Higher FruitScore = lower-hanging fruit.

---

### Candidate A: CICD-WPM-004 verdict_key_mismatch diagnostic — regression protection

| Variable | Score | Rationale |
|---|---|---|
| Impact | 3 | Prevents silent 0.0 fitness from masking non-conformance; already baked into LSP tests |
| ProofReadiness | 5 | 5 passing tests in `diagnostics_verdict_key.rs`; schema struct exists in `cargo-cicd-core`; diagnostic code CICD-WPM-004 registered |
| UserVisibility | 2 | Schema-level protection; invisible to CLI users unless they read LSP diagnostics |
| Risk | 1 | Tests already pass; no new code needed for LSP side |
| Scope | 2 | Narrow: only the publish path's verdict reading needs to be confirmed schema-aligned |

**FruitScore = (3 × 5 × 2) / (1 × 2) = 15.0**

**Assessment:** The regression is already protected. Day3 value is extending the protection to cover the `ReceiptDoctorVerdict` path in `publish.rs` — confirming it reads `overall_fitness`, not `fitness`, from the wpm JSON. Small, safe, high-proof-value.

---

### Candidate B: LSP editor proof — produce diagnostic JSON from real workspace fixture

| Variable | Score | Rationale |
|---|---|---|
| Impact | 4 | Proves the LSP server actually works end-to-end; closes the gap between "unit tests pass" and "editor receives diagnostics" |
| ProofReadiness | 3 | `backend.rs` is complete; `run_all(WorkspaceSnapshot)` has passing unit tests; gap is JSON-RPC wire protocol test |
| UserVisibility | 5 | Direct editor integration proof — visible to any editor user |
| Risk | 3 | tower-lsp initialization over stdio is not trivial to test without a JSON-RPC client; capability duplicate defect must be fixed first |
| Scope | 3 | Medium: need fixture workspace + JSON-RPC client script or `tower-lsp` test harness |

**FruitScore = (4 × 3 × 5) / (3 × 3) = 6.67**

**Assessment:** High value but non-trivial. The capability duplicate defect (Law 5) blocks full proof — fix `backend.rs` to use `build_server_capabilities()` first, then write a fixture test that sends `initialize` + `textDocument/didOpen` and asserts diagnostic JSON output.

---

### Candidate C: Conformance precision gap — implement or declare UNSUPPORTED

| Variable | Score | Rationale |
|---|---|---|
| Impact | 2 | Precision is informational; TRUTHFUL/VARIANCE verdict is the gate; precision null is already explicitly documented |
| ProofReadiness | 4 | `has_precision()` exists; `precision_null_is_explicit_not_silent_zero` test passes; declaring UNSUPPORTED is one doc line |
| UserVisibility | 1 | Precision not surfaced in any CLI output |
| Risk | 1 | Zero risk for UNSUPPORTED declaration |
| Scope | 1 | Minimal: update doc comment in `verdict.rs` to say "precision: UNSUPPORTED in current wpm implementation" |

**FruitScore = (2 × 4 × 1) / (1 × 1) = 8.0**

**Assessment:** Low-effort, low-risk, low-impact. Do it as a one-liner if touching `verdict.rs` for another reason. Not a standalone Day3 target.

---

### Candidate D: Publish gate as adjudicated receipt — dry-run dependent on Admitted receipt

| Variable | Score | Rationale |
|---|---|---|
| Impact | 5 | Closes the actual release gate: `cargo cicd publish run` → wpm adjudicates → Admitted before `cargo publish --dry-run` proceeds |
| ProofReadiness | 3 | `ReceiptDoctor` exists in `evidence.rs`; `publish.rs` calls it; CICD-PUBLISH-002 is defined; but `cargo publish --dry-run` is never invoked by publish verb |
| UserVisibility | 5 | Directly visible: publish fails loudly if receipt not admitted |
| Risk | 4 | Touching the publish gate is high-risk; if `ReceiptDoctor` incorrectly refuses, all publishes break |
| Scope | 4 | Broad: need dry-run invocation, error handling, test fixture for admitted vs refused paths |

**FruitScore = (5 × 3 × 5) / (4 × 4) = 4.69**

**Assessment:** Highest business impact but lowest FruitScore due to risk and scope. The `ReceiptDoctorVerdict::Refused` path in `publish.rs` already bails correctly. The missing piece is the dry-run invocation. Day3 is feasible only if scoped to: "add `cargo publish --dry-run` call after Admitted verdict, add one test asserting it is not called when verdict is Blocked."

---

### Candidate E: Spec Kit integration — task → evidence → receipt trace

| Variable | Score | Rationale |
|---|---|---|
| Impact | 3 | Closes the spec → evidence → receipt traceability gap |
| ProofReadiness | 1 | No `specs/` directory, no `.specify/` directory, no Spec Kit integration code; CICD-SPEC-001/002 are catalog entries only |
| UserVisibility | 3 | Visible to anyone using spec-driven workflow |
| Risk | 2 | No existing code to break |
| Scope | 5 | Wide: need Spec Kit integration layer, directory conventions, evidence linkage |

**FruitScore = (3 × 1 × 3) / (2 × 5) = 0.90**

**Assessment:** No seam exists. Not Day3 fruit. Defer to v26.6.3.

---

### Summary Table

| Candidate | FruitScore | Status |
|---|---|---|
| A: CICD-WPM-004 regression | 15.0 | LIVE — already baked in; Day3 extends to publish path |
| C: Conformance precision UNSUPPORTED | 8.0 | Easy one-liner; low value standalone |
| B: LSP editor proof | 6.67 | High value, medium effort; Law 5 fix required first |
| D: Publish gate dry-run | 4.69 | High impact, high risk; scope down to minimum viable |
| E: Spec Kit integration | 0.90 | No seam — DEFER |

---

## 4. RECOMMENDED DAY 3 FIRST TARGET

**Candidate B: LSP editor proof — produce diagnostic JSON from a real workspace fixture**

**Rationale:**

1. **Law 5 is a real defect with a one-line fix.** `backend.rs` uses `server_capabilities()` (no diagnostic capability) instead of `build_server_capabilities()` (declares `DiagnosticServerCapabilities`). This is a silent defect: the LSP crate's unit tests all pass, but editors cannot request pull-mode diagnostics. Fixing it requires changing one line in `backend.rs` and is immediately verifiable.

2. **The fix unlocks the proof.** Once `build_server_capabilities()` is active, a fixture test can send a real `initialize` JSON-RPC request to the binary and assert the `InitializeResult` contains `diagnosticProvider`. This is concrete, falsifiable proof that the editor integration path is live, not merely compiled.

3. **All prerequisites are already passing.** `run_all(WorkspaceSnapshot)` has 9 passing analyzer unit tests. `finding_to_lsp` maps findings to LSP `Diagnostic`. The backend compiles and the binary exists at `target/debug/cargo-cicd-lsp`. The only gap is the wire-level proof.

4. **High FruitScore with mandatory prerequisite already scoped.** The Law 5 fix is not blocked on any external dependency. The fixture test requires only a JSON-RPC client call — achievable with a raw `stdin` write in a Rust test using `Command` + `Child::stdin`.

5. **Candidate A's protection is already baked in.** Spending Day3 on A would be reinforcing a door that is already locked. Spending it on B opens a door that is currently blocked.

**Day3 execution sequence for Candidate B:**
1. Fix `backend.rs` line 62: change `server_capabilities()` to `build_server_capabilities()` — one line, `src/server/capabilities.rs` can be marked `// superseded by initialize.rs`.
2. Add `tests/lsp_initialize_fixture.rs`: spawn `cargo-cicd-lsp`, write `initialize` JSON-RPC over stdin, read response from stdout, assert `capabilities.diagnosticProvider` is present and `capabilities.textDocumentSync` is FULL.
3. Add `tests/lsp_did_open_fixture.rs`: after initialize, send `textDocument/didOpen` for a real fixture workspace file, assert `window/publishDiagnostics` notification is received (or assert no crash for empty workspace).
4. Emit ProcessEvent for each step, verify with `emit_xes` + `assert_wpm_verdict` under `REQUIRE_WPM_ORACLE=1`.

---

## 5. RISKS FOR DAY 3 EXECUTION

**Risk 1: Oracle absent in clean environment — Accept path never exercised**  
- **Severity:** High  
- **Detail:** All 9 `wasm4pm_evidence_gate` tests pass but via the Blocked path (oracle not found). Setting `REQUIRE_WPM_ORACLE=1` forces the Accept path. Without this, the evidence gate is untested in its operational mode.  
- **Mitigation:** Run `REQUIRE_WPM_ORACLE=1 cargo test --test wasm4pm_evidence_gate` at Day3 start to confirm `/Users/sac/wasm4pm/target/release/wpm` is still present and returns exit 0 for valid XES.

**Risk 2: LSP binary spawn in tests is flaky on stdio**  
- **Severity:** Medium  
- **Detail:** JSON-RPC over stdin/stdout with a spawned process requires careful buffering. `tower-lsp` sends framed messages; a naive `write_all` + `read_to_string` will hang. tower-lsp expects Content-Length framing.  
- **Mitigation:** Use the `tower-lsp` test harness crate or write a minimal framing wrapper. Alternatively, test only `InitializeResult` deserialization from the binary's stdout by piping a single `initialize` request and immediately closing stdin.

**Risk 3: `ReceiptDoctor` path in `publish.rs` reads wrong key**  
- **Severity:** Medium  
- **Detail:** `publish.rs` extracts `state` from `stdout_json` via `serde_json::Value::get("state")`. If wpm's `receipt doctor` output schema changes, this silently falls back to "Admitted". This is the same key-mismatch pattern CICD-WPM-004 was designed to catch, but in a different location.  
- **Mitigation:** Add a `ReceiptDoctorVerdict` schema test (analogous to `diagnostics_verdict_key.rs`) asserting the correct JSON key is read.

**Risk 4: Capability duplicate causes silent diagnostic gap**  
- **Severity:** Medium  
- **Detail:** Law 5 is active right now. Until `backend.rs` is fixed, no editor will receive pull-mode diagnostics. If Day3 LSP proof is attempted without fixing this, the fixture test will assert a capability that is not advertised, causing the test to fail for the wrong reason.  
- **Mitigation:** Fix Law 5 as the first commit of Day3 before writing any test.

**Risk 5: `wpm receipt doctor` BLOCKED path proceeds silently**  
- **Severity:** Medium  
- **Detail:** `publish.rs` L80: when `ReceiptDoctor::discover()` returns None, publish proceeds with a warning (`BLOCKED:oracle_unavailable`). This means `cargo cicd publish run` can succeed without any oracle adjudication. This is documented as "proceed with a warning" but the warning goes to stderr only.  
- **Mitigation:** Add a CICD-WPM-001 diagnostic emission when oracle is unavailable, so the LSP can surface it as an active Error finding. This is a Day3 gap, not a blocker.
