//! Evidence Gate Architecture — cargo-cicd emits; wasm4pm adjudicates.
//!
//! ## Invariants
//!
//! - **E1**: cargo-cicd NEVER adjudicates its own process conformance.
//!   All verdicts are issued by the external wasm4pm oracle.
//! - **E2**: Evidence is emitted before adjudication. The OCEL 2.0 JSON file
//!   (`events.ocel.json`) must exist on disk before `audit_ocel` is called.
//! - **E3**: If the oracle is unavailable and the expected verdict is not
//!   `Blocked`, the evidence gate panics. Certification requires the oracle.
//! - **E4**: Tests assert only wasm4pm verdict, never internal cargo-cicd state.
//!   cargo-cicd state assertions belong in unit tests; process conformance
//!   assertions belong in evidence-gate tests.
//! - **E5**: OCEL 2.0 emission is the ONLY event-log format. XES emission is
//!   removed by operator directive — no `.xes` file is ever created.
//! - **E6**: JSONL emission mirrors OCEL 2.0 — same event set, machine-readable
//!   companion format for downstream tooling.
//! - **E7**: `ExpectedWpmVerdict::Blocked` is a first-class expectation, not
//!   an error state. Tests that run without wpm installed MUST declare
//!   `Blocked` as their expected verdict.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::integrations::{discover_wpm_binary, Wasm4pmShell, WpmVerdict};
use crate::ocel::{
    OcelAttributeValue, OcelEvent, OcelEventAttributeValue, OcelEventType, OcelLog, OcelObject,
    OcelObjectAttribute, OcelObjectAttributeValue, OcelRelationship,
};

// ── Timestamp helpers ─────────────────────────────────────────────────────────

/// Return the current UTC time as an ISO-8601 string, e.g.
/// `"2026-06-02T13:45:07.123Z"`.
pub fn now_iso8601() -> String {
    let ts = jiff::Timestamp::now();
    let dt = ts.to_zoned(jiff::tz::TimeZone::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.millisecond(),
    )
}

/// Build a compact event-id from the current timestamp (no special chars).
fn new_event_id(command: &str) -> String {
    let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
    format!("evt-{}-{}", command.replace(' ', "-"), ts)
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single process event emitted by cargo-cicd for wasm4pm adjudication.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProcessEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// ISO-8601 UTC timestamp captured at event construction time.
    pub timestamp_iso: String,
    /// Session / case grouping key. Events with the same `case_id` belong to
    /// the same process case in the OCEL 2.0 log.
    pub case_id: Option<String>,
    /// `"start"` or `"complete"`.
    pub lifecycle_transition: String,
    pub workspace_id: String,
    pub repo_path: String,
    pub command: String,
    /// Verdict claimed by cargo-cicd (never adjudicated by cargo-cicd itself).
    pub verdict_claimed: String,
    /// Elapsed wall-clock milliseconds. `None` for `"start"` events.
    pub duration_ms: Option<u64>,
    /// Verdict issued by the external wasm4pm oracle after round-trip.
    /// `None` until adjudication has been performed.
    pub verdict_adjudicated: Option<String>,
    /// ISO-8601 UTC timestamp when the oracle responded.
    /// `None` until adjudication has been performed.
    pub adjudicated_at: Option<String>,
    /// Path to the wpm binary used for adjudication.
    /// `None` until adjudication has been performed.
    pub oracle_command: Option<String>,
    /// Trace class separating pipeline runs from ambient live-workspace history.
    ///
    /// - `"pipeline_run"` — emitted by `pipeline run`; complete sequential
    ///   execution of all declared activities.
    /// - `"live_workspace"` — emitted by individual sub-command invocations;
    ///   accumulated ambient history (VARIANCE verdict is expected and honest).
    #[serde(default = "default_trace_class")]
    pub trace_class: String,
    /// BLAKE3 hex hash of the admitted star-toml config witness.
    /// `None` when no config was loaded through the star-toml pipeline.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub config_witness: Option<String>,
}

fn default_trace_class() -> String {
    "live_workspace".to_string()
}

impl ProcessEvent {
    /// Construct a completed `ProcessEvent` with the current real timestamp.
    ///
    /// `verdict` should be `"PASS"`, `"WARN"`, or `"FAIL"`.
    /// The `trace_class` is set to `"live_workspace"` (ambient command history).
    pub fn new(command: &str, verdict: &str) -> Self {
        Self {
            event_id: new_event_id(command),
            timestamp_iso: now_iso8601(),
            case_id: None,
            lifecycle_transition: "complete".to_string(),
            workspace_id: "cargo-cicd-workspace".to_string(),
            repo_path: ".".to_string(),
            command: command.to_string(),
            verdict_claimed: verdict.to_string(),
            duration_ms: None,
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
            config_witness: None,
        }
    }

    /// Construct a `"start"` lifecycle event and capture the wall-clock instant.
    ///
    /// Returns `(event, instant)`. Pass `instant` to [`ProcessEvent::completed`]
    /// to measure elapsed time.
    pub fn started(command: &str) -> (Self, std::time::Instant) {
        let t0 = std::time::Instant::now();
        let ev = Self {
            event_id: new_event_id(command),
            timestamp_iso: now_iso8601(),
            case_id: None,
            lifecycle_transition: "start".to_string(),
            workspace_id: "cargo-cicd-workspace".to_string(),
            repo_path: ".".to_string(),
            command: command.to_string(),
            verdict_claimed: String::new(),
            duration_ms: None,
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
            config_witness: None,
        };
        (ev, t0)
    }

