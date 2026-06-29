//! Maps CicdFinding → LSP CodeActions.

use tower_lsp::lsp_types::{CodeAction, CodeActionKind, Command, Diagnostic, Url};

use cargo_cicd_core::diagnostics::CicdFinding;

/// Convert a [`CicdFinding`] into a list of LSP [`CodeAction`]s.
///
/// One quick-fix action is produced per repair command in `finding.repairs`.
/// Each action links back to `diagnostic` so the editor can highlight the
/// associated squiggle when the action is selected.
pub fn finding_to_actions(
    f: &CicdFinding,
    _uri: &Url,
    diagnostic: Option<&Diagnostic>,
) -> Vec<CodeAction> {
    let linked_diagnostics = diagnostic.map(|d| vec![d.clone()]);

    f.repairs
        .iter()
        .enumerate()
        .map(|(i, cmd)| CodeAction {
            title: format!("cargo-cicd: {}", cmd),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: linked_diagnostics.clone(),
            edit: None,
            command: Some(Command {
                title: cmd.clone(),
                command: "cargo-cicd.execute".to_string(),
                arguments: Some(vec![serde_json::json!(cmd)]),
            }),
            is_preferred: Some(i == 0),
            disabled: None,
            data: None,
        })
        .collect()
}
