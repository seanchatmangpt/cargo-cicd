---
description: Build and run the UI demo and dashboard, then explain the design-system modules under src/ui/.
allowed-tools: Bash, Read, Glob
---

You are exploring and demonstrating the cargo-cicd terminal UI design system.

---

## Step 1 — Run the UI demo

```
cargo cicd ui demo
```

Capture the full output. This exercises all design-system primitives: panels, badges, progress bars, tables, trees, and charts.

---

## Step 2 — Run the dashboard

```
cargo cicd ui dashboard
```

The dashboard is a composed view that lays out multiple UI components in a single terminal frame. Capture its output.

---

## Step 3 — Map the design-system modules

List every file under `src/ui/`:

```bash
find src/ui -type f -name '*.rs' | sort
```

Then read each module's top-level doc comment (the first `//!` block) to understand its role. Summarise the purpose of each module:

| Module | File | Purpose |
|--------|------|---------|
| `style` | `src/ui/style.rs` | … |
| `symbols` | `src/ui/symbols.rs` | … |
| `text` | `src/ui/text.rs` | … |
| `table` | `src/ui/table.rs` | … |
| `panel` | `src/ui/panel.rs` | … |
| `badge` | `src/ui/badge.rs` | … |
| `progress` | `src/ui/progress.rs` | … |
| `chart` | `src/ui/chart.rs` | … |
| `tree` | `src/ui/tree.rs` | … |
| `theme` | `src/ui/theme.rs` | … |
| `layout` | `src/ui/layout.rs` | … |
| `diagnostics` | `src/ui/diagnostics.rs` | … |
| `dashboard` | `src/ui/dashboard.rs` | … |

Fill in any missing rows by reading the actual files.

---

## Step 4 — Design-system architecture

After reading the modules, explain:

1. **Layering** — how the modules depend on each other (e.g. `panel` uses `style` + `symbols`; `dashboard` uses `layout` + `panel` + `table`).
2. **Zero-dependency constraint** — the UI system has no external crate dependencies; describe how that shapes the implementation (e.g. ANSI codes written by hand in `style`, box-drawing characters in `symbols`).
3. **Theme system** — how `theme.rs` allows switching between colour palettes without touching individual components.
4. **Adding a new component** — the steps a developer would take to add, e.g., a `spinner` component: create `src/ui/spinner.rs`, export from `src/ui/mod.rs`, and exercise it in the `ui demo` command path.

---

## Step 5 — Live output annotation

Return the captured output from Step 1 (`ui demo`) and annotate which design-system module produced each visible section. For example:

```
╔══════════════════════╗   ← panel::render_panel()
║  cargo-cicd status   ║
╚══════════════════════╝
  ● PASS  3 targets    ← badge::render_badge()
  [████░░░░] 60%       ← progress::render_bar()
```