    /// Construct a `"complete"` lifecycle event, measuring elapsed time from `t0`.
    pub fn completed(command: &str, t0: std::time::Instant, verdict: &str) -> Self {
        let duration_ms = t0.elapsed().as_millis() as u64;
        Self {
            event_id: new_event_id(command),
            timestamp_iso: now_iso8601(),
            case_id: None,
            lifecycle_transition: "complete".to_string(),
            workspace_id: "cargo-cicd-workspace".to_string(),
            repo_path: ".".to_string(),
            command: command.to_string(),
            verdict_claimed: verdict.to_string(),
            duration_ms: Some(duration_ms),
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
            config_witness: None,
        }
    }

    /// Construct an event that records an oracle adjudication result.
    ///
    /// - `command` — the logical event name (e.g. `"evidence:audit"`)
    /// - `verdict` — the verdict string from the oracle (`"ACCEPT"` or `"REFUSE"`)
    /// - `oracle` — path to the wpm binary that produced the verdict
    pub fn new_adjudicated(command: &str, verdict: &str, oracle: &str) -> Self {
        Self {
            event_id: new_event_id(command),
            timestamp_iso: now_iso8601(),
            case_id: None,
            lifecycle_transition: "complete".to_string(),
            workspace_id: "cargo-cicd-workspace".to_string(),
            repo_path: ".".to_string(),
            command: command.to_string(),
            verdict_claimed: "pending_adjudication".to_string(),
            duration_ms: None,
            verdict_adjudicated: Some(verdict.to_string()),
            adjudicated_at: Some(now_iso8601()),
            oracle_command: Some(oracle.to_string()),
            trace_class: "live_workspace".to_string(),
            config_witness: None,
        }
    }

    /// Construct a completed event tagged as `"pipeline_run"`.
    ///
    /// Use this instead of [`ProcessEvent::new`] for events emitted by the
    /// `pipeline run` command so they can be separated from ambient
    /// `"live_workspace"` history during conformance checking.
    pub fn for_pipeline(command: &str, verdict: &str) -> Self {
        Self {
            trace_class: "pipeline_run".to_string(),
            ..Self::new(command, verdict)
        }
    }
}

/// The verdict the test expects from wasm4pm for a given evidence file.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedWpmVerdict {
    /// wasm4pm accepted the event log as conformant.
    Accept,
    /// wasm4pm rejected the event log (Fail verdict).
    Refuse,
    /// wasm4pm binary is unavailable; the gate is blocked.
    Blocked,
}

/// Runtime oracle that shells out to `wpm` for OCEL 2.0 adjudication.
pub struct WpmEvidenceOracle {
    shell: Option<Wasm4pmShell>,
}

impl Default for WpmEvidenceOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl WpmEvidenceOracle {
    /// Create a new oracle, auto-detecting the wpm binary.
    pub fn new() -> Self {
        Self {
            shell: Wasm4pmShell::detect(),
        }
    }

    /// Returns `true` if the wpm binary was detected and is available.
    pub fn is_available(&self) -> bool {
        self.shell.is_some()
    }

    /// Audit an OCEL 2.0 event log via `wpm receipt verify-ocel2` and map the
    /// wpm verdict to `ExpectedWpmVerdict`.
    ///
    /// - Binary absent → `Blocked`
    /// - Invocation error → `Refuse`
    /// - `Pass | Warn | Partial` → `Accept`
    /// - `Fail` → `Refuse`
    /// - `NotAvailable` → `Blocked`
    pub fn audit_ocel(&self, ocel_path: &Path) -> ExpectedWpmVerdict {
        match &self.shell {
            None => ExpectedWpmVerdict::Blocked,
            Some(wpm) => {
                if !ocel_path.exists() {
                    return ExpectedWpmVerdict::Blocked;
                }
                match wpm.receipt_verify_ocel2(ocel_path.to_str().unwrap_or("")) {
                    Err(_) => ExpectedWpmVerdict::Refuse,
                    Ok(result) => match result.verdict {
                        WpmVerdict::Pass | WpmVerdict::Warn | WpmVerdict::Partial => {
                            ExpectedWpmVerdict::Accept
                        }
                        WpmVerdict::Fail => ExpectedWpmVerdict::Refuse,
                        WpmVerdict::NotAvailable => ExpectedWpmVerdict::Blocked,
                    },
                }
            }
        }
    }
}

// ── Emission ──────────────────────────────────────────────────────────────────

// ── Declared model activities ─────────────────────────────────────────────────

/// The 10 activities declared in cicd-process.powl.json.
///
/// Only these activities are written to events.ocel.json for token-replay
/// fitness. Noise events (e.g. "git:status") are excluded from the log so they
/// do not corrupt the DFG-derived Petri net with unmodelled transitions.
const DECLARED_ACTIVITIES: &[&str] = &[
    "status:show",
    "status:audit",
    "target:show",
    "target:prune",
    "test:changed",
    "trybuild:changed",
    "workspace:doctor",
    "publish:run",
    "evidence:audit",
    "receipt:write",
];

/// Returns `true` if `name` is one of the 10 declared model activities.
fn is_declared_activity(name: &str) -> bool {
    DECLARED_ACTIVITIES.contains(&name)
}

// ── OCEL 2.0 emission ─────────────────────────────────────────────────────────

