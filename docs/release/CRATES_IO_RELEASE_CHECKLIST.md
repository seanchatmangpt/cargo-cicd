# cargo-cicd v26.6.2 — crates.io Release Checklist

**Date:** 2026-06-02
**Version:** 26.6.2
**Repository:** https://github.com/seanchatmangpt/cargo-cicd

Law: "cargo-cicd is crates.io-ready only when the packaged crate installs cleanly,
exposes a public-safe Cargo subcommand, preserves the wasm4pm evidence gate,
and passes cargo publish --dry-run."

---

## ALIVE Conditions — v26.6.2

| # | Condition | Status |
|---|---|---|
| 1 | `name` field present and available on crates.io | [x] |
| 2 | `version` field follows SemVer — 26.6.2 | [x] |
| 3 | `description` field present and non-empty | [x] |
| 4 | `license` field present — `MIT OR Apache-2.0` | [x] |
| 5 | LICENSE-MIT file present in repository root | [x] |
| 6 | LICENSE-APACHE file present in repository root | [x] |
| 7 | `repository` field points to valid URL | [x] |
| 8 | `readme` field set to README.md; README.md present | [x] |
| 9 | `keywords` list present — 5 keywords, within crates.io limit | [x] |
| 10 | `categories` list present and valid crates.io category slugs | [x] |
| 11 | No path dependencies in published Cargo.toml | [x] |
| 12 | `cargo publish --dry-run` passes — 178 files, 70.4 KiB compressed | [x] |
| 13 | `cargo install --path .` succeeds — binary at ~/.cargo/bin/cargo-cicd | [x] |
| 14 | wasm4pm evidence gate — 20/20 tests pass (8 gate + 5 mutation + 7 refusal) | [x] |
| 15 | Public boundary audit clean — no forbidden terms in public doc comments | [x] |
| 16 | `cargo cicd <cmd>` subcommand dispatch working end-to-end | [ ] PARTIAL |

---

## Condition 16 Detail

The re-exec fix is present in src/main.rs (committed as a70c639 — "fix(cli): strip cargo
subcommand prefix before noun-verb dispatch"). The uncommitted diff in the working tree
adds a second guard for the `cargo cicd` re-exec loop. This diff must be committed and
`cargo install --path . && cargo cicd status` re-verified before this item is [x].

---

## Summary

15/16 conditions fully met.

**Verdict: PARTIAL** — advance to PUBLISH_READY after committing src/main.rs fix and
confirming `cargo cicd status` exits 0.
