//! Pretty diagnostics: rustc-style severity blocks with notes and help lines,
//! plus error-chain rendering for `anyhow`-style causes.
//!
//! The header is a severity-colored `error[CODE]: title`; subsequent notes and
//! help lines hang under a dim gutter with a colored leading marker. All color
//! is applied through [`Style::paint`], so it disappears off-TTY and the plain
//! text stays clean and aligned.

use crate::ui::style::{Color, Style};
use crate::ui::symbols;

/// Diagnostic severity levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
    Success,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
            Severity::Success => "success",
        }
    }
    pub fn glyph(self) -> &'static str {
        match self {
            Severity::Error => symbols::failure(),
            Severity::Warning => symbols::warning(),
            Severity::Note => symbols::info(),
            Severity::Help => symbols::arrow_small(),
            Severity::Success => symbols::success(),
        }
    }

    /// The accent color used for this severity's header and markers.
    fn color(self) -> Color {
        match self {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Note => Color::Cyan,
            Severity::Help => Color::Green,
            Severity::Success => Color::Green,
        }
    }

    /// Bold, severity-colored style for the header label.
    fn header_style(self) -> Style {
        Style::new().fg(self.color()).bold()
    }
}

/// Dim style for the hanging gutter (`│` / `=`) that links continuation lines.
fn gutter_style() -> Style {
    Style::new().dim()
}

/// The continuation gutter glyph (a vertical bar, ASCII `|` fallback).
fn gutter_bar() -> &'static str {
    symbols::box_chars(symbols::BoxStyle::Light).v
}

impl Diagnostic {
    /// Render this diagnostic as a multi-line, rustc-style block.
    fn render_block(&self) -> String {
        let sev = self.severity;
        // Header: `error[E123]: title`, with the `error[E123]:` part colored.
        let code = self
            .code
            .as_deref()
            .map(|c| format!("[{c}]"))
            .unwrap_or_default();
        let head_label = format!("{}{}", sev.label(), code);
        let mut out = format!("{}: {}", sev.header_style().paint(&head_label), self.title);

        // A 3-space hang aligns continuation gutters under the header text.
        let bar = gutter_style().paint(gutter_bar());
        let eq = gutter_style().paint("=");

        for n in &self.notes {
            let marker = Style::new().fg(Severity::Note.color()).bold().paint("note");
            out.push_str(&format!("\n   {bar} {marker}: {n}"));
        }
        for h in &self.helps {
            let marker = Style::new().fg(Severity::Help.color()).bold().paint("help");
            out.push_str(&format!("\n   {eq} {marker}: {h}"));
        }
        out
    }
}

/// A structured diagnostic with optional code, notes, and help lines.
pub struct Diagnostic {
    severity: Severity,
    title: String,
    code: Option<String>,
    notes: Vec<String>,
    helps: Vec<String>,
}

impl Diagnostic {
    pub fn new(sev: Severity, title: impl Into<String>) -> Self {
        Self {
            severity: sev,
            title: title.into(),
            code: None,
            notes: Vec::new(),
            helps: Vec::new(),
        }
    }
    pub fn code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }
    pub fn note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
    pub fn help(mut self, help: &str) -> Self {
        self.helps.push(help.to_string());
        self
    }
    pub fn render(&self) -> String {
        self.render_block()
    }
}

/// A one-line severity-tagged message: a colored glyph, a colored
/// `label:` prefix, then the message.
pub fn line(sev: Severity, msg: &str) -> String {
    let glyph = Style::new().fg(sev.color()).paint(sev.glyph());
    let label = sev.header_style().paint(&format!("{}:", sev.label()));
    format!("{glyph} {label} {msg}")
}

/// Render an error and its `causes` as an indented chain. The head is an
/// error-severity line; each cause hangs under a dim `caused by:` gutter.
pub fn error_chain(err: &str, causes: &[&str]) -> String {
    let mut out = line(Severity::Error, err);
    let bar = gutter_style().paint(gutter_bar());
    for c in causes {
        let label = gutter_style().paint("caused by:");
        out.push_str(&format!("\n   {bar} {label} {c}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps;
    use crate::ui::text::strip_ansi;
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
    fn plain_render_is_clean() {
        let _g = CapsGuard::acquire(false, true);
        let d = Diagnostic::new(Severity::Error, "broken build")
            .code("E001")
            .note("the workspace failed to compile")
            .help("run `cargo check` for details");
        let out = d.render();
        // No escapes leak in plain mode.
        assert!(!out.contains('\u{1b}'));
        assert!(out.starts_with("error[E001]: broken build"));
        assert!(out.contains("note: the workspace failed to compile"));
        assert!(out.contains("help: run `cargo check` for details"));
    }

    #[test]
    fn render_without_code_omits_brackets() {
        let _g = CapsGuard::acquire(false, true);
        let out = Diagnostic::new(Severity::Warning, "deprecated flag").render();
        assert_eq!(out, "warning: deprecated flag");
    }

    #[test]
    fn forced_color_colors_header_and_markers() {
        let _g = CapsGuard::acquire(true, true);
        let out = Diagnostic::new(Severity::Error, "x")
            .code("E9")
            .note("n")
            .render();
        assert!(out.contains('\u{1b}'), "should contain SGR escapes");
        // Red header (31) and reset are present.
        assert!(out.contains("31"));
        assert!(out.contains("\u{1b}[0m"));
        // Stripping ANSI yields the plain, structured form.
        let plain = strip_ansi(&out);
        assert!(plain.starts_with("error[E9]: x"));
        assert!(plain.contains("note: n"));
    }

    #[test]
    fn line_has_glyph_label_message() {
        // Force ASCII so the glyph is stable for assertion.
        let _g = CapsGuard::acquire(false, false);
        let out = line(Severity::Success, "all checks passed");
        assert_eq!(out, "+ success: all checks passed");
    }

    #[test]
    fn error_chain_indents_causes() {
        let _g = CapsGuard::acquire(false, false);
        let out = error_chain("top-level failure", &["disk full", "io error"]);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "x error: top-level failure");
        assert!(lines[1].contains("caused by: disk full"));
        assert!(lines[2].contains("caused by: io error"));
        // Continuation lines are indented under the head.
        assert!(lines[1].starts_with("   "));
    }
}
