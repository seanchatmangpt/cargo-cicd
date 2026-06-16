//! Known command URIs for cargo-cicd-lsp commands.

/// Command identifier for running a repair command in the terminal.
pub const CMD_RUN_REPAIR: &str = "cargo-cicd-lsp.runRepair";

/// Command identifier for opening the explain panel.
pub const CMD_EXPLAIN: &str = "cargo-cicd.explain";

/// Command identifier for refreshing diagnostics.
pub const CMD_REFRESH: &str = "cargo-cicd-lsp.refresh";

/// Command identifier for running the full pipeline.
pub const CMD_RUN_PIPELINE: &str = "cargo-cicd.pipeline.run";

/// Command identifier for checking git status via the LSP.
pub const CMD_GIT_STATUS: &str = "cargo-cicd.git.status";

/// Command identifier for pruning the target directory.
pub const CMD_TARGET_PRUNE: &str = "cargo-cicd.target.prune";

/// Command identifier for running the wpm evidence doctor.
pub const CMD_EVIDENCE_DOCTOR: &str = "cargo-cicd.evidence.doctor";

/// Returns all known command identifiers.
pub fn all_commands() -> &'static [&'static str] {
    &[
        CMD_RUN_REPAIR,
        CMD_EXPLAIN,
        CMD_REFRESH,
        CMD_RUN_PIPELINE,
        CMD_GIT_STATUS,
        CMD_TARGET_PRUNE,
        CMD_EVIDENCE_DOCTOR,
    ]
}
