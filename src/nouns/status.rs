use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::evidence::ProcessEvent;
use crate::nouns::evidence_helpers::{finish_evidence, init_evidence};
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
        vec![Box::new(StatusShowVerb), Box::new(StatusAuditVerb)]
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
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("status:show");

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
        finish_evidence(start_evt, t0, case_id, ev_verdict, "status:show", &evidence_dir);
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

// ── audit verb ────────────────────────────────────────────────────────────────

/// `cargo cicd status audit` — shell out to wpm to adjudicate the current
/// evidence XES file.  Emits an `evidence:audit` event (with oracle provenance)
/// back into the log, then fails if the oracle refuses.
pub struct StatusAuditVerb;

impl StatusAuditVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let xes = evidence_dir.join("events.xes");

        if !xes.exists() {
            println!("BLOCKED: no evidence at {}", xes.display());
            return Ok(());
        }

        // Detect wpm oracle.
        let wpm = crate::integrations::Wasm4pmShell::detect();
        let Some(wpm_shell) = wpm else {
            println!("BLOCKED: wpm oracle not found");
            return Ok(());
        };

        println!("wasm4pm evidence audit");
        println!("======================");
        println!("evidence: {}", xes.display());
        println!("wpm:      {}", wpm_shell.binary_path());

        let result = wpm_shell
            .audit(xes.to_str().unwrap_or(""))
            .unwrap_or_else(|e| crate::integrations::WpmResult {
                command: "wpm audit".to_string(),
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                verdict: crate::integrations::WpmVerdict::Fail,
            });

        println!(
            "exit:     {}",
            if result.success { "0" } else { "non-zero" }
        );
        println!("stdout:   {}", result.stdout.trim());
        if !result.stderr.trim().is_empty() {
            println!("stderr:   {}", result.stderr.trim());
        }

        let oracle_verdict = if result.success { "ACCEPT" } else { "REFUSE" };
        println!("verdict:  {}", oracle_verdict);

        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        // 1. Emit status:audit — the declared required_stage event.
        let (mut sa_start, sa_t0) = crate::evidence::ProcessEvent::started("status:audit");
        sa_start.case_id = Some(case_id.clone());
        let mut sa_complete =
            crate::evidence::ProcessEvent::completed("status:audit", sa_t0, oracle_verdict);
        sa_complete.case_id = Some(case_id.clone());

        // 2. Emit evidence:audit — adjudicated event with oracle provenance.
        let mut ea_evt = crate::evidence::ProcessEvent::new_adjudicated(
            "evidence:audit",
            oracle_verdict,
            wpm_shell.binary_path(),
        );
        ea_evt.case_id = Some(case_id.clone());

        // 3. Emit receipt:write — only if oracle accepted.
        let mut events_to_append: Vec<crate::evidence::ProcessEvent> =
            vec![sa_start, sa_complete, ea_evt];

        if result.success {
            let mut rw_evt = crate::evidence::ProcessEvent::new("receipt:write", "COMPLETE");
            rw_evt.case_id = Some(case_id.clone());
            events_to_append.push(rw_evt);
        }

        // Append all three/four events into the main evidence log so they are
        // included in the next XES re-build.
        if let Err(e) = crate::evidence::append_events(&events_to_append, &evidence_dir) {
            eprintln!("warning: audit evidence emission failed: {}", e);
        }

        if !result.success {
            anyhow::bail!("wasm4pm REFUSED evidence");
        }
        Ok(())
    }
}

impl VerbCommand for StatusAuditVerb {
    fn name(&self) -> &'static str {
        "audit"
    }
    fn about(&self) -> &'static str {
        "Adjudicate current evidence XES file via the wasm4pm oracle"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}
