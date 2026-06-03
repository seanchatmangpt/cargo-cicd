//! Evidence Gate Architecture — cargo-cicd emits; wasm4pm adjudicates.
//!
//! ## Invariants
//!
//! - **E1**: cargo-cicd NEVER adjudicates its own process conformance.
//!   All verdicts are issued by the external wasm4pm oracle.
//! - **E2**: Evidence is emitted before adjudication. The XES file must exist
//!   on disk before `audit_xes` is called.
//! - **E3**: If the oracle is unavailable and the expected verdict is not
//!   `Blocked`, the evidence gate panics. Certification requires the oracle.
//! - **E4**: Tests assert only wasm4pm verdict, never internal cargo-cicd state.
//!   cargo-cicd state assertions belong in unit tests; process conformance
//!   assertions belong in evidence-gate tests.
//! - **E5**: XES emission groups events by `case_id` into separate `<trace>`
//!   elements. Events without a `case_id` go into a default trace.
//! - **E6**: JSONL emission mirrors XES — same event set, machine-readable
//!   companion format for downstream tooling.
//! - **E7**: `ExpectedWpmVerdict::Blocked` is a first-class expectation, not
//!   an error state. Tests that run without wpm installed MUST declare
//!   `Blocked` as their expected verdict.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::integrations::{Wasm4pmShell, WpmVerdict};

// ── Timestamp helpers (std-only, no chrono) ───────────────────────────────────

/// Return the current UTC time as an ISO-8601 string, e.g.
/// `"2026-06-02T13:45:07.123Z"`.
pub fn now_iso8601() -> String {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let ms = d.subsec_millis();
    let (y, mo, day) = epoch_secs_to_ymd(secs);
    let h = (secs % 86400) / 3600;
    let mi = (secs % 3600) / 60;
    let s = secs % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, mo, day, h, mi, s, ms
    )
}

fn epoch_secs_to_ymd(secs: u64) -> (u64, u64, u64) {
    let mut days = secs / 86400;
    let mut y = 1970u64;
    loop {
        let dy = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
            366
        } else {
            365
        };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mdays: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u64;
    for md in &mdays {
        if days < *md {
            break;
        }
        days -= md;
        mo += 1;
    }
    (y, mo, days + 1)
}

/// Build a compact event-id from the current timestamp (no special chars).
fn new_event_id(command: &str) -> String {
    let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
    format!("evt-{}-{}", command.replace(' ', "-"), ts)
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single process event emitted by cargo-cicd for wasm4pm adjudication.
pub struct ProcessEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// ISO-8601 UTC timestamp captured at event construction time.
    pub timestamp_iso: String,
    /// Session / case grouping key. Events with the same `case_id` are written
    /// into the same XES `<trace>`.
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
}

impl ProcessEvent {
    /// Construct a completed `ProcessEvent` with the current real timestamp.
    ///
    /// `verdict` should be `"PASS"`, `"WARN"`, or `"FAIL"`.
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

/// Runtime oracle that shells out to `wpm` for XES adjudication.
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

    /// Audit an XES file and map the wpm verdict to `ExpectedWpmVerdict`.
    ///
    /// - Binary absent → `Blocked`
    /// - Invocation error → `Refuse`
    /// - `Pass | Warn | Partial` → `Accept`
    /// - `Fail` → `Refuse`
    /// - `NotAvailable` → `Blocked`
    pub fn audit_xes(&self, xes_path: &Path) -> ExpectedWpmVerdict {
        match &self.shell {
            None => ExpectedWpmVerdict::Blocked,
            Some(wpm) => match wpm.audit(xes_path.to_str().unwrap_or("")) {
                Err(_) => ExpectedWpmVerdict::Refuse,
                Ok(result) => match result.verdict {
                    WpmVerdict::Pass | WpmVerdict::Warn | WpmVerdict::Partial => {
                        ExpectedWpmVerdict::Accept
                    }
                    WpmVerdict::Fail => ExpectedWpmVerdict::Refuse,
                    WpmVerdict::NotAvailable => ExpectedWpmVerdict::Blocked,
                },
            },
        }
    }
}

