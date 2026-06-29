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
