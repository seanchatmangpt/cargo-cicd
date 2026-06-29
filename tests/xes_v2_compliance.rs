//! XES 2.0 compliance tests for the `evidence_xes_v2` and `evidence_jsonl` modules.
//!
//! These tests validate that the serializers produce output that conforms to
//! the XES 2.0 standard (ISO/IEC 20880:2013) and that companion JSONL files
//! are well-formed and contain required fields.
//!
//! Each test targets a specific structural requirement from the specification.

use cargo_cicd::evidence::ProcessEvent;
use cargo_cicd::evidence_jsonl::{event_to_jsonl_line, write_jsonl};
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

/// Test 2: Root <log> element has xmlns:xes namespace
#[test]
fn xes_v2_log_has_xmlns_namespace() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());
    assert!(
        xml.contains("xmlns:xes=\"http://www.xes-standard.org/\""),
        "Root <log> must carry xmlns:xes namespace; got:\n{}",
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

/// Test 4: <trace> contains workspace_id
#[test]
fn xes_v2_trace_contains_workspace_id() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());
    assert!(
        xml.contains("cargo_cicd:workspace_id"),
        "<trace> must contain cargo_cicd:workspace_id; got:\n{}",
        &xml[..xml.len().min(800)]
    );
}

/// Test 5: <trace> carries all required workspace metadata fields
#[test]
fn xes_v2_trace_contains_all_workspace_meta_fields() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());

    let required = [
        "cargo_cicd:workspace_id",
        "cargo_cicd:workspace_root",
        "cargo_cicd:git_branch",
        "cargo_cicd:git_commit_sha",
        "cargo_cicd:toolchain_version",
        "cargo_cicd:cargo_version",
        "cargo_cicd:os_version",
        "cargo_cicd:session_id",
    ];

    for field in &required {
        assert!(
            xml.contains(field),
            "Trace must contain {}; got:\n{}",
            field,
            &xml[..xml.len().min(1200)]
        );
    }
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

/// Test 7: Start event has lifecycle_transition="start"
#[test]
fn xes_v2_start_event_has_lifecycle_start() {
    let events = vec![make_start_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());
    assert!(
        xml.contains("lifecycle:transition") && xml.contains("\"start\""),
        "Start event must have lifecycle:transition=start; got:\n{}",
        &xml[..xml.len().min(1000)]
    );
}

/// Test 8: Complete event has lifecycle_transition="complete" and verdict_claimed
#[test]
fn xes_v2_complete_event_has_lifecycle_complete_and_verdict() {
    let events = vec![make_complete_event()];
    let xml = to_xes_v2(&events, "compliance_case", &test_meta());
    assert!(
        xml.contains("lifecycle:transition") && xml.contains("\"complete\""),
        "Complete event must have lifecycle:transition=complete"
    );
    assert!(
        xml.contains("cargo_cicd:verdict_claimed") && xml.contains("PASS"),
        "Complete event must have cargo_cicd:verdict_claimed=PASS"
    );
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

/// Test 10: File naming convention uses evt-{case_id}-{timestamp}.xes
#[test]
fn write_xes_v2_file_name_follows_convention() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_complete_event()];
    let path = write_xes_v2_with_meta(&events, "my_case", dir.path(), &test_meta())
        .expect("write_xes_v2 must succeed");

    let filename = path.file_name().unwrap().to_string_lossy();
    assert!(
        filename.starts_with("evt-my_case-"),
        "File must start with 'evt-{{case_id}}-'; got '{}'",
        filename
    );
    assert!(
        filename.ends_with(".xes"),
        "File must end with '.xes'; got '{}'",
        filename
    );
}

// ── JSONL tests ───────────────────────────────────────────────────────────────

/// Test 11: JSONL output is valid JSON (parses without error)
#[test]
fn jsonl_output_is_valid_json() {
    let event = make_complete_event();
    let line = event_to_jsonl_line(&event);

    // Parse with serde_json to verify structural validity.
    let parsed: serde_json::Value =
        serde_json::from_str(&line).expect("JSONL line must be valid JSON");
    assert!(parsed.is_object(), "JSONL line must parse as a JSON object");
}

/// Test 12: JSONL output contains event_id
#[test]
fn jsonl_output_contains_event_id() {
    let event = make_complete_event();
    let line = event_to_jsonl_line(&event);
    assert!(
        line.contains("event_id"),
        "JSONL line must contain 'event_id'; got: {}",
        line
    );
    assert!(
        line.contains("evt-compliance-complete"),
        "JSONL line must contain the event_id value"
    );
}

/// Test 13: JSONL output contains timestamp
#[test]
fn jsonl_output_contains_timestamp() {
    let event = make_complete_event();
    let line = event_to_jsonl_line(&event);
    assert!(
        line.contains("timestamp_iso"),
        "JSONL line must contain 'timestamp_iso'"
    );
    assert!(
        line.contains("2026-06-17"),
        "JSONL line must contain the timestamp value"
    );
}

/// Test 14: JSONL output contains verdict_claimed
#[test]
fn jsonl_output_contains_verdict_claimed() {
    let event = make_complete_event();
    let line = event_to_jsonl_line(&event);
    assert!(
        line.contains("verdict_claimed"),
        "JSONL line must contain 'verdict_claimed'"
    );
    assert!(
        line.contains("PASS"),
        "JSONL line must contain the verdict value"
    );
}

/// Test 15: write_jsonl creates a file in the specified directory
#[test]
fn write_jsonl_creates_file() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_complete_event()];
    let path = write_jsonl(&events, "jsonl_case", dir.path()).expect("write_jsonl must succeed");

    assert!(path.exists(), "JSONL file must exist at {}", path.display());
    let content = std::fs::read_to_string(&path).expect("must read JSONL file");
    assert!(!content.is_empty(), "JSONL file must not be empty");
}

/// Test 16: JSONL file naming follows evt-{case_id}-{timestamp}.jsonl convention
#[test]
fn write_jsonl_file_name_follows_convention() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_complete_event()];
    let path = write_jsonl(&events, "my_jsonl_case", dir.path()).expect("write_jsonl must succeed");

    let filename = path.file_name().unwrap().to_string_lossy();
    assert!(
        filename.starts_with("evt-my_jsonl_case-"),
        "File must start with 'evt-{{case_id}}-'; got '{}'",
        filename
    );
    assert!(
        filename.ends_with(".jsonl"),
        "File must end with '.jsonl'; got '{}'",
        filename
    );
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

/// Test 18: Each JSONL line in a multi-event file is independently valid JSON
#[test]
fn write_jsonl_each_line_is_valid_json() {
    let dir = TempDir::new().unwrap();
    let events = vec![make_start_event(), make_complete_event()];
    let path =
        write_jsonl(&events, "json_lines_case", dir.path()).expect("write_jsonl must succeed");

    let content = std::fs::read_to_string(&path).expect("must read JSONL file");
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(
            parsed.is_ok(),
            "Line {} must be valid JSON; got: {}",
            i + 1,
            line
        );
    }
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

/// Test 20: XesWorkspaceMeta::from_env() returns without panicking
#[test]
fn xes_workspace_meta_from_env_does_not_panic() {
    // This test verifies graceful degradation: even without git, rustc, etc.,
    // from_env() must return a valid (possibly defaulted) struct.
    let meta = XesWorkspaceMeta::from_env();
    assert!(
        !meta.workspace_id.is_empty(),
        "workspace_id must have a non-empty default"
    );
    assert!(!meta.session_id.is_empty(), "session_id must be generated");
}
