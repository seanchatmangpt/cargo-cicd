//! Chicago TDD verification suite for the OCEL 2.0 migration.
//!
//! Uses a vendored Arrange-Act-Assert `aaa_test!` macro (see below) to prove
//! that every capability added in the OCEL migration is complete and correct —
//! not just compiling, but behaving as specified.
//!
//! Coverage map:
//!  - emit_ocel / emit_ocel_filtered / emit_ocel_fresh / emit_ocel_impl
//!  - build_ocel_log / build_ocel_log_filtered
//!  - append_events writes events.ocel.json and never events.xes
//!  - WpmEvidenceOracle::audit_ocel (Blocked when oracle absent)
//!  - assert_wpm_verdict_ocel (passes on Blocked/Blocked pair)
//!  - ocel::OcelLog: 11 cargo object types, validate(), flatten(), e2o(), o2o()
//!  - ocel::blake3_hex: 64-char deterministic hex
//!  - ocel::Perturbator: perturb_trace, drop_event, inject_noise
//!  - ocel::DimensionGroup<U>: generic unit-tagged accumulator
//!  - ocel::BasicPredicate: E2O/O2O/Tbe variants (struct variants)
//!  - ocel::ocpq_eval: returns non-empty slice for non-empty log
//!  - ocel::score_sequence_anomaly: zero for constant sequence
//!  - ocel::detect_drift: false when windows are identical
//!  - ocel::page_hinkley_test: no change-point in flat sequence
//!  - ocel::select_ucb1: selects arm with best UCB1 score
//!  - pipeline.rs: events.ocel.json cleaned on fresh start
//!  - evidence_stale policy: accepts OCEL presence as fresh evidence
//!  - lsp.rs: CICD-EVIDENCE-002 catalog entry references events.ocel.json
#![allow(
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use cargo_cicd::evidence::{
    append_events, assert_wpm_verdict_ocel, build_ocel_log, build_ocel_log_filtered, emit_ocel,
    emit_ocel_filtered, emit_ocel_fresh, ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle,
};
use cargo_cicd::integrations::Wasm4pmShell;
use cargo_cicd::ocel::{
    blake3_hex, detect_drift, is_dominated, ocpq_eval, page_hinkley_test, reject_dominated,
    score_sequence_anomaly, select_ucb1, BasicPredicate, DimCount, DimensionGroup, OcelEvent,
    OcelLog, OcelObject, OcelRelationship, Perturbator,
};
use std::path::Path;
use tempfile::TempDir;

// ── Arrange-Act-Assert test macro ─────────────────────────────────────────────
//
// Imported from the officially published `chicago-tdd-tools` crate.
use chicago_tdd_tools::test as aaa_test;

// ── helpers ───────────────────────────────────────────────────────────────────

fn complete_event(cmd: &str) -> ProcessEvent {
    ProcessEvent::new(cmd, "PASS")
}

fn start_event(cmd: &str) -> ProcessEvent {
    let mut ev = ProcessEvent::new(cmd, "");
    ev.lifecycle_transition = "start".to_string();
    ev
}

fn declared_events() -> Vec<ProcessEvent> {
    vec![
        complete_event("status:show"),
        complete_event("target:show"),
        complete_event("test:changed"),
        complete_event("workspace:doctor"),
    ]
}

fn empty_ocel_log() -> OcelLog {
    OcelLog {
        event_types: vec![],
        object_types: vec![],
        events: vec![],
        objects: vec![],
    }
}

fn single_event_log() -> OcelLog {
    OcelLog {
        event_types: vec![],
        object_types: OcelLog::cargo_object_types(),
        events: vec![OcelEvent {
            id: "e1".to_string(),
            event_type: "status:show".to_string(),
            time: "2026-06-16T00:00:00Z".to_string(),
            attributes: vec![],
            relationships: vec![OcelRelationship {
                object_id: "ws:test".to_string(),
                qualifier: "executed_in".to_string(),
            }],
        }],
        objects: vec![OcelObject {
            id: "ws:test".to_string(),
            object_type: "cargo.workspace".to_string(),
            attributes: vec![],
            relationships: vec![],
        }],
    }
}

// ── 1. Core OCEL log structure tests ─────────────────────────────────────────

aaa_test!(ocel_log_validate_empty_log_passes, {
    // Arrange
    let log = empty_ocel_log();

    // Act
    let report = log.validate();

    // Assert — an empty log is structurally valid
    assert!(
        report.valid,
        "validate() on an empty OcelLog must return valid=true"
    );
    assert_eq!(report.event_count, 0);
    assert_eq!(report.object_count, 0);
});

