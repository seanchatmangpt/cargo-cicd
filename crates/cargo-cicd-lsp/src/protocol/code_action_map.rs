//! Maps CicdFinding → LSP CodeActions.

use tower_lsp::lsp_types::{CodeAction, CodeActionKind, Url};

use cargo_cicd_core::diagnostics::CicdFinding;

/// Convert a [`CicdFinding`] into a list of LSP [`CodeAction`]s.
///
/// One quick-fix action is produced per repair command in `finding.route_commands`.
pub fn finding_to_actions(f: &CicdFinding, _uri: &Url) -> Vec<CodeAction> {
    f.route_commands
        .iter()
        .map(|cmd| CodeAction {
            title: format!("Run: {}", cmd),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        })
        .collect()
}