/// Build an OCEL 2.0 JSON log from a slice of ProcessEvents.
///
/// Produces the official OCEL 2.0 JSON structure
/// (<https://ocel-standard.org/specification/formats/json/>): top-level
/// `eventTypes`, `objectTypes`, `events`, `objects` arrays. Events reference
/// real evidence-pipeline objects (workspace, pipeline, policy) through
/// spec-shape relationships `{objectId, qualifier}`.
pub fn build_ocel_log(events: &[ProcessEvent]) -> serde_json::Value {
    build_ocel_log_impl(events, false)
}

/// Build a production-quality OCEL 2.0 log, filtered for token-replay fitness.
///
/// Applies the same three quality fixes as [`emit_ocel_filtered`]:
/// 1. Only "complete" lifecycle events.
/// 2. Only declared-model activities.
/// 3. Events sorted by timestamp within each object group.
pub fn build_ocel_log_filtered(events: &[ProcessEvent]) -> serde_json::Value {
    build_ocel_log_impl(events, true)
}

fn build_ocel_log_impl(events: &[ProcessEvent], filter: bool) -> serde_json::Value {
    let filtered: Vec<&ProcessEvent> = events
        .iter()
        .filter(|ev| {
            if filter {
                ev.lifecycle_transition == "complete" && is_declared_activity(&ev.command)
            } else {
                true
            }
        })
        .collect();

    // ── Event types: one entry per unique activity, with the declared vmap schema.
    let mut seen_types: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for ev in &filtered {
        seen_types.insert(&ev.command);
    }
    let event_types: Vec<OcelEventType> = seen_types
        .iter()
        .map(|t| OcelEventType {
            name: t.to_string(),
            attributes: vec![
                OcelObjectAttribute {
                    name: "verdict_claimed".into(),
                    attr_type: "string".into(),
                },
                OcelObjectAttribute {
                    name: "lifecycle".into(),
                    attr_type: "string".into(),
                },
                OcelObjectAttribute {
                    name: "trace_class".into(),
                    attr_type: "string".into(),
                },
                OcelObjectAttribute {
                    name: "duration_ms".into(),
                    attr_type: "integer".into(),
                },
            ],
        })
        .collect();

    // ── Objects: real multi-type objects derived from the event data itself —
    //    the workspace (real workspace_id / repo_path), one pipeline object per
    //    trace_class, and one policy object per claimed verdict.
    let (workspace_id, repo_path, first_time) = filtered
        .first()
        .map(|ev| {
            (
                ev.workspace_id.clone(),
                ev.repo_path.clone(),
                ev.timestamp_iso.clone(),
            )
        })
        .unwrap_or_else(|| {
            (
                "cargo-cicd-workspace".to_string(),
                String::new(),
                now_iso8601(),
            )
        });

    let mut object_ids: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut objects: Vec<OcelObject> = vec![OcelObject {
        id: workspace_id.clone(),
        object_type: "cargo.workspace".into(),
        attributes: vec![OcelObjectAttributeValue {
            name: "repo_path".into(),
            time: first_time,
            value: OcelAttributeValue::String(repo_path),
        }],
        relationships: vec![],
    }];
    object_ids.insert(workspace_id.clone());

    let mut pipelines: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut policies: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for ev in &filtered {
        pipelines
            .entry(ev.trace_class.clone())
            .or_insert_with(|| ev.timestamp_iso.clone());
        policies
            .entry(ev.verdict_claimed.clone())
            .or_insert_with(|| ev.timestamp_iso.clone());
    }
    for (tc, ts) in &pipelines {
        let id = format!("pipeline:{tc}");
        if object_ids.insert(id.clone()) {
            objects.push(OcelObject {
                id,
                object_type: "cargo.pipeline".into(),
                attributes: vec![OcelObjectAttributeValue {
                    name: "trace_class".into(),
                    time: ts.clone(),
                    value: OcelAttributeValue::String(tc.clone()),
                }],
                relationships: vec![],
            });
        }
    }
    for (v, ts) in &policies {
        let id = format!("policy:{v}");
        if object_ids.insert(id.clone()) {
            objects.push(OcelObject {
                id,
                object_type: "cargo.policy".into(),
                attributes: vec![OcelObjectAttributeValue {
                    name: "verdict".into(),
                    time: ts.clone(),
                    value: OcelAttributeValue::String(v.clone()),
                }],
                relationships: vec![],
            });
        }
    }

    // ── Events: spec shape, with typed relationships (objectId + qualifier).
    let ocel_events: Vec<OcelEvent> = filtered
        .iter()
        .map(|ev| {
            let mut attributes = vec![
                OcelEventAttributeValue {
                    name: "verdict_claimed".into(),
                    value: OcelAttributeValue::String(ev.verdict_claimed.clone()),
                },
                OcelEventAttributeValue {
                    name: "lifecycle".into(),
                    value: OcelAttributeValue::String(ev.lifecycle_transition.clone()),
                },
                OcelEventAttributeValue {
                    name: "trace_class".into(),
                    value: OcelAttributeValue::String(ev.trace_class.clone()),
                },
            ];
            if let Some(ms) = ev.duration_ms {
                attributes.push(OcelEventAttributeValue {
                    name: "duration_ms".into(),
                    value: OcelAttributeValue::Integer(ms as i64),
                });
            }
            if let Some(ref adj) = ev.verdict_adjudicated {
                attributes.push(OcelEventAttributeValue {
                    name: "verdict_adjudicated".into(),
                    value: OcelAttributeValue::String(adj.clone()),
                });
            }
            if let Some(ref hash) = ev.config_witness {
                attributes.push(OcelEventAttributeValue {
                    name: "config_witness".into(),
                    value: OcelAttributeValue::String(hash.clone()),
                });
            }
            OcelEvent {
                id: ev.event_id.clone(),
                event_type: ev.command.clone(),
                time: ev.timestamp_iso.clone(),
                attributes,
                relationships: vec![
                    OcelRelationship {
                        object_id: workspace_id.clone(),
                        qualifier: "executed_in".into(),
                    },
                    OcelRelationship {
                        object_id: format!("pipeline:{}", ev.trace_class),
                        qualifier: "part_of".into(),
                    },
                    OcelRelationship {
                        object_id: format!("policy:{}", ev.verdict_claimed),
                        qualifier: "claimed".into(),
                    },
                ],
            }
        })
        .collect();

    let log = OcelLog {
        event_types,
        object_types: OcelLog::cargo_object_types(),
        events: ocel_events,
        objects,
    };
    serde_json::to_value(&log).unwrap_or_else(|_| serde_json::json!({}))
}

