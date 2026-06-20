//! `list_commands` tool — returns the available CLI subcommands as JSON.
//!
//! The command registry is static: it mirrors the noun-verb grammar baked
//! into the binary.  When a new noun is added to the project, add a matching
//! entry to [`COMMAND_REGISTRY`].

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::McpError;

// ---------------------------------------------------------------------------
// CommandEntry
// ---------------------------------------------------------------------------

/// A single CLI command entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// Top-level noun (e.g. `"status"`, `"target"`, `"workspace"`).
    pub noun: String,

    /// Verb within the noun (e.g. `"show"`, `"prune"`, `"doctor"`).
    pub verb: String,

    /// Short human-readable description of what the command does.
    pub description: String,

    /// Whether this command may mutate workspace state or run a subprocess.
    pub is_destructive: bool,

    /// Example invocation string.
    pub example: String,
}

// ---------------------------------------------------------------------------
// Static registry
// ---------------------------------------------------------------------------

/// All commands exposed by the project binary.
///
/// Update this list whenever a noun or verb is added or removed.
static COMMAND_REGISTRY: &[(&str, &str, &str, bool, &str)] = &[
    // (noun, verb, description, is_destructive, example)
    (
        "status",
        "show",
        "Print a workspace health snapshot: toolchain, git phase, target size, changed files.",
        false,
        "cargo cicd status show",
    ),
    (
        "target",
        "show",
        "Show the size and composition of the target/ directory.",
        false,
        "cargo cicd target show",
    ),
    (
        "target",
        "prune",
        "Remove stale build artifacts from target/. Pass --confirm to apply.",
        true,
        "cargo cicd target prune --confirm",
    ),
    (
        "test",
        "changed",
        "Run only the test files that correspond to changed Rust source files.",
        false,
        "cargo cicd test changed",
    ),
    (
        "trybuild",
        "changed",
        "Run trybuild fixtures that are touched by the current diff.",
        false,
        "cargo cicd trybuild changed",
    ),
    (
        "trybuild",
        "full",
        "Run all trybuild compile-error snapshot fixtures.",
        false,
        "cargo cicd trybuild full",
    ),
    (
        "git",
        "status",
        "Show git phase: branch, dirty/staged files, ahead/behind counts.",
        false,
        "cargo cicd git status",
    ),
    (
        "git",
        "close",
        "Advance the git phase gate — commits, squashes, and marks the phase clean.",
        true,
        "cargo cicd git close",
    ),
    (
        "git",
        "phase",
        "Print the current git phase label from cicd.toml.",
        false,
        "cargo cicd git phase",
    ),
    (
        "workspace",
        "doctor",
        "Run all workspace diagnostics and emit policy recommendations.",
        false,
        "cargo cicd workspace doctor",
    ),
    (
        "publish",
        "run",
        "Run the publish gate: verify metadata, license, readme, and registry access.",
        false,
        "cargo cicd publish run",
    ),
    (
        "evidence",
        "doctor",
        "Inspect emitted XES evidence files and report their structure.",
        false,
        "cargo cicd evidence doctor",
    ),
    (
        "evidence",
        "audit",
        "Invoke the wasm4pm oracle to adjudicate all pending evidence.",
        false,
        "cargo cicd evidence audit",
    ),
    (
        "pipeline",
        "run",
        "Execute all CI/CD activities in sequence (check, test, trybuild, publish-gate).",
        true,
        "cargo cicd pipeline run",
    ),
    (
        "lsp",
        "explain",
        "Return an explanation of a Rust compiler diagnostic (IDE integration).",
        false,
        "cargo cicd lsp explain --code E0308",
    ),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the full list of [`CommandEntry`] values from the static registry.
pub fn list_commands() -> Vec<CommandEntry> {
    COMMAND_REGISTRY
        .iter()
        .map(|(noun, verb, description, is_destructive, example)| CommandEntry {
            noun: noun.to_string(),
            verb: verb.to_string(),
            description: description.to_string(),
            is_destructive: *is_destructive,
            example: example.to_string(),
        })
        .collect()
}

/// Serialize the command list to a JSON [`Value`].
pub fn commands_to_json(entries: &[CommandEntry]) -> Result<Value, McpError> {
    serde_json::to_value(entries).map_err(McpError::SerializationError)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_is_non_empty() {
        let cmds = list_commands();
        assert!(!cmds.is_empty());
    }

    #[test]
    fn all_entries_have_non_empty_fields() {
        for cmd in list_commands() {
            assert!(!cmd.noun.is_empty(), "noun should not be empty");
            assert!(!cmd.verb.is_empty(), "verb should not be empty");
            assert!(!cmd.description.is_empty(), "description should not be empty");
            assert!(!cmd.example.is_empty(), "example should not be empty");
        }
    }

    #[test]
    fn status_show_present() {
        let cmds = list_commands();
        let found = cmds.iter().any(|c| c.noun == "status" && c.verb == "show");
        assert!(found, "status show should be in the registry");
    }

    #[test]
    fn destructive_commands_flagged() {
        let cmds = list_commands();
        let prune = cmds.iter().find(|c| c.noun == "target" && c.verb == "prune");
        assert!(prune.is_some());
        assert!(prune.unwrap().is_destructive);
    }

    #[test]
    fn read_only_commands_not_flagged() {
        let cmds = list_commands();
        let status = cmds.iter().find(|c| c.noun == "status" && c.verb == "show");
        assert!(status.is_some());
        assert!(!status.unwrap().is_destructive);
    }

    #[test]
    fn serialises_to_json_array() {
        let cmds = list_commands();
        let json = commands_to_json(&cmds).unwrap();
        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), cmds.len());
        assert!(arr[0].get("noun").is_some());
        assert!(arr[0].get("verb").is_some());
        assert!(arr[0].get("description").is_some());
    }
}
