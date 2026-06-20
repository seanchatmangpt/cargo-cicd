//! Named colour palette.

use crate::ui::caps::Caps;

/// ANSI colour codes used by the design system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub &'static str);

// ─────────────────────────────────────────────────────────────────────────────
// Palette constants
// ─────────────────────────────────────────────────────────────────────────────

pub const GREEN: Color = Color("\x1b[32m");
pub const YELLOW: Color = Color("\x1b[33m");
pub const RED: Color = Color("\x1b[31m");
pub const CYAN: Color = Color("\x1b[36m");
pub const BOLD: Color = Color("\x1b[1m");
pub const DIM: Color = Color("\x1b[2m");
pub const RESET: Color = Color("\x1b[0m");

/// The active colour theme for the current process.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// `true` when ANSI escape sequences are safe to emit.
    pub color: bool,
    /// `true` when Unicode glyphs are safe to emit.
    pub unicode: bool,
}

impl Theme {
    /// Detect the appropriate theme from current terminal capabilities.
    pub fn detect() -> Self {
        let caps = Caps::detect();
        Self { color: caps.has_color, unicode: caps.has_unicode }
    }

    /// A theme that always produces plain ASCII output.
    pub fn plain() -> Self {
        Self { color: false, unicode: false }
    }

    /// Apply a colour to a string, respecting the current theme.
    ///
    /// When `color` is `false`, the string is returned unchanged.
    pub fn paint(&self, color: Color, text: &str) -> String {
        if self.color {
            format!("{}{}{}", color.0, text, RESET.0)
        } else {
            text.to_owned()
        }
    }

    /// Wrap text in bold, respecting the current theme.
    pub fn bold(&self, text: &str) -> String {
        self.paint(BOLD, text)
    }

    /// Wrap text in dim/faint, respecting the current theme.
    pub fn dim(&self, text: &str) -> String {
        self.paint(DIM, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_theme_no_escapes() {
        let theme = Theme::plain();
        let result = theme.paint(GREEN, "hello");
        assert_eq!(result, "hello");
        assert!(!result.contains('\x1b'));
    }

    #[test]
    fn color_theme_includes_escapes() {
        let theme = Theme { color: true, unicode: true };
        let result = theme.paint(GREEN, "hello");
        assert!(result.contains('\x1b'));
        assert!(result.contains("hello"));
    }

    #[test]
    fn bold_plain() {
        let theme = Theme::plain();
        assert_eq!(theme.bold("text"), "text");
    }
}
