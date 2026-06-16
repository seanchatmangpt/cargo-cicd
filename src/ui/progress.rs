//! Progress feedback: spinners, progress bars, and step checklists.
//!
//! Animated output goes to **stderr** and must no-op when stderr is not a TTY,
//! so captured stdout stays clean. Determinate bars and checklists are pure
//! string builders (no I/O) and degrade gracefully to plain, aligned ASCII when
//! color or Unicode is disabled.
//!
//! Frozen public API: [`Spinner::new`]/[`Spinner::tick`]/[`Spinner::finish`],
//! [`ProgressBar::new`]/[`ProgressBar::set`]/[`ProgressBar::inc`]/
//! [`ProgressBar::finish`]/[`ProgressBar::render`], and the free functions
//! [`bar`] and [`steps`].

use std::io::{stderr, IsTerminal, Write};

use crate::ui::caps;
use crate::ui::style::{Color, Style};
use crate::ui::symbols;
use crate::ui::text::{self, Align};

/// Color for the spinning glyph while work is in progress.
const SPINNER_STYLE: Style = Style::new().fg(Color::Cyan).bold();
/// Color for the success glyph printed when a spinner finishes.
const SUCCESS_STYLE: Style = Style::new().fg(Color::Green).bold();
/// Dim style for secondary / pending text.
const DIM_STYLE: Style = Style::new().dim();

/// True when animated spinner output should be drawn: stderr is a real terminal
/// *and* color/ANSI control is enabled. When false, [`Spinner`] is inert so that
/// captured output stays pristine.
fn animation_enabled() -> bool {
    stderr().is_terminal() && caps::color_enabled()
}

/// An animated single-line spinner (renders to stderr when interactive).
///
/// Each [`tick`](Spinner::tick) repaints the current line in place using a
/// carriage return plus a clear-to-end-of-line, so successive frames overwrite
/// one another without scrolling. When stderr is not a terminal (e.g. piped to a
/// file or captured by a test), every method is a no-op.
pub struct Spinner {
    message: String,
    frame: usize,
    enabled: bool,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frame: 0,
            enabled: animation_enabled(),
        }
    }

    /// Advance one frame and (when enabled) repaint the spinner line on stderr.
    pub fn tick(&mut self) {
        if self.enabled {
            let line = spinner_line(self.frame, &self.message);
            // `\r` returns to column 0, `\x1b[K` clears the stale tail so a
            // shorter message never leaves leftover characters behind.
            let mut err = stderr();
            let _ = write!(err, "\r\u{1b}[K{line}");
            let _ = err.flush();
        }
        self.frame = self.frame.wrapping_add(1);
    }

    /// Stop the spinner, clearing its line and printing `final_msg`.
    ///
    /// When interactive, the in-progress line is erased and replaced by a green
    /// success glyph followed by `final_msg` (terminated by a newline). When not
    /// interactive this is a no-op so stdout stays clean.
    pub fn finish(self, final_msg: &str) {
        if self.enabled {
            let glyph = SUCCESS_STYLE.paint(symbols::success());
            let mut err = stderr();
            let _ = write!(err, "\r\u{1b}[K{glyph} {final_msg}\n");
            let _ = err.flush();
        }
    }
}

/// A determinate progress bar.
///
/// Tracks a position out of a total and renders a labeled, sub-cell-precise bar
/// with a trailing percentage. Rendering is pure (no I/O); callers decide where
/// to print the returned string.
pub struct ProgressBar {
    total: u64,
    pos: u64,
    label: Option<String>,
    width: usize,
}

/// Default inner width (in cells) of a rendered [`ProgressBar`].
const DEFAULT_BAR_WIDTH: usize = 24;

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            pos: 0,
            label: None,
            width: DEFAULT_BAR_WIDTH,
        }
    }

    /// Attach a leading label rendered to the left of the bar.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Override the bar's inner width in cells (clamped to a sane minimum).
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width.max(4);
        self
    }

    pub fn set(&mut self, pos: u64) {
        self.pos = pos.min(self.total);
    }

    pub fn inc(&mut self, delta: u64) {
        self.pos = (self.pos + delta).min(self.total);
    }

    pub fn finish(self) {}

    /// Current completion as a fraction in `0.0..=1.0`.
    fn fraction(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            (self.pos as f64 / self.total as f64).clamp(0.0, 1.0)
        }
    }

    /// Render a labeled bar with a trailing percentage, e.g.
    /// `build [██████░░░░] 60%`. The fill is colored by completion threshold
    /// (red → yellow → green) when color is enabled.
    pub fn render(&self) -> String {
        let frac = self.fraction();
        let pct = (frac * 100.0).round() as u64;
        let gauge = colored_bar(frac, self.width, threshold_color(frac));
        let pct_text = format!("{pct:>3}%");
        match &self.label {
            Some(label) => format!("{label} [{gauge}] {pct_text}"),
            None => format!("[{gauge}] {pct_text}"),
        }
    }
}

