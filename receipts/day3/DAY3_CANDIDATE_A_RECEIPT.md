# DAY3 CANDIDATE A RECEIPT

**Date:** 2026-06-02
**Candidate:** LSP editor diagnostics proof tests

---

## What Was Fixed

1. `LspExplainVerb::run` — unknown diagnostic codes previously emitted an error
   on stderr but returned `Ok(())` (exit 0). Fixed to return
   `Err(NounVerbError::execution_error(...))` so the process exits non-zero,
   satisfying the exit-code contract tested by `lsp_explain_unknown_code_exits_nonzero`.

2. `crates/cargo-cicd-lsp/src/server/mod.rs` — `capabilities` submodule was not
   exported. Added `pub mod capabilities;` so integration tests in
   `crates/cargo-cicd-lsp/tests/protocol_initialize.rs` can call
   `cargo_cicd_lsp::server::capabilities::build_server_capabilities()`.

---

## Tests That Prove It

### Test 1 — `cargo cicd lsp explain` catalog coverage
**File:** `tests/lsp_explain.rs`
**Registration:** `[[test]] name = "lsp_explain"` in root `Cargo.toml`

| Test | Assertion |
|------|-----------|
| `lsp_explain_git_001_dirty_tree` | stdout contains "dirty_tree_blocks_close", "Code:", "Repair:" |
| `lsp_explain_evidence_003_hardcoded_timestamp` | stdout contains "hardcoded_timestamp", "Repair:" |
| `lsp_explain_wpm_001_unconfirmed_receipt_court` | stdout contains "unconfirmed_receipt_court", "Repair:" |
| `lsp_explain_public_001_private_term_leak` | stdout contains "private_term_leak", "Repair:" |
| `lsp_explain_close_001_false_close_risk` | stdout contains "false_close_risk", "Repair:" |
| `lsp_explain_unknown_code_exits_nonzero_with_stderr` | stderr contains "unknown diagnostic code", exit non-zero |
| `lsp_explain_known_code_exits_zero` | known code exits 0 |
| `lsp_explain_unknown_code_exits_nonzero` | unknown code exits non-zero |

Result: **8 passed, 0 failed**

### Test 2 — LSP initialize capabilities proof
**File:** `crates/cargo-cicd-lsp/tests/protocol_initialize.rs`

| Test | Assertion |
|------|-----------|
| `build_server_capabilities_declares_diagnostic_provider` | `diagnostic_provider` is `Some` |
| `build_server_capabilities_declares_code_action_provider` | `code_action_provider` is `Some` |
| `build_server_capabilities_declares_text_document_sync` | `text_document_sync` is `Some` |
| `build_server_capabilities_all_required_present` | all three present together |

Result: **4 passed, 0 failed**

---

## Commands Run

```sh
cargo test --test lsp_explain
# test result: ok. 8 passed; 0 failed

cargo test -p cargo-cicd-lsp --test protocol_initialize
# test result: ok. 4 passed; 0 failed
```

---

## Files Changed / Created

- `tests/lsp_explain.rs` — new integration test file (8 tests)
- `crates/cargo-cicd-lsp/tests/protocol_initialize.rs` — new unit tests for capabilities (4 tests)
- `Cargo.toml` — added `[[test]] name = "lsp_explain"` declaration
- `src/nouns/lsp.rs` — fixed `LspExplainVerb::run` to exit non-zero on unknown code
- `crates/cargo-cicd-lsp/src/server/mod.rs` — exported `pub mod capabilities`

---

## Verdict

**DAY3_FRUIT_DELIVERED**

Both proof test families pass. The CLI exit-code contract is enforced. The LSP
`build_server_capabilities()` function is unit-tested and confirmed to declare
`diagnosticProvider`, `codeActionProvider`, and `textDocumentSync`.
