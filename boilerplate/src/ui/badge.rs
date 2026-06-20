//! Inline status badges: `[PASS]`, `[WARN]`, `[FAIL]`, `[SKIP]`.
//!
//! Badges are rendered respecting the active [`crate::ui::theme::Theme`].
//! In plain mode they appear as `[PASS]`; in colour mode they are coloured.

use crate::ui::theme::{Theme, GREEN, RED, YELLOW};
use project_core::Verdict;

/// An inline status badge.
#[derive(Debug, Clone)]
pub struct Badge {
    /// The text label inside the brackets.
    pub label: String,
    /// The rendered string (includes colour if the theme allows it).
    rendered: String,
}

impl Badge {
    /// Create a badge from a [`Verdict`] and the active [`Theme`].
    pub fn from_verdict(verdict: &Verdict, theme: &Theme) -> Self {
        let label = verdict.label().to_owned();
        let color = match verdict {
            Verdict::Pass => GREEN,
            Verdict::Warn | Verdict::Blocked => YELLOW,
            Verdict::Fail => RED,
        };
        let text = format!("[{label}]");
        let rendered = theme.paint(color, &text);
        Self { label, rendered }
    }

    /// Create a named badge with an explicit colour.
    pub fn new(label: impl Into<String>, theme: &Theme) -> Self {
        let label = label.into();
        let text = format!("[{}]", label.to_ascii_uppercase());
        let rendered = theme.paint(crate::ui::theme::CYAN, &text);
        Self { label, rendered }
    }

    /// Returns the rendered badge string.
    pub fn as_str(&self) -> &str {
        &self.rendered
    }
}

impl std::fmt::Display for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_plain_contains_label() {
        let theme = Theme::plain();
        let badge = Badge::from_verdict(&Verdict::Pass, &theme);
        assert!(badge.rendered.contains("PASS"));
        assert!(badge.rendered.contains('['));
        assert!(badge.rendered.contains(']'));
        assert!(!badge.rendered.contains('\x1b'));
    }

    #[test]
    fn badge_fail_plain() {
        let theme = Theme::plain();
        let badge = Badge::from_verdict(&Verdict::Fail, &theme);
        assert!(badge.rendered.contains("FAIL"));
    }

    #[test]
    fn badge_color_contains_escapes() {
        let theme = Theme { color: true, unicode: true };
        let badge = Badge::from_verdict(&Verdict::Pass, &theme);
        assert!(badge.rendered.contains('\x1b'));
        assert!(badge.rendered.contains("PASS"));
    }

    #[test]
    fn badge_display_equals_as_str() {
        let theme = Theme::plain();
        let badge = Badge::from_verdict(&Verdict::Warn, &theme);
        assert_eq!(badge.to_string(), badge.as_str());
    }
}
