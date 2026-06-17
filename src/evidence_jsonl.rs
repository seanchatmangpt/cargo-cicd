//! JSONL companion format for cargo-cicd process evidence.
//!
//! Each line is a complete JSON object representing a single `ProcessEvent`.
//! The JSONL format mirrors the XES emission — same event set, machine-readable
//! companion for streaming parsers and downstream tooling.
//!
//! ## Invariant E6
//!
//! JSONL emission mirrors XES: same events, same fields, machine-readable.
//! This module fulfils E6 for the XES 2.0 evidence path.
//!
//! ## File naming
//!
//! Files are written as `evt-{case_id}-{timestamp}.jsonl` to match the XES v2
//! naming convention established by `evidence_xes_v2`.

use crate::evidence::{now_iso8601, ProcessEvent};
use std::io;
use std::path::{Path, PathBuf};

// ── JSON helpers (std-only, no serde_json in the hot path) ───────────────────

/// Escape a string for embedding inside a JSON double-quoted value.
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Produce a JSON string literal including the surrounding quotes.
fn json_str(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

// ── Core serialisation ────────────────────────────────────────────────────────

/// Serialize a `ProcessEvent` to a single JSONL line (no trailing newline).
///
/// All fields present on `ProcessEvent` are included. Optional fields are
/// omitted entirely when `None` rather than serialised as `null`, keeping the
/// output compact and compatible with streaming JSON parsers that skip unknown
/// keys.
pub fn event_to_jsonl_line(event: &ProcessEvent) -> String {
    let mut fields: Vec<String> = Vec::new();

    // Required fields — always present.
    fields.push(format!("{}:{}", json_str("event_id"), json_str(&event.event_id)));
    fields.push(format!("{}:{}", json_str("timestamp_iso"), json_str(&event.timestamp_iso)));
    fields.push(format!(
        "{}:{}",
        json_str("lifecycle_transition"),
        json_str(&event.lifecycle_transition)
    ));
    fields.push(format!("{}:{}", json_str("command"), json_str(&event.command)));
    fields.push(format!(
        "{}:{}",
        json_str("verdict_claimed"),
        json_str(&event.verdict_claimed)
    ));
    fields.push(format!(
        "{}:{}",
        json_str("workspace_id"),
        json_str(&event.workspace_id)
    ));
    fields.push(format!("{}:{}", json_str("repo_path"), json_str(&event.repo_path)));
    fields.push(format!("{}:{}", json_str("trace_class"), json_str(&event.trace_class)));

    // Optional fields — omitted when None.
    if let Some(ref case_id) = event.case_id {
        fields.push(format!("{}:{}", json_str("case_id"), json_str(case_id)));
    }
    if let Some(ms) = event.duration_ms {
        fields.push(format!("{}:{}", json_str("duration_ms"), ms));
    }
    if let Some(ref v) = event.verdict_adjudicated {
        fields.push(format!("{}:{}", json_str("verdict_adjudicated"), json_str(v)));
    }
    if let Some(ref ts) = event.adjudicated_at {
        fields.push(format!("{}:{}", json_str("adjudicated_at"), json_str(ts)));
    }
    if let Some(ref oracle) = event.oracle_command {
        fields.push(format!("{}:{}", json_str("oracle_command"), json_str(oracle)));
    }

    format!("{{{}}}", fields.join(","))
}

/// Write a slice of `ProcessEvent`s as JSONL to `evidence_dir`.
///
/// The file is named `evt-{case_id}-{timestamp}.jsonl` following the cargo-cicd
/// evidence naming convention. The directory is created if absent.
///
/// Returns the path to the written file.
pub fn write_jsonl(
    events: &[ProcessEvent],
    case_id: &str,
    evidence_dir: &Path,
) -> io::Result<PathBuf> {
    std::fs::create_dir_all(evidence_dir)?;

    let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
    let safe_case_id = case_id.replace(['/', '\\', ':', ' '], "_");
    let filename = format!("evt-{}-{}.jsonl", safe_case_id, ts);
    let path = evidence_dir.join(filename);

    let mut lines = String::new();
    for event in events {
        lines.push_str(&event_to_jsonl_line(event));
        lines.push('\n');
    }

    std::fs::write(&path, lines)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::ProcessEvent;

    fn sample_event() -> ProcessEvent {
        ProcessEvent {
            event_id: "evt-jsonl-001".to_string(),
            timestamp_iso: "2026-06-17T12:00:00.000Z".to_string(),
            case_id: Some("jsonl_case".to_string()),
            lifecycle_transition: "complete".to_string(),
            workspace_id: "test-workspace".to_string(),
            repo_path: "/repo".to_string(),
            command: "status show".to_string(),
            verdict_claimed: "PASS".to_string(),
            duration_ms: Some(99),
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
        }
    }

    #[test]
    fn jsonl_line_contains_event_id() {
        let line = event_to_jsonl_line(&sample_event());
        assert!(line.contains("event_id"), "JSONL line must contain event_id");
        assert!(line.contains("evt-jsonl-001"), "event_id value must be present");
    }

    #[test]
    fn jsonl_line_contains_timestamp() {
        let line = event_to_jsonl_line(&sample_event());
        assert!(line.contains("timestamp_iso"), "must contain timestamp_iso");
        assert!(line.contains("2026-06-17"), "timestamp value must be present");
    }

    #[test]
    fn jsonl_line_contains_verdict_claimed() {
        let line = event_to_jsonl_line(&sample_event());
        assert!(line.contains("verdict_claimed"), "must contain verdict_claimed");
        assert!(line.contains("PASS"), "verdict value must be present");
    }

    #[test]
    fn jsonl_line_is_valid_json_object() {
        let line = event_to_jsonl_line(&sample_event());
        assert!(line.starts_with('{'), "JSONL line must start with '{'");
        assert!(line.ends_with('}'), "JSONL line must end with '}'");
    }
}