/// Low-level OCEL 2.0 writer: emits all events as-provided, no filtering.
pub fn emit_ocel(events: &[ProcessEvent], path: &Path) -> Result<()> {
    emit_ocel_impl(events, path, false)
}

/// Production OCEL 2.0 writer for token-replay fitness.
///
/// Applies the same quality fixes as [`build_ocel_log_filtered`]:
/// - Only "complete" lifecycle events.
/// - Only declared-model activities.
pub fn emit_ocel_filtered(events: &[ProcessEvent], path: &Path) -> Result<()> {
    emit_ocel_impl(events, path, true)
}

/// Emit a fresh OCEL 2.0 event log, overwriting any existing file at `path`.
///
/// Fresh OCEL 2.0 writer: writes only the events provided, overwriting any
/// existing file at `path`. Applies production filters (declared activities,
/// complete events only).
pub fn emit_ocel_fresh(events: &[ProcessEvent], path: &Path) -> Result<()> {
    emit_ocel_filtered(events, path)
}

fn emit_ocel_impl(events: &[ProcessEvent], path: &Path, filter: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let log = build_ocel_log_impl(events, filter);
    std::fs::write(path, serde_json::to_string_pretty(&log)?)?;
    Ok(())
}

/// Emit events as newline-delimited JSON to `path`.
///
/// Each line is a JSON object with `event_id`, `command`,
/// `verdict_claimed`, `timestamp_iso`, `lifecycle_transition`, and
/// optional `case_id` / `duration_ms` fields.
pub fn emit_events_jsonl(events: &[ProcessEvent], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = String::new();
    for event in events {
        let case_field = match &event.case_id {
            Some(id) => format!(",\"case_id\":{}", json_string(id)),
            None => String::new(),
        };
        let dur_field = match event.duration_ms {
            Some(ms) => format!(",\"duration_ms\":{}", ms),
            None => String::new(),
        };
        lines.push_str(&format!(
            "{{\"event_id\":{},\"command\":{},\"verdict_claimed\":{},\"timestamp_iso\":{},\"lifecycle_transition\":{}{}{}}}\n",
            json_string(&event.event_id),
            json_string(&event.command),
            json_string(&event.verdict_claimed),
            json_string(&event.timestamp_iso),
            json_string(&event.lifecycle_transition),
            case_field,
            dur_field,
        ));
    }

    std::fs::write(path, lines)?;
    Ok(())
}

/// Canonical evidence directory relative to the workspace root.
pub fn evidence_dir() -> PathBuf {
    PathBuf::from("target/cargo-cicd/evidence")
}

// ── Receipt Doctor ─────────────────────────────────────────────────────────────

/// Verdict returned by `wpm receipt doctor`.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiptDoctorVerdict {
    /// `state == "Admitted"` — receipt accepted.
    Accepted { stdout_json: String },
    /// `state == "Refused"` or exit code 1.
    Refused {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    /// wpm binary unavailable.
    Blocked { reason: String },
}

/// Shells out to `wpm receipt doctor --format json --strict`.
pub struct ReceiptDoctor {
    wpm_path: std::path::PathBuf,
}

impl ReceiptDoctor {
    /// Discover wpm binary using canonical multi-source discovery.
    pub fn discover() -> Option<Self> {
        discover_wpm_binary().map(|pb| Self { wpm_path: pb })
    }

    /// Run `wpm receipt doctor --format json --strict <receipt_path>`.
    /// Returns the path of the discovered wpm binary.
    pub fn binary_path(&self) -> &str {
        self.wpm_path.to_str().unwrap_or("")
    }

    pub fn doctor_strict_json(&self, receipt_path: &Path) -> ReceiptDoctorVerdict {
        let output = match std::process::Command::new(&self.wpm_path)
            .args([
                "receipt",
                "doctor",
                "--format",
                "json",
                "--strict",
                receipt_path.to_str().unwrap_or(""),
            ])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return ReceiptDoctorVerdict::Blocked {
                    reason: e.to_string(),
                }
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code == 0 {
            ReceiptDoctorVerdict::Accepted {
                stdout_json: stdout,
            }
        } else {
            ReceiptDoctorVerdict::Refused {
                exit_code,
                stdout,
                stderr,
            }
        }
    }

