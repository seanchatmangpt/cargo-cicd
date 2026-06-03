//! Graceful shutdown handling.

/// Perform any cleanup needed on shutdown.
/// Currently a no-op; expanded as resources are added.
pub fn on_shutdown() {
    tracing::info!("cargo-cicd-lsp shutting down");
}
