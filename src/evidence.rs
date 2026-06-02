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
//! - **E5**: XES emission is append-safe. Each call to `emit_xes` produces a
//!   complete, self-contained log for the event slice passed.
//! - **E6**: JSONL emission mirrors XES — same event set, machine-readable
//!   companion format for downstream tooling.
//! - **E7**: `ExpectedWpmVerdict::Blocked` is a first-class expectation, not
//!   an error state. Tests that run without wpm installed MUST declare
//!   `Blocked` as their expected verdict.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::integrations::{Wasm4pmShell, WpmVerdict};

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single process event emitted by cargo-cicd for wasm4pm adjudication.
pub struct ProcessEvent {
    pub event_id: String,
    pub timestamp_iso: String,
    pub workspace_id: String,
    pub repo_path: String,
    pub command: String,
    pub verdict_claimed_by_cargo_cicd: String,
    pub duration_ms: u64,
}

impl ProcessEvent {
    /// Construct a new `ProcessEvent` with canonical defaults.
    ///
    /// - `event_id` is `"evt-{command}"` with spaces replaced by dashes.
    /// - `timestamp_iso` is fixed to `"2026-06-02T00:00:00.000Z"`.
    /// - `workspace_id` is `"cargo-cicd-workspace"`.
    /// - `repo_path` is `"."`.
    /// - `duration_ms` is `0`.
    pub fn new(command: &str, verdict: &str) -> Self {
        Self {
            event_id: format!("evt-{}", command.replace(' ', "-")),
            timestamp_iso: "2026-06-02T00:00:00.000Z".to_string(),
            workspace_id: "cargo-cicd-workspace".to_string(),
            repo_path: ".".to_string(),
            command: command.to_string(),
            verdict_claimed_by_cargo_cicd: verdict.to_string(),
            duration_ms: 0,
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

/// Emit a minimal valid XES event log to `path`.
///
/// The file is created (with parent directories) and overwritten if it exists.
/// Each `ProcessEvent` becomes one `<event>` inside a single `<trace>`.
pub fn emit_xes(events: &[ProcessEvent], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<log xes.version=\"1.0\" xes.features=\"\">\n");
    xml.push_str("  <extension name=\"Concept\" prefix=\"concept\" uri=\"http://www.xes-standard.org/concept.xesext\"/>\n");
    xml.push_str("  <extension name=\"Time\" prefix=\"time\" uri=\"http://www.xes-standard.org/time.xesext\"/>\n");
    xml.push_str("  <trace>\n");
    xml.push_str("    <string key=\"concept:name\" value=\"cargo-cicd-run\"/>\n");

    for event in events {
        xml.push_str("    <event>\n");
        xml.push_str(&format!(
            "      <string key=\"concept:name\" value=\"{}\"/>\n",
            escape_xml(&event.command)
        ));
        xml.push_str(&format!(
            "      <string key=\"cargo_cicd:verdict\" value=\"{}\"/>\n",
            escape_xml(&event.verdict_claimed_by_cargo_cicd)
        ));
        xml.push_str(&format!(
            "      <date key=\"time:timestamp\" value=\"{}\"/>\n",
            escape_xml(&event.timestamp_iso)
        ));
        xml.push_str("      <string key=\"lifecycle:transition\" value=\"complete\"/>\n");
        xml.push_str("    </event>\n");
    }

    xml.push_str("  </trace>\n");
    xml.push_str("</log>\n");

    std::fs::write(path, xml)?;
    Ok(())
}

/// Emit events as newline-delimited JSON to `path`.
///
/// Each line is a JSON object with `event_id`, `command`, and
/// `verdict_claimed_by_cargo_cicd` fields.
pub fn emit_events_jsonl(events: &[ProcessEvent], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = String::new();
    for event in events {
        lines.push_str(&format!(
            "{{\"event_id\":{},\"command\":{},\"verdict_claimed_by_cargo_cicd\":{}}}\n",
            json_string(&event.event_id),
            json_string(&event.command),
            json_string(&event.verdict_claimed_by_cargo_cicd),
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