/// Pick a fill color based on completion: low fractions read as red, the middle
/// band as yellow, and near-complete as green.
fn threshold_color(frac: f64) -> Color {
    if frac >= 0.999 {
        Color::Green
    } else if frac >= 0.66 {
        Color::BrightGreen
    } else if frac >= 0.33 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Build the raw (uncolored, ANSI-free) glyph string for `fraction` over
/// `width` cells, using eighth-block sub-cell precision for the partial cell.
fn raw_bar(fraction: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let f = fraction.clamp(0.0, 1.0);
    let blocks = symbols::hblocks();
    let full = blocks[8];
    let empty = blocks[0];

    // Total fill measured in eighths of a cell across the whole bar.
    let total_eighths = (f * width as f64 * 8.0).round() as usize;
    let full_cells = (total_eighths / 8).min(width);
    let remainder = total_eighths % 8;

    let mut out = String::new();
    out.push_str(&full.repeat(full_cells));

    let mut used = full_cells;
    // A non-zero remainder draws one partial cell from the eighth-block ramp,
    // but only if there is still room for it.
    if remainder > 0 && used < width {
        out.push_str(blocks[remainder]);
        used += 1;
    }
    // Pad the rest with the empty-cell glyph so the bar is always `width` wide.
    if used < width {
        out.push_str(&empty.repeat(width - used));
    }
    out
}

/// Like [`raw_bar`] but paints the filled portion in `color` (when enabled). The
/// trailing empty cells are dimmed so the track reads as background.
fn colored_bar(fraction: f64, width: usize, color: Color) -> String {
    if width == 0 {
        return String::new();
    }
    let f = fraction.clamp(0.0, 1.0);
    let blocks = symbols::hblocks();
    let full = blocks[8];
    let empty = blocks[0];

    let total_eighths = (f * width as f64 * 8.0).round() as usize;
    let full_cells = (total_eighths / 8).min(width);
    let remainder = total_eighths % 8;

    let fill_style = Style::new().fg(color);

    // Filled run (solid cells + a partial cell) gets the threshold color.
    let mut filled = String::new();
    filled.push_str(&full.repeat(full_cells));
    let mut used = full_cells;
    if remainder > 0 && used < width {
        filled.push_str(blocks[remainder]);
        used += 1;
    }

    let mut out = fill_style.paint(&filled);
    if used < width {
        let track = empty.repeat(width - used);
        out.push_str(&DIM_STYLE.paint(&track));
    }
    out
}

/// A static progress bar string of `width` columns for `fraction` in `0.0..=1.0`.
///
/// The final partial cell uses eighth-block precision (so a half-filled cell
/// renders as a half block, not all-or-nothing). The filled portion is colored
/// by completion threshold when color is enabled; otherwise the bar is plain,
/// fixed-width text.
pub fn bar(fraction: f64, width: usize) -> String {
    colored_bar(fraction, width, threshold_color(fraction.clamp(0.0, 1.0)))
}

/// A checklist: `(label, done)` rows rendered with status glyphs, followed by a
/// `done/total complete` summary line.
///
/// Completed rows are marked with a green success glyph and a dimmed label;
/// pending rows use a dim radio glyph and a normal label. In plain mode the
/// glyphs fall back to ASCII and no color is emitted, so the list stays readable
/// when captured.
pub fn steps(items: &[(&str, bool)]) -> String {
    let total = items.len();
    let done = items.iter().filter(|(_, d)| *d).count();

    let mut lines: Vec<String> = items
        .iter()
        .map(|(label, is_done)| {
            if *is_done {
                let glyph = SUCCESS_STYLE.paint(symbols::success());
                let text = DIM_STYLE.paint(label);
                format!("{glyph} {text}")
            } else {
                let glyph = DIM_STYLE.paint(symbols::radio_off());
                format!("{glyph} {label}")
            }
        })
        .collect();

    let summary_text = format!("{done}/{total} complete");
    let summary_style = if total > 0 && done == total {
        SUCCESS_STYLE
    } else {
        DIM_STYLE
    };
    lines.push(summary_style.paint(&summary_text));
    lines.join("\n")
}

/// Pure, I/O-free renderer for a single spinner frame: `"<frame> <msg>"`.
///
/// `frame_idx` selects a frame from [`symbols::spinner_frames`] (wrapping), and
/// the glyph is styled when color is enabled. Exposed for testing and for
/// callers that drive their own rendering loop.
pub fn spinner_line(frame_idx: usize, msg: &str) -> String {
    let frames = symbols::spinner_frames();
    // `spinner_frames()` is always non-empty, but guard defensively.
    let frame = if frames.is_empty() {
        ""
    } else {
        frames[frame_idx % frames.len()]
    };
    let styled = SPINNER_STYLE.paint(frame);
    format!("{styled} {msg}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps;
    use crate::ui::text::display_width;

    #[test]
    fn bar_endpoints_have_exact_width() {
        // Plain mode: no ANSI, so the rendered width equals the cell width.
        caps::set_color_override(Some(false));
        assert_eq!(display_width(&bar(0.0, 10)), 10);
        assert_eq!(display_width(&bar(1.0, 10)), 10);
        // A mid value still occupies exactly `width` cells.
        assert_eq!(display_width(&bar(0.5, 10)), 10);
        caps::set_color_override(None);
    }

    #[test]
    fn bar_zero_is_all_empty_and_full_is_all_filled() {
        caps::set_color_override(Some(false));
        caps::set_unicode_override(Some(true));
        let blocks = symbols::hblocks();
        let empty = blocks[0];
        let full = blocks[8];

        let z = bar(0.0, 10);
        assert_eq!(z, empty.repeat(10));

        let f = bar(1.0, 10);
        assert_eq!(f, full.repeat(10));

        caps::set_unicode_override(None);
        caps::set_color_override(None);
    }

    #[test]
    fn bar_uses_subcell_partial_block() {
        // 1/16 of a 1-cell bar rounds to ~half an eighth → a partial glyph that
        // is neither fully empty nor fully full.
        caps::set_color_override(Some(false));
        caps::set_unicode_override(Some(true));
        let blocks = symbols::hblocks();
        let half = bar(0.5, 1);
        assert_ne!(half, blocks[0]);
        assert_ne!(half, blocks[8]);
        caps::set_unicode_override(None);
        caps::set_color_override(None);
    }

    #[test]
    fn steps_contains_both_labels_and_summary() {
        caps::set_color_override(Some(false));
        let out = steps(&[("a", true), ("b", false)]);
        assert!(out.contains('a'), "expected label 'a' in: {out:?}");
        assert!(out.contains('b'), "expected label 'b' in: {out:?}");
        assert!(out.contains("1/2 complete"), "expected summary in: {out:?}");
        // One line per item plus the summary line.
        assert_eq!(out.lines().count(), 3);
        caps::set_color_override(None);
    }

    #[test]
    fn steps_all_done_reports_full_count() {
        caps::set_color_override(Some(false));
        let out = steps(&[("x", true), ("y", true)]);
        assert!(out.contains("2/2 complete"), "got: {out:?}");
        caps::set_color_override(None);
    }

    #[test]
    fn steps_empty_reports_zero() {
        caps::set_color_override(Some(false));
        let out = steps(&[]);
        assert_eq!(out, "0/0 complete");
        caps::set_color_override(None);
    }

    #[test]
    fn spinner_line_selects_expected_frame() {
        caps::set_color_override(Some(false));
        let frames = symbols::spinner_frames();

        let first = spinner_line(0, "loading");
        assert!(first.starts_with(frames[0]), "got: {first:?}");
        assert!(first.ends_with("loading"));

        // Index wraps around the frame set.
        let wrapped = spinner_line(frames.len(), "loading");
        assert!(wrapped.starts_with(frames[0]), "got: {wrapped:?}");

        // A distinct index yields a distinct frame (frame sets have >1 entry).
        if frames.len() > 1 {
            let second = spinner_line(1, "x");
            assert!(second.starts_with(frames[1]), "got: {second:?}");
        }
        caps::set_color_override(None);
    }

    #[test]
    fn progress_bar_render_shows_percentage_and_label() {
        caps::set_color_override(Some(false));
        let mut pb = ProgressBar::new(10).with_label("build");
        pb.set(6);
        let out = pb.render();
        assert!(out.starts_with("build "), "got: {out:?}");
        assert!(out.contains("60%"), "got: {out:?}");
        assert!(out.contains('['), "got: {out:?}");
        assert!(out.contains(']'), "got: {out:?}");
        caps::set_color_override(None);
    }

    #[test]
    fn progress_bar_clamps_and_handles_zero_total() {
        caps::set_color_override(Some(false));
        // inc beyond total clamps to 100%.
        let mut pb = ProgressBar::new(4);
        pb.inc(10);
        assert!(pb.render().contains("100%"));

        // A zero total is treated as already complete (avoids div-by-zero).
        let zero = ProgressBar::new(0);
        assert!(zero.render().contains("100%"));
        caps::set_color_override(None);
    }

    #[test]
    fn spinner_off_tty_is_noop_constructible() {
        // In the test harness stderr is captured (not a TTY), so the spinner is
        // inert and these calls must neither panic nor emit to stdout.
        let mut s = Spinner::new("working");
        s.tick();
        s.tick();
        s.finish("done");
    }
}
