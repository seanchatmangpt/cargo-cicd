use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct StatusNoun;
impl StatusNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for StatusNoun {
    fn default() -> Self {
        Self::new()
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
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("status:show");
        start_evt.case_id = Some(case_id.clone());

        println!("cargo-cicd workspace status");
        println!("===========================");
        let toolchain = ToolchainDetector::active_toolchain();
        println!("toolchain:    {}", toolchain);
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let verdict_str = TargetScannerAdapter::verdict(target_gb, 20.0);
        println!("target:       {:.2} GB [{}]", target_gb, verdict_str);
        let git = GitStatusAdapter::query().unwrap_or_default();
        println!("branch:       {}", git.branch);
        println!("dirty files:  {}", git.dirty_files.len());
        println!("untracked:    {}", git.untracked_files.len());
        let dirty = !git.dirty_files.is_empty() || !git.untracked_files.is_empty();
        let dirty_word = if dirty { "dirty" } else { "clean" };
        println!("git:          {}", dirty_word);

        let ev_verdict = if dirty { "WARN" } else { "PASS" };
        let mut complete_evt = ProcessEvent::completed("status:show", t0, ev_verdict);
        complete_evt.case_id = Some(case_id.clone());

        let evidence_path = evidence_dir.join("events.xes");
        if let Err(e) = crate::evidence::emit_xes(&[start_evt, complete_evt], &evidence_path) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
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