// ── Emission ──────────────────────────────────────────────────────────────────

/// Emit a valid XES event log to `path`, grouping events by `case_id`.
///
/// - Events that share the same `case_id` (or both have `None`) are placed in
///   the same `<trace>`.
/// - The file is always overwritten (no partial append).
pub fn emit_xes(events: &[ProcessEvent], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Group events by case_id, preserving insertion order.
    let mut case_order: Vec<String> = Vec::new();
    let mut by_case: std::collections::HashMap<String, Vec<&ProcessEvent>> =
        std::collections::HashMap::new();

    for ev in events {
        let key = ev
            .case_id
            .clone()
            .unwrap_or_else(|| "cargo-cicd-run".to_string());
        if !by_case.contains_key(&key) {
            case_order.push(key.clone());
        }
        by_case.entry(key).or_default().push(ev);
    }

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<log xes.version=\"1.0\" xes.features=\"\">\n");
    xml.push_str("  <extension name=\"Concept\" prefix=\"concept\" uri=\"http://www.xes-standard.org/concept.xesext\"/>\n");
    xml.push_str("  <extension name=\"Time\" prefix=\"time\" uri=\"http://www.xes-standard.org/time.xesext\"/>\n");
    xml.push_str("  <extension name=\"Lifecycle\" prefix=\"lifecycle\" uri=\"http://www.xes-standard.org/lifecycle.xesext\"/>\n");

    for case_id in &case_order {
        let trace_events = &by_case[case_id];
        xml.push_str("  <trace>\n");
        xml.push_str(&format!(
            "    <string key=\"concept:name\" value=\"{}\"/>\n",
            escape_xml(case_id)
        ));

        for event in trace_events {
            xml.push_str("    <event>\n");
            xml.push_str(&format!(
                "      <string key=\"concept:name\" value=\"{}\"/>\n",
                escape_xml(&event.command)
            ));
            xml.push_str(&format!(
                "      <date key=\"time:timestamp\" value=\"{}\"/>\n",
                escape_xml(&event.timestamp_iso)
            ));
            xml.push_str(&format!(
                "      <string key=\"lifecycle:transition\" value=\"{}\"/>\n",
                escape_xml(&event.lifecycle_transition)
            ));
            xml.push_str(&format!(
                "      <string key=\"cargo_cicd:verdict_claimed\" value=\"{}\"/>\n",
                escape_xml(&event.verdict_claimed)
            ));
            if let Some(ms) = event.duration_ms {
                xml.push_str(&format!(
                    "      <int key=\"cargo_cicd:duration_ms\" value=\"{}\"/>\n",
                    ms
                ));
            }
            if let Some(ref v) = event.verdict_adjudicated {
                xml.push_str(&format!(
                    "      <string key=\"wasm4pm:verdict_adjudicated\" value=\"{}\"/>\n",
                    escape_xml(v)
                ));
            }
            if let Some(ref ts) = event.adjudicated_at {
                xml.push_str(&format!(
                    "      <string key=\"wasm4pm:adjudicated_at\" value=\"{}\"/>\n",
                    escape_xml(ts)
                ));
            }
            if let Some(ref oracle) = event.oracle_command {
                xml.push_str(&format!(
                    "      <string key=\"wasm4pm:oracle_command\" value=\"{}\"/>\n",
                    escape_xml(oracle)
                ));
            }
            xml.push_str("    </event>\n");
        }

        xml.push_str("  </trace>\n");
    }

    xml.push_str("</log>\n");

    std::fs::write(path, xml)?;
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

// ── Assertion ─────────────────────────────────────────────────────────────────

/// Assert that the wasm4pm oracle returns the expected verdict for an XES file.
///
/// Panics with a detailed message if:
/// - The oracle is `Blocked` and the expected verdict is not `Blocked` (E3 violation).
/// - The actual verdict does not match the expected verdict.
pub fn assert_wpm_verdict(
    oracle: &WpmEvidenceOracle,
    evidence_path: &Path,
    expected: &ExpectedWpmVerdict,
) {
    let actual = oracle.audit_xes(evidence_path);

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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

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
