---
name: ui-component
description: Adds a new component to the `src/ui/` zero-dependency terminal design system, wires it into `src/ui/mod.rs`, enforces the color/glyph/width rules, adds `#[cfg(test)]` tests with caps overrides, and showcases it in `cargo cicd ui demo`. Use when the user says "add a UI component", "new widget", "add a <name> component to the design system", or needs a new terminal rendering primitive.
---

# UI Component

Trigger: "add a UI component", "new widget", "add a <name> component to the design system".

Reference: read `src/ui/badge.rs` and `src/ui/panel.rs` before writing.

## Design System Rules (violations cause test failures)

| Rule | Correct | Forbidden |
|------|---------|-----------|
| Color | `Style::paint` / `theme::paint` | Hard-coded `\x1b[...m` sequences |
| Glyphs | `symbols::success()`, `symbols::bullet()`, etc. | Embedded Unicode literals in output strings |
| Width | `text::display_width(s)` | `s.len()` on styled strings |
| Plain mode | Text label alongside colored indicator | Color as sole meaning carrier |

## Step 1 — Create `src/ui/<name>.rs`

```rust
//! `<name>` — brief description.

use crate::ui::{caps, style::{Style, Color}, symbols, text, theme::{self, Role}};

pub struct <Name> {
    // fields
}

impl <Name> {
    pub fn new() -> Self { Self { /* defaults */ } }

    pub fn render(&self) -> String {
        let mut out = String::new();
        // symbols::*() for every glyph
        // Style::paint / theme::paint for every colored span
        // text::display_width for width calculations
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{caps, text::strip_ansi};

    #[test]
    fn renders_in_plain_mode() {
        caps::set_color_override(Some(false));
        caps::set_unicode_override(Some(false));
        let out = <Name>::new().render();
        assert!(!strip_ansi(&out).contains('\u{1b}'), "no ANSI in plain mode");
        // assert key substrings present
        caps::set_color_override(None);
        caps::set_unicode_override(None);
    }

    #[test]
    fn renders_color_when_forced() {
        caps::set_color_override(Some(true));
        let out = <Name>::new().render();
        assert!(out.contains('\u{1b}'), "expected ANSI with color on");
        caps::set_color_override(None);
    }
}
```

`caps::set_color_override` / `caps::set_unicode_override` must be reset to `None` after every test, including on panic paths.

## Step 2 — Register in `src/ui/mod.rs`

```rust
pub mod <name>;  // alphabetical order
// Optional re-export:
pub use <name>::<function>;
```

## Step 3 — Showcase in `cargo cicd ui demo`

In `src/nouns/ui.rs`, inside `render_demo()`:

```rust
push_section(&mut out, "<Name>", w);
let example = crate::ui::<name>::<Name>::new().render();
out.push_str(&indent_block(&example, 2));
out.push('\n');
```

Update the section-presence assertion in `demo_renders_every_section_in_plain_mode`:

```rust
assert!(plain.contains("<Name>"), "missing section: <Name>");
```

## Step 4 — Verify

```sh
cargo build
cargo test --test cli
cargo cicd ui demo | cat    # must produce clean plain text, no escape chars
cargo cicd ui demo          # must render styled output
```

## Checklist

- [ ] `src/ui/<name>.rs`: all color via `Style::paint`, all glyphs via `symbols::*`
- [ ] Width math uses `text::display_width`, not `.len()`
- [ ] `src/ui/mod.rs` declares `pub mod <name>;`
- [ ] Two `#[cfg(test)]` tests: plain-mode and forced-color
- [ ] `caps` overrides reset to `None` after every test
- [ ] Showcased in `render_demo()` and assertion updated
- [ ] `cargo cicd ui demo | cat` produces clean plain text
