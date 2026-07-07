//! Panels, banners, section headers, dividers, and key/value blocks.
//!
//! Polished, zero-dependency containers built on the shared foundation
//! ([`caps`](crate::ui::caps), [`style`](crate::ui::style),
//! [`symbols`](crate::ui::symbols), [`text`](crate::ui::text)). Every helper
//! degrades gracefully: color is emitted only through [`Style::paint`] (so it
//! auto-disables off-TTY) and borders go through
//! [`symbols::box_chars`](crate::ui::symbols::box_chars) (so ASCII fallback
//! works). Because color is suppressed on non-terminals, captured/piped output
//! is plain — which is why [`header`] and [`banner`] keep their `title`
//! argument as a contiguous substring of the plain-text output (the
//! public-boundary tests assert on title text).

use crate::ui::caps;
use crate::ui::style::{Color, Style};
use crate::ui::symbols::{self, BoxChars, BoxStyle};
use crate::ui::text::{self, Align};

/// Default inner content width when a [`Panel`] does not pin its own width.
fn default_inner_width() -> usize {
    // Bounded, terminal-aware width: at most 80 columns, never below 20.
    caps::content_width(80)
}

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

/// Fit `line` to exactly `width` display columns: pad short lines on the left,
/// and clip overlong ones (with an ellipsis) so a panel's right border always
/// lands in the same column. ANSI styling in `line` is measured by display
/// width; when a clip is required the surviving text is plain (escapes are
/// dropped by [`text::truncate`]), which is the safe choice for a hard border.
fn fit_line(line: &str, width: usize) -> String {
    if text::display_width(line) <= width {
        text::pad(line, width, Align::Left)
    } else {
        // Truncate to the inner width, then pad in case the ellipsis logic left
        // us a column short (e.g. width below the ellipsis width).
        let cut = text::truncate(line, width, symbols::ellipsis());
        text::pad(&cut, width, Align::Left)
    }
}

/// A bordered panel with an optional title and a list of body lines.
///
/// The title is embedded into the top border, e.g. `┌─ Title ───────┐`. Body
/// lines are each padded to the inner width and wrapped with vertical borders
/// and one column of inner padding on either side. Body lines may themselves
/// contain ANSI styling; widths are measured with [`text::display_width`] so
/// styled cells still align.
pub struct Panel {
    title: Option<String>,
    lines: Vec<String>,
    box_style: BoxStyle,
    width: Option<usize>,
    border: Option<Style>,
    padding: usize,
}

impl Panel {
    /// An empty light-bordered panel with one column of inner padding.
    pub fn new() -> Self {
        Self {
            title: None,
            lines: Vec::new(),
            box_style: BoxStyle::Light,
            width: None,
            border: None,
            padding: 1,
        }
    }

    /// Set the title embedded in the top border.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Choose the box-drawing family for the border.
    pub fn box_style(mut self, s: BoxStyle) -> Self {
        self.box_style = s;
        self
    }

    /// Pin the inner content width (columns between the inner padding), instead
    /// of deriving it from the terminal.
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Color the border with `style`. Honors [`caps::color_enabled`], so the
    /// border is plain box-drawing when color is off.
    pub fn style(mut self, style: Style) -> Self {
        self.border = Some(style);
        self
    }

    /// Set the number of blank columns between the vertical border and the
    /// content on each side (default `1`).
    #[allow(dead_code, reason = "builder-completeness sibling of width()/style(), which are used")]
    pub fn padding(mut self, n: usize) -> Self {
        self.padding = n;
        self
    }

