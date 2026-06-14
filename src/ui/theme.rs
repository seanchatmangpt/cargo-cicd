//! Semantic theme: maps roles to concrete styles so the whole CLI stays
//! visually consistent. Callers should prefer roles over hard-coded colors.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent replaces the body
//! of [`style`] with a real palette; signatures must not change.

use crate::ui::style::Style;

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

/// Resolve a semantic role to a concrete [`Style`].
pub fn style(role: Role) -> Style {
    // STUB: only attributes, no palette yet.
    match role {
        Role::Heading | Role::Strong => Style::new().bold(),
        Role::Subheading => Style::new().underline(),
        Role::Muted => Style::new().dim(),
        _ => Style::new(),
    }
}

/// Paint `text` according to a semantic role.
pub fn paint(text: &str, role: Role) -> String {
    style(role).paint(text)
}
