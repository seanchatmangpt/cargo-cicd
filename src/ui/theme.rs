//! Semantic theme: maps roles to concrete styles so the whole CLI stays
//! visually consistent. Callers should prefer roles over hard-coded colors.
//!
//! The palette is built once, here, as `const` [`Style`] tokens and resolved
//! by [`style`]. Every color is emitted through [`Style::paint`], so it
//! auto-disables off-TTY and in captured output. Accents prefer 256-color
//! ([`Color::Fixed`]) and truecolor ([`Color::Rgb`]) values for a cohesive,
//! modern look that still degrades to the basic 16 colors where needed.

use crate::ui::style::{Color, Style};

/// A semantic styling role, independent of concrete colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Heading,
    Subheading,
    Success,
    Warning,
    Danger,
    Info,
    Muted,
    Accent,
    Strong,
    Link,
    Label,
    Value,
}

// ── palette tokens ──────────────────────────────────────────────────────────
//
// A small, deliberately limited set of accent colors keeps the CLI coherent.
// 256-color indices are used for the saturated brand accents and truecolor for
// the few places that benefit from a precise hue; both fall back cleanly when
// the terminal cannot render them (color is dropped entirely off-TTY).

/// Brand cyan accent (256-color), used for headings and informational text.
pub const ACCENT_CYAN: Color = Color::Fixed(45);
/// Brand magenta accent (256-color), used for decorative emphasis.
pub const ACCENT_MAGENTA: Color = Color::Fixed(170);
/// Positive/“ok” green (256-color).
pub const ACCENT_GREEN: Color = Color::Fixed(78);
/// Caution amber (256-color).
pub const ACCENT_AMBER: Color = Color::Fixed(214);
/// Error red (truecolor) — a slightly soft, readable red.
pub const ACCENT_RED: Color = Color::Rgb(0xE0, 0x4A, 0x4A);
/// Link blue (256-color).
pub const ACCENT_BLUE: Color = Color::Fixed(75);

/// Bold, bright-cyan heading.
pub const HEADING: Style = Style::new().fg(ACCENT_CYAN).bold();
/// Bold + underlined subheading.
pub const SUBHEADING: Style = Style::new().fg(ACCENT_CYAN).bold().underline();
/// Success / “passing” text.
pub const SUCCESS: Style = Style::new().fg(ACCENT_GREEN);
/// Warning / “needs attention” text.
pub const WARNING: Style = Style::new().fg(ACCENT_AMBER);
/// Danger / “failing” text — bold red for maximum salience.
pub const DANGER: Style = Style::new().fg(ACCENT_RED).bold();
/// Informational text.
pub const INFO: Style = Style::new().fg(ACCENT_CYAN);
/// De-emphasized / secondary text.
pub const MUTED: Style = Style::new().dim();
/// Decorative accent.
pub const ACCENT: Style = Style::new().fg(ACCENT_MAGENTA);
/// Strong emphasis without recoloring.
pub const STRONG: Style = Style::new().bold();
/// Hyperlink-style text: underlined blue.
pub const LINK: Style = Style::new().fg(ACCENT_BLUE).underline();
/// Field label: dim, paired with a bold [`VALUE`].
pub const LABEL: Style = Style::new().dim();
/// Field value: bold, paired with a dim [`LABEL`].
pub const VALUE: Style = Style::new().bold();

/// Resolve a semantic role to a concrete [`Style`].
pub fn style(role: Role) -> Style {
    match role {
        Role::Heading => HEADING,
        Role::Subheading => SUBHEADING,
        Role::Success => SUCCESS,
        Role::Warning => WARNING,
        Role::Danger => DANGER,
        Role::Info => INFO,
        Role::Muted => MUTED,
        Role::Accent => ACCENT,
        Role::Strong => STRONG,
        Role::Link => LINK,
        Role::Label => LABEL,
        Role::Value => VALUE,
    }
}

/// Paint `text` according to a semantic role.
pub fn paint(text: &str, role: Role) -> String {
    style(role).paint(text)
}

/// Convenience: render a `label: value` pair with the [`Role::Label`] and
/// [`Role::Value`] styles, separated by a single space.
pub fn field(label: &str, value: &str) -> String {
    format!(
        "{} {}",
        paint(&format!("{label}:"), Role::Label),
        paint(value, Role::Value)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::caps;
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

    fn all_roles() -> [Role; 12] {
        [
            Role::Heading,
            Role::Subheading,
            Role::Success,
            Role::Warning,
            Role::Danger,
            Role::Info,
            Role::Muted,
            Role::Accent,
            Role::Strong,
            Role::Link,
            Role::Label,
            Role::Value,
        ]
    }

    #[test]
    fn plain_mode_is_passthrough() {
        let _g = CapsGuard::acquire(false, true);
        for r in all_roles() {
            assert_eq!(paint("text", r), "text", "role {r:?} must not color off-TTY");
        }
    }

    #[test]
    fn forced_color_wraps_every_role() {
        let _g = CapsGuard::acquire(true, true);
        for r in all_roles() {
            let out = paint("x", r);
            assert!(out.starts_with("\u{1b}["), "role {r:?} should emit SGR");
            assert!(out.ends_with("\u{1b}[0m"), "role {r:?} should reset");
            assert!(out.contains('x'));
        }
    }

    #[test]
    fn key_roles_have_expected_attributes() {
        // Structural assertions on the resolved Style (independent of TTY).
        assert!(style(Role::Heading).bold);
        assert!(style(Role::Subheading).bold && style(Role::Subheading).underline);
        assert!(style(Role::Danger).bold);
        assert!(style(Role::Strong).bold);
        assert!(style(Role::Muted).dim);
        assert!(style(Role::Label).dim);
        assert!(style(Role::Value).bold);
        assert!(style(Role::Link).underline);
        assert_eq!(style(Role::Success).fg, Some(ACCENT_GREEN));
        assert_eq!(style(Role::Warning).fg, Some(ACCENT_AMBER));
        assert_eq!(style(Role::Info).fg, Some(ACCENT_CYAN));
        assert_eq!(style(Role::Accent).fg, Some(ACCENT_MAGENTA));
    }

    #[test]
    fn field_pairs_label_and_value() {
        let _g = CapsGuard::acquire(false, true);
        assert_eq!(field("name", "demo"), "name: demo");
    }
}
