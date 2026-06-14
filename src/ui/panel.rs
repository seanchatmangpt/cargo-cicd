//! Panels, banners, section headers, dividers, and key/value blocks.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent draws real
//! bordered boxes; signatures must not change. [`header`] and [`banner`] MUST
//! keep their `title` argument as a contiguous substring of the plain-text
//! output (the public-boundary tests assert on title text).

use crate::ui::symbols::{self, BoxStyle};
use crate::ui::text::{self, Align};

/// A bordered panel with an optional title and a list of body lines.
pub struct Panel {
    title: Option<String>,
    lines: Vec<String>,
    box_style: BoxStyle,
    width: Option<usize>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            title: None,
            lines: Vec::new(),
            box_style: BoxStyle::Light,
            width: None,
        }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn box_style(mut self, s: BoxStyle) -> Self {
        self.box_style = s;
        self
    }
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }
    pub fn push(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }
    pub fn render(&self) -> String {
        // STUB: title + lines, no border. Agent draws a real box using
        // `symbols::box_chars(self.box_style)` and `self.width`.
        let _ = (self.box_style, self.width);
        let mut out = String::new();
        if let Some(t) = &self.title {
            out.push_str(t);
            out.push('\n');
        }
        out.push_str(&self.lines.join("\n"));
        out
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

/// A section header: `title` on its own line above an underline rule.
pub fn header(title: &str) -> String {
    let w = text::display_width(title);
    format!("{title}\n{}", "=".repeat(w.max(1)))
}

/// A full-width banner with a title and optional subtitle.
pub fn banner(title: &str, subtitle: &str) -> String {
    // STUB: title + subtitle; agent frames it. Title stays contiguous.
    if subtitle.is_empty() {
        title.to_string()
    } else {
        format!("{title}\n{subtitle}")
    }
}

/// A labeled horizontal divider spanning `width` columns.
pub fn divider(label: &str, width: usize) -> String {
    let h = symbols::box_chars(BoxStyle::Light).h;
    if label.is_empty() {
        text::fill(h, width)
    } else {
        let prefix = h.repeat(3);
        let used = text::display_width(label) + 5;
        format!("{prefix} {label} {}", text::fill(h, width.saturating_sub(used)))
    }
}

/// An aligned key/value block.
pub fn kv(pairs: &[(&str, &str)]) -> String {
    let w = pairs
        .iter()
        .map(|(k, _)| text::display_width(k))
        .max()
        .unwrap_or(0);
    pairs
        .iter()
        .map(|(k, v)| format!("{}  {}", text::pad(k, w, Align::Left), v))
        .collect::<Vec<_>>()
        .join("\n")
}
