//! Maps CicdFinding → LSP Diagnostic.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use cargo_cicd_core::diagnostics::{CicdFinding, CicdSeverity};

/// Convert a [`CicdFinding`] into an LSP [`Diagnostic`].
///
/// Findings without a source location are placed at the top of the document (line 0).
pub fn finding_to_lsp(f: &CicdFinding) -> Diagnostic {
    let severity = cicd_severity_to_lsp(f.severity);
    let code = NumberOrString::String(f.code.code_str().to_string());

    // Default position: top of file. Findings from the analyzer carry no column info yet.
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(code),
        code_description: None,
        source: Some("cargo-cicd-lsp".to_string()),
        message: f.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn cicd_severity_to_lsp(s: CicdSeverity) -> DiagnosticSeverity {
    match s {
        CicdSeverity::Error => DiagnosticSeverity::ERROR,
        CicdSeverity::Warning => DiagnosticSeverity::WARNING,
        CicdSeverity::Information => DiagnosticSeverity::INFORMATION,
        CicdSeverity::Hint => DiagnosticSeverity::HINT,
    }
}
