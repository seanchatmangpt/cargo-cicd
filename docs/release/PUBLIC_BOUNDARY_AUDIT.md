# cargo-cicd v26.6.2 — Public Boundary Audit

**Date:** 2026-06-02
**Version:** 26.6.2

Forbidden terms scanned: `ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`,
`Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`

Scan command:
```
grep -rn "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8" \
  README.md LICENSE-MIT LICENSE-APACHE Cargo.toml src/
```

---

## Results per File

| File | Classification | Verdict |
|---|---|---|
| `README.md` | Public / crates.io rendered | CLEAN |
| `LICENSE-MIT` | Public / crates.io rendered | CLEAN |
| `LICENSE-APACHE` | Public / crates.io rendered | CLEAN |
| `Cargo.toml` | Public / crates.io metadata | CLEAN |
| `src/main.rs` | Public / compiled binary | CLEAN |
| `src/lib.rs` | Public / compiled binary | CLEAN |
| `src/evidence.rs` | Public / compiled binary | CLEAN |
| `src/integrations/wasm4pm_current.rs` | Public / compiled binary | CLEAN (ALIVE removed in f931629) |
| `src/integrations/wasm4pm_exchange.rs` | Public / compiled binary | CLEAN |
| `src/integrations/wasm4pm_shell.rs` | Public / compiled binary | CLEAN |
| `src/engine/*` | Public / compiled binary | CLEAN |
| `src/nouns/*` | Public / compiled binary | CLEAN |
| `src/adapters/*` | Public / compiled binary | CLEAN |
| `src/autonomic/*` | Public / compiled binary | CLEAN |
| `src/policies/*` | Public / compiled binary | CLEAN |
| `src/state/*` | Public / compiled binary | CLEAN |
| `docs/commands/*.md` | Public / included in package | CLEAN |
| `CLAUDE.md` | Internal / excluded from crates.io | INTERNAL — excluded |
| `cicd.toml` | Internal / excluded from crates.io | INTERNAL — excluded |
| `receipts/*` | Internal / excluded from crates.io | INTERNAL — excluded |

---

## Notable Fix

`src/integrations/wasm4pm_current.rs` contained the forbidden term `ALIVE` in a public
doc comment. Removed in commit f931629. The file is confirmed clean.

---

## Verdict: CLEAN

No forbidden internal terms appear in any public-facing file that is included in the
published crate package.
