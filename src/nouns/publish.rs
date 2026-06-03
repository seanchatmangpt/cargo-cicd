use crate::adapters::{
    ChangedFileDetector, GitStatusAdapter, TargetScannerAdapter, ToolchainDetector,
};
use crate::cicd_toml::CicdToml;
use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct PublishNoun;
impl PublishNoun {
    pub fn new() -> Self {
        Self
    }
    pub fn run_direct() -> anyhow::Result<()> {
        let matches = clap::Command::new("publish").get_matches_from(vec!["publish"]);
        let args = clap_noun_verb::VerbArgs::new(matches);
        PublishRunVerb
            .run(&args)
            .map_err(|e| anyhow::anyhow!("{}", e))
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
        // ── Adjudicated publish gate ──────────────────────────────────────────
        // Before writing cicd.toml we ask the wasm4pm oracle to adjudicate the
        // current evidence XES.  This closes the gap between "cargo-cicd claims
        // PASS" and "an independent process oracle agrees".
        //
        // Gate outcomes:
        //   ADJUDICATED:accept       — oracle accepted the evidence; proceed.
        //   WARN:oracle_unavailable  — binary not found; proceed with a warning.
        //   (bail!)                  — oracle refused; publish is blocked.
        let evidence_xes = crate::evidence::evidence_dir().join("events.xes");
        let publish_readiness = if evidence_xes.exists() {
            match crate::integrations::Wasm4pmShell::detect() {
                None => {
                    eprintln!(
                        "warning: wasm4pm oracle unavailable — publish proceeding without \
                         adjudication (BLOCKED:oracle_unavailable)"
                    );
                    "BLOCKED:oracle_unavailable"
                }
                Some(wpm) => match wpm.audit(evidence_xes.to_str().unwrap_or("")) {
                    Err(e) => {
                        eprintln!("warning: oracle invocation failed: {e} — proceeding without adjudication");
                        "WARN:oracle_error"
                    }
                    Ok(result) => {
                        use crate::integrations::WpmVerdict;
                        match result.verdict {
                            WpmVerdict::Pass | WpmVerdict::Warn | WpmVerdict::Partial => {
                                "ADJUDICATED:accept"
                            }
                            WpmVerdict::Fail => {
                                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                                    "AndonPull: wasm4pm refused evidence — publish blocked"
                                        .to_string(),
                                ));
                            }
                            WpmVerdict::NotAvailable => "BLOCKED:oracle_unavailable",
                        }
                    }
                },
            }
        } else {
            // No evidence file yet — first-run; proceed without adjudication.
            "WARN:no_evidence"
        };
        println!("  adjudication: {}", publish_readiness);

        cicd.write_default()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("published cicd.toml");
        println!("  workspace:    {}", cicd.workspace.name);
        println!("  toolchain:    {}", cicd.workspace.toolchain);
        println!("  target:       {:.2} GB", cicd.state.target_size_gb);
        println!("  dirty:        {}", cicd.state.dirty);
        println!("  changed:      {}", cicd.state.changed_files);

        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let mut event = ProcessEvent::new("publish:run", "PASS");
        event.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[event], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}
