//! Tutorial anchor for `docs/tutorials/02-ocel-evidence.md`.
//!
//! Run:
//!   cargo run --example 02_ocel_evidence --features process-data
//!
//! What you will see: an OCEL 2.0 evidence file written to
//! `target/cargo-cicd/evidence/events.ocel.json` and its absolute path printed.
//! Oracle verification (`wpm`) is a separate, optional step.

use cargo_cicd::evidence::{emit_ocel, evidence_dir, now_iso8601, ProcessEvent};

fn main() {
    // Build one process event representing a successful `status show` run.
    let mut event = ProcessEvent::new("status show", "PASS");
    event.case_id = Some("tutorial_ocel_evidence".to_string());

    let events = vec![event];

    // Emit to the standard evidence directory.
    let dir = evidence_dir();
    std::fs::create_dir_all(&dir).expect("creates evidence dir");
    let out = dir.join("events.ocel.json");

    emit_ocel(&events, &out).expect("emit OCEL 2.0 evidence file");

    println!("OCEL 2.0 evidence written to:");
    println!("  {}", out.display());
    println!();
    println!("To verify with the oracle (if wpm is on PATH):");
    println!("  wpm receipt verify-ocel2 {}", out.display());
    println!();
    println!("Without wpm, the verdict is: Blocked (expected — oracle is optional)");
    println!("timestamp: {}", now_iso8601());
}
