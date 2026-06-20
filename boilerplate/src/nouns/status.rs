//! `cargo project status show` — workspace health snapshot.
//!
//! Reads the current [`EngineState`] (or a lightweight fallback when the
//! `process-data` feature is not enabled) and prints a one-screen summary of
//! workspace health.
//!
//! Exit codes:
//! - `0` — workspace is healthy (PASS or WARN)
//! - `1` — workspace is unhealthy (FAIL) or an unexpected error occurred

use anyhow::Result;
use clap::{Args, Subcommand};

// ─────────────��─────────────────────────────────────��─────────────────────────
// Clap structures
// ──────────────────────────────────��───────────────────────────────���──────────

/// Workspace health snapshot.
#[derive(Debug, Args)]
pub struct StatusArgs {
    /// The sub-verb to execute.  Defaults to `show` when omitted.
    #[command(subcommand)]
    pub verb: Option<StatusVerb>,
}

/// Verbs available under `status`.
#[derive(Debug, Subcommand)]
pub enum StatusVerb {
    /// Display the current workspace health snapshot (default).
    Show(ShowArgs),
}

/// Arguments for `status show`.
#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Emit machine-readable JSON instead of the human-friendly table.
    #[arg(long)]
    pub json: bool,

    /// Show verbose output including all adapter readings.
    #[arg(long, short)]
    pub verbose: bool,
}

// ─────��─────────────────────────���─────────────────────────────────────────────
// Entry point
// ─���────────────────────��──────────────────────────────────────────────────────

/// Dispatch for all `status` sub-verbs.
pub fn run(args: StatusArgs) -> Result<()> {
    match args.verb.unwrap_or(StatusVerb::Show(ShowArgs { json: false, verbose: false })) {
        StatusVerb::Show(show_args) => run_show(show_args),
    }
}

// ─────────────────────────────────���───────────────────────────────────────────
// `status show`
// ───────��──────────────────��──────────────────────────────────────────────────

fn run_show(args: ShowArgs) -> Result<()> {
    #[cfg(feature = "process-data")]
    {
        run_show_full(args)
    }
    #[cfg(not(feature = "process-data"))]
    {
        run_show_minimal(args)
    }
}

/// Full implementation when the `process-data` feature is active.
///
/// Populates the complete [`crate::engine::EngineState`] via all registered
/// adapters and renders a rich health table.
#[cfg(feature = "process-data")]
fn run_show_full(args: ShowArgs) -> Result<()> {
    use crate::engine::EngineState;
    use crate::ui::{badge::Badge, symbols, theme::Theme};

    let state = EngineState::from_workspace();
    let theme = Theme::detect();

    if args.json {
        // Machine-readable output: emit the state as JSON.
        let json = serde_json::json!({
            "workspace": state.workspace.name,
            "root_path": state.workspace.root_path,
            "git_phase": {
                "branch": state.git.branch,
                "dirty_files": state.git.dirty_files.len(),
                "staged_files": state.git.staged_files.len(),
                "untracked_files": state.git.untracked_files.len(),
                "ahead": state.git.ahead,
                "behind": state.git.behind,
            },
            "verdict": determine_verdict(&state).label(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    // Human-readable output.
    let verdict = determine_verdict(&state);
    let badge = Badge::from_verdict(&verdict, &theme);

    println!(
        "{} {} {}",
        symbols::PROJECT_GLYPH,
        state.workspace.name,
        badge
    );
    println!();

    // Git summary
    let dirty = state.git.dirty_files.len();
    let staged = state.git.staged_files.len();
    let untracked = state.git.untracked_files.len();

    let git_glyph = if dirty > 0 || staged > 0 {
        symbols::WARN
    } else {
        symbols::CHECK
    };

    println!(
        "  {} git  branch={} dirty={dirty} staged={staged} untracked={untracked}",
        git_glyph, state.git.branch
    );

    if args.verbose && dirty > 0 {
        for f in &state.git.dirty_files {
            println!("       {} {f}", symbols::BULLET);
        }
    }

    println!();

    // Toolchain summary
    println!(
        "  {} toolchain  {}",
        symbols::INFO,
        state.toolchain.rust_version
    );

    Ok(())
}

/// Determine an overall verdict for the workspace given its current state.
#[cfg(feature = "process-data")]
fn determine_verdict(state: &crate::engine::EngineState) -> project_core::Verdict {
    if !state.git.dirty_files.is_empty() || !state.git.staged_files.is_empty() {
        return project_core::Verdict::Warn;
    }
    project_core::Verdict::Pass
}

/// Minimal implementation when `process-data` is not enabled.
///
/// Shows a brief one-liner using only information available without the
/// Level 5 engine.
#[cfg(not(feature = "process-data"))]
#[allow(unused_variables)]
fn run_show_minimal(args: ShowArgs) -> Result<()> {
    use crate::ui::symbols;

    // Determine workspace name from Cargo.toml without the full adapter stack.
    let name = read_workspace_name_simple().unwrap_or_else(|| "unknown".to_owned());
    println!("{} {name}  status: OK (basic mode)", symbols::CHECK);
    Ok(())
}

/// Read the `[package] name` from the nearest `Cargo.toml` using only `std`.
///
/// This is intentionally simple — it avoids pulling in `toml` or any adapter
/// so the minimal build remains lean.
#[cfg(not(feature = "process-data"))]
fn read_workspace_name_simple() -> Option<String> {
    let content = std::fs::read_to_string("Cargo.toml").ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(val) = line.splitn(2, '=').nth(1) {
                return Some(val.trim().trim_matches('"').to_owned());
            }
        }
    }
    None
}

// ���──────────────────────────────────────────────────────────────���─────────────
// Tests
// ─────────────────────���─────────────────────────────────���─────────────────────

#[cfg(test)]
mod tests {
    #[test]
    fn show_args_default_values() {
        use super::ShowArgs;
        let args = ShowArgs { json: false, verbose: false };
        assert!(!args.json);
        assert!(!args.verbose);
    }
}
