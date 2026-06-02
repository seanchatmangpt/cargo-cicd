use crate::adapters::TargetScannerAdapter;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct TargetNoun;
impl TargetNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TargetNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for TargetNoun {
    fn name(&self) -> &'static str {
        "target"
    }
    fn about(&self) -> &'static str {
        "Manage target directory"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(TargetShowVerb), Box::new(TargetPruneVerb)]
    }
}

pub struct TargetShowVerb;
impl VerbCommand for TargetShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show target directory size and state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let target_dir = "target";
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        let max_gb = 20.0_f64;
        let verdict = TargetScannerAdapter::verdict(size_gb, max_gb);
        println!("target directory: {}", target_dir);
        println!("total size:       {:.2} GB", size_gb);
        println!("max configured:   {:.1} GB", max_gb);
        println!("verdict:          {}", verdict);
        if verdict != "pass" {
            println!("recommendation:   run 'cargo cicd target prune' to free space");
        }
        Ok(())
    }
}

pub struct TargetPruneVerb;
impl VerbCommand for TargetPruneVerb {
    fn name(&self) -> &'static str {
        "prune"
    }
    fn about(&self) -> &'static str {
        "Plan target directory cleanup (safe by default)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let target_dir = "target";
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        println!("target prune plan");
        println!("=================");
        println!("current size: {:.2} GB", size_gb);
        println!("mode:         suggest (use --apply to execute)");
        println!();
        println!("suggested candidates:");
        for profile in &["debug/incremental", "debug/.fingerprint", "debug/deps"] {
            let p = format!("{}/{}", target_dir, profile);
            if std::path::Path::new(&p).exists() {
                let sz = TargetScannerAdapter::total_size_bytes(&p);
                println!("  {} ({:.2} GB)", p, sz as f64 / 1_073_741_824.0);
            }
        }
        println!();
        println!("to execute: cargo cicd target prune --apply");
        println!("note: release artifacts are never deleted automatically");
        Ok(())
    }
}
