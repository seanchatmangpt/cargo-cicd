//! Binary entry point for `cargo-project-mcp`.
//!
//! Reads [`ServerConfig`] from environment variables, initialises `tracing`,
//! builds a [`ProjectMcpServer`], and runs it on the chosen transport.
//!
//! # Environment variables
//!
//! | Variable              | Default                    | Effect                         |
//! |-----------------------|----------------------------|--------------------------------|
//! | `MCP_BIND_ADDR`       | *(unset → stdio)*          | Switch to HTTP/SSE transport.  |
//! | `MCP_WORKSPACE_ROOT`  | current working directory  | Override workspace root.       |
//! | `MCP_TIMEOUT_SECS`    | `60`                       | Subprocess timeout.            |
//! | `MCP_MAX_FILE_BYTES`  | `1048576` (1 MiB)          | Max `read_file` payload.       |
//! | `RUST_LOG`            | `info`                     | Tracing filter.                |
//!
//! # Usage
//!
//! ```sh
//! # stdio transport (Claude Desktop, Cursor, etc.)
//! cargo-project-mcp
//!
//! # HTTP/SSE transport on a custom address
//! MCP_BIND_ADDR=127.0.0.1:8080 cargo-project-mcp
//!
//! # Custom workspace and timeout
//! MCP_WORKSPACE_ROOT=/my/project MCP_TIMEOUT_SECS=120 cargo-project-mcp
//! ```

use project_mcp_server::{ProjectMcpServer, ServerConfig, TransportKind};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialise tracing (structured logging to stderr so it does not
    //    corrupt the stdio MCP stream).
    init_tracing();

    // 2. Load configuration from environment variables.
    let config = ServerConfig::from_env();
    info!(
        workspace_root = %config.workspace_root.display(),
        timeout_secs = config.timeout_secs,
        max_file_bytes = config.max_file_size_bytes,
        "project-mcp-server starting",
    );

    // 3. Build the server.
    let server = ProjectMcpServer::new(config.clone());

    // 4. Start the transport.
    match &config.transport {
        TransportKind::Stdio => {
            info!("using stdio transport");
            run_stdio(server).await?;
        }
        TransportKind::Http { bind_addr } => {
            info!(bind_addr = %bind_addr, "using HTTP/SSE transport");
            run_http(server, bind_addr).await?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Transport runners
// ---------------------------------------------------------------------------

/// Run the server on stdin/stdout.
///
/// This is the most widely supported MCP transport.
async fn run_stdio(server: ProjectMcpServer) -> anyhow::Result<()> {
    use rmcp::transport::StdioServerTransport;

    // NOTE: `StdioServerTransport::new()` reads from stdin and writes to
    // stdout.  Tracing logs go to stderr so they don't interfere.
    //
    // If the rmcp API differs (e.g. it uses `serve_stdio(handler)` directly),
    // replace the body of this function accordingly.
    let transport = StdioServerTransport::new();
    server
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("stdio transport error: {e}"))?;

    Ok(())
}

/// Run the server on an HTTP/SSE transport.
async fn run_http(server: ProjectMcpServer, bind_addr: &str) -> anyhow::Result<()> {
    use rmcp::transport::SseServerTransport;

    // NOTE: The SSE transport constructor name may vary across rmcp versions.
    // Common variants: `SseServerTransport::new(addr)` or
    // `SseServerTransport::bind(addr)`.  Adjust if needed.
    let transport = SseServerTransport::new(bind_addr);
    server
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP/SSE transport error: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tracing initialisation
// ---------------------------------------------------------------------------

/// Initialise the global tracing subscriber.
///
/// - Uses `RUST_LOG` for the filter (defaults to `info`).
/// - Outputs to **stderr** so it does not pollute the stdio MCP stream.
/// - Uses compact format for readability.
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .compact()
        .init();
}
