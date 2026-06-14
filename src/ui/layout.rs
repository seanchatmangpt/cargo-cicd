//! Layout helpers: indentation, centering, rules, multi-column composition,
//! and terminal hyperlinks.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent implements true
//! side-by-side [`columns`] and OSC-8 [`hyperlink`]; signatures must not change.

use crate::ui::text::{self, Align};

/// Indent every line of `text` by `n` spaces.
pub fn indent(text: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    text.lines()
        .map(|l| format!("{pad}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Center a single line within `width` columns.
pub fn center(text: &str, width: usize) -> String {
    text::pad(text, width, Align::Center)
}

/// A horizontal rule of `width` columns built from `ch`.
pub fn rule(width: usize, ch: &str) -> String {
    text::fill(ch, width)
}

/// Join multi-line `blocks` side by side with `gap` spaces between them.
pub fn columns(blocks: &[&str], gap: usize) -> String {
    // STUB: stack vertically; agent implements real column layout.
    let _ = gap;
    blocks.join("\n")
}

/// A terminal hyperlink (OSC-8) with visible `label`, falling back to the label
/// alone where unsupported.
pub fn hyperlink(label: &str, url: &str) -> String {
    // STUB: label only; agent emits OSC-8 when color is enabled.
    let _ = url;
    label.to_string()
}
