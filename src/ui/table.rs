//! Tables: aligned columns with optional headers and box-drawn borders.
//!
//! A small, zero-dependency table renderer with real box-drawing borders, a
//! styled header row, optional zebra striping, width-capped cells, and a
//! borderless "rule" variant. All width math is ANSI-aware (via
//! [`crate::ui::text::display_width`]) so pre-styled cells align correctly, and
//! every color goes through [`Style::paint`] so output is plain off-TTY.

use crate::ui::style::{Color, Style};
use crate::ui::symbols::{self, BoxStyle};
use crate::ui::text::{self, Align};

/// Default ellipsis used when a cell is truncated to fit [`Table::max_width`].
const ELLIPSIS: &str = "\u{2026}"; // falls back to a plain cut in narrow caps

/// Horizontal padding applied inside every boxed cell (one space each side).
const CELL_PAD: usize = 1;

/// Accent style for header text: bright cyan, bold.
fn header_style() -> Style {
    Style::new().fg(Color::BrightCyan).bold()
}

/// Style for the border glyphs themselves (muted, so data stays prominent).
fn border_style() -> Style {
    Style::new().fg(Color::BrightBlack)
}

/// Style applied to dimmed (zebra) body rows.
fn zebra_style() -> Style {
    Style::new().dim()
}

