use std::path::PathBuf;

use crate::evidence::{
    emit_events_jsonl, emit_receipt_json, evidence_dir, ProcessEvent, ReceiptDoctor,
    ReceiptDoctorVerdict,
};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct EvidenceNoun;

impl EvidenceNoun {
    pub fn new() -> Self {
        Self
    }

    pub fn run_direct() -> anyhow::Result<()> {
        // Dispatch directly to the doctor verb without going through clap
        // subcommand parsing — the bare-noun path has no subcommand context.
        let matches = clap::Command::new("evidence").get_matches_from(vec!["evidence"]);
        let args = VerbArgs::new(matches);
        DoctorVerb.run(&args).map_err(|e| anyhow::anyhow!("{}", e))
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
        "Adjudicate runtime process evidence via wasm4pm receipt doctor"
    }

    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(DoctorVerb), Box::new(AuditVerb)]
    }
}

pub struct DoctorVerb;

impl VerbCommand for DoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }

    fn about(&self) -> &'static str {
        "Run wpm receipt doctor --format json --strict on the latest process receipt"
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let receipt_path = PathBuf::from("target/cargo-cicd/evidence/receipts/latest.json");

        // Locate wpm oracle.
        let doctor = match ReceiptDoctor::discover() {
            None => {
                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                    "BLOCKED: wpm binary not found — set WPM_PATH env var or install wasm4pm"
                        .to_string(),
                ));
            }
            Some(d) => d,
        };

        // Ensure a receipt exists; seed one if the directory is empty.
        if !receipt_path.exists() {
            let sentinel = ProcessEvent::new("evidence:doctor:init", "PASS");
            if let Err(e) = emit_receipt_json(&[&sentinel], "cargo cicd evidence doctor", 0) {
                eprintln!("warning: receipt emission failed: {e}");
            }
        }

        println!("  adjudicating: {}", receipt_path.display());

        let verdict = doctor.doctor_strict_json(&receipt_path);

        // Emit the adjudication outcome as a process event.
        let (verdict_str, oracle_path) = match &verdict {
            ReceiptDoctorVerdict::Accepted { .. } => ("ACCEPT", doctor.binary_path().to_string()),
            ReceiptDoctorVerdict::Refused { .. } => ("REFUSE", doctor.binary_path().to_string()),
            ReceiptDoctorVerdict::Blocked { .. } => ("BLOCKED", String::new()),
        };
        let adj_event = ProcessEvent::new_adjudicated("evidence:doctor", verdict_str, &oracle_path);
        let jsonl_path = evidence_dir().join("events.jsonl");
        if let Err(e) = emit_events_jsonl(&[adj_event], &jsonl_path) {
            eprintln!("warning: evidence emission failed: {e}");
        }

        match verdict {
            ReceiptDoctorVerdict::Accepted { stdout_json } => {
                println!("{}", stdout_json);
                println!("  verdict: ACCEPTED");
                Ok(())
            }
            ReceiptDoctorVerdict::Refused {
                exit_code,
                stdout,
                stderr,
            } => {
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                if !stderr.is_empty() {
                    eprintln!("{}", stderr);
                }
                Err(clap_noun_verb::error::NounVerbError::execution_error(
                    format!(
                        "AndonPull: receipt doctor refused admission (exit {})",
                        exit_code
                    ),
                ))
            }
            ReceiptDoctorVerdict::Blocked { reason } => Err(
                clap_noun_verb::error::NounVerbError::execution_error(format!("BLOCKED: {reason}")),
            ),
        }
    }
}

/// `audit` is the canonical public-facing verb for evidence adjudication.
/// It delegates to the same implementation as `doctor`.
pub struct AuditVerb;

impl VerbCommand for AuditVerb {
    fn name(&self) -> &'static str {
        "audit"
    }

    fn about(&self) -> &'static str {
        "Audit process evidence receipts (alias for doctor — canonical public-facing verb)"
    }

    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        DoctorVerb.run(args)
    }
}
