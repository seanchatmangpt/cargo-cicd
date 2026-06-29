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

