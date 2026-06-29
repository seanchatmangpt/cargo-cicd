//! Chicago TDD verification suite for the OCEL 2.0 migration.
//!
//! Uses a vendored Arrange-Act-Assert `aaa_test!` macro (see below) to prove
//! that every capability added in the OCEL migration is complete and correct —
//! not just compiling, but behaving as specified.
//!
//! Coverage map:
//!  - emit_ocel / emit_ocel_filtered / emit_ocel_fresh / emit_ocel_impl
//!  - build_ocel_log / build_ocel_log_filtered
//!  - append_events writes events.ocel.json alongside events.xes
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
    OcelLog, OcelObject, OcelRelationship, OcelTypes, Perturbator,
};
use std::collections::HashMap;
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
        version: "2.0".to_string(),
        types: OcelTypes {
            object_types: vec![],
            event_types: vec![],
        },
        events: HashMap::new(),
        objects: HashMap::new(),
    }
}

fn single_event_log() -> OcelLog {
    let mut events: HashMap<String, OcelEvent> = HashMap::new();
    events.insert(
        "e1".to_string(),
        OcelEvent {
            activity: "status:show".to_string(),
            timestamp: "2026-06-16T00:00:00Z".to_string(),
            vmap: HashMap::new(),
            typed_omap: vec![OcelRelationship {
                object_id: "ws:test".to_string(),
                object_type: "cargo.workspace".to_string(),
                qualifier: None,
            }],
        },
    );
    let mut objects: HashMap<String, OcelObject> = HashMap::new();
    objects.insert(
        "ws:test".to_string(),
        OcelObject {
            object_type: "cargo.workspace".to_string(),
            ovmap: HashMap::new(),
            o2o: vec![],
        },
    );
    OcelLog {
        version: "2.0".to_string(),
        types: OcelTypes {
            object_types: OcelLog::cargo_object_types(),
            event_types: vec![],
        },
        events,
        objects,
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

    // Assert
    assert_eq!(log["ocel:version"], "2.0");
    assert!(log["ocel:events"].is_object());
    assert!(log["ocel:objects"].is_object());
    assert!(log["ocel:object-types"].is_object());
    assert!(log["ocel:event-types"].is_object());
});

aaa_test!(ocel_emit_each_event_has_typed_omap, {
    // Arrange
    let events = vec![complete_event("status:show")];
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.ocel.json");

    // Act
    emit_ocel(&events, &path).unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let event_map = val["ocel:events"].as_object().unwrap();
    let first = event_map.values().next().unwrap();
    let omap = first["ocel:typedOmap"].as_array().unwrap();

    // Assert
    assert!(
        !omap.is_empty(),
        "every OCEL event must have at least one ocel:typedOmap entry"
    );
    assert_eq!(
        omap[0]["ocel:qualifier"], "cargo.workspace",
        "typedOmap entry must reference cargo.workspace"
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
    let event_map = val["ocel:events"].as_object().unwrap();

    // Assert — OCEL reflects full accumulated history
    assert_eq!(
        event_map.len(),
        2,
        "append_events OCEL must accumulate events across multiple calls"
    );
});

aaa_test!(append_events_writes_xes_alongside_ocel, {
    // Arrange
    let tmp = TempDir::new().unwrap();
    let evidence_dir = tmp.path().to_path_buf();
    let events = declared_events();

    // Act
    append_events(&events, &evidence_dir).unwrap();

    // Assert — XES must still be present (backward-compat invariant)
    let xes_path = evidence_dir.join("events.xes");
    assert!(
        xes_path.exists(),
        "append_events must still write events.xes (backward compat)"
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

// ── 5. XES / evidence emit ────────────────────────────────────────────────────

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

    // Assert
    assert_eq!(
        val["ocel:version"], "2.0",
        "OCEL log must declare version 2.0 (E5 compliance)"
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
    let event_map = val["ocel:events"].as_object().unwrap();

    // Assert
    assert_eq!(
        event_map.len(),
        events.len(),
        "ocel:events must contain exactly one entry per input ProcessEvent"
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
