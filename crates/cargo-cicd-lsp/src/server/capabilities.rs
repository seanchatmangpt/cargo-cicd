//! Server capability declarations.

use tower_lsp::lsp_types::*;

/// Returns the ServerCapabilities advertised during initialization.
pub fn server_capabilities() -> ServerCapabilities {
    build_server_capabilities()
}

/// Returns the ServerCapabilities advertised during initialization,
/// including diagnosticProvider for Law 5 editor diagnostics proof.
pub fn build_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: Some("cargo-cicd-lsp".to_string()),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
            work_done_progress_options: Default::default(),
        })),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        ..Default::default()
    }
}
