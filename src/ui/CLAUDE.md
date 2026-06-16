# src/ui — Terminal UI Design System

This directory is the zero-dependency terminal UI toolkit for cargo-cicd.
Every noun that produces terminal output must render through these modules.
No external crates — only `std`.

---

## Module Split: Foundation vs Components

**Foundation modules** (no rendering logic — every other module depends on these):

| Module | What it owns |
|---|---|
| `caps.rs` | Runtime terminal capability detection: colour support, Unicode support, TTY vs pipe. Query `caps::color_enabled()` and `caps::unicode_enabled()` before rendering. |
| `style.rs` | `Style` type and `Style::paint(text, style)` — the sole entry point for coloured output. Reads `caps` at call time; returns a plain string when not a TTY. |
| `symbols.rs` | Named glyph constants. At startup, `symbols` selects Unicode or ASCII variants based on `caps::unicode_enabled()`. |
| `text.rs` | String utilities: `display_width(s)` (Unicode-aware), `truncate(s, max_cols)`. |
| `theme.rs` | Named colour palette. `Theme::default()` returns a colour-aware palette; `Theme::plain()` returns a no-colour palette. Noun modules should accept a `&Theme` parameter rather than hard-coding colours. |

**Component modules** (rendering — depend on foundation):

| Module | What it renders |
|---|---|
| `table.rs` | Columnar table. Always use `text::display_width` for column sizing, never `.len()`. |
| `panel.rs` | Bordered panel with an optional title string. |
| `badge.rs` | Inline status badge: `[PASS]`, `[FAIL]`, `[SKIP]`, `[WARN]`, etc. |
| `progress.rs` | Single-line progress indicator / spinner for long-running verbs. |
| `chart.rs` | Horizontal bar chart for numeric summaries (e.g., target sizes, test counts). |
| `tree.rs` | Hierarchical tree for workspace/package structures. |
| `diagnostics.rs` | Structured diagnostic messages: error/warn/info/hint with optional source location. |
| `layout.rs` | Composes panels, tables, and trees into a multi-section layout for dashboard use. |
| `dashboard.rs` | Full-workspace status dashboard; invoked by `cargo cicd ui dashboard`. |

---

## The Three Rendering Rules

These rules are non-negotiable. Tests in `tests/invariants.rs` and the CLI tests
capture non-TTY output and will fail if any rule is broken.

### 1. Colour via `Style::paint` only

```rust
// Correct
use crate::ui::style::{Style, Color};
println!("{}", Style::paint("OK", Style::bold().fg(Color::Green)));

// Wrong — never do this
println!("\x1b[32mOK\x1b[0m");
```

`Style::paint` checks `caps::color_enabled()` internally and returns a plain
string when stdout is not a TTY. Raw ANSI codes bypass that check and corrupt
piped output.

### 2. Glyphs via `symbols::*` only

```rust
// Correct
use crate::ui::symbols;
println!("{} all checks passed", symbols::CHECK);

// Wrong — never do this
println!("✓ all checks passed");
println!("✔ all checks passed");
```

`symbols` selects Unicode or ASCII at runtime. Hard-coded glyphs break terminals
that report no Unicode support and corrupt ASCII-only log captures.

### 3. Column widths via `text::display_width`

```rust
// Correct
use crate::ui::text::display_width;
let pad = col_width - display_width(cell);

// Wrong
let pad = col_width - cell.len();
```

Multi-byte characters (e.g., box-drawing, CJK, emoji) have a display width that
differs from their byte length. Using `.len()` produces misaligned tables.

---

## Adding a New Component

1. Create `src/ui/<name>.rs`.
2. Import only from `std` and from the foundation modules (`caps`, `style`,
   `symbols`, `text`, `theme`). Do not import from other component modules unless
   absolutely necessary and clearly justified.
3. Expose a single primary function or struct that accepts a `&Theme` parameter
   so callers can pass `Theme::plain()` in tests.
4. Re-export from `src/ui/mod.rs`:
   ```rust
   pub mod name;
   pub use name::YourType;
   ```
5. Add at least one rendering example to the `cargo cicd ui demo` command
   (`src/nouns/ui.rs`, `UiDemoVerb`) so the component appears in the live preview.
6. Write a non-TTY test: capture output with stdout redirected to a pipe, assert
   no `\x1b` escape bytes, and assert no non-ASCII bytes if the content is
   expected to be ASCII-safe.

---

## Plain-Mode Output Contract

When stdout is not a TTY (pipe, file redirect, CI log capture):
- Zero ANSI escape codes in output.
- Glyphs are ASCII fallbacks (e.g., `[+]` instead of `✓`).
- Table borders use ASCII (`+`, `-`, `|`) not box-drawing characters.
- `Style::paint` and `symbols::*` handle this automatically — no `if tty` guards
  needed in component code.

The `cargo cicd ui demo` and `cargo cicd ui dashboard` commands both respect this
contract. Run them in a pipe (`cargo cicd ui demo | cat`) to verify plain mode.
