use crate::adapters::{
    ChangedFileDetector, GitStatusAdapter, TargetScannerAdapter, ToolchainDetector,
};
use crate::cicd_toml::CicdToml;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct PublishNoun;
impl PublishNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PublishNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for PublishNoun {
    fn name(&self) -> &'static str {
        "publish"
    }
    fn about(&self) -> &'static str {
        "Publish cicd.toml with current workspace state"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(PublishRunVerb)]
    }
}

pub struct PublishRunVerb;
impl VerbCommand for PublishRunVerb {
    fn name(&self) -> &'static str {
        "run"
    }
    fn about(&self) -> &'static str {
        "Emit cicd.toml with current workspace state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let mut cicd = CicdToml::from_current_workspace();
        let target_gb = TargetScannerAdapter::total_size_gb(&cicd.workspace.target_dir);
        cicd.state.target_size_gb = (target_gb * 100.0).round() / 100.0;
        let git = GitStatusAdapter::query().unwrap_or_default();
        cicd.state.dirty = !git.dirty_files.is_empty() || !git.untracked_files.is_empty();
        let changed = ChangedFileDetector::changed_rs_files(&cicd.test.changed.base);
        cicd.state.changed_files = changed.len();
        cicd.state.changed_tests = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_test_file(f))
            .count();
        cicd.state.changed_trybuild_fixtures = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_trybuild_fixture(f))
            .count();
        cicd.workspace.toolchain = ToolchainDetector::active_toolchain();
        cicd.write_default()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("published cicd.toml");
        println!("  workspace:    {}", cicd.workspace.name);
        println!("  toolchain:    {}", cicd.workspace.toolchain);
        println!("  target:       {:.2} GB", cicd.state.target_size_gb);
        println!("  dirty:        {}", cicd.state.dirty);
        println!("  changed:      {}", cicd.state.changed_files);
        Ok(())
    }
}
