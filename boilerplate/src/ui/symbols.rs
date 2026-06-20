//! Named glyph constants with ASCII fallbacks.
//!
//! All glyphs used in noun output **must** come from this module.
//! Hard-coding Unicode characters inside noun modules is forbidden.
//!
//! Selection at runtime: call [`glyph()`] which respects the current
//! [`crate::ui::caps::Caps`] and returns the Unicode or ASCII variant
//! accordingly.

use crate::ui::caps::Caps;

/// A glyph that has both a Unicode and an ASCII representation.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// The Unicode representation, used when the terminal supports it.
    pub unicode: &'static str,
    /// The ASCII fallback, used in plain/pipe mode.
    pub ascii: &'static str,
}

impl Glyph {
    /// Return the appropriate string for the current terminal capabilities.
    pub fn render(self) -> &'static str {
        if Caps::detect().has_unicode {
            self.unicode
        } else {
            self.ascii
        }
    }
}

impl std::fmt::Display for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.render())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Glyph catalogue
// ─────────────────────────────────────────────────────────────────────────────

/// Checkmark — indicates success.
pub const CHECK: Glyph = Glyph { unicode: "✓", ascii: "[ok]" };

/// Cross — indicates failure.
pub const CROSS: Glyph = Glyph { unicode: "✗", ascii: "[x]" };

/// Warning triangle — indicates a non-blocking warning.
pub const WARN: Glyph = Glyph { unicode: "⚠", ascii: "[!]" };

/// Information symbol — neutral informational item.
pub const INFO: Glyph = Glyph { unicode: "ℹ", ascii: "[i]" };

/// Bullet — unordered list item marker.
pub const BULLET: Glyph = Glyph { unicode: "•", ascii: "-" };

/// Arrow — directional indicator.
pub const ARROW: Glyph = Glyph { unicode: "→", ascii: "->" };

/// Project-level heading glyph.
pub const PROJECT_GLYPH: Glyph = Glyph { unicode: "⚙", ascii: ">>" };

/// Horizontal rule glyph (repeated to form a divider).
pub const RULE: Glyph = Glyph { unicode: "─", ascii: "-" };

// ─────────────────────────────────────────────────────────────────────────────
// Convenience function
// ─────────────────────────────────────────────────────────────────────────────

/// Return the runtime-appropriate string for a [`Glyph`].
///
/// Prefer referencing the constant directly (`symbols::CHECK`) since it
/// implements `Display`, but this function is useful when you need a `&str`.
pub fn glyph(g: Glyph) -> &'static str {
    g.render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_display_does_not_panic() {
        // Each glyph must produce non-empty output regardless of caps.
        for g in [CHECK, CROSS, WARN, INFO, BULLET, ARROW, PROJECT_GLYPH, RULE] {
            let rendered = g.render();
            assert!(!rendered.is_empty(), "glyph should not be empty");
        }
    }

    #[test]
    fn ascii_fallbacks_are_ascii() {
        for g in [CHECK, CROSS, WARN, INFO, BULLET, ARROW, PROJECT_GLYPH, RULE] {
            assert!(
                g.ascii.is_ascii(),
                "ASCII fallback `{}` must be pure ASCII",
                g.ascii
            );
        }
    }
}
