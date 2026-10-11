//! OCEL 2.0 conformance court.
//!
//! Runs the real `cargo-cicd` binary as a subprocess (`ocel export`), then
//! validates the emitted JSON against the vendored OCEL 2.0 schema
//! (`tests/fixtures/ocel20.schema.json`, transcribed from
//! <https://ocel-standard.org/specification/formats/json/>) using the real
//! `jsonschema` validator. Chicago-style: real binary, real file I/O, real
//! schema validation — no stubs.
#![allow(
    warnings,
    clippy::all,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use assert_cmd::Command;
use std::path::Path;

/// Seed a real ProcessEvent journal the same way the evidence pipeline writes
/// it (JSON lines of `ProcessEvent` under `.cargo-cicd/evidence/events.jsonl`).
fn seed_journal(evidence_dir: &Path) {
    std::fs::create_dir_all(evidence_dir).unwrap();
    let events = serde_json::json!([
        {
            "event_id": "evt-conf-001",
            "timestamp_iso": "2026-10-10T12:00:00.000Z",
            "case_id": "conformance_case",
            "lifecycle_transition": "complete",
            "workspace_id": "ws:conformance",
            "repo_path": "/tmp/conformance-repo",
            "command": "status:show",
            "verdict_claimed": "PASS",
            "trace_class": "pipeline_run"
        },
        {
            "event_id": "evt-conf-002",
            "timestamp_iso": "2026-10-10T12:00:05.000Z",
            "case_id": "conformance_case",
            "lifecycle_transition": "complete",
            "workspace_id": "ws:conformance",
            "repo_path": "/tmp/conformance-repo",
            "command": "test:changed",
            "verdict_claimed": "PASS",
            "duration_ms": 4211,
            "trace_class": "pipeline_run"
        },
        {
            "event_id": "evt-conf-003",
            "timestamp_iso": "2026-10-10T12:00:09.000Z",
            "case_id": "conformance_case",
            "lifecycle_transition": "start",
            "workspace_id": "ws:conformance",
            "repo_path": "/tmp/conformance-repo",
            "command": "receipt:write",
            "verdict_claimed": "WARN",
            "trace_class": "live_workspace"
        }
    ]);
    let mut body = String::new();
    for ev in events.as_array().unwrap() {
        body.push_str(&serde_json::to_string(ev).unwrap());
        body.push('\n');
    }
    std::fs::write(evidence_dir.join("events.jsonl"), body).unwrap();
}

#[test]
fn ocel_export_output_conforms_to_ocel20_schema() {
    let tmp = tempfile::TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    let evidence_dir = repo.join(".cargo-cicd").join("evidence");
    seed_journal(&evidence_dir);

    let out_path = tmp.path().join("out").join("events.ocel.json");

    // Act: real subprocess export.
    let mut cmd = Command::cargo_bin("cargo-cicd").unwrap();
    let assert = cmd
        .args([
            "ocel",
            "export",
            "--repo",
            repo.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.contains("\"events_exported\":3"),
        "export summary must report 3 events, got: {stdout}"
    );

    // Assert: validate against the vendored OCEL 2.0 schema with the real
    // jsonschema validator.
    let schema_val: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/ocel20.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::validator_for(&schema_val)
        .expect("vendored OCEL 2.0 schema must itself be a valid JSON Schema");

    let log: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();

    let errors: Vec<String> = validator
        .iter_errors(&log)
        .map(|e| format!("{} at {}", e, e.instance_path))
        .collect();
    assert!(
        errors.is_empty(),
        "exported OCEL log must conform to the OCEL 2.0 schema; violations: {errors:#?}"
    );

    // Structural spec assertions beyond the schema floor.
    assert_eq!(log["events"].as_array().unwrap().len(), 3);
    let object_types: std::collections::BTreeSet<String> = log["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["type"].as_str().map(String::from))
        .collect();
    assert!(
        object_types.len() >= 3,
        "expected multi-type objects (workspace/pipeline/policy), got {object_types:?}"
    );
    let event_type_names: Vec<&str> = log["eventTypes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(event_type_names.contains(&"status:show"));
    assert!(event_type_names.contains(&"test:changed"));
    // Every event relationship objectId must resolve to an existing object.
    let object_ids: std::collections::BTreeSet<&str> = log["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["id"].as_str())
        .collect();
    for ev in log["events"].as_array().unwrap() {
        for rel in ev["relationships"].as_array().unwrap() {
            let oid = rel["objectId"].as_str().unwrap();
            assert!(
                object_ids.contains(oid),
                "relationship target {oid} missing from objects"
            );
        }
    }
}

#[test]
fn ocel_export_with_empty_journal_still_conforms() {
    let tmp = tempfile::TempDir::new().unwrap();
    let repo = tmp.path().join("empty-repo");
    std::fs::create_dir_all(repo.join(".cargo-cicd").join("evidence")).unwrap();
    let out_path = tmp.path().join("out").join("empty.ocel.json");

    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args([
            "ocel",
            "export",
            "--repo",
            repo.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let schema_val: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/ocel20.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::validator_for(&schema_val).unwrap();
    let log: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
    assert!(
        validator.is_valid(&log),
        "empty-journal export must still be a valid OCEL 2.0 log"
    );
}
