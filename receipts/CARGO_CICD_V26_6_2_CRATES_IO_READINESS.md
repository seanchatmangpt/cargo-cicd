# cargo-cicd v26.6.2 — crates.io Readiness Receipt

**Date:** 2026-06-02
**Version:** 26.6.2
**Repository:** https://github.com/seanchatmangpt/cargo-cicd
**Crate name:** cargo-cicd
**Name available:** yes

---

## Cargo.toml Metadata

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

---

## Dependency Audit

| Dependency | Resolved to | Path dep? |
|---|---|---|
| clap | 4.x (crates.io) | no |
| clap-noun-verb | 26.5.19 (crates.io) | no |
| serde | 1.x (crates.io) | no |
| toml | 0.8 (crates.io) | no |
| anyhow | 1.x (crates.io) | no |
| walkdir | 2.x (crates.io) | no |
| serde_json | 1.x (crates.io) | no |

**Verdict: CLEAN** — no path dependencies in published Cargo.toml.

---

## Feature Matrix

| Feature combo | Build result |
|---|---|
| (default — `formats` off) | PASS |
| `--features process-data` | PASS |
| `--features autonomic` | PASS |
| `--features wasm4pm` | PASS |
| `--all-features` | PASS |
| Clippy `--all-features` | CLEAN (no errors) |
| Fmt check | CLEAN (no diff) |

---

## Package Contents

| Metric | Value |
|---|---|
| File count | 178 |
| Uncompressed size | 280.3 KiB |
| Compressed .crate size | 70.4 KiB |

Private paths excluded: `/receipts`, `/docs/testing`, `/docs/release`, `/docs/deferred`,
`/docs/wasm4pm`, `/ontology`, `/queries`, `/templates`, `/cicd.toml`, `/CLAUDE.md`,
`/ggen.toml`, `/tests/wasm4pm_evidence`.

---

## Public Boundary Audit

Forbidden terms scanned in all public-facing `src/` and `docs/commands/` files:
`ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`,
`Truex`, `CONSTRUCT8`.

**Result: CLEAN**

Notable: `src/integrations/wasm4pm_current.rs` had a forbidden term in a doc comment.
Removed in commit f931629. All public source confirmed clean as of this receipt.

---

## License

| File | Status |
|---|---|
| LICENSE-MIT | present |
| LICENSE-APACHE | present |
| Cargo.toml `license` field | `MIT OR Apache-2.0` |

---

## Install Verification

**cargo install --path .** — SUCCESS
Binary installed to: `/Users/sac/.cargo/bin/cargo-cicd`

**cargo cicd subcommand dispatch** — PARTIAL
`cargo cicd <cmd>` fails with "unrecognized subcommand 'cicd'" under direct `cargo`
dispatch in some environments. Re-exec fix committed as a70c639 and a pending working-tree
change. Full end-to-end verification pending after committing the working-tree diff and
re-installing.

---

## wasm4pm Evidence Gate

| Suite | Result |
|---|---|
| wasm4pm_evidence_gate | 8/8 PASS |
| wasm4pm_evidence_mutation | 5/5 PASS (all mutations REFUSED) |
| wasm4pm_refusal_cases | 7/7 PASS |
| **Total** | **20/20 PASS** |

TOTAL TESTS (full suite): 113 passed, 0 failed.

---

## cargo publish --dry-run

```
    Updating crates.io index
   Packaging cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
    Updating crates.io index
    Packaged 178 files, 280.3KiB (70.4KiB compressed)
   Verifying cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
   Compiling cargo-cicd v26.6.2 (/Users/sac/cargo-cicd/target/package/cargo-cicd-26.6.2)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.10s
   Uploading cargo-cicd v26.6.2 (/Users/sac/cargo-cicd)
warning: aborting upload due to dry run
```

**Verdict: PASS**

---

## Known Gaps

1. **Subcommand dispatch (PARTIAL):** `cargo cicd <cmd>` re-exec loop fix is in the
   working tree but not yet committed. Must commit src/main.rs, reinstall, and confirm
   `cargo cicd status` exits 0 to close this gap.
2. **Working tree dirty:** `src/main.rs` has uncommitted changes at time of this receipt.
   The dry-run was run with `--allow-dirty`. Clean commit required before actual publish.

---

## Verdict

**PARTIAL**

dry-run passed, 113/113 tests passed, install succeeds, evidence gate ALIVE, public
boundary clean, all Cargo metadata fields correct. Two items block PUBLISH_READY:

1. Commit the src/main.rs subcommand re-exec fix.
2. Re-verify `cargo cicd status` exits 0 after reinstall.

Law: "cargo-cicd is crates.io-ready only when the packaged crate installs cleanly,
exposes a public-safe Cargo subcommand, preserves the wasm4pm evidence gate,
and passes cargo publish --dry-run."
