//! Tests that CicdFinding source location fields propagate to LSP Diagnostic range.

use cargo_cicd_core::diagnostics::code::CicdCode;
use cargo_cicd_core::diagnostics::CicdFinding;
use cargo_cicd_lsp::protocol::diagnostic_map::finding_to_lsp;

/// A finding built with `.at_line(5)` must produce an LSP diagnostic whose
/// `range.start.line` equals 5.
#[test]
fn finding_at_line_produces_correct_diagnostic_range() {
    let finding = CicdFinding::minimal(CicdCode::EvidenceMissing, "test message").at_line(5);
    let diagnostic = finding_to_lsp(&finding);
    assert_eq!(
        diagnostic.range.start.line, 5,
        "expected range.start.line == 5, got {}",
        diagnostic.range.start.line
    );
    assert_eq!(
        diagnostic.range.end.line, 5,
        "expected range.end.line == 5, got {}",
        diagnostic.range.end.line
    );
}

/// A finding with `.at_line(3).at_character(7)` must set both start position fields.
#[test]
fn finding_at_line_and_character_produces_correct_range() {
    let finding = CicdFinding::minimal(CicdCode::EvidenceMissing, "test message")
        .at_line(3)
        .at_character(7);
    let diagnostic = finding_to_lsp(&finding);
    assert_eq!(diagnostic.range.start.line, 3);
    assert_eq!(diagnostic.range.start.character, 7);
    assert_eq!(
        diagnostic.range.end.character, 8,
        "end character should be start + 1"
    );
}

/// A finding without location info defaults to line 0, character 0.
#[test]
fn finding_without_location_defaults_to_origin() {
    let finding = CicdFinding::minimal(CicdCode::EvidenceMissing, "test message");
    let diagnostic = finding_to_lsp(&finding);
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
}
