use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};
use crate::adapters::{ToolchainDetector, TargetScannerAdapter, GitStatusAdapter};
use crate::autonomic::policies::{run_all_policies, PolicyVerdict, WorkspaceInfo, GitState};

pub struct WorkspaceNoun;
impl WorkspaceNoun { pub fn new() -> Self { Self } }

impl NounCommand for WorkspaceNoun {
    fn name(&self) -> &'static str { "workspace" }
    fn about(&self) -> &'static str { "Workspace diagnostics" }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(WorkspaceDoctorVerb)]
    }
}

pub struct WorkspaceDoctorVerb;
impl VerbCommand for WorkspaceDoctorVerb {
    fn name(&self) -> &'static str { "doctor" }
    fn about(&self) -> &'static str { "Diagnose workspace health" }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        println!("workspace doctor");
        println!("================");

        let has_cargo = std::path::Path::new("Cargo.toml").exists();
        println!("[{}] Cargo.toml", if has_cargo { "OK" } else { "FAIL" });

        let toolchain = ToolchainDetector::active_toolchain();
        println!("[OK] toolchain: {}", toolchain);

        let has_toolchain_file = std::path::Path::new("rust-toolchain.toml").exists()
            || std::path::Path::new("rust-toolchain").exists();
        println!("[{}] rust-toolchain file", if has_toolchain_file { "OK" } else { "WARN" });

        let has_git = std::path::Path::new(".git").exists();
        println!("[{}] git repository", if has_git { "OK" } else { "FAIL" });

        let has_cicd = std::path::Path::new("cicd.toml").exists();
        println!("[{}] cicd.toml (run 'cargo cicd publish' to generate)", if has_cicd { "OK" } else { "WARN" });

        // ── autonomic policy checks ──────────────────────────────────────────
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let pinned_toolchain = read_pinned_toolchain();
        let git_dirty = GitStatusAdapter::query()
            .map(|r| r.dirty_files.len())
            .unwrap_or(0);

        let workspace_info = WorkspaceInfo {
            target_gb,
            max_gb: 20.0,
            active_toolchain: toolchain.clone(),
            pinned_toolchain,
            changed_trybuild_fixtures: 0,
        };
        let git_state = GitState { dirty_count: git_dirty };

        let results = run_all_policies(&workspace_info, &git_state);

        if !results.is_empty() {
            println!();
            println!("autonomic policy results");
            println!("------------------------");
            for r in &results {
                let tag = match r.verdict {
                    PolicyVerdict::Pass    => "PASS",
                    PolicyVerdict::Warn    => "WARN",
                    PolicyVerdict::Suggest => "SUGGEST",
                };
                if r.recommendation.is_empty() {
                    println!("[{}] {}", tag, r.name);
                } else {
                    println!("[{}] {}: {}", tag, r.name, r.recommendation);
                }
            }
        }

        println!();
        if !has_cargo || !has_git {
            println!("FAIL: workspace has critical issues");
        } else {
            println!("workspace is healthy");
        }
        Ok(())
    }
}

/// Read the channel from `rust-toolchain.toml` if it exists.
fn read_pinned_toolchain() -> Option<String> {
    if std::path::Path::new("rust-toolchain.toml").exists() {
        std::fs::read_to_string("rust-toolchain.toml").ok().and_then(|s| {
            s.lines()
                .find(|l| l.contains("channel"))
                .and_then(|l| l.split('"').nth(1))
                .map(|s| s.to_string())
        })
    } else {
        None
    }
}
