use crate::evidence::{append_events, evidence_dir, ProcessEvent, WpmEvidenceOracle};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct EvidenceNoun;
impl EvidenceNoun {
    pub fn new() -> Self {
        Self
    }
    pub fn run_direct() -> anyhow::Result<()> {
        EvidenceAuditVerb.execute()
    }
}
impl Default for EvidenceNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for EvidenceNoun {
    fn name(&self) -> &'static str {
        "evidence"
    }
    fn about(&self) -> &'static str {
        "Manage process evidence and wasm4pm adjudication"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(EvidenceAuditVerb)]
    }
}

pub struct EvidenceAuditVerb;

impl EvidenceAuditVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let ev_dir = evidence_dir();
        let xes = ev_dir.join("events.xes");

        if !xes.exists() {
            println!("BLOCKED: no evidence at {}", xes.display());
            println!("  run a cargo cicd command first to emit evidence");
            return Ok(());
        }

        let oracle = WpmEvidenceOracle::new();
        if !oracle.is_available() {
            println!("BLOCKED: wpm oracle not found");
            return Ok(());
        }

        let wpm_shell = crate::integrations::Wasm4pmShell::detect().unwrap();
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

        println!("exit:     {}", if result.success { "0" } else { "non-zero" });
        println!("stdout:   {}", result.stdout.trim());
        if !result.stderr.trim().is_empty() {
            println!("stderr:   {}", result.stderr.trim());
        }
        let oracle_verdict = if result.success { "ACCEPT" } else { "REFUSE" };
        println!("verdict:  {}", oracle_verdict);

        let case_id = crate::session::read_or_create_session_id(&ev_dir);
        let mut evt = ProcessEvent::new_adjudicated(
            "evidence:audit",
            oracle_verdict,
            wpm_shell.binary_path(),
        );
        evt.case_id = Some(case_id);
        if let Err(e) = append_events(&[evt], &ev_dir) {
            eprintln!("warning: audit evidence emission failed: {}", e);
        }

        if !result.success {
            anyhow::bail!("wasm4pm REFUSED evidence");
        }
        Ok(())
    }
}

impl VerbCommand for EvidenceAuditVerb {
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
