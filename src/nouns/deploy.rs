//! Projects the cargo-cicd command graph onto deployment surfaces provided by
//! `clap-noun-verb-deploy` (MCP stdio today; http/kubernetes/container are
//! schema-only render surfaces the crate exposes for later use).
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use clap_noun_verb_deploy::mcp::McpServer;
use clap_noun_verb_deploy::{CliSchema, ProcessExecutor};
use std::sync::OnceLock;

static SCHEMA: OnceLock<CliSchema> = OnceLock::new();

/// Populate the schema snapshot. Must be called once, before
/// `CommandRegistry::run` takes its lock — see the comment in `main.rs`. The
/// registry's `Mutex` is not reentrant, so verbs below cannot re-derive the
/// schema from the live registry while `run()` is executing them.
pub fn set_schema(schema: CliSchema) {
    let _ = SCHEMA.set(schema);
}

fn schema() -> anyhow::Result<&'static CliSchema> {
    SCHEMA
        .get()
        .ok_or_else(|| anyhow::anyhow!("deploy schema was not initialized before dispatch"))
}

fn current_binary() -> std::ffi::OsString {
    std::env::current_exe()
        .map(|p| p.into_os_string())
        .unwrap_or_else(|_| "cargo-cicd".into())
}

fn print_schema() -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(schema()?)?;
    println!("{}", json);
    Ok(())
}

fn serve_mcp() -> anyhow::Result<()> {
    let executor = ProcessExecutor::new(current_binary());
    let server = McpServer::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        schema()?.clone(),
        executor,
    );
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    server
        .serve_stdio(stdin.lock(), stdout.lock())
        .map_err(|e| anyhow::anyhow!("mcp stdio server failed: {}", e))
}

/// Print the deployment-projected CLI schema (tool names, args) as JSON.
#[verb("schema")]
pub fn cmd_schema() -> Result<()> {
    print_schema().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

/// Serve the cargo-cicd command graph as an MCP stdio server.
#[verb("mcp")]
pub fn cmd_mcp() -> Result<()> {
    serve_mcp().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
