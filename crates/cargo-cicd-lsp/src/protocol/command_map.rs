//! Known command URIs for cargo-cicd-lsp commands.

/// Command identifier for running a repair command in the terminal.
pub const CMD_RUN_REPAIR: &str = "cargo-cicd-lsp.runRepair";

/// Command identifier for opening the explain panel.
pub const CMD_EXPLAIN: &str = "cargo-cicd-lsp.explain";

/// Command identifier for refreshing diagnostics.
pub const CMD_REFRESH: &str = "cargo-cicd-lsp.refresh";

/// Returns all known command identifiers.
pub fn all_commands() -> &'static [&'static str] {
    &[CMD_RUN_REPAIR, CMD_EXPLAIN, CMD_REFRESH]
}
