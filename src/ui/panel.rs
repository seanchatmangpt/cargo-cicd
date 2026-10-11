//! Section headers and key/value blocks.
//!
//! Polished, zero-dependency containers built on the shared foundation
//! ([`caps`](crate::ui::caps), [`style`](crate::ui::style),
//! [`symbols`](crate::ui::symbols), [`text`](crate::ui::text)). Every helper
//! degrades gracefully: color is emitted only through [`Style::paint`] (so it
//! auto-disables off-TTY) and borders go through
//! [`symbols::box_chars`](crate::ui::symbols::box_chars) (so ASCII fallback
//! works). Because color is suppressed on non-terminals, captured/piped output
//! is plain — which is why [`header`] keeps its `title` argument as a
//! contiguous substring of the plain-text output (the public-boundary tests
//! assert on title text).

use crate::ui::caps;
use crate::ui::style::{Color, Style};
use crate::ui::symbols::{self, BoxStyle};
use crate::ui::text::{self, Align};

/// Accent style for titles and emphasized rules (bold, bright cyan).
fn accent_style() -> Style {
    Style::new().fg(Color::Cyan).bold()
}

/// Muted style for subtitles, labels, and inset divider text.
fn muted_style() -> Style {
    Style::new().dim()
}

/// Rule style for plain horizontal lines (a single cyan tone, not bold) so
/// rules read as structure rather than emphasis.
fn rule_style() -> Style {
    Style::new().fg(Color::Cyan)
}

/// A section header: an accent-styled `title` on its own line above a
/// full-width horizontal rule.
///
/// In plain mode the title is emitted verbatim, so it is always a contiguous
/// substring of the output.
pub fn header(title: &str) -> String {
    let bx = symbols::box_chars(BoxStyle::Light);
    let width = caps::content_width(80);
    let rule = rule_style().paint(text::fill(bx.h, width));
    format!("{}\n{}", accent_style().paint(title), rule)
}

/// An aligned key/value block: keys are dimmed and left-padded to a common
/// width, values follow at a fixed gutter and are emitted normally.
pub fn kv(pairs: &[(&str, &str)]) -> String {
    kv_aligned(pairs, Align::Left)
}

/// Like [`kv`] but with explicit key alignment within the key column. `Right`
/// alignment yields a flush-right "label: value" ledger look; `Left` keeps the
/// default block.
pub fn kv_aligned(pairs: &[(&str, &str)], key_align: Align) -> String {
    if pairs.is_empty() {
        return String::new();
    }
    let key_style = muted_style();
    let key_w = pairs
        .iter()
        .map(|(k, _)| text::display_width(k))
        .max()
        .unwrap_or(0);

    pairs
        .iter()
        .map(|(k, v)| {
            // Pad the *plain* key to alignment width, then paint, so styling
            // never shifts the value column.
            let padded = text::pad(k, key_w, key_align);
            format!("{}  {}", key_style.paint(&padded), v)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
