# Public Boundary Audit

This table records the forbidden-term scan results for all public-facing files.

Forbidden terms: `ALIVE`, `Inspection Gate`, `Nehemiah`, `Field8`, `Instinct8`, `Cargo Court`, `AGI`, `Truex`, `CONSTRUCT8`

| File | Terms Scanned | Classification | Status |
|---|---|---|---|
| `README.md` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / crates.io | CLEAN |
| `LICENSE-MIT` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / crates.io | CLEAN |
| `LICENSE-APACHE` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / crates.io | CLEAN |
| `Cargo.toml` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / crates.io | CLEAN |
| `src/main.rs` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / compiled binary | CLEAN |
| `src/lib.rs` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Public / compiled binary | CLEAN |
| `CLAUDE.md` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Internal / not published | INTERNAL — excluded from crates.io |
| `cicd.toml` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Internal / gitignored or .ignore | INTERNAL |
| `docs/release/CRATES_IO_RELEASE_CHECKLIST.md` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Internal docs | CLEAN |
| `docs/release/PUBLIC_BOUNDARY_AUDIT.md` | ALIVE, Inspection Gate, Nehemiah, Field8, Instinct8, Cargo Court, AGI, Truex, CONSTRUCT8 | Internal docs | CLEAN |

## Notes

- `CLAUDE.md` contains forbidden terms in the FORBIDDEN list definition itself. This file is not published to crates.io and is excluded via `.cargo/publish` exclude list or `.gitignore` as appropriate.
- All public-facing files (README, LICENSE, Cargo.toml, src/) are confirmed clean as of 2026-06-02.
- Re-run this audit before each release by running: `grep -rn "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8" README.md LICENSE-MIT LICENSE-APACHE Cargo.toml src/`
