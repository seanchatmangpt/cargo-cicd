---
description: Run UI demo and dashboard; map src/ui/ modules; annotate output with source module.
allowed-tools: Bash, Read, Glob
---

Trigger: user says "ui demo", "show UI", or runs `/ui-demo`.

## 1 — Run demo and dashboard

```bash
cargo cicd ui demo
cargo cicd ui dashboard
```

Capture full output of both commands.

## 2 — Map design-system modules

```bash
find src/ui -type f -name '*.rs' | sort
```

Read `//!` doc comment from each file. Produce table:

| Module | File | Purpose |
|--------|------|---------|
| `style` | `src/ui/style.rs` | ANSI colour codes, `Style::paint` |
| `symbols` | `src/ui/symbols.rs` | box-drawing + glyph constants |
| `text` | `src/ui/text.rs` | `display_width`, truncation |
| `table` | `src/ui/table.rs` | columnar layout |
| `panel` | `src/ui/panel.rs` | bordered panels via `render_panel()` |
| `badge` | `src/ui/badge.rs` | inline status badges |
| `progress` | `src/ui/progress.rs` | progress bar |
| `chart` | `src/ui/chart.rs` | sparkline/bar chart |
| `tree` | `src/ui/tree.rs` | hierarchical tree |
| `theme` | `src/ui/theme.rs` | colour palette switching |
| `layout` | `src/ui/layout.rs` | multi-column composition |
| `diagnostics` | `src/ui/diagnostics.rs` | error/warning surfaces |
| `dashboard` | `src/ui/dashboard.rs` | composed full-frame view |

Fill any missing rows from actual file content.

## 3 — Annotate demo output

Return captured `ui demo` output with inline annotations:

```
╔══════════════════════╗   ← panel::render_panel()
║  cargo-cicd status   ║
╚══════════════════════╝
  ● PASS  3 targets    ← badge::render_badge()
  [████░░░░] 60%       ← progress::render_bar()
```

## 4 — Adding a new component

1. Create `src/ui/<name>.rs` with `//!` doc comment.
2. Export from `src/ui/mod.rs`.
3. Add a demo case to the `ui demo` command path.
4. All colour via `Style::paint`. All glyphs via `symbols::*`. Width via `text::display_width`.
5. Plain output when not TTY (no ANSI escapes).
6. No external crate dependencies.
