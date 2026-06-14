//! cargo-cicd-lsp: author-time witness for local software manufacturing state.
pub mod analyzers;
pub mod commands;
pub mod lifecycle;
pub mod protocol;
pub mod server;
pub mod state;
pub mod watcher;

pub use analyzers::run_all;

/// Start the LSP server over stdin/stdout.
pub fn run_server() {
    server::start();
}
