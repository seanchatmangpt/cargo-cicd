//! Build ServerCapabilities for the initialize response.

use tower_lsp::lsp_types::*;

/// Build the full ServerCapabilities announced during initialize.
pub fn build_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: Some("cargo-cicd-lsp".to_string()),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        ..Default::default()
    }
}