/// A table builder: configure headers/alignment/box style, push rows, render.
///
/// ```ignore
/// let out = Table::new()
///     .headers(&["Name", "Count"])
///     .align(&[Align::Left, Align::Right])
///     .zebra(true)
///     .row(&["alpha", "12"])
///     .row(&["beta", "340"])
///     .render();
/// ```
pub struct Table {
    headers: Vec<String>,
    aligns: Vec<Align>,
    rows: Vec<Vec<String>>,
    box_style: BoxStyle,
    zebra: bool,
    borderless: bool,
    max_width: Option<usize>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            aligns: Vec::new(),
            rows: Vec::new(),
            box_style: BoxStyle::Light,
            zebra: false,
            borderless: false,
            max_width: None,
        }
    }

    pub fn headers(mut self, cols: &[&str]) -> Self {
        self.headers = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn align(mut self, aligns: &[Align]) -> Self {
        self.aligns = aligns.to_vec();
        self
    }

    pub fn box_style(mut self, s: BoxStyle) -> Self {
        self.box_style = s;
        self
    }

    pub fn row(mut self, cells: &[&str]) -> Self {
        self.rows
            .push(cells.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn push_row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }

    /// Replace all body rows at once.
    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    /// Dim every other body row for easier horizontal scanning.
    pub fn zebra(mut self, on: bool) -> Self {
        self.zebra = on;
        self
    }

    /// Render a header + underline rule + aligned rows, with no outer box.
    pub fn borderless(mut self, on: bool) -> Self {
        self.borderless = on;
        self
    }

    /// Cap each cell to at most `max` display columns, truncating with an
    /// ellipsis. Applies per-cell, before column widths are computed.
    pub fn max_width(mut self, max: usize) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Number of columns, derived from the widest of headers/rows.
    fn ncol(&self) -> usize {
        self.headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0))
    }

    fn align_of(&self, i: usize) -> Align {
        self.aligns.get(i).copied().unwrap_or(Align::Left)
    }

    /// Apply the per-cell width cap (if any) to a single cell.
    fn cap(&self, cell: &str) -> String {
        match self.max_width {
            Some(max) => text::truncate(cell, max, ELLIPSIS),
            None => cell.to_string(),
        }
    }

    /// Build `(headers_capped, rows_capped, widths)` so layout and rendering
    /// share one source of truth. `widths` is the visible content width of each
    /// column (excluding any cell padding).
    fn measure(&self) -> (Vec<String>, Vec<Vec<String>>, Vec<usize>) {
        let ncol = self.ncol();
        let mut widths = vec![0usize; ncol];

        let headers: Vec<String> = self.headers.iter().map(|c| self.cap(c)).collect();
        let rows: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|r| r.iter().map(|c| self.cap(c)).collect())
            .collect();

        for (i, c) in headers.iter().enumerate() {
            widths[i] = widths[i].max(text::display_width(c));
        }
        for r in &rows {
            for (i, c) in r.iter().enumerate() {
                widths[i] = widths[i].max(text::display_width(c));
            }
        }
        (headers, rows, widths)
    }

    /// Pad `cell` to `width` with `align`, then paint with `style` (if any).
    /// Padding happens on the unstyled string so visible width is exact; the
    /// style then wraps the already-sized content, preserving alignment.
    fn cell(cell: &str, width: usize, align: Align, style: Option<Style>) -> String {
        let padded = text::pad(cell, width, align);
        match style {
            Some(s) => s.paint(padded),
            None => padded,
        }
    }

    pub fn render(&self) -> String {
        if self.ncol() == 0 {
            return String::new();
        }
        if self.borderless {
            self.render_borderless()
        } else {
            self.render_boxed()
        }
    }

    /// Boxed layout: top rule, header band, separator, body rows, bottom rule.
    fn render_boxed(&self) -> String {
        let bx = symbols::box_chars(self.box_style);
        let (headers, rows, widths) = self.measure();
        let ncol = widths.len();
        let bstyle = border_style();
        let pad = " ".repeat(CELL_PAD);

        // A horizontal rule with the given corner/connector glyphs.
        let rule = |left: &str, mid: &str, right: &str| -> String {
            let mut s = String::new();
            s.push_str(left);
            for (i, w) in widths.iter().enumerate() {
                s.push_str(&bx.h.repeat(w + CELL_PAD * 2));
                if i + 1 < ncol {
                    s.push_str(mid);
                }
            }
            s.push_str(right);
            bstyle.paint(s)
        };

        let v = bstyle.paint(bx.v);
        // Render one content line from already-padded column strings.
        let content_line = |cols: &[String]| -> String {
            let mut s = String::new();
            s.push_str(&v);
            for col in cols {
                s.push_str(&pad);
                s.push_str(col);
                s.push_str(&pad);
                s.push_str(&v);
            }
            s
        };

        let mut out: Vec<String> = Vec::new();
        out.push(rule(bx.tl, bx.tee_down, bx.tr));

        let has_header = !self.headers.is_empty();
        if has_header {
            let hstyle = header_style();
            let cols: Vec<String> = (0..ncol)
                .map(|i| {
                    let text = headers.get(i).map(String::as_str).unwrap_or("");
                    Self::cell(text, widths[i], self.align_of(i), Some(hstyle))
                })
                .collect();
            out.push(content_line(&cols));
            out.push(rule(bx.tee_right, bx.cross, bx.tee_left));
        }

        for (ri, r) in rows.iter().enumerate() {
            // Zebra dims odd-indexed body rows; only paint when nonempty so we
            // never emit dangling escapes around blank cells.
            let row_style = if self.zebra && ri % 2 == 1 {
                Some(zebra_style())
            } else {
                None
            };
            let cols: Vec<String> = (0..ncol)
                .map(|i| {
                    let text = r.get(i).map(String::as_str).unwrap_or("");
                    Self::cell(text, widths[i], self.align_of(i), row_style)
                })
                .collect();
            out.push(content_line(&cols));
        }

        out.push(rule(bx.bl, bx.tee_up, bx.br));
        out.join("\n")
    }

    /// Borderless layout: styled header, a single underline rule spanning the
    /// columns, then aligned rows — no outer frame, two-space gutters.
    fn render_borderless(&self) -> String {
        let bx = symbols::box_chars(self.box_style);
        let (headers, rows, widths) = self.measure();
        let ncol = widths.len();
        let gutter = "  ";

        let join_cols = |cols: &[String]| cols.join(gutter);

        let mut out: Vec<String> = Vec::new();
        let has_header = !self.headers.is_empty();

        if has_header {
            let hstyle = header_style();
            let cols: Vec<String> = (0..ncol)
                .map(|i| {
                    let text = headers.get(i).map(String::as_str).unwrap_or("");
                    Self::cell(text, widths[i], self.align_of(i), Some(hstyle))
                })
                .collect();
            out.push(join_cols(&cols));

            // Underline rule: one h-run per column, gutter-aligned.
            let bstyle = border_style();
            let segments: Vec<String> = widths.iter().map(|w| text::fill(bx.h, *w)).collect();
            out.push(bstyle.paint(segments.join(gutter)));
        }

        for (ri, r) in rows.iter().enumerate() {
            let row_style = if self.zebra && ri % 2 == 1 {
                Some(zebra_style())
            } else {
                None
            };
            let cols: Vec<String> = (0..ncol)
                .map(|i| {
                    let text = r.get(i).map(String::as_str).unwrap_or("");
                    Self::cell(text, widths[i], self.align_of(i), row_style)
                })
                .collect();
            out.push(join_cols(&cols));
        }

        out.join("\n")
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: a borderless table from string headers + rows.
///
/// Renders a styled header, an underline rule, and aligned rows with no outer
/// box — handy for compact, log-friendly listings.
pub fn simple(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    Table::new()
        .headers(headers)
        .rows(rows)
        .borderless(true)
        .render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps;
    use crate::ui::text::{display_width, strip_ansi};
    use std::sync::{Mutex, MutexGuard};

    // Color/unicode overrides are process-wide atomics; serialize the tests in
    // this module so concurrent cases don't race that shared state. The guard
    // resets both overrides to auto on drop (even on panic).
    static CAPS_LOCK: Mutex<()> = Mutex::new(());

    struct CapsGuard(#[allow(dead_code)] MutexGuard<'static, ()>);

    impl CapsGuard {
        fn acquire(color: bool, unicode: bool) -> Self {
            let g = CAPS_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            caps::set_color_override(Some(color));
            caps::set_unicode_override(Some(unicode));
            CapsGuard(g)
        }
    }

    impl Drop for CapsGuard {
        fn drop(&mut self) {
            caps::set_color_override(None);
            caps::set_unicode_override(None);
        }
    }

    #[test]
    fn empty_table_is_empty() {
        assert_eq!(Table::new().render(), "");
        // No headers, only alignment hints → still empty.
        assert_eq!(Table::new().align(&[Align::Right]).render(), "");
    }

    #[test]
    fn header_is_styled_when_color_forced() {
        let _g = CapsGuard::acquire(true, true);
        let out = Table::new()
            .headers(&["Name", "Count"])
            .row(&["alpha", "1"])
            .render();
        // The header band must carry SGR escapes (bright cyan / bold).
        assert!(
            out.contains('\u{1b}'),
            "expected ANSI escapes in colored output"
        );
        let first_content = out.lines().nth(1).unwrap_or("");
        assert!(
            first_content.contains("\u{1b}["),
            "header row should contain ANSI styling: {first_content:?}"
        );
    }

    #[test]
    fn plain_mode_aligns_and_draws_box() {
        let _g = CapsGuard::acquire(false, true);
        let out = Table::new()
            .headers(&["Name", "N"])
            .align(&[Align::Left, Align::Right])
            .row(&["a", "10"])
            .row(&["bbbb", "2"])
            .render();

        // No ANSI escapes leak into plain output.
        assert!(
            !out.contains('\u{1b}'),
            "plain output must be escape-free: {out:?}"
        );

        // Real box-drawing glyphs appear (corners + light horizontal/vertical).
        assert!(out.contains('\u{250c}'), "missing top-left corner: {out:?}");
        assert!(
            out.contains('\u{2518}'),
            "missing bottom-right corner: {out:?}"
        );
        assert!(out.contains('\u{2500}'), "missing horizontal rule: {out:?}");
        assert!(
            out.contains('\u{2502}'),
            "missing vertical separator: {out:?}"
        );
        assert!(
            out.contains('\u{252c}'),
            "missing top tee connector: {out:?}"
        );

        // Every line has identical display width → columns are aligned.
        let widths: Vec<usize> = out.lines().map(display_width).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven line widths: {widths:?}"
        );

        // Right-aligned numeric column: the narrow "2" is space-padded so it
        // sits flush-right under the wider "10" (no immediate vertical bar).
        let row_two = out
            .lines()
            .find(|l| l.contains("bbbb"))
            .expect("could not locate the 'bbbb' data row");
        assert!(
            row_two.contains(" 2 \u{2502}"),
            "right-aligned cell not padded flush-right: {row_two:?}"
        );
    }

    #[test]
    fn borderless_has_underline_rule_no_box() {
        let _g = CapsGuard::acquire(false, true);
        let out = simple(
            &["Key", "Value"],
            vec![vec!["a".into(), "longvalue".into()]],
        );

        // Underline rule present, but no box corners.
        assert!(
            out.contains('\u{2500}'),
            "borderless should still draw an underline: {out:?}"
        );
        assert!(
            !out.contains('\u{250c}'),
            "borderless must not draw a top-left corner: {out:?}"
        );
        assert!(
            !out.contains('\u{2502}'),
            "borderless must not draw vertical bars: {out:?}"
        );

        // Header + rule + one data row.
        assert_eq!(
            out.lines().count(),
            3,
            "expected header, rule, one row: {out:?}"
        );

        // Columns still line up.
        let widths: Vec<usize> = out.lines().map(display_width).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven widths: {widths:?}"
        );
    }

    #[test]
    fn max_width_truncates_with_ellipsis() {
        let _g = CapsGuard::acquire(false, true);
        let out = Table::new()
            .headers(&["Path"])
            .max_width(6)
            .row(&["a/very/long/path"])
            .render();
        // The long cell was cut and carries the ellipsis glyph.
        assert!(
            out.contains('\u{2026}'),
            "expected ellipsis after truncation: {out:?}"
        );
        // No content line exceeds the cap + borders/padding.
        for line in out.lines() {
            assert!(
                display_width(line) <= 6 + 2 * CELL_PAD + 2,
                "line too wide: {line:?}"
            );
        }
    }

    #[test]
    fn zebra_dims_alternating_rows_only_with_color() {
        let _g = CapsGuard::acquire(true, true);
        let out = Table::new()
            .headers(&["X"])
            .zebra(true)
            .rows(vec![
                vec!["one".into()],
                vec!["two".into()],
                vec!["three".into()],
            ])
            .render();
        // The dim SGR parameter (2) should appear for the striped row.
        assert!(
            out.contains("\u{1b}[2"),
            "expected a dim escape for zebra row: {out:?}"
        );
        // Stripping ANSI must not change column alignment.
        let plain = strip_ansi(&out);
        let widths: Vec<usize> = plain.lines().map(display_width).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven widths under zebra: {widths:?}"
        );
    }

    #[test]
    fn ascii_fallback_uses_plus_and_dash() {
        let _g = CapsGuard::acquire(false, false);
        let out = Table::new().headers(&["A", "B"]).row(&["1", "2"]).render();
        assert!(out.contains('+'), "ASCII box should use '+': {out:?}");
        assert!(out.contains('-'), "ASCII box should use '-': {out:?}");
        assert!(out.contains('|'), "ASCII box should use '|': {out:?}");
        assert!(
            !out.contains('\u{250c}'),
            "no unicode corners in ASCII mode: {out:?}"
        );
    }
}
