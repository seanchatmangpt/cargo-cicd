//! Explain command — delegates to cargo_cicd_core diagnostics.

use cargo_cicd_core::diagnostics::explain_code;
use tower_lsp::lsp_types::{ExecuteCommandParams, MessageType};

/// Handle the `cargo-cicd.explain` command from the LSP client.
///
/// Extracts the diagnostic code from the first argument and sends a prose
/// explanation back to the client via `showMessage`.
pub async fn handle_explain(client: &tower_lsp::Client, params: ExecuteCommandParams) {
    let code = params
        .arguments
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let message =
        explain_code(code).unwrap_or_else(|| format!("Unknown diagnostic code: {}", code));

    client.show_message(MessageType::INFO, message).await;
}
