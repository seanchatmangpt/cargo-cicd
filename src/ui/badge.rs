//! Status badges: the semantic [`Verdict`] vocabulary shared across cargo-cicd
//! output, rendered as bracket tags, glyph tags, or filled pills.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent adds color to
//! [`tag`], [`pill`], and [`inline`]; [`Verdict`], [`Verdict::label`], and
//! [`Verdict::from_tag`] are part of the contract and must not change meaning.

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
    /// output that the public-boundary tests assert on.
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

    /// The glyph associated with this verdict.
    pub fn glyph(self) -> &'static str {
        match self {
            Verdict::Pass | Verdict::Accept => symbols::success(),
            Verdict::Warn | Verdict::Suggest => symbols::warning(),
            Verdict::Fail | Verdict::Refuse | Verdict::Blocked => symbols::failure(),
            Verdict::Info | Verdict::Skip => symbols::info(),
        }
    }
}

/// A plain bracket tag like `[PASS]` (always ASCII-safe label).
pub fn bracket(v: Verdict) -> String {
    format!("[{}]", v.label())
}

/// A glyph + label bracket tag like `[✔ PASS]`.
pub fn tag(v: Verdict) -> String {
    // STUB: no color; agent colors the bracket by verdict.
    format!("[{} {}]", v.glyph(), v.label())
}

/// A glyph + label without brackets, e.g. `✔ PASS`.
pub fn inline(v: Verdict) -> String {
    format!("{} {}", v.glyph(), v.label())
}

/// A filled "pill" badge like ` PASS ` (agent renders on a colored background).
pub fn pill(v: Verdict) -> String {
    // STUB: padded label; agent adds background color.
    format!(" {} ", v.label())
}
