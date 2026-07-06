//! `cargo cicd affidavit` — seal and certify the process-evidence journal with
//! the **affidavit** cryptographic provenance engine via its `affi` CLI.
//!
//! - `affidavit seal`   replay the accumulated evidence journal into affidavit
//!                      (`affi receipt emit` per event) and seal it into a
//!                      content-addressed BLAKE3 receipt (`affi receipt assemble`).
//! - `affidavit verify` certify the sealed receipt (`affi receipt verify`) and
//!                      report ACCEPT / REJECT.
//!
//! affidavit is invoked as an external binary — never linked — so this works on
//! the stable toolchain and degrades gracefully when `affi` is not installed
//! (the verbs report the gap and claim `WARN`, never crashing the workspace).
//! cargo-cicd never grades the receipt; affidavit does (invariant **E1**).

use std::path::Path;

use crate::integrations::affidavit_shell::{
    affidavit_receipt_dir, event_type_for, object_ref_for, AffidavitShell, AffidavitVerdict,
};
use crate::legacy_nouns::evidence_helpers::{finish_evidence, init_evidence};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct AffidavitNoun;

impl AffidavitNoun {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AffidavitNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for AffidavitNoun {
    fn name(&self) -> &'static str {
        "affidavit"
    }
    fn about(&self) -> &'static str {
        "Seal and certify the process-evidence journal as a cryptographic receipt"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(AffidavitSealVerb), Box::new(AffidavitVerifyVerb)]
    }
}

/// Message shown when the `affi` binary is not on the system.
fn affi_missing_hint() -> &'static str {
    "affi binary not found — install affidavit (https://github.com/seanchatmangpt/affidavit) \
     or set AFFI_PATH"
}

// ---------------------------------------------------------------------------
// affidavit seal — emit every journal event, then assemble a BLAKE3 receipt
// ---------------------------------------------------------------------------

pub struct AffidavitSealVerb;

impl VerbCommand for AffidavitSealVerb {
    fn name(&self) -> &'static str {
        "seal"
    }
    fn about(&self) -> &'static str {
        "Replay the evidence journal into affidavit and seal a BLAKE3 receipt"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("affidavit:seal");
        let receipt_dir = affidavit_receipt_dir(&evidence_dir);

        let verdict = match AffidavitShell::detect() {
            None => {
                println!("affidavit seal: {}", affi_missing_hint());
                "WARN"
            }
            Some(shell) => seal_with(&shell, &receipt_dir, &evidence_dir),
        };

        finish_evidence(
            start_evt,
            t0,
            case_id,
            verdict,
            "affidavit:seal",
            &evidence_dir,
        );
        Ok(())
    }
}

/// Emit each journaled [`ProcessEvent`](crate::evidence::ProcessEvent) into
/// affidavit and seal a receipt. Returns the claimed verdict string.
fn seal_with(shell: &AffidavitShell, receipt_dir: &Path, evidence_dir: &Path) -> &'static str {
    if let Err(e) = std::fs::create_dir_all(receipt_dir) {
        eprintln!(
            "affidavit seal: cannot create {}: {e}",
            receipt_dir.display()
        );
        return "FAIL";
    }

    let events = crate::evidence::read_journal(evidence_dir);
    println!("affidavit seal");
    println!("  affi            : {}", shell.binary_path());
    println!("  events to emit  : {}", events.len());

    for (i, ev) in events.iter().enumerate() {
        let payload = receipt_dir.join(format!("payload-{i}.json"));
        let body = serde_json::to_vec(ev).unwrap_or_else(|_| ev.event_id.clone().into_bytes());
        if let Err(e) = std::fs::write(&payload, &body) {
            eprintln!("  payload write failed at event {i}: {e}");
            return "FAIL";
        }
        let etype = event_type_for(&ev.command, &ev.lifecycle_transition);
        let object = object_ref_for(ev);
        match shell.emit(receipt_dir, &etype, &object, &payload) {
            Ok(r) if r.success => {}
            Ok(r) => {
                eprintln!("  affi emit rejected event {i}: {}", r.stderr.trim());
                return "FAIL";
            }
            Err(e) => {
                eprintln!("  affi emit errored at event {i}: {e}");
                return "FAIL";
            }
        }
    }

    let out = receipt_dir.join("receipt.json");
    match shell.assemble(receipt_dir, &out) {
        Ok(r) if r.success => {
            println!("  receipt         : {}", out.display());
            println!("  sealed          : OK  (run `cargo cicd affidavit verify` to certify)");
            "PASS"
        }
        Ok(r) => {
            eprintln!("  affi assemble failed: {}", r.stderr.trim());
            "FAIL"
        }
        Err(e) => {
            eprintln!("  affi assemble errored: {e}");
            "FAIL"
        }
    }
}

// ---------------------------------------------------------------------------
// affidavit verify — certify the sealed receipt (exit 0 = ACCEPT)
// ---------------------------------------------------------------------------

pub struct AffidavitVerifyVerb;

impl VerbCommand for AffidavitVerifyVerb {
    fn name(&self) -> &'static str {
        "verify"
    }
    fn about(&self) -> &'static str {
        "Certify the sealed receipt through affidavit's pipeline (ACCEPT/REJECT)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("affidavit:verify");
        let receipt_dir = affidavit_receipt_dir(&evidence_dir);
        let receipt = receipt_dir.join("receipt.json");

        let verdict = match AffidavitShell::detect() {
            None => {
                println!("affidavit verify: {}", affi_missing_hint());
                "WARN"
            }
            Some(_) if !receipt.exists() => {
                println!(
                    "affidavit verify: no receipt at {} — run `cargo cicd affidavit seal` first",
                    receipt.display()
                );
                "WARN"
            }
            Some(shell) => match shell.verify(&receipt) {
                Ok(r) => {
                    println!("affidavit verify");
                    println!("  receipt         : {}", receipt.display());
                    if !r.stdout.trim().is_empty() {
                        for line in r.stdout.lines() {
                            println!("  | {line}");
                        }
                    }
                    println!("  verdict         : {}", r.verdict);
                    match r.verdict {
                        AffidavitVerdict::Accept => "PASS",
                        _ => "FAIL",
                    }
                }
                Err(e) => {
                    eprintln!("affidavit verify: affi errored: {e}");
                    "FAIL"
                }
            },
        };

        finish_evidence(
            start_evt,
            t0,
            case_id,
            verdict,
            "affidavit:verify",
            &evidence_dir,
        );
        Ok(())
    }
}
