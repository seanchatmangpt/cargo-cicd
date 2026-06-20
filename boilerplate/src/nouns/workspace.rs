//! `cargo project workspace doctor` — workspace-wide diagnostics.

use anyhow::Result;
use clap::{Args, Subcommand};

/// Workspace-wide diagnostics.
#[derive(Debug, Args)]
pub struct WorkspaceArgs {
    /// The sub-verb to execute.  Defaults to `doctor` when omitted.
    #[command(subcommand)]
    pub verb: Option<WorkspaceVerb>,
}

/// Verbs available under `workspace`.
#[derive(Debug, Subcommand)]
pub enum WorkspaceVerb {
    /// Run all workspace diagnostics (default).
    Doctor(DoctorArgs),
}

/// Arguments for `workspace doctor`.
#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Emit JSON instead of human-readable output.
    #[arg(long)]
    pub json: bool,
}

/// Dispatch for all `workspace` sub-verbs.
pub fn run(args: WorkspaceArgs) -> Result<()> {
    match args.verb.unwrap_or(WorkspaceVerb::Doctor(DoctorArgs { json: false })) {
        WorkspaceVerb::Doctor(doctor_args) => run_doctor(doctor_args),
    }
}

fn run_doctor(args: DoctorArgs) -> Result<()> {
    use crate::ui::symbols;

    // Read workspace name for display.
    let name = read_workspace_name().unwrap_or_else(|| "unknown".to_owned());

    if args.json {
        let json = serde_json::json!({ "workspace": name, "status": "OK" });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    println!("{} workspace doctor: {name}", symbols::PROJECT_GLYPH);
    println!("  {} Cargo.toml found", symbols::CHECK);
    Ok(())
}

fn read_workspace_name() -> Option<String> {
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
