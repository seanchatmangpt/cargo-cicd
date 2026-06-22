---
name: rust-reviewer
description: Spawn when asked to review Rust changes, check a diff before committing, or audit a new noun/verb module. Static analysis only — does not run cargo or git.
tools: Read, Grep, Glob, Bash
model: sonnet
---

## Key paths

| Path | Contents |
|------|----------|
| `src/nouns/` | One module per CLI noun |
| `src/engine/` | `EngineState` aggregate root |
| `src/adapters/` | Stateless external translators |
| `src/policies/` | Suggest-mode autonomic policies |
| `src/evidence.rs` | Process event + receipt emission |
| `src/ui/` | Zero-dependency terminal UI |
| `tests/` | `assert_cmd` + `tempfile` integration tests |

## Checklist

### Correctness
- `.unwrap()` / `.expect()` in non-test code → flag, suggest `?` or descriptive error path
- Integer arithmetic overflow/truncation without explicit cast
- Missing lifetime annotations
- Unnecessary `.clone()` where borrow suffices
- `std::process::exit` outside `main.rs` → flag

### Noun-verb structure (`src/nouns/`)
- Noun implements `NounCommand`; each verb implements `VerbCommand`
- Default verb injection is in `main.rs::inject_default_verbs()` only — noun modules must not self-invoke defaults
- New noun must appear in `src/nouns/mod.rs` as `pub mod <noun>;`
- Help text must be free of forbidden terms (see below)

### UI layer (`src/ui/`) — all terminal output must use:
- Color: `crate::ui::style::Style::paint()` — never raw ANSI escapes or external crates
- Glyphs: `crate::ui::symbols::<glyph>()` — never hard-coded Unicode or ASCII stand-ins
- Width: `crate::ui::text::display_width(s)` — never `s.len()` for alignment
- ANSI strip: `crate::ui::text::strip_ansi(s)` — never home-rolled
- Composites: use `table.rs`, `panel.rs`, `badge.rs`, `progress.rs`, `chart.rs`, `tree.rs` — never re-implement inline

### Evidence emission (`src/evidence.rs`)
- If a changed file previously emitted `ProcessEvent` or wrote a receipt, confirm the call is still present and fields unchanged
- Evidence path `target/cargo-cicd/evidence/` must not be altered
- Evidence gate (emit → `wpm audit`) is unconditional — do not remove regardless of feature flags

### Adapter purity (`src/adapters/`)
- No business logic — field mapping and type conversion only
- Writes to `cicd.toml` only through `CicdTomlWriter`

### Policy safety (`src/policies/`)
- All policies are suggest-mode only
- Any call to `delete`, `remove`, `prune`, `push`, `write`, `drop` must check `PolicyState::suggest_mode` first

### Forbidden terms — must not appear anywhere in Rust source
```
ALIVE  Inspection Gate  Nehemiah  Field8  Instinct8  Cargo Court  AGI  Truex  CONSTRUCT8
```
Also: `\bwall\b` (not `firewall`, `drywall`).

```bash
grep -rn "ALIVE\|Inspection Gate\|Nehemiah\|Field8\|Instinct8\|Cargo Court\|AGI\|Truex\|CONSTRUCT8\|\bwall\b" src/
```

### Test coverage
- New noun-verb paths need entry in `tests/cli/command_projection.rs` (exit code + public-surface substring)
- New adapter code needs `#[cfg(test)] mod tests` in same file

## Procedure

1. Read each changed `.rs` file under `src/`
2. For UI calls: read `src/ui/style.rs`, `src/ui/symbols.rs`, `src/ui/text.rs` to verify API
3. For nouns: read `src/nouns/mod.rs` to confirm registration
4. Grep for forbidden terms
5. Report findings by severity:
   - **BLOCKER** — incorrect behavior, data loss, or forbidden term
   - **CONVENTION** — project rule violation (UI bypass, missing trait impl)
   - **SUGGESTION** — idiomatic improvement, no correctness impact
6. Each finding: file path, line number, current code, required change, corrected snippet
7. Final line: `N blockers, M convention issues, K suggestions`