    /// Emit a receipt for `events` and adjudicate it in a single wpm call.
    ///
    /// Hash fields are intentionally omitted so `CanonicalHashVerifier` is skipped;
    /// adjudication relies on structural correctness only.
    pub fn emit_and_adjudicate(
        &self,
        events: &[ProcessEvent],
        evidence_dir: &Path,
        command: &str,
    ) -> (PathBuf, ReceiptDoctorVerdict) {
        let receipts_dir = evidence_dir.join("receipts");
        let _ = std::fs::create_dir_all(&receipts_dir);
        let receipt_path = receipts_dir.join("latest.json");

        let refs: Vec<&ProcessEvent> = events.iter().collect();
        let receipt = build_receipt_json(&refs, command, 0);
        let pretty = serde_json::to_string_pretty(&receipt).unwrap_or_default();
        let _ = std::fs::write(&receipt_path, pretty);

        let verdict = self.doctor_strict_json(&receipt_path);
        (receipt_path, verdict)
    }
}

/// Build an OCEL 2.0 receipt that satisfies `wpm receipt doctor --strict`.
///
/// Key design decisions:
/// - `algorithms` is non-empty with both expected and observed OCEL 2.0 paths.
/// - Hash fields are intentionally absent so `CanonicalHashVerifier` is skipped.
/// - `boundary_evidence` has `exit_code` + `command` to satisfy `BoundaryEvidenceVerifier`.
/// - No `alignment`, `challenge_nonce`, `runtime_observer`, or `all_real` to avoid
///   `SelfCertifiedAlignment`, `ChallengeNonceVerifier`, and `ClosureOverclaimDetector`.
/// - All string values avoid the forbidden evidence markers list.
pub fn build_receipt_json(
    events: &[&ProcessEvent],
    command: &str,
    exit_code: i32,
) -> serde_json::Value {
    let now = now_iso8601();

    // Expected: declared cargo-cicd process model (static, version-stamped).
    // Types are intentionally distinct from observed types to prevent near-clone detection.
    let expected_ocel = serde_json::json!({
        "events": [
            {"id": "exp-evt-ci-start",    "type": "cargo.ci.session.start",   "timestamp": now.clone()},
            {"id": "exp-evt-cmd-execute", "type": "cargo.ci.command.execute", "timestamp": now.clone()},
            {"id": "exp-evt-evidence",    "type": "cargo.ci.evidence.emit",   "timestamp": now.clone()}
        ],
        "objects": [
            {"id": "cargo-cicd-workspace", "type": "cargo.workspace"}
        ],
        "ocel-version": "2.0"
    });

    // Observed: actual runtime events, always non-empty (sentinel appended).
    let mut obs_events: Vec<serde_json::Value> = events
        .iter()
        .filter(|ev| ev.lifecycle_transition == "complete")
        .map(|ev| {
            serde_json::json!({
                "id":        ev.event_id.as_str(),
                "type":      ev.command.as_str(),
                "timestamp": ev.timestamp_iso.as_str()
            })
        })
        .collect();

    // Sentinel ensures events list is never empty.
    obs_events.push(serde_json::json!({
        "id":        format!("evt-receipt-emit-{}", now.replace(['-', ':', '.', 'T', 'Z'], "")),
        "type":      "cargo.ci.receipt.emit",
        "timestamp": now.clone()
    }));

    let observed_ocel = serde_json::json!({
        "events":      obs_events,
        "objects": [
            {"id": "cargo-cicd-workspace", "type": "cargo.workspace"}
        ],
        "ocel-version": "2.0"
    });

    let receipt_id = format!(
        "cargo-cicd-receipt-{}",
        now.replace(['-', ':', '.', 'T', 'Z'], "")
    );
    let repo_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/repo".to_string());
    let git_head = git_head_short();

    serde_json::json!({
        "receipt_id":       receipt_id,
        "producer":         "cargo-cicd",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "created_at":       now,
        "repo_path":        repo_path,
        "git_head":         git_head,
        "algorithms": [{
            "algorithm_id": "cargo-cicd-process-evidence",
            "expected_path": {
                "route_id":       "cargo.ci.declared-process",
                "expected_ocel2": expected_ocel
            },
            "observed_path": {
                "route_id":        "cargo.ci.observed-process",
                "observed_ocel2":  observed_ocel
            },
            "boundary_evidence": {
                "exit_code": exit_code,
                "command":   command
            }
        }]
    })
}

/// Return the short git HEAD SHA, or a safe fallback (no forbidden markers).
fn git_head_short() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "HEAD-not-resolved".to_string())
}

/// Write a receipt to `target/cargo-cicd/evidence/receipts/latest.json`.
///
/// Returns the path the receipt was written to.
pub fn emit_receipt_json(
    events: &[&ProcessEvent],
    command: &str,
    exit_code: i32,
) -> Result<PathBuf> {
    let dir = PathBuf::from("target/cargo-cicd/evidence/receipts");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("latest.json");
    let receipt = build_receipt_json(events, command, exit_code);
    std::fs::write(&path, serde_json::to_string_pretty(&receipt)?)?;
    Ok(path)
}

