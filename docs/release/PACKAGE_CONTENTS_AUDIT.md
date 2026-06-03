# cargo-cicd v26.6.2 — Package Contents Audit

**Date:** 2026-06-02
**Version:** 26.6.2
**Command:** `cargo package --list --allow-dirty`

---

## Package Size

| Metric | Value |
|---|---|
| File count | 178 |
| Uncompressed | 280.3 KiB |
| Compressed (.crate) | 70.4 KiB |

---

## Exclusions Applied (Cargo.toml `exclude` list)

| Excluded path | Reason |
|---|---|
| `/receipts` | Internal manufacturing receipts — not public API |
| `/docs/testing` | Internal testing notes |
| `/docs/release` | Internal release documentation (this file) |
| `/docs/deferred` | Deferred capability notes |
| `/docs/wasm4pm` | wasm4pm integration docs (internal) |
| `/ontology` | Internal ontology files |
| `/queries` | Internal SPARQL/query files |
| `/templates` | Internal code generation templates |
| `/cicd.toml` | Internal CI/CD config — not for consumers |
| `/CLAUDE.md` | Internal Claude Code instructions |
| `/ggen.toml` | Internal code generator config |
| `/tests/wasm4pm_evidence` | Internal wasm4pm evidence fixtures |

---

## Included File Categories

| Category | Example paths |
|---|---|
| Source | src/main.rs, src/lib.rs, src/evidence.rs, src/engine/*, src/nouns/*, src/adapters/*, src/autonomic/*, src/integrations/*, src/policies/*, src/state/* |
| Tests | tests/wasm4pm_evidence_gate.rs, tests/wasm4pm_evidence_mutation.rs, tests/wasm4pm_refusal_cases.rs, tests/cli/*, tests/fixtures/* |
| Commands documentation | docs/commands/git.md, status.md, publish.md, target.md, test.md, trybuild.md, workspace.md |
| Build metadata | Cargo.toml, Cargo.lock, .cargo_vcs_info.json, .gitignore |
| License | LICENSE-MIT, LICENSE-APACHE |
| README | README.md |

---

## Verification

`cargo package --list --allow-dirty` output used (working tree has uncommitted src/main.rs
changes). When src/main.rs is committed the `--allow-dirty` flag will not be needed and
the file list will be identical.

**Audit verdict: CLEAN** — no private receipts, no internal docs, no CLAUDE.md, no
cicd.toml, no ontology/query/template files in the published package.
