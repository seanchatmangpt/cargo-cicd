//! Maps CicdFinding → LSP Diagnostic.

use serde_json::json;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use cargo_cicd_core::diagnostics::{CicdFinding, CicdSeverity};

/// Convert a [`CicdFinding`] into an LSP [`Diagnostic`].
///
/// Findings without a source location are placed at the top of the document (line 0).
/// The `data` field carries the CicdCode string so hover providers and code-action
/// handlers can retrieve the code without re-parsing the `code` field.
/// The `source` field is always `"cargo-cicd"`.
pub fn finding_to_lsp(f: &CicdFinding) -> Diagnostic {
    let severity = cicd_severity_to_lsp(f.severity);
    let code_str = f.code.as_str().to_string();
    let code = NumberOrString::String(code_str.clone());

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

    // Attach the code string as structured data so code-action and hover providers
    // can retrieve it without re-parsing the `code` field.
    let data = Some(json!({ "code": code_str }));

    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(code),
        code_description: None,
        source: Some("cargo-cicd".to_string()),
        message: f.message.clone(),
        related_information: None,
        tags: None,
        data,
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
