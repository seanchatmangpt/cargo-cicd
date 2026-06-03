# Receipt: cargo-cicd v26.6.2 — crates.io Readiness

**Date:** 2026-06-02
**Version:** 26.6.2
**Repository:** https://github.com/seanchatmangpt/cargo-cicd
**Crate name:** cargo-cicd

---

## Dependency Chain Published

The following upstream crates are already published on crates.io and resolved via the
registry (no path dependencies remain):

| Crate | Version | Source |
|---|---|---|
| clap-noun-verb-macros | 26.6.2 | crates.io registry |
| clap-noun-verb | 26.6.2 | crates.io registry |

c8-* dependencies were removed as unused prior to this release. All c8-family crates are
absent from Cargo.toml and Cargo.lock.

---

## Cargo.toml Status

| Field | Value |
|---|---|
| name | cargo-cicd |
| version | 26.6.2 |
| edition | 2021 |
| rust-version | 1.85 |
| description | Local-first CI/CD helper for Rust workspaces: clean target dirs, run changed tests, check git state, and publish cicd.toml. |
| license | MIT OR Apache-2.0 |
| repository | https://github.com/seanchatmangpt/cargo-cicd |
| homepage | https://github.com/seanchatmangpt/cargo-cicd |
| readme | README.md |
| keywords | ["cargo", "ci", "testing", "workspace", "cleanup"] |
| categories | ["command-line-utilities", "development-tools"] |

All 11 crates.io-required and recommended metadata fields are present and well-formed.
No path dependencies in `[dependencies]`. All dependencies resolve to crates.io registry.

---

## Evidence Gate

**Total test suite:** 113 tests, 113 passed, 0 failed.

| Suite | Tests | Result |
|---|---|---|
| unit tests (lib + main) | 8 | PASS |
| autonomic_policies | 23 | PASS |
| changed_tests | 4 | PASS |
| cicd_toml_truth | 3 | PASS |
| cli/command_projection | 8 | PASS |
| feature_projection | 4 | PASS |
| feature_projections | 4 | PASS |
| fixture_workspaces | 8 | PASS |
| git_phase_closure | 3 | PASS |
| interactions | 7 | PASS |
| invariants | 5 | PASS |
| policies | 3 | PASS |
| wasm4pm_evidence_gate | 8 | PASS |
| wasm4pm_evidence_mutation | 5 | PASS |
| wasm4pm_harness | 7 | PASS |
| wasm4pm_refusal_cases | 7 | PASS |
| wasm4pm_shell | 5 | PASS |
| doc-tests | 1 | PASS |

wasm4pm gate status: **ALIVE** — 8/8 positive evidence cases accepted, 5/5 mutations
refused, 7/7 refusal invariants confirmed. Receipt:
`receipts/CARGO_CICD_V26_6_2_WASM4PM_EVIDENCE_GATE.md`.

---

## CARGO_REGISTRY_TOKEN Note

Before running `cargo publish`, one of the following must be in place:

- Environment variable `CARGO_REGISTRY_TOKEN` set to a valid crates.io API token, OR
- `~/.cargo/credentials.toml` updated with a valid token under `[registry]`

The token must belong to the owner of the `cargo-cicd` crate (seanchatmangpt). Without
this, `cargo publish` will fail with an authentication error regardless of dry-run status.

---

## Known Gaps

1. **Working tree dirty (PARTIAL):** `Cargo.toml`, `Cargo.lock`, and `cicd.toml` have
   uncommitted modifications; several files under `queries/` are untracked. A clean commit
   and push to `origin` are required before actual publish. The dry-run was run
   `--allow-dirty`.

2. **Subcommand dispatch not re-verified (PARTIAL):** The `cargo cicd <cmd>` re-exec fix
   is committed (a70c639) but the working tree has additional changes. After committing and
   reinstalling, `cargo cicd status` must exit 0 to confirm end-to-end subcommand dispatch.
   This has not been re-verified since the working-tree changes were introduced.

---

## Verdict

**PARTIAL**

All Cargo metadata fields correct, no path dependencies, both license files present,
dry-run passes (178 files, 70.4 KiB compressed), 113/113 tests pass, wasm4pm evidence
gate ALIVE, public boundary clean.

Two items block PUBLISH_READY:
1. Commit and push the dirty working tree.
2. Reinstall and confirm `cargo cicd status` exits 0.

Law: "cargo-cicd is crates.io-ready only when the packaged crate installs cleanly,
exposes a public-safe Cargo subcommand, preserves the wasm4pm evidence gate,
and passes cargo publish --dry-run."