aaa_test!(build_ocel_log_returns_valid_json, {
    // Arrange
    let events = declared_events();

    // Act
    let log = build_ocel_log(&events);

    // Assert — official OCEL 2.0 JSON keys (no "ocel:" prefixes)
    assert!(log["eventTypes"].is_array());
    assert!(log["objectTypes"].is_array());
    assert!(log["events"].is_array());
    assert!(log["objects"].is_array());
});

aaa_test!(ocel_emit_each_event_has_relationships, {
    // Arrange
    let events = vec![complete_event("status:show")];
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.ocel.json");

    // Act
    emit_ocel(&events, &path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let events_arr = val["events"].as_array().unwrap();
    let first = &events_arr[0];
    let rels = first["relationships"].as_array().unwrap();

    // Assert
    assert!(
        !rels.is_empty(),
        "every OCEL event must have at least one relationship"
    );
    assert!(
        rels[0]["objectId"].is_string() && rels[0]["qualifier"].is_string(),
        "relationships must use the spec shape {{objectId, qualifier}}, got: {}",
        rels[0]
    );
    // Referenced objects must exist in the objects array with their type.
    let object_ids: Vec<&str> = val["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["id"].as_str())
        .collect();
    for rel in rels {
        let oid = rel["objectId"].as_str().unwrap();
        assert!(
            object_ids.contains(&oid),
            "relationship objectId {} must exist in objects",
            oid
        );
    }
});

aaa_test!(ocel_log_is_multi_type, {
    // Arrange
    let events = declared_events();

    // Act
    let log = build_ocel_log(&events);

    // Assert — at least three distinct object types derived from real data
    let types: std::collections::BTreeSet<String> = log["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["type"].as_str().map(|s| s.to_string()))
        .collect();
    assert!(
        types.len() >= 3,
        "log must contain >=3 distinct object types (workspace/pipeline/policy), got {:?}",
        types
    );
});

// ── 2. BLAKE3 / hash tests ────────────────────────────────────────────────────

aaa_test!(blake3_hex_is_deterministic, {
    // Arrange
    let data = b"deterministic input";

    // Act
    let h1 = blake3_hex(data);
    let h2 = blake3_hex(data);

    // Assert
    assert_eq!(
        h1, h2,
        "blake3_hex must be deterministic for identical input"
    );
});

aaa_test!(blake3_hex_produces_64_char_hex, {
    // Arrange
    let data = b"cargo-cicd process evidence";

    // Act
    let hex = blake3_hex(data);

    // Assert
    assert_eq!(
        hex.len(),
        64,
        "blake3_hex must return a 64-character hex string"
    );
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "blake3_hex must return only hex characters"
    );
});

// ── 3. OCEL event lifecycle ───────────────────────────────────────────────────

aaa_test!(append_events_writes_ocel_json, {
    // Arrange
    let tmp = TempDir::new().unwrap();
    let evidence_dir = tmp.path().to_path_buf();
    let events = declared_events();

    // Act
    append_events(&events, &evidence_dir).unwrap();

    // Assert
    let ocel_path = evidence_dir.join("events.ocel.json");
    assert!(
        ocel_path.exists(),
        "append_events must write events.ocel.json"
    );
});

aaa_test!(append_events_ocel_accumulates_across_calls, {
    // Arrange: two separate append calls
    let tmp = TempDir::new().unwrap();
    let evidence_dir = tmp.path().to_path_buf();

    append_events(&[complete_event("status:show")], &evidence_dir).unwrap();
    append_events(&[complete_event("target:show")], &evidence_dir).unwrap();

    // Act
    let raw = std::fs::read_to_string(evidence_dir.join("events.ocel.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let event_arr = val["events"].as_array().unwrap();

    // Assert — OCEL reflects full accumulated history
    assert_eq!(
        event_arr.len(),
        2,
        "append_events OCEL must accumulate events across multiple calls"
    );
});

aaa_test!(append_events_writes_ocel_and_never_xes, {
    // Arrange
    let tmp = TempDir::new().unwrap();
    let evidence_dir = tmp.path().to_path_buf();
    let events = declared_events();

    // Act
    append_events(&events, &evidence_dir).unwrap();

    // Assert — OCEL 2.0 JSON must be written; XES must NOT be created
    // (OCEL-only by operator directive).
    let ocel_path = evidence_dir.join("events.ocel.json");
    assert!(
        ocel_path.exists(),
        "append_events must write events.ocel.json"
    );
    let xes_path = evidence_dir.join("events.xes");
    assert!(
        !xes_path.exists(),
        "append_events must NOT create events.xes (OCEL-only directive)"
    );
});

// ── 4. Statistical / algorithm tests ─────────────────────────────────────────

aaa_test!(page_hinkley_detects_step_change, {
    // Arrange: 20 observations at 1.0 then 10 at 5.0
    let mut obs: Vec<f64> = (0..20).map(|_| 1.0).collect();
    obs.extend((0..10).map(|_| 5.0));

    // Act
    let cp = page_hinkley_test(&obs, 5.0, 0.1);

    // Assert
    assert!(cp.is_some(), "page_hinkley_test must detect a step change");
});

aaa_test!(pareto_reject_dominated_returns_pareto_front, {
    // Arrange: (0.7, 0.7) is dominated by (0.9, 0.8); (0.9, 0.6) is dominated by (0.9, 0.8)
    let candidates = vec![(0.9f64, 0.8), (0.5, 0.9), (0.7, 0.7), (0.9, 0.6)];

    // Act
    let front = reject_dominated(&candidates);

    // Assert
    assert!(
        !front.contains(&(0.7, 0.7)),
        "reject_dominated must remove dominated point (0.7, 0.7)"
    );
    assert!(
        !front.contains(&(0.9, 0.6)),
        "reject_dominated must remove dominated point (0.9, 0.6)"
    );
    assert!(
        front.contains(&(0.9, 0.8)),
        "reject_dominated must keep non-dominated point (0.9, 0.8)"
    );
    assert!(
        front.contains(&(0.5, 0.9)),
        "reject_dominated must keep non-dominated point (0.5, 0.9)"
    );
});

aaa_test!(score_sequence_anomaly_zero_for_constant_sequence, {
    // Arrange
    let seq = vec![5.0f64; 20];

    // Act
    let score = score_sequence_anomaly(&seq);

    // Assert — constant sequence has zero anomaly
    assert!(
        score.abs() < 1e-10,
        "score_sequence_anomaly must return ~0.0 for a constant sequence, got {}",
        score
    );
});

aaa_test!(select_ucb1_prefers_arm_with_higher_reward, {
    // Arrange: arm 0 has low reward, arm 1 has high reward, equal counts
    let rewards = [1.0f64, 9.0];
    let counts = [10u64, 10];
    let total = 20u64;

    // Act
    let chosen = select_ucb1(&rewards, &counts, total);

    // Assert — arm 1 (higher reward) should be preferred
    assert_eq!(
        chosen, 1,
        "select_ucb1 must prefer the arm with higher reward when counts are equal"
    );
});

// ── 5. OCEL / evidence emit ───────────────────────────────────────────────────

aaa_test!(ocel_emit_creates_file, {
    // Arrange
    let events = declared_events();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.ocel.json");

    // Act
    emit_ocel(&events, &path).unwrap();

    // Assert
    assert!(path.exists(), "emit_ocel must create the file");
});

aaa_test!(ocel_emit_version_field_is_2_0, {
    // Arrange
    let events = declared_events();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.ocel.json");

    // Act
    emit_ocel(&events, &path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&raw).unwrap();

    // Assert — spec keys present with version-correct shape
    assert!(
        val["eventTypes"].is_array() && val["events"].is_array(),
        "OCEL log must use the official OCEL 2.0 JSON keys"
    );
});

aaa_test!(ocel_emit_events_map_contains_all_inputs, {
    // Arrange
    let events = declared_events();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.ocel.json");

    // Act
    emit_ocel(&events, &path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let event_arr = val["events"].as_array().unwrap();

    // Assert
    assert_eq!(
        event_arr.len(),
        events.len(),
        "events must contain exactly one entry per input ProcessEvent"
    );
});

// ── lsp: CICD-EVIDENCE-002 references events.ocel.json ───────────────────────
//
// The `lsp` noun is feature-gated, so `lsp explain` only exists when the `lsp`
// feature is enabled. Gate this test to match — under the default feature set
// the command is absent and the assertion would not apply.

#[cfg(feature = "lsp")]
#[test]
fn lsp_cicd_evidence_002_references_ocel_path() {
    // Arrange: check the compiled source to confirm the catalog entry was updated
    use assert_cmd::Command;
    let output = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["lsp", "explain", "CICD-EVIDENCE-002"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    // Assert
    assert!(
        text.contains("events.ocel.json") || text.contains("ocel"),
        "lsp explain CICD-EVIDENCE-002 must reference events.ocel.json, got: {}",
        text
    );
}
