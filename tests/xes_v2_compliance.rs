//! XES 2.0 compliance tests for the `evidence_xes_v2` and `evidence_jsonl` modules.
//!
//! These tests validate that the serializers produce output that conforms to
//! the XES 2.0 standard (ISO/IEC 20880:2013) and that companion JSONL files
//! are well-formed and contain required fields.
//!
//! Each test targets a specific structural requirement from the specification.

use cargo_cicd::evidence::ProcessEvent;
use cargo_cicd::evidence_jsonl::write_jsonl;
use cargo_cicd::evidence_xes_v2::{to_xes_v2, write_xes_v2_with_meta, XesWorkspaceMeta};
use tempfile::TempDir;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn make_event(lifecycle: &str, verdict: &str) -> ProcessEvent {
    ProcessEvent {
        event_id: format!("evt-compliance-{}", lifecycle),
        timestamp_iso: "2026-06-17T10:00:00.000Z".to_string(),
        case_id: Some("compliance_case".to_string()),
        lifecycle_transition: lifecycle.to_string(),
        workspace_id: "compliance-workspace".to_string(),
        repo_path: "/repo/test".to_string(),
        command: "status show".to_string(),
        verdict_claimed: verdict.to_string(),
        duration_ms: if lifecycle == "complete" {
            Some(123)
        } else {
            None
        },
        verdict_adjudicated: None,
        adjudicated_at: None,
        oracle_command: None,
        trace_class: "live_workspace".to_string(),
        config_witness: None,
    }
}

fn make_start_event() -> ProcessEvent {
    make_event("start", "")
}

fn make_complete_event() -> ProcessEvent {
    make_event("complete", "PASS")
}

fn test_meta() -> XesWorkspaceMeta {
    XesWorkspaceMeta::for_testing()
}

// ── XES 2.0 structural tests ──────────────────────────────────────────────────

/// Test 1: Root <log> element has xes.version="2.0"
#[test]
fn xes_v2_log_has_version_attribute() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());
    assert!(
        xml.contains("xes.version=\"2.0\""),
        "Root <log> must carry xes.version=\"2.0\"; got:\n{}",
        &xml[..xml.len().min(500)]
    );
}

/// Test 3: <trace> contains case_id as concept:name
#[test]
fn xes_v2_trace_contains_case_id() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "my_test_case", &test_meta());
    assert!(
        xml.contains("concept:name") && xml.contains("my_test_case"),
        "<trace> must contain the case_id as concept:name; got:\n{}",
        &xml[..xml.len().min(800)]
    );
}

/// Test 6: <event> elements contain required attributes
#[test]
fn xes_v2_event_contains_required_attributes() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());

    let required = [
        "cargo_cicd:event_id",
        "concept:name",
        "time:timestamp",
        "lifecycle:transition",
        "cargo_cicd:verdict_claimed",
    ];

    for attr in &required {
        assert!(
            xml.contains(attr),
            "<event> must contain {}; got:\n{}",
            attr,
            &xml[..xml.len().min(1200)]
        );
    }
}

/// Test 9: write_xes_v2 creates a file in the specified directory
#[test]
fn write_xes_v2_creates_file() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_complete_event()];
    let path = write_xes_v2_with_meta(&events, "test_case", dir.path(), &test_meta())
        .expect("write_xes_v2 must succeed");

    assert!(
        path.exists(),
        "XES v2 file must exist at {}",
        path.display()
    );
    let content = std::fs::read_to_string(&path).expect("must be able to read XES file");
    assert!(!content.is_empty(), "XES v2 file must not be empty");
}

/// Test 17: JSONL file with multiple events has one line per event
#[test]
fn write_jsonl_multiple_events_one_per_line() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_start_event(), make_complete_event()];
    let path = write_jsonl(&events, "multi_case", dir.path()).expect("write_jsonl must succeed");

    let content = std::fs::read_to_string(&path).expect("must read JSONL file");
    let non_empty_lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        non_empty_lines.len(),
        2,
        "Two events must produce two non-empty lines; got:\n{}",
        content
    );
}

/// Test 19: XES v2 output produces well-formed XML (has opening and closing tags)
#[test]
fn xes_v2_output_is_well_formed_xml() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "wf_case", &test_meta());

    // Basic well-formedness: starts with XML declaration, has log open/close.
    assert!(
        xml.starts_with("<?xml"),
        "XES output must start with XML declaration"
    );
    assert!(
        xml.contains("<log "),
        "XES output must have an opening <log> tag"
    );
    assert!(
        xml.contains("</log>"),
        "XES output must have a closing </log> tag"
    );
    assert!(
        xml.contains("<trace>"),
        "XES output must have an opening <trace> tag"
    );
    assert!(
        xml.contains("</trace>"),
        "XES output must have a closing </trace> tag"
    );
    assert!(
        xml.contains("<event>"),
        "XES output must have an opening <event> tag"
    );
    assert!(
        xml.contains("</event>"),
        "XES output must have a closing </event> tag"
    );
}
