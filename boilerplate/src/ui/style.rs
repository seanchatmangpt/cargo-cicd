//! `Style::paint` — the single entry point for all coloured output.
//!
//! Every piece of coloured text in the binary must go through this module.

use crate::ui::caps::Caps;

/// Named text styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Success / positive state.
    Success,
    /// Warning / attention needed.
    Warning,
    /// Error / failure.
    Error,
    /// Informational / neutral.
    Info,
    /// Subdued / secondary information.
    Dim,
    /// Bold emphasis.
    Bold,
    /// No special styling.
    Plain,
}

impl Style {
    /// Apply this style to `text`.
    ///
    /// When stdout is not a TTY or `NO_COLOR` is set, returns the text
    /// unchanged (no ANSI escape codes).
    pub fn paint(self, text: &str) -> String {
        let caps = Caps::detect();
        if !caps.has_color {
            return text.to_owned();
        }

        let (open, close) = self.escape_codes();
        format!("{open}{text}{close}")
    }

    /// Apply this style to an owned `String`.
    pub fn paint_owned(self, text: String) -> String {
        self.paint(&text)
    }

    fn escape_codes(self) -> (&'static str, &'static str) {
        match self {
            Self::Success => ("\x1b[32m", "\x1b[0m"),
            Self::Warning => ("\x1b[33m", "\x1b[0m"),
            Self::Error => ("\x1b[31m", "\x1b[0m"),
            Self::Info => ("\x1b[36m", "\x1b[0m"),
            Self::Dim => ("\x1b[2m", "\x1b[0m"),
            Self::Bold => ("\x1b[1m", "\x1b[0m"),
            Self::Plain => ("", ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps::{Caps, PLAIN};

    fn with_plain_caps<F: FnOnce()>(f: F) {
        // Tests run in a non-TTY environment so caps detection returns PLAIN
        // automatically.  This is here for documentation / clarity.
        let _ = PLAIN;
        f()
    }

    #[test]
    fn plain_style_no_escapes() {
        with_plain_caps(|| {
            let result = Style::Plain.paint("hello");
            assert!(!result.contains('\x1b'));
        });
    }

    #[test]
    fn all_styles_contain_text() {
        for style in [
            Style::Success,
            Style::Warning,
            Style::Error,
            Style::Info,
            Style::Dim,
            Style::Bold,
            Style::Plain,
        ] {
            let result = style.paint("test");
            assert!(result.contains("test"), "{style:?} should contain the input text");
        }
    }
}
