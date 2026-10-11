# crates.io Release Checklist — cargo-cicd v26.10.10

**Date:** 2026-10-10
**Version:** 26.10.10 (final numbering set by release lane)
**Repository:** https://github.com/seanchatmangpt/cargo-cicd

Mechanized checklist — every gate is a command that exits non-zero on failure.
No manual PASS/FAIL table; run each step, record the real output in the release
receipt. Do not proceed past a failing step.

---

## 1. Version parity

```bash
cargo test -p cargo-cicd --test version_parity_proof
```
- All parity tests pass: workspace version, sub-crate versions
  (`crates/cargo-cicd-core`, `crates/cargo-cicd-lsp`), and `ggen.toml` parity
  agree on the release version.
- Failing parity = stop; fix versions, re-run. Never publish with skew.

## 2. Full test suite

```bash
cargo test --workspace
```
- 0 failures, 0 compile aborts. A compile abort is a blocked build, not a
  failure count — fix the broken crate before continuing.

## 3. Lint

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
- Clean. Warnings are Andon: fix, do not suppress.

## 4. Status audit

```bash
cargo cicd status audit
```
- Green: real receipt chain verified (blake3), no self-minted or sentinel
  receipts, OCEL evidence conforms to the jsonschema court.

## 5. Per-crate publish dry-run (dependency order)

```bash
cargo publish --dry-run -p cargo-cicd-core
cargo publish --dry-run -p cargo-cicd-lsp
cargo publish --dry-run -p cargo-cicd
```
- Each compiles the packaged crate in a temp dir with no errors.
- `cargo package --list` spot-check: no private docs, no target/evidence dirs
  in any package.
- `cargo cicd publish check` passes (includes the doc-drift gate: docs changed
  since the last tag require a passing `[docs] verify_cmd` from cicd.toml).

## 6. Publish (user-gated)

Each publish is an explicit human-gated transition:

```bash
cargo publish -p cargo-cicd-core
cargo publish -p cargo-cicd-lsp
cargo publish -p cargo-cicd
```
- Publish in dependency order; wait for each to be visible on crates.io before
  the next (registry index lag).

## 7. Tag

```bash
git tag v26.10.10
git push origin v26.10.10
```
- Tag on the exact published commit. Then mint the release receipt and file it
  under `docs/release/`.
