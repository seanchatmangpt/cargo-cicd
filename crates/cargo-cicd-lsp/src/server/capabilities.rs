//! Server capability declarations.

use tower_lsp::lsp_types::*;

/// Returns the ServerCapabilities advertised during initialization.
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        ..Default::default()
    }
}
