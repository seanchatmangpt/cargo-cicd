use std::path::PathBuf;

pub use crate::evidence::{
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
        vec![
            Box::new(DoctorVerb),
            Box::new(AuditVerb),
            Box::new(EvidenceShowVerb),
            Box::new(EvidenceListVerb),
            Box::new(EvidenceResetVerb),
        ]
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

    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name()).about(self.about()).arg(
            clap::Arg::new("json")
                .long("json")
                .action(clap::ArgAction::SetTrue)
                .help("Output machine-readable JSON"),
        )
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let json_mode = std::env::args().any(|a| a == "--json");
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

        if !json_mode {
            println!("  adjudicating: {}", receipt_path.display());
        }

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
                if json_mode {
                    println!("{}", stdout_json);
                } else {
                    println!("{}", stdout_json);
                    println!("  verdict: ACCEPTED");
                }
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

pub struct EvidenceShowVerb;

impl VerbCommand for EvidenceShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }

    fn about(&self) -> &'static str {
        "Show a summary of the current process evidence"
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let jsonl_path = evidence_dir.join("events.jsonl");

        if !jsonl_path.exists() {
            println!("no evidence found — run 'cargo cicd pipeline run' first");
            return Ok(());
        }

        let content = std::fs::read_to_string(&jsonl_path)
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        println!("process evidence");
        println!("================");
        let mut count = 0;
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let event_type = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                let verdict = val
                    .get("verdict_claimed")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let ts = val
                    .get("timestamp_iso")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                println!("  [{:>3}] {} → {} ({})", count + 1, event_type, verdict, ts);
                count += 1;
            }
        }
        println!();
        println!("total events: {}", count);

        let show_event = crate::evidence::ProcessEvent::new("evidence:show", "PASS");
        let _ = crate::evidence::emit_events_jsonl(&[show_event], &jsonl_path);

        Ok(())
    }
}

pub struct EvidenceListVerb;

impl VerbCommand for EvidenceListVerb {
    fn name(&self) -> &'static str {
        "list"
    }

    fn about(&self) -> &'static str {
        "List evidence files in the evidence directory"
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        println!("evidence directory: {}", evidence_dir.display());
        println!();
        if !evidence_dir.exists() {
            println!("  (no evidence directory — run 'cargo cicd pipeline run' first)");
            return Ok(());
        }

        fn walk_dir(path: &std::path::Path, indent: usize) {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut sorted: Vec<_> = entries.flatten().collect();
                sorted.sort_by_key(|e| e.file_name());
                for entry in sorted {
                    let meta = entry.metadata().ok();
                    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let spaces = " ".repeat(indent * 2);
                    if meta.map(|m| m.is_dir()).unwrap_or(false) {
                        println!("{}{}/", spaces, entry.file_name().to_string_lossy());
                        walk_dir(&entry.path(), indent + 1);
                    } else {
                        println!(
                            "{}{}  ({} bytes)",
                            spaces,
                            entry.file_name().to_string_lossy(),
                            size
                        );
                    }
                }
            }
        }

        walk_dir(&evidence_dir, 1);
        Ok(())
    }
}

pub struct EvidenceResetVerb;

impl VerbCommand for EvidenceResetVerb {
    fn name(&self) -> &'static str {
        "reset"
    }

    fn about(&self) -> &'static str {
        "Clear process events for a fresh pipeline run (preserves receipts)"
    }

    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();

        // Emit reset event before clearing so it is recorded in the outgoing session.
        let reset_event = crate::evidence::ProcessEvent::new("evidence:reset", "PASS");
        let jsonl_path = evidence_dir.join("events.jsonl");
        let _ = crate::evidence::emit_events_jsonl(&[reset_event], &jsonl_path);

        // Remove the mutable evidence files; leave receipts/ intact (permanent records).
        let _ = std::fs::remove_file(evidence_dir.join("events.jsonl"));
        let _ = std::fs::remove_file(evidence_dir.join("events.xes"));
        let _ = std::fs::remove_file(evidence_dir.join("events.ocel.json"));
        let session_file = evidence_dir.join(".session");
        let _ = std::fs::remove_file(&session_file);

        let new_id = crate::session::read_or_create_session_id(&evidence_dir);
        println!("evidence cleared");
        println!("new session: {}", new_id);
        Ok(())
    }
}
