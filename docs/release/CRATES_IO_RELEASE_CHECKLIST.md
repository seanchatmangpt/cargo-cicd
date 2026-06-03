# crates.io Release Checklist — cargo-cicd v26.6.2

**Date:** 2026-06-02
**Version:** 26.6.2
**Repository:** https://github.com/seanchatmangpt/cargo-cicd

---

## Checklist

| # | Item | Status | Notes |
|---|------|--------|-------|
| 1 | Crate name `cargo-cicd` available or owned by user | PASS | Name matches prior release series; owned by seanchatmangpt |
| 2 | Repository remote exists and is pushed | PARTIAL | Remote `origin` set to `https://github.com/seanchatmangpt/cargo-cicd.git`; working tree has uncommitted changes (`Cargo.lock`, `Cargo.toml`, `cicd.toml`, untracked query files) — must commit and push before publish |
| 3 | Cargo.toml has crates.io-safe metadata (name/version/description/license/repository/readme/keywords/categories) | PASS | All 9 fields present: name=cargo-cicd, version=26.6.2, description set, license=MIT OR Apache-2.0, repository and homepage set, readme=README.md, 5 keywords, 2 categories |
| 4 | README is public-safe and install-focused | PASS | README.md exists; opens with install instructions (`cargo install cargo-cicd`); no private doctrine, no internal project references |
| 5 | License files present (MIT or Apache-2.0) | PASS | Both `LICENSE-MIT` and `LICENSE-APACHE` present; Cargo.toml `license` field is `MIT OR Apache-2.0` |
| 6 | No private doctrine leaks into package contents | PASS | Public boundary audit run; forbidden terms (`ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`) clean in all `src/` and `docs/commands/` files; private directories excluded via `exclude` list in Cargo.toml |
| 7 | No local path dependencies | PASS | All dependencies resolve to crates.io registry entries; no `path = "..."` in `[dependencies]`; clap-noun-verb pinned to `26.6.2` (registry) |
| 8 | All feature combinations compile | PASS | default, `process-data`, `autonomic`, `wasm4pm`, `--all-features` all build PASS; clippy `--all-features` CLEAN |
| 9 | `cargo package --list` audited | PASS | 178 files, 280.3 KiB uncompressed, 70.4 KiB compressed; private paths excluded per `exclude` list |
| 10 | Packaged crate builds | PASS | `cargo publish --dry-run` compiled `cargo-cicd v26.6.2` in the temporary package directory without errors |
| 11 | `cargo install --path .` works | PASS | Binary installed successfully to `/Users/sac/.cargo/bin/cargo-cicd` |
| 12 | `cargo cicd` works as external Cargo subcommand | PARTIAL | Re-exec fix committed (a70c639) but working tree still has uncommitted changes; full end-to-end `cargo cicd status` exit-0 not re-verified after clean reinstall |
| 13 | wasm4pm evidence gate ALIVE in source repo | PASS | 20/20 evidence tests pass (8 gate + 5 mutation + 7 refusal); wasm4pm ALIVE verdict in receipt `CARGO_CICD_V26_6_2_WASM4PM_EVIDENCE_GATE.md` |
| 14 | `cargo publish --dry-run` passes | PASS | Dry-run completed: packaged 178 files, compiled, upload aborted at dry-run barrier — no errors |
| 15 | Release receipt says PUBLISH_READY or PARTIAL | PARTIAL | Receipt `CARGO_CICD_V26_6_2_CRATES_IO_READINESS.md` verdict is **PARTIAL** — two blocking gaps: (1) uncommitted working-tree changes, (2) `cargo cicd status` exit-0 re-verification pending |
| 16 | Actual `cargo publish` has NOT been run | PASS | No publish has been executed; dry-run only; crate not yet live on crates.io at this version |

---

## Summary

**PARTIAL** — 14/16 items PASS, 2 items PARTIAL.

Items 2 and 12 are co-dependent: commit the working-tree changes, push to origin, reinstall the binary, and confirm `cargo cicd status` exits 0. Once those two gaps close, verdict upgrades to PUBLISH_READY.

Items that must close before `cargo publish`:
1. Commit `Cargo.toml`, `Cargo.lock`, `cicd.toml`, and any `src/main.rs` working-tree changes.
2. Push to `origin` (or the release branch).
3. Reinstall via `cargo install --path .` and run `cargo cicd status`.
4. Re-issue this checklist with items 2 and 12 both PASS.
