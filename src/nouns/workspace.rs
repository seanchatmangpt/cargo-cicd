use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};
use crate::adapters::ToolchainDetector;

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
        println!();
        if !has_cargo || !has_git {
            println!("FAIL: workspace has critical issues");
        } else {
            println!("workspace is healthy");
        }
        Ok(())
    }
}
