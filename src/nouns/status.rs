use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct StatusNoun;
impl StatusNoun {
    pub fn new() -> Self {
        Self
    }
}

impl NounCommand for StatusNoun {
    fn name(&self) -> &'static str {
        "status"
    }
    fn about(&self) -> &'static str {
        "Show workspace CI/CD status"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(StatusShowVerb)]
    }
}

impl StatusNoun {
    pub fn run_direct() -> anyhow::Result<()> {
        StatusShowVerb.execute()
    }
}

pub struct StatusShowVerb;

impl StatusShowVerb {
    fn execute(&self) -> anyhow::Result<()> {
        println!("cargo-cicd workspace status");
        println!("===========================");
        let toolchain = ToolchainDetector::active_toolchain();
        println!("toolchain:    {}", toolchain);
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let verdict = TargetScannerAdapter::verdict(target_gb, 20.0);
        println!("target:       {:.2} GB [{}]", target_gb, verdict);
        let git = GitStatusAdapter::query().unwrap_or_default();
        println!("branch:       {}", git.branch);
        println!("dirty files:  {}", git.dirty_files.len());
        println!("untracked:    {}", git.untracked_files.len());
        let dirty_word = if git.dirty_files.is_empty() && git.untracked_files.is_empty() {
            "clean"
        } else {
            "dirty"
        };
        println!("git:          {}", dirty_word);
        Ok(())
    }
}

impl VerbCommand for StatusShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show full CI/CD status"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}