    /// Append a body line. Lines may contain ANSI styling.
    pub fn push(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    /// Append a blank separator row, for visual grouping inside a panel.
    #[allow(dead_code, reason = "builder-completeness sibling of push(), which is used")]
    pub fn push_blank(self) -> Self {
        self.push(String::new())
    }

    /// Append a full-width inner rule (a horizontal line spanning the content
    /// area), for sectioning a panel's body. The rule uses the panel's own box
    /// family so it matches the border.
    #[allow(dead_code, reason = "builder-completeness sibling of push(), which is used")]
    pub fn push_rule(self) -> Self {
        let bx = symbols::box_chars(self.box_style);
        let inner = self.inner_width();
        let rule = rule_style().paint(text::fill(bx.h, inner));
        self.push(rule)
    }

    /// Color a border segment, falling back to the raw glyph when no border
    /// style is set or color is disabled.
    fn paint_border(&self, s: &str) -> String {
        match self.border {
            Some(style) => style.paint(s),
            None => s.to_string(),
        }
    }

    /// Inner width available to content, accounting for left/right padding.
    fn inner_width(&self) -> usize {
        let total = self.width.unwrap_or_else(default_inner_width);
        let pad = self.padding.saturating_mul(2);
        total.saturating_sub(pad).max(1)
    }

    /// Build the top border, embedding the title when present:
    /// `┌─ Title ──────┐`. The run between corners always spans `span` columns.
    fn top_border(&self, bx: &BoxChars, span: usize) -> String {
        let mid = match &self.title {
            Some(title) if !title.is_empty() => {
                // Layout: `<h> <title> ` then fill the remainder with `h`, so the
                // visible run is exactly `span` columns. The lead glyph and fill
                // are border-colored; the title is accent-colored. A title too
                // wide for the box is truncated so the border stays rectangular.
                let fixed = 1 + 1 + 1; // lead glyph + space on each side
                let title = if text::display_width(title) + fixed > span {
                    text::truncate(title, span.saturating_sub(fixed), symbols::ellipsis())
                } else {
                    title.clone()
                };
                let label_w = fixed + text::display_width(&title);
                let rest = span.saturating_sub(label_w);
                format!(
                    "{} {} {}",
                    self.paint_border(bx.h),
                    accent_style().paint(&title),
                    self.paint_border(&bx.h.repeat(rest)),
                )
            }
            _ => self.paint_border(&bx.h.repeat(span)),
        };
        format!(
            "{}{}{}",
            self.paint_border(bx.tl),
            mid,
            self.paint_border(bx.tr)
        )
    }

    /// Render one body row: `│ <padded-content> │` (padding columns scale with
    /// [`Panel::padding`]).
    fn body_row(&self, bx: &BoxChars, line: &str, inner: usize, pad: &str) -> String {
        let body = fit_line(line, inner);
        format!(
            "{}{}{}{}{}",
            self.paint_border(bx.v),
            pad,
            body,
            pad,
            self.paint_border(bx.v),
        )
    }

    /// Render the panel as a real bordered box.
    pub fn render(&self) -> String {
        let bx = symbols::box_chars(self.box_style);
        let inner = self.inner_width();
        let pad = " ".repeat(self.padding);
        // Span between the two corner glyphs of the box.
        let span = inner + self.padding * 2;

        let mut rows: Vec<String> = Vec::with_capacity(self.lines.len() + 2);
        rows.push(self.top_border(&bx, span));
        for line in &self.lines {
            rows.push(self.body_row(&bx, line, inner, &pad));
        }
        rows.push(format!(
            "{}{}{}",
            self.paint_border(bx.bl),
            self.paint_border(&bx.h.repeat(span)),
            self.paint_border(bx.br),
        ));
        rows.join("\n")
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
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

/// A full-width framed banner: a heavy box with the `title` centered and
/// accent-colored over a centered, dimmed `subtitle`, with a blank row of
/// breathing space framing the text block.
///
/// The `title` is centered with leading/trailing spaces only, so it remains a
/// contiguous substring of the plain (uncolored) output.
pub fn banner(title: &str, subtitle: &str) -> String {
    let bx = symbols::box_chars(BoxStyle::Heavy);
    let inner = caps::content_width(80).saturating_sub(4).max(1);
    let border = rule_style();

    // A horizontal border run of `inner + 2` columns (the body rows carry one
    // space of padding on each side, matching the `+2`).
    let edge = |corner_l: &str, corner_r: &str| {
        format!(
            "{}{}{}",
            border.paint(corner_l),
            border.paint(bx.h.repeat(inner + 2)),
            border.paint(corner_r),
        )
    };
    let top = edge(bx.tl, bx.tr);
    let bottom = edge(bx.bl, bx.br);

    // A body row with one space of padding either side of `content` (already
    // padded to `inner`), framed by vertical borders.
    let row = |content: &str| format!("{} {} {}", border.paint(bx.v), content, border.paint(bx.v),);
    let blank = row(&" ".repeat(inner));

    // Center the *plain* title to compute even padding, then paint the centered
    // string so styling never disturbs the contiguous title.
    let title_centered = text::pad(title, inner, Align::Center);
    let title_row = row(&accent_style().paint(&title_centered));

    let mut rows = vec![top, blank.clone(), title_row];
    if !subtitle.is_empty() {
        let sub_centered = text::pad(subtitle, inner, Align::Center);
        rows.push(row(&muted_style().paint(&sub_centered)));
    }
    rows.push(blank);
    rows.push(bottom);
    rows.join("\n")
}

/// A labeled horizontal divider spanning `width` columns:
/// `── label ─────`. The label is inset after a short lead rule and dimmed.
///
/// With an empty label the divider is a plain full-width rule.
pub fn divider(label: &str, width: usize) -> String {
    let bx = symbols::box_chars(BoxStyle::Light);

    if label.is_empty() {
        return rule_style().paint(text::fill(bx.h, width));
    }

    // `<lead> label <tail rule>` where lead is a fixed short rule and the tail
    // fills the remaining width.
    let lead_w = 2usize;
    let label_w = text::display_width(label);
    // lead + 1 space + label + 1 space, then the remainder.
    let used = lead_w + 1 + label_w + 1;
    let tail = width.saturating_sub(used);

    let lead = rule_style().paint(bx.h.repeat(lead_w));
    let painted_label = muted_style().paint(label);
    let tail_rule = rule_style().paint(text::fill(bx.h, tail));
    format!("{lead} {painted_label} {tail_rule}")
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