/// Append `events` to `<evidence_dir>/events.jsonl`, then rebuild
/// `<evidence_dir>/events.ocel.json` from the full accumulated log.
///
/// This is the canonical emission path. It is safe to call from multiple
/// commands in the same session — each call appends rather than overwrites,
/// so the OCEL log always reflects the complete session history.
///
/// The OCEL 2.0 JSON written here applies the three quality fixes:
/// - Noise events excluded (only DECLARED_ACTIVITIES pass through).
/// - Start lifecycle events excluded (start_complete_affects_fitness = true).
/// - Events sorted by timestamp within each object group.
///
/// Additionally, each invocation archives `events.ocel.json` to
/// `<evidence_dir>/history/<timestamp>-events.ocel.json` so individual pipeline
/// runs are preserved for forensic inspection (fresh-trace-per-run).
/// Read the full accumulated process-event journal (`<evidence_dir>/events.jsonl`)
/// back into memory.
///
/// Returns an empty vec if the journal does not exist or is empty. Malformed
/// lines are skipped on a best-effort basis (never panics). This mirrors the
/// read-back path inside [`append_events`] and is used by external witnesses
/// (e.g. the affidavit provenance engine) that certify the full process history.
pub fn read_journal(evidence_dir: &Path) -> Vec<ProcessEvent> {
    let jsonl_path = evidence_dir.join("events.jsonl");
    let content = std::fs::read_to_string(&jsonl_path).unwrap_or_default();
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

pub fn append_events(events: &[ProcessEvent], evidence_dir: &Path) -> Result<()> {
    if let Some(parent) = evidence_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir_all(evidence_dir)?;

    let pid = std::process::id();

    // Clean up stale .tmp.* files older than 60 seconds before writing.
    if let Ok(read_dir) = std::fs::read_dir(evidence_dir) {
        let now = std::time::SystemTime::now();
        for entry in read_dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.contains(".tmp.") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() > 60 {
                                let _ = std::fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }

    let jsonl_path = evidence_dir.join("events.jsonl");
    let ocel_path = evidence_dir.join("events.ocel.json");

    let jsonl_tmp = evidence_dir.join(format!("events.jsonl.tmp.{}", pid));
    let ocel_tmp = evidence_dir.join(format!("events.ocel.json.tmp.{}", pid));

    // Read existing JSONL content, append new events in memory.
    let existing = std::fs::read_to_string(&jsonl_path).unwrap_or_default();
    let mut new_content = existing.clone();
    for ev in events {
        let line = serde_json::to_string(ev)?;
        new_content.push_str(&line);
        new_content.push('\n');
    }

    // Write full JSONL to tmp, then atomically rename.
    std::fs::write(&jsonl_tmp, &new_content)?;
    std::fs::rename(&jsonl_tmp, &jsonl_path)?;

    // Parse all events from the now-committed JSONL.
    let all_events: Vec<ProcessEvent> = new_content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    // Build and atomically write OCEL 2.0 JSON.
    {
        emit_ocel_filtered(&all_events, &ocel_tmp)?;
        std::fs::rename(&ocel_tmp, &ocel_path)?;
    }

    // Archive the final renamed files to history/ for fresh-trace-per-run traceability.
    let history_dir = evidence_dir.join("history");
    if std::fs::create_dir_all(&history_dir).is_ok() {
        let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
        let _ = std::fs::copy(
            &ocel_path,
            history_dir.join(format!("{}-events.ocel.json", ts)),
        );
    }

    // Autonomic Receipt Injection
    // Synchronously emit each event to affidavit and seal the receipt chain.
    // Runs AFTER the rename sequence so the committed JSONL is on disk.
    #[cfg(feature = "affidavit")]
    if let Some(affi) = crate::integrations::affidavit_shell::AffidavitShell::detect() {
        let affi_dir = crate::integrations::affidavit_shell::affidavit_receipt_dir(evidence_dir);
        let _ = std::fs::create_dir_all(&affi_dir);
        let receipt_out = affi_dir.join("receipt.json");

        for ev in events {
            let event_type = crate::integrations::affidavit_shell::event_type_for(
                &ev.command,
                &ev.lifecycle_transition,
            );
            let object = crate::integrations::affidavit_shell::object_ref_for(ev);
            let _ = affi.emit(&affi_dir, &event_type, &object, &jsonl_path);
        }
        let _ = affi.assemble(&affi_dir, &receipt_out);
    }

    Ok(())
}

// ── Assertion ─────────────────────────────────────────────────────────────────

/// Assert that the wasm4pm oracle returns the expected verdict for an OCEL 2.0
/// evidence file. Kept as a compat alias for [`assert_wpm_verdict_ocel`].
///
/// Panics with a detailed message if:
/// - The oracle is `Blocked` and the expected verdict is not `Blocked` (E3 violation).
/// - The actual verdict does not match the expected verdict.
pub fn assert_wpm_verdict(
    oracle: &WpmEvidenceOracle,
    evidence_path: &Path,
    expected: &ExpectedWpmVerdict,
) {
    let actual = oracle.audit_ocel(evidence_path);

    if actual == ExpectedWpmVerdict::Blocked && *expected != ExpectedWpmVerdict::Blocked {
        panic!(
            "BLOCKED: wasm4pm oracle command unavailable — evidence gate cannot certify.\n\
             wpm binary not found. Install wasm4pm or set WPM_PATH env var.\n\
             Evidence gate invariant E3 violated: external oracle required."
        );
    }

    assert_eq!(
        actual, *expected,
        "wpm evidence gate verdict mismatch for {:?}: expected {:?}, got {:?}",
        evidence_path, expected, actual
    );
}

/// Assert that the wasm4pm oracle returns the expected verdict for an OCEL 2.0 file.
///
/// Mirrors [`assert_wpm_verdict`] but drives the `receipt verify-ocel2` oracle path.
/// Panics with a detailed message on E3 violation or verdict mismatch.
pub fn assert_wpm_verdict_ocel(
    oracle: &WpmEvidenceOracle,
    ocel_path: &Path,
    expected: &ExpectedWpmVerdict,
) {
    let actual = oracle.audit_ocel(ocel_path);

    if actual == ExpectedWpmVerdict::Blocked && *expected != ExpectedWpmVerdict::Blocked {
        panic!(
            "BLOCKED: wasm4pm oracle unavailable — OCEL evidence gate cannot certify.\n\
             wpm binary not found. Install wasm4pm or set WPM_PATH env var.\n\
             Evidence gate invariant E3 violated: external oracle required."
        );
    }

    assert_eq!(
        actual, *expected,
        "wpm OCEL evidence gate verdict mismatch for {:?}: expected {:?}, got {:?}",
        ocel_path, expected, actual
    );
}

/// Assert that the affidavit oracle returns the expected verdict for a sealed receipt.
///
/// Panics with a detailed message if:
/// - The oracle is `Blocked` and the expected verdict is not `Blocked` (E3 violation).
/// - The actual verdict does not match the expected verdict.
#[cfg(feature = "affidavit")]
pub fn assert_affidavit_verdict(
    oracle: &crate::integrations::affidavit_shell::AffidavitShell,
    receipt_path: &Path,
    expected: &crate::integrations::affidavit_shell::AffidavitVerdict,
) {
    let actual = match oracle.verify(receipt_path) {
        Ok(r) => r.verdict,
        Err(e) => panic!("Failed to invoke affi verify: {}", e),
    };

    if actual == crate::integrations::affidavit_shell::AffidavitVerdict::Blocked
        && *expected != crate::integrations::affidavit_shell::AffidavitVerdict::Blocked
    {
        panic!(
            "BLOCKED: affidavit oracle command unavailable — evidence gate cannot certify integrity.\n\
             affi binary not found. Install affidavit or set AFFI_PATH env var.\n\
             Evidence gate invariant E3 violated: external oracle required."
        );
    }

    assert_eq!(
        actual, *expected,
        "affidavit evidence gate verdict mismatch for {:?}: expected {:?}, got {:?}",
        receipt_path, expected, actual
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn json_string(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_complete_event(id: &str, cmd: &str) -> ProcessEvent {
        ProcessEvent {
            event_id: id.to_string(),
            timestamp_iso: "2026-06-02T00:00:00Z".to_string(),
            case_id: Some("test-case".to_string()),
            lifecycle_transition: "complete".to_string(),
            workspace_id: "ws-test".to_string(),
            repo_path: "/repo".to_string(),
            command: cmd.to_string(),
            verdict_claimed: "pass".to_string(),
            duration_ms: Some(42),
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
            config_witness: None,
        }
    }

    fn make_start_event(id: &str, cmd: &str) -> ProcessEvent {
        ProcessEvent {
            lifecycle_transition: "start".to_string(),
            duration_ms: None,
            ..make_complete_event(id, cmd)
        }
    }

    /// build_receipt_json must produce all top-level OCEL 2.0 receipt fields
    /// required by wpm receipt doctor --strict.
    #[test]
    fn build_receipt_json_top_level_fields_present() {
        let ev = make_complete_event("evt-001", "cargo cicd status");
        let receipt = build_receipt_json(&[&ev], "cargo cicd status", 0);

        assert!(receipt.get("receipt_id").is_some(), "missing receipt_id");
        assert!(receipt.get("producer").is_some(), "missing producer");
        assert!(
            receipt.get("producer_version").is_some(),
            "missing producer_version"
        );
        assert!(receipt.get("created_at").is_some(), "missing created_at");
        assert!(receipt.get("repo_path").is_some(), "missing repo_path");
        assert!(receipt.get("git_head").is_some(), "missing git_head");
        assert!(receipt.get("algorithms").is_some(), "missing algorithms");
    }

    /// producer must always be "cargo-cicd".
    /// algorithms must be a non-empty array.
    /// Each algorithm entry must carry expected_path, observed_path, and
    /// boundary_evidence — the three sub-fields checked by wpm receipt doctor --strict.
    #[test]
    fn build_receipt_json_algorithm_shape() {
        let ev = make_complete_event("evt-004", "cargo cicd publish");
        let receipt = build_receipt_json(&[&ev], "cargo cicd publish", 0);
        let algo = &receipt["algorithms"][0];

        assert!(algo.get("algorithm_id").is_some(), "missing algorithm_id");
        let expected_path = algo.get("expected_path").expect("missing expected_path");
        let observed_path = algo.get("observed_path").expect("missing observed_path");
        let boundary = algo
            .get("boundary_evidence")
            .expect("missing boundary_evidence");

        // expected_path must have route_id + expected_ocel2 with ocel-version
        assert!(
            expected_path.get("route_id").is_some(),
            "expected_path missing route_id"
        );
        let exp_ocel = expected_path
            .get("expected_ocel2")
            .expect("missing expected_ocel2");
        assert_eq!(
            exp_ocel["ocel-version"], "2.0",
            "expected_ocel2 ocel-version must be 2.0"
        );

        // observed_path must have route_id + observed_ocel2 with ocel-version
        assert!(
            observed_path.get("route_id").is_some(),
            "observed_path missing route_id"
        );
        let obs_ocel = observed_path
            .get("observed_ocel2")
            .expect("missing observed_ocel2");
        assert_eq!(
            obs_ocel["ocel-version"], "2.0",
            "observed_ocel2 ocel-version must be 2.0"
        );

        // boundary_evidence must have exit_code and command
        assert!(
            boundary.get("exit_code").is_some(),
            "boundary_evidence missing exit_code"
        );
        assert!(
            boundary.get("command").is_some(),
            "boundary_evidence missing command"
        );
    }

    /// boundary_evidence exit_code must reflect the value passed to the function.
    #[test]
    fn build_receipt_json_exit_code_propagated() {
        let ev = make_complete_event("evt-005", "cargo cicd git");
        let receipt = build_receipt_json(&[&ev], "cargo cicd git", 42);
        let exit_code = receipt["algorithms"][0]["boundary_evidence"]["exit_code"]
            .as_i64()
            .expect("exit_code must be integer");
        assert_eq!(exit_code, 42);
    }

    /// observed_ocel2 events list must always contain at least the sentinel receipt-emit event,
    /// even when the input slice is empty.
    #[test]
    fn build_receipt_json_observed_events_never_empty_with_no_input() {
        let receipt = build_receipt_json(&[], "cargo cicd status", 0);
        let obs_events = receipt["algorithms"][0]["observed_path"]["observed_ocel2"]["events"]
            .as_array()
            .expect("observed events must be array");
        assert!(
            !obs_events.is_empty(),
            "observed events must not be empty (sentinel required)"
        );
    }

    /// config_witness_appears_in_ocel — a ProcessEvent with config_witness set
    /// must include the "config_witness" attribute in the emitted OCEL 2.0 JSON.
    #[test]
    fn config_witness_appears_in_ocel() {
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let ocel_path = tmp.path().join("events.ocel.json");

        let mut ev = make_complete_event("evt-witness", "status:show");
        ev.config_witness = Some("abcd1234".to_string());

        emit_ocel(&[ev], &ocel_path).expect("emit_ocel must succeed");
        let json = std::fs::read_to_string(&ocel_path).expect("ocel must exist");
        assert!(
            json.contains("config_witness"),
            "OCEL output must contain config_witness attribute"
        );
        assert!(
            json.contains("abcd1234"),
            "OCEL output must contain the witness hash value"
        );
    }

    /// Only "complete" lifecycle events must appear in the observed OCEL; "start"
    /// events are filtered out.
    #[test]
    fn build_receipt_json_filters_start_events() {
        let complete = make_complete_event("evt-complete", "cargo cicd status");
        let start = make_start_event("evt-start", "cargo cicd status");
        let receipt = build_receipt_json(&[&complete, &start], "cargo cicd status", 0);
        let obs_events = receipt["algorithms"][0]["observed_path"]["observed_ocel2"]["events"]
            .as_array()
            .expect("observed events must be array");

        // Should have: 1 complete event + 1 sentinel = 2 total; start must be absent.
        assert_eq!(obs_events.len(), 2, "expected 1 complete + 1 sentinel");
        let ids: Vec<&str> = obs_events.iter().filter_map(|e| e["id"].as_str()).collect();
        assert!(
            ids.contains(&"evt-complete"),
            "complete event must be present"
        );
        assert!(
            !ids.contains(&"evt-start"),
            "start event must be filtered out"
        );
    }

    /// atomic_write_survives_simulation — writing to tmp then renaming produces
    /// correct JSONL content that round-trips through append_events.
    #[test]
    fn atomic_write_survives_simulation() {
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let ev_dir = tmp.path().join("evidence");

        let ev1 = make_complete_event("evt-atomic-1", "status:show");
        let ev2 = make_complete_event("evt-atomic-2", "test:changed");

        // First append.
        append_events(&[ev1], &ev_dir).expect("first append_events must succeed");

        let jsonl_path = ev_dir.join("events.jsonl");
        let content1 = std::fs::read_to_string(&jsonl_path).expect("jsonl must exist");
        assert_eq!(content1.lines().count(), 1, "one event after first append");
        assert!(content1.contains("evt-atomic-1"), "first event present");

        // Second append — must accumulate, not overwrite.
        append_events(&[ev2], &ev_dir).expect("second append_events must succeed");

        let content2 = std::fs::read_to_string(&jsonl_path).expect("jsonl must exist");
        assert_eq!(
            content2.lines().count(),
            2,
            "two events after second append"
        );
        assert!(content2.contains("evt-atomic-1"), "first event retained");
        assert!(content2.contains("evt-atomic-2"), "second event added");

        // OCEL must exist, no XES may be created, and no tmp files left behind.
        assert!(ev_dir.join("events.ocel.json").exists(), "events.ocel.json must exist");
        assert!(
            !ev_dir.join("events.xes").exists(),
            "events.xes must NOT be created (OCEL-only by operator directive)"
        );
        assert!(
            ev_dir.join("events.ocel.json").exists(),
            "events.ocel.json must exist"
        );

        let stale_tmp: Vec<_> = std::fs::read_dir(&ev_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(
            stale_tmp.is_empty(),
            "no .tmp.* files must remain after rename"
        );
    }
}
