//! CicdFinding structure.

use crate::diagnostics::code::CicdCode;
use crate::diagnostics::lifecycle::DiagnosticLifecycle;
use crate::diagnostics::severity::CicdSeverity;

/// A single diagnostic finding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CicdFinding {
    pub code: CicdCode,
    pub severity: CicdSeverity,
    pub location: String,
    pub source: String,
    pub repairs: Vec<String>,
    pub message: String,
    pub uri: Option<String>,
    pub route: Option<RepairRoute>,
    pub lifecycle: DiagnosticLifecycle,
    /// 0-indexed LSP line number of the offending location, if known.
    pub source_line: Option<u32>,
    /// 0-indexed LSP character offset of the offending location, if known.
    pub source_character: Option<u32>,
}

/// A suggested repair route for a finding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepairRoute {
    pub command: String,
    pub explanation: String,
}

impl CicdFinding {
    pub fn new(
        code: CicdCode,
        location: impl Into<String>,
        source: impl Into<String>,
        repairs: Vec<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: CicdSeverity::Warning,
            location: location.into(),
            source: source.into(),
            repairs,
            message: message.into(),
            uri: None,
            route: None,
            lifecycle: DiagnosticLifecycle::Raised,
            source_line: None,
            source_character: None,
        }
    }

    /// Construct a minimal finding for test use.
    pub fn minimal(code: CicdCode, message: impl Into<String>) -> Self {
        Self::new(code, "", "", vec![], message)
    }

    /// Set the severity of this finding.
    pub fn with_severity(mut self, severity: CicdSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Attach a file URI to this finding.
    pub fn at_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Attach a repair route to this finding.
    pub fn with_route(mut self, route: RepairRoute) -> Self {
        self.route = Some(route);
        self
    }

    /// Set the 0-indexed LSP line number where the issue was found.
    pub fn at_line(mut self, line: u32) -> Self {
        self.source_line = Some(line);
        self
    }

    /// Set the 0-indexed LSP character offset where the issue was found.
    pub fn at_character(mut self, char: u32) -> Self {
        self.source_character = Some(char);
        self
    }
}
