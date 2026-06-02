---
artifact: NEGATIVE_FIXTURE_LEDGER
date: 2026-06-02
---

# Negative Fixture Ledger

Each row is one negative-path fixture. Columns: **Fixture Name**, **Setup**, **Expected
Verdict**, **Expected Event** (the observable signal that confirms the verdict).

A fixture that lacks an expected event is not a valid receipt. "exit non-0" alone is not an
event — the event must name a reason, law, or message fragment.

---

| Fixture Name | Setup | Expected Verdict | Expected Event |
|---|---|---|---|
| clean_workspace | Fresh single-crate workspace, no uncommitted changes, no `target/`, no `cicd.toml` | PASS | stdout contains "all green" or equivalent; exit 0 |
| dirty_workspace | Single-crate workspace with one tracked file modified but not committed | REFUSE (git close) | stderr contains dirty-state law name; exit non-0 |
| missing_manifest | Directory with no `Cargo.toml` | FAIL with explanation | stderr contains "Cargo.toml not found" or equivalent; exit non-0 |
| toolchain_mismatch | `rust-toolchain.toml` specifies nightly channel; active toolchain is stable | WARN | stdout or stderr contains "toolchain mismatch"; exit 0 or non-0 per command |
| target_over_limit | `target/` directory exceeds configured size threshold | WARN + SUGGEST | stdout contains size report and suggests prune; no files deleted; exit 0 |
| trybuild_changed_only | Multiple trybuild fixtures; exactly one `.rs` source changed | PLAN (changed-only) | stdout lists only the changed fixture, not the full estate; exit 0 |
| trybuild_huge_fixture_set | 500+ trybuild fixtures; exactly one changed | PLAN (changed-only) | stdout lists only changed fixture; full count not executed; exit 0 |
| corrupted_cicd_toml | `cicd.toml` present but contains invalid TOML bytes | REFUSE or REPAIR OFFER | stderr contains "corrupted" or unparseable signal; no silent overwrite; exit non-0 |
| stale_cicd_toml | `cicd.toml` present; workspace inputs have changed since last write | REGENERATE | stdout indicates stale detection and regeneration; old content not retained; exit 0 |
| git_unrelated_dirty | Workspace has unrelated untracked or modified files; `git close` invoked | REFUSE | stderr names the blocking files and states close refused; exit non-0 |
| wasm4pm_missing | `wasm4pm` feature enabled; wasm4pm binary absent from PATH | PARTIAL (honest) | stdout contains PARTIAL signal and states binary not found; no fabricated capability; exit 0 |
| wasm4pm_mock_cli | `wasm4pm` feature enabled; mock binary present responding to scan but not to exchange | PARTIAL (honest) | stdout contains PARTIAL signal; exchange capability absent is named; exit 0 |
