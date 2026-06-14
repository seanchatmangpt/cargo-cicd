//! Pretty diagnostics: rustc-style severity blocks with notes and help lines,
//! plus error-chain rendering for `anyhow`-style causes.
//!
//! STUB IMPLEMENTATION — frozen public API. The owning agent adds a colored
//! gutter and severity colors; signatures must not change.

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
        // STUB: flat multi-line; agent adds gutter & color.
        let code = self
            .code
            .as_deref()
            .map(|c| format!("[{c}]"))
            .unwrap_or_default();
        let mut out = format!(
            "{} {}{}: {}",
            self.severity.glyph(),
            self.severity.label(),
            code,
            self.title
        );
        for n in &self.notes {
            out.push_str(&format!("\n  note: {n}"));
        }
        for h in &self.helps {
            out.push_str(&format!("\n  help: {h}"));
        }
        out
    }
}

/// A one-line severity-tagged message.
pub fn line(sev: Severity, msg: &str) -> String {
    format!("{} {}: {}", sev.glyph(), sev.label(), msg)
}

/// Render an error and its `causes` as an indented chain.
pub fn error_chain(err: &str, causes: &[&str]) -> String {
    let mut out = format!("{} error: {err}", symbols::failure());
    for c in causes {
        out.push_str(&format!("\n  caused by: {c}"));
    }
    out
}
