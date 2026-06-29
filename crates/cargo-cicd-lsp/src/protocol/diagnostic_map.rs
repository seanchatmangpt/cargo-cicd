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

    let start_line = f.source_line.unwrap_or(0);
    let start_char = f.source_character.unwrap_or(0);
    let range = Range {
        start: Position {
            line: start_line,
            character: start_char,
        },
        end: Position {
            line: start_line,
            character: start_char.saturating_add(1),
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
