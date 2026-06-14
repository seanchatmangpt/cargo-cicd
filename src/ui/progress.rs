//! Progress feedback: spinners, progress bars, and step checklists.
//!
//! Animated output goes to **stderr** and must no-op when stderr is not a TTY,
//! so captured stdout stays clean.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent implements live
//! animation; signatures must not change.

use crate::ui::caps;
use crate::ui::symbols;

/// An animated single-line spinner (renders to stderr when interactive).
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
            enabled: caps::color_enabled(),
        }
    }
    /// Advance one frame and (when enabled) repaint the spinner line.
    pub fn tick(&mut self) {
        // STUB: advance only; agent repaints stderr in-place when enabled.
        let _ = (&self.message, self.enabled);
        self.frame = self.frame.wrapping_add(1);
    }
    /// Stop the spinner, clearing its line and printing `final_msg`.
    pub fn finish(self, final_msg: &str) {
        // STUB: no-op to keep stdout clean; agent clears the stderr line.
        let _ = final_msg;
    }
}

/// A determinate progress bar.
pub struct ProgressBar {
    total: u64,
    pos: u64,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self { total, pos: 0 }
    }
    pub fn set(&mut self, pos: u64) {
        self.pos = pos.min(self.total);
    }
    pub fn inc(&mut self, delta: u64) {
        self.pos = (self.pos + delta).min(self.total);
    }
    pub fn finish(self) {}
    pub fn render(&self) -> String {
        let frac = if self.total == 0 {
            0.0
        } else {
            self.pos as f64 / self.total as f64
        };
        bar(frac, 20)
    }
}

/// A static progress bar string of `width` columns for `fraction` in `0.0..=1.0`.
pub fn bar(fraction: f64, width: usize) -> String {
    let f = fraction.clamp(0.0, 1.0);
    let filled = (f * width as f64).round() as usize;
    format!(
        "{}{}",
        symbols::gauge_full().repeat(filled),
        symbols::gauge_empty().repeat(width.saturating_sub(filled))
    )
}

/// A checklist: `(label, done)` rows rendered with status glyphs.
pub fn steps(items: &[(&str, bool)]) -> String {
    items
        .iter()
        .map(|(label, done)| {
            let g = if *done {
                symbols::success()
            } else {
                symbols::radio_off()
            };
            format!("{g} {label}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
