//! src/nouns/evidence.rs — cargo cicd evidence verbs (inlined from the retired legacy layer; see CCICD-106)

use std::path::PathBuf;

pub use crate::evidence::{
    emit_events_jsonl, evidence_dir, ProcessEvent, ReceiptDoctor,
    ReceiptDoctorVerdict,
};

pub struct EvidenceNoun;

impl EvidenceNoun {
    /// Direct doctor dispatch for `doctor evidence` — typed json flag,
    /// no clap re-parse of process args.
    pub fn evidence_doctor_direct(json: bool) -> anyhow::Result<()> {
        DoctorVerb.execute(json).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

pub struct DoctorVerb;

impl DoctorVerb {

    pub fn execute(&self, json: bool) -> clap_noun_verb::error::Result<()> {
        let json_mode = json;
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

        // Missing evidence is a typed BLOCKED, never a minted sentinel
        // receipt — self-attestation is not evidence.
        if !receipt_path.exists() {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                format!(
                    "BLOCKED: no process receipt at {} — run 'cargo cicd pipeline run' first",
                    receipt_path.display()
                ),
            ));
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

impl AuditVerb {

    pub fn execute(json: bool) -> clap_noun_verb::error::Result<()> {
        DoctorVerb.execute(json)
    }
}

pub struct EvidenceShowVerb;

impl EvidenceShowVerb {

    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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

impl EvidenceListVerb {

    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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

impl EvidenceResetVerb {

    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();

        // Emit reset event before clearing so it is recorded in the outgoing session.
        let reset_event = crate::evidence::ProcessEvent::new("evidence:reset", "PASS");
        let jsonl_path = evidence_dir.join("events.jsonl");
        let _ = crate::evidence::emit_events_jsonl(&[reset_event], &jsonl_path);

        // Remove the mutable evidence files (OCEL only — no XES); leave
        // receipts/ intact (permanent records).
        let _ = std::fs::remove_file(evidence_dir.join("events.jsonl"));
        let _ = std::fs::remove_file(evidence_dir.join("events.ocel.json"));
        let session_file = evidence_dir.join(".session");
        let _ = std::fs::remove_file(&session_file);

        let new_id = crate::session::read_or_create_session_id(&evidence_dir);
        println!("evidence cleared");
        println!("new session: {}", new_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fail-closed: with no recorded receipt, doctor must NOT mint a
    /// sentinel PASS receipt — missing evidence stays missing on disk.
    #[test]
    fn doctor_without_evidence_never_creates_a_receipt() {
        let dir = tempfile::TempDir::new().unwrap();
        let prev_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = DoctorVerb.execute(false);
        let _ = std::env::set_current_dir(prev_cwd);

        // Either way it must be a typed error, never a minted receipt.
        assert!(result.is_err(), "missing evidence must be an error, not PASS");
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("BLOCKED"), "error must be typed BLOCKED, got: {msg}");
        assert!(
            !dir.path().join("target/cargo-cicd/evidence/receipts/latest.json").exists(),
            "no receipt may be created for missing evidence"
        );
    }
}

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("doctor")]
pub fn cmd_doctor(json: bool) -> Result<()> {
    DoctorVerb.execute(json)
}

#[verb("audit")]
pub fn cmd_audit(json: bool) -> Result<()> {
    AuditVerb::execute(json)
}

#[verb("show")]
pub fn cmd_show() -> Result<()> {
    EvidenceShowVerb.execute()
}

#[verb("list")]
pub fn cmd_list() -> Result<()> {
    EvidenceListVerb.execute()
}

#[verb("reset")]
pub fn cmd_reset() -> Result<()> {
    EvidenceResetVerb.execute()
}
