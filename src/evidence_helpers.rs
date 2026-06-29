//! Evidence lifecycle helpers — init/finish wrappers used by noun-verb handlers.

use crate::evidence::ProcessEvent;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn init_evidence(noun_verb: &str) -> (PathBuf, String, ProcessEvent, Instant) {
    let evidence_dir = crate::evidence::evidence_dir();
    let case_id = crate::session::read_or_create_session_id(&evidence_dir);
    let (mut start_evt, t0) = ProcessEvent::started(noun_verb);
    start_evt.case_id = Some(case_id.clone());
    (evidence_dir, case_id, start_evt, t0)
}

pub fn finish_evidence(
    start_evt: ProcessEvent,
    t0: Instant,
    case_id: String,
    verdict: &str,
    noun_verb: &str,
    evidence_dir: &Path,
) {
    let mut complete_evt = ProcessEvent::completed(noun_verb, t0, verdict);
    complete_evt.case_id = Some(case_id);
    if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], evidence_dir) {
        eprintln!("warning: evidence emission failed: {}", e);
    }
}
