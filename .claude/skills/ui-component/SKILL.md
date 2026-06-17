---
name: ui-component
description: Adds a new component to the `src/ui/` zero-dependency terminal design system, wires it into `src/ui/mod.rs`, enforces the color/glyph/width rules, adds `#[cfg(test)]` tests with caps overrides, and showcases it in `cargo cicd ui demo`. Use when the user says "add a UI component", "new widget", "add a <name> component to the design system", or needs a new terminal rendering primitive.
---
# UI Component

Step-by-step instructions for adding a new component to cargo-cicd's terminal design system at `src/ui/`.

## 1. Understand the design system architecture

The `src/ui/` crate is zero-dependency (std-only). It has two layers:

**Foundation** (never change these without understanding the whole system):
- `caps.rs` — color/unicode capability detection + process-global overrides for tests.
- `style.rs` — `Style` (fg/bg/bold/dim/italic/underline) + free `paint()` fn.
- `symbols.rs` — all glyphs with Unicode↔ASCII fallback via `glyph!` macro.
- `text.rs` — `display_width`, `pad`, `truncate`, `Align`, `strip_ansi`.

**Components** (each is a self-contained module):
`badge`, `chart`, `dashboard`, `diagnostics`, `layout`, `panel`, `progress`, `table`, `theme`, `tree`.

Read `src/ui/badge.rs` and `src/ui/panel.rs` as starting-point references before writing code.

## 2. Decide on the component's public API

A component typically exposes either:
- A builder struct (like `Table::new().headers(...).row(...).render()`) for structured output.
- A set of free functions (like `badge::tag(Verdict)`, `chart::gauge(val, max, width)`) for inline use.

Keep the public API minimal. Every function that returns a `String` is renderable and testable.

## 3. Create `src/ui/<name>.rs`

Follow these mandatory rules:

**Color**: route ALL color through `Style::paint` or `theme::paint`. Never hard-code ANSI escape sequences (`\u{1b}[...m`). Check `caps::color_enabled()` implicitly via `Style::paint` — it returns plain text when color is off.

**Glyphs**: use `symbols::success()`, `symbols::warning()`, `symbols::bullet()`, etc. for every non-alphanumeric glyph. Never embed Unicode literals directly in component output strings. For box-drawing, use `symbols::box_chars(BoxStyle::*)`.

**Width arithmetic**: measure string widths with `text::display_width(s)` (ANSI-aware), never `s.len()`. This matters for right-aligning columns and building fixed-width borders.

**Plain-mode safety**: when color is off (piped output, CI), output must be clean plain text. Do not rely on escape sequences to convey meaning — always include a text label alongside a colored indicator.

**Minimal allocations**: prefer building into a `String` buffer with `push_str` over repeated `format!()` concatenations in hot paths.

Example skeleton:

```rust
//! `<name>` — brief description of what this component renders.

use crate::ui::{caps, style::{Style, Color}, symbols, text, theme::{self, Role}};

/// Builder for a <Name>.
pub struct <Name> {
    // fields
}

impl <Name> {
    pub fn new() -> Self { Self { /* defaults */ } }

    pub fn render(&self) -> String {
        let mut out = String::new();
        // Use symbols::*() for every glyph
        // Use Style::paint / theme::paint for every colored span
        // Use text::display_width for width calculations
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps;
    use crate::ui::text::strip_ansi;

    #[test]
    fn renders_in_plain_mode() {
        caps::set_color_override(Some(false));
        caps::set_unicode_override(Some(false));
        let out = <Name>::new().render();
        let plain = strip_ansi(&out);
        assert!(!plain.contains('\u{1b}'), "no ANSI in plain mode");
        // assert key substrings are present in plain output
        caps::set_color_override(None);
        caps::set_unicode_override(None);
    }

    #[test]
    fn renders_color_when_forced() {
        caps::set_color_override(Some(true));
        let out = <Name>::new().render();
        assert!(out.contains('\u{1b}'), "expected ANSI escapes with color on");
        caps::set_color_override(None);
    }
}
```

Note: `caps::set_color_override` and `caps::set_unicode_override` take `Option<bool>` and must be reset to `None` after each test (use a guard or explicit reset even on panic paths).

## 4. Register in `src/ui/mod.rs`

Add one `pub mod` line in the components section, in alphabetical order:

```rust
pub mod <name>;
```

If the component has a commonly-used free function, optionally re-export it in the curated prelude at the bottom of `mod.rs`:

```rust
pub use <name>::<function>;
```

## 5. Add a showcase section in `cargo cicd ui demo`

Open `src/nouns/ui.rs`. Inside `render_demo()`:

1. Add a new section call:
```rust
push_section(&mut out, "<Name>", w);
```

2. Render a representative example of the component:
```rust
let example = crate::ui::<name>::<Name>::new()
    /* .builder_method(...) */
    .render();
out.push_str(&indent_block(&example, 2));
out.push('\n');
```

3. Update the section-presence assertion in the existing `demo_renders_every_section_in_plain_mode` test in `src/nouns/ui.rs` to include the new section title:
```rust
assert!(plain.contains("<Name>"), "missing section: <Name>");
```

## 6. Verify

```sh
cargo build
cargo test --test cli
cargo cicd ui demo
```

The demo must render cleanly with no panics and must look correct in both color (terminal) and plain (piped) modes:

```sh
cargo cicd ui demo | cat    # plain mode — no escape characters visible
cargo cicd ui demo          # color mode — styled output
```

## 7. Checklist before done

- [ ] `src/ui/<name>.rs` created with all color via `Style::paint`, all glyphs via `symbols::*`.
- [ ] Width math uses `text::display_width`, not `.len()`.
- [ ] `src/ui/mod.rs` declares `pub mod <name>;`.
- [ ] `#[cfg(test)]` block included with at least plain-mode and forced-color tests.
- [ ] `caps::set_color_override(None)` (and unicode) reset after every test.
- [ ] Showcased in `render_demo()` in `src/nouns/ui.rs`.
- [ ] `demo_renders_every_section_in_plain_mode` test updated to assert the new section title.
- [ ] `cargo cicd ui demo | cat` produces clean plain text.
