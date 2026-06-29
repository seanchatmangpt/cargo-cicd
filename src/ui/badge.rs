//! Status badges: the semantic [`Verdict`] vocabulary shared across cargo-cicd
//! output, rendered as bracket tags, glyph tags, dots, or filled pills.
//!
//! [`Verdict`] is the canonical status vocabulary. Each verdict carries a
//! stable uppercase [`label`](Verdict::label) (used in plain-text output the
//! public-boundary tests assert on), a [`glyph`](Verdict::glyph) with ASCII
//! fallback, and a semantic color exposed through [`style_for`].
//!
//! Renderers come in two families:
//! * **foreground** — [`tag`], [`inline`], [`dot`]: colored text on the default
//!   background.
//! * **filled** — [`pill`]: the label on a colored background.
//!
//! [`bracket`] is the deliberately plain, never-colored ASCII form for places
//! where the literal label must survive intact (logs, machine-readable lines).
//!
//! All color flows through [`Style::paint`], so it auto-disables when stdout is
//! not a TTY, when `NO_COLOR` is set, or when `--no-color` was passed; captured
//! and piped output stays clean ASCII.

use crate::ui::style::{Color, Style};
use crate::ui::symbols;

/// A semantic verdict used across status, policy, and evidence output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
    Suggest,
    Blocked,
    Accept,
    Refuse,
    Info,
    Skip,
}

impl Verdict {
    /// The canonical uppercase label, e.g. `"PASS"`. Stable: used in plain-text
    /// output that the public-boundary tests assert on. Always ASCII so the
    /// substring survives in piped output.
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
            Verdict::Suggest => "SUGGEST",
            Verdict::Blocked => "BLOCKED",
            Verdict::Accept => "ACCEPT",
            Verdict::Refuse => "REFUSE",
            Verdict::Info => "INFO",
            Verdict::Skip => "SKIP",
        }
    }

    /// Parse a free-form status tag (`"OK"`, `"WARN:dry_run"`, `"ACCEPTED"`, …)
    /// into a [`Verdict`].
    ///
    /// Only the leading token (up to the first `:`, space, or `_`) is matched,
    /// case-insensitively; anything unrecognized falls back to [`Verdict::Info`].
    pub fn from_tag(tag: &str) -> Verdict {
        let head = tag
            .trim()
            .split([':', ' ', '_'])
            .next()
            .unwrap_or("")
            .to_uppercase();
        match head.as_str() {
            "PASS" | "OK" | "GOOD" | "CLEAN" | "HEALTHY" => Verdict::Pass,
            "WARN" | "WARNING" => Verdict::Warn,
            "FAIL" | "ERROR" | "BAD" | "DIRTY" => Verdict::Fail,
            "SUGGEST" | "HINT" => Verdict::Suggest,
            "BLOCKED" | "BLOCK" => Verdict::Blocked,
            "ACCEPT" | "ACCEPTED" => Verdict::Accept,
            "REFUSE" | "REFUSED" => Verdict::Refuse,
            "SKIP" | "SKIPPED" => Verdict::Skip,
            _ => Verdict::Info,
        }
    }

    /// The glyph associated with this verdict (Unicode, ASCII fallback).
    pub fn glyph(self) -> &'static str {
        match self {
            Verdict::Pass | Verdict::Accept => symbols::success(),
            Verdict::Warn | Verdict::Suggest => symbols::warning(),
            Verdict::Fail | Verdict::Refuse | Verdict::Blocked => symbols::failure(),
            Verdict::Info | Verdict::Skip => symbols::info(),
        }
    }

    /// The semantic accent color for this verdict.
    ///
    /// * Pass / Accept → green
    /// * Warn / Suggest → yellow
    /// * Fail / Refuse / Blocked → red
    /// * Info → cyan/blue
    /// * Skip → dim gray
    fn color(self) -> Color {
        match self {
            Verdict::Pass | Verdict::Accept => Color::Green,
            Verdict::Warn | Verdict::Suggest => Color::Yellow,
            Verdict::Fail | Verdict::Refuse | Verdict::Blocked => Color::Red,
            Verdict::Info => Color::Cyan,
            Verdict::Skip => Color::BrightBlack,
        }
    }
}

/// The semantic foreground [`Style`] for a verdict: its accent color, bold.
///
/// Skip is rendered dim rather than bold so it visually recedes. Use this as
/// the single source of truth when coloring verdict text in other components.
pub fn style_for(v: Verdict) -> Style {
    let base = Style::new().fg(v.color());
    match v {
        Verdict::Skip => base.dim(),
        _ => base.bold(),
    }
}

/// The filled-pill [`Style`] for a verdict: bold bright-white text on the
/// verdict's color as a background.
fn pill_style(v: Verdict) -> Style {
    Style::new().fg(Color::BrightWhite).bg(v.color()).bold()
}

/// A plain bracket tag like `[PASS]` — always uncolored, ASCII-safe label.
///
/// This is the deliberately literal form: use it where the exact bytes matter
/// (machine-readable lines, logs) and color would be noise or harmful.
pub fn bracket(v: Verdict) -> String {
    format!("[{}]", v.label())
}

/// A glyph + label bracket tag like `[✔ PASS]`, colored by verdict (bold).
///
/// In plain mode the color is dropped and the result is just `[✔ PASS]` (or
/// `[+ PASS]` with ASCII glyphs).
pub fn tag(v: Verdict) -> String {
    let body = format!("[{} {}]", v.glyph(), v.label());
    style_for(v).paint(body)
}

/// A glyph + label without brackets, e.g. `✔ PASS`, colored by verdict.
///
/// In plain mode the color is dropped, leaving `✔ PASS` / `+ PASS`.
pub fn inline(v: Verdict) -> String {
    let body = format!("{} {}", v.glyph(), v.label());
    style_for(v).paint(body)
}

/// A filled "pill" badge like ` PASS `: the label on the verdict's color as a
/// background, with one space of padding each side.
///
/// In plain mode the background is dropped, leaving the padded label ` PASS `
/// so spacing in captured output stays consistent with the colored form.
pub fn pill(v: Verdict) -> String {
    let body = format!(" {} ", v.label());
    pill_style(v).paint(body)
}

/// A colored status dot followed by the label, e.g. `● PASS`.
///
/// Uses a round bullet glyph (ASCII fallback `*`); the dot carries the verdict
/// color while the label stays in the default foreground for readability.
pub fn dot(v: Verdict) -> String {
    let mark = Style::new().fg(v.color()).bold().paint(symbols::bullet());
    format!("{} {}", mark, v.label())
}
