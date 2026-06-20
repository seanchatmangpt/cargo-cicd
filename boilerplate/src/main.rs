//! Entry point for `cargo-project`.
//!
//! # Cargo external subcommand convention
//!
//! When invoked as `cargo project <noun> <verb>`, Cargo passes the argv as
//! `["cargo-project", "project", <noun>, <verb>, ...]`.  The first positional
//! argument is therefore `"project"` and must be stripped before our parser
//! sees it.  `prepare_args()` handles this transparently so both
//! `cargo project status` and `cargo-project status` work identically.
//!
//! # Default verb injection
//!
//! Bare nouns without an explicit verb are upgraded to their default verb:
//!
//! ```text
//! cargo project status          → status show
//! cargo project workspace       → workspace doctor
//! ```
//!
//! This is implemented in `inject_default_verbs()` before clap parses argv.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use clap::{Parser, Subcommand};

mod nouns;
#[cfg(feature = "process-data")]
mod engine;
#[cfg(feature = "process-data")]
mod adapters;
mod ui;

// ─────────────────────────────────────────────────────────────────────────────
// Default verb table
// ─────────────────────────────────────────────────────────────────────────────

/// Maps each bare noun to the verb that should be injected when none is given.
const DEFAULT_VERBS: &[(&str, &str)] = &[
    ("status", "show"),
    ("workspace", "doctor"),
    ("target", "show"),
    ("evidence", "doctor"),
    ("publish", "run"),
    ("pipeline", "run"),
];

/// Pre-processes `argv` to:
/// 1. Strip the `project` passthrough token inserted by `cargo`.
/// 2. Inject a default verb for bare nouns.
fn prepare_args(mut args: Vec<String>) -> Vec<String> {
    // Drop the cargo passthrough: ["cargo-project", "project", ...]
    if args.get(1).map(String::as_str) == Some("project") {
        args.remove(1);
    }

    inject_default_verbs(&mut args);
    args
}

/// If `argv[1]` is a known noun and `argv[2]` is absent or starts with `--`,
/// splice in the default verb so clap sees a complete noun-verb pair.
fn inject_default_verbs(args: &mut Vec<String>) {
    let noun = match args.get(1) {
        Some(n) if !n.starts_with('-') => n.clone(),
        _ => return,
    };

    let needs_verb = match args.get(2) {
        None => true,
        Some(v) => v.starts_with('-'),
    };

    if !needs_verb {
        return;
    }

    for &(n, default_verb) in DEFAULT_VERBS {
        if noun == n {
            args.insert(2, default_verb.to_owned());
            return;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Clap top-level CLI definition
// ─────────────────────────────────────────────────────────────────────────────

/// PROJECT keeps your Rust workspace clean, fast, and push-ready.
#[derive(Debug, Parser)]
#[command(
    name = "cargo-project",
    bin_name = "cargo project",
    version,
    author,
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    noun: NounCommands,
}

#[derive(Debug, Subcommand)]
enum NounCommands {
    /// Workspace health snapshot.
    Status(nouns::status::StatusArgs),
    /// Workspace-wide diagnostics.
    Workspace(nouns::workspace::WorkspaceArgs),
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    // Initialise structured logging.  RUST_LOG controls the filter;
    // defaults to "warn" so normal users see only actionable output.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .compact()
        .init();

    let args: Vec<String> = std::env::args().collect();
    let args = prepare_args(args);

    let cli = Cli::parse_from(args);

    match cli.noun {
        NounCommands::Status(args) => nouns::status::run(args),
        NounCommands::Workspace(args) => nouns::workspace::run(args),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_default_verb_status() {
        let mut args =
            vec!["cargo-project".to_owned(), "status".to_owned()];
        inject_default_verbs(&mut args);
        assert_eq!(args, ["cargo-project", "status", "show"]);
    }

    #[test]
    fn inject_default_verb_not_duplicated_when_verb_present() {
        let mut args = vec![
            "cargo-project".to_owned(),
            "status".to_owned(),
            "show".to_owned(),
        ];
        inject_default_verbs(&mut args);
        // Should be unchanged — verb already present
        assert_eq!(args, ["cargo-project", "status", "show"]);
    }

    #[test]
    fn inject_default_verb_flag_after_noun() {
        // `cargo project status --json` → inject "show" before the flag
        let mut args = vec![
            "cargo-project".to_owned(),
            "status".to_owned(),
            "--json".to_owned(),
        ];
        inject_default_verbs(&mut args);
        assert_eq!(args, ["cargo-project", "status", "show", "--json"]);
    }

    #[test]
    fn prepare_args_strips_cargo_passthrough() {
        let args = vec![
            "cargo-project".to_owned(),
            "project".to_owned(),
            "status".to_owned(),
        ];
        let result = prepare_args(args);
        assert_eq!(result[1], "status");
        assert_eq!(result[2], "show"); // default verb injected
    }

    #[test]
    fn prepare_args_no_passthrough_unchanged() {
        let args =
            vec!["cargo-project".to_owned(), "status".to_owned(), "show".to_owned()];
        let result = prepare_args(args);
        assert_eq!(result[1], "status");
        assert_eq!(result[2], "show");
    }
}
