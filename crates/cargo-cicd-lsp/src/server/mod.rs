//! LSP server entry point.
pub mod backend;
pub mod capabilities;
pub mod initialize;
pub mod shutdown;

use backend::Backend;
use tower_lsp::{LspService, Server};

/// Start the LSP server over stdin/stdout using the tower-lsp runtime.
pub fn start() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
        .block_on(async {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            let (service, socket) = LspService::new(|client| Backend::new(client));
            Server::new(stdin, stdout, socket).serve(service).await;
        });
}
