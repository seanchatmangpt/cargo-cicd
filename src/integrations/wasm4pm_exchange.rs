//! SHELL_OUT integration adapter for wasm4pm (wpm) binary.
//!
//! Implements a four-stage shell-out pipeline for cargo-cicd v26.6.2:
//!
//! 1. PRE-FLIGHT     — `wpm doctor` from cargo project root
//! 2. CONFORMANCE    — `wpm audit <xes> --activity-key concept:name`
//! 3. RECEIPT DOCTOR — `wpm receipt doctor <receipt.json> --audience ci --format json`
//! 4. TELCO HEALTH   — `wpm telco status`
//!
//! NOTE: `wpm mining conformance`, `wpm oracle check`, and `wpm oracle watch` are
//! confirmed stubs returning exit code 0 regardless of input — they are NOT invoked
//! here to avoid false-positive CI passes.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// ProcessEvent — file exchange artifact written to JSONL
// ---------------------------------------------------------------------------

/// A single process event emitted by the wasm4pm integration pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    /// Stage identifier: "pre_flight" | "conformance_audit" | "receipt_doctor" | "telco_health"
    pub kind: String,
    /// RFC 3339 timestamp at the moment the event was recorded
    pub timestamp: String,
    /// Verdict: "PASS" | "WARN" | "FAIL" | "TRUTHFUL" | "VARIANCE" | "DECEPTIVE" | "ACTIVE" | "DEGRADED"
    pub verdict: String,
    /// Human-readable detail line from wpm stdout (may be None on clean pass)
    pub details: Option<String>,
}

// ---------------------------------------------------------------------------
// File-exchange helpers
// ---------------------------------------------------------------------------

/// Returns the canonical path for the JSONL process-event exchange file.
///
/// Path: `target/cargo-cicd/process/events.jsonl` relative to the project root
/// supplied by the caller (or the current working directory when `None`).
pub fn exchange_path(project_root: Option<&Path>) -> PathBuf {
    let base = project_root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("target/cargo-cicd/process/events.jsonl")
}

/// Appends a slice of [`ProcessEvent`] records to the JSONL exchange file,
/// creating parent directories as needed.
pub fn emit_events_jsonl(events: &[ProcessEvent], project_root: Option<&Path>) -> Result<()> {
    let path = exchange_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create_dir_all: {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open JSONL exchange file: {}", path.display()))?;
    for event in events {
        let line = serde_json::to_string(event)
            .with_context(|| "serialize ProcessEvent to JSON")?;
        writeln!(file, "{}", line)
            .with_context(|| format!("write to JSONL exchange file: {}", path.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// wpm binary detection
// ---------------------------------------------------------------------------

/// Resolves the path to the `wpm` binary.
///
/// Resolution order:
///   1. `WPM_BIN` environment variable (allows test injection)
///   2. `/Users/sac/wasm4pm/target/release/wpm` (known release path)
///   3. `which wpm` via PATH
///
/// Returns `Err` if the binary cannot be found.
pub fn resolve_wpm_bin() -> Result<PathBuf> {
    // 1. Environment override
    if let Ok(val) = std::env::var("WPM_BIN") {
        let p = PathBuf::from(&val);
        if p.is_file() {
            return Ok(p);
        }
        bail!("WPM_BIN={} is set but the file does not exist", val);
    }

    // 2. Known release path
    let known = PathBuf::from("/Users/sac/wasm4pm/target/release/wpm");
    if known.is_file() {
        return Ok(known);
    }

    // 3. PATH lookup — avoid the `which` crate dependency; shell out once
    let output = Command::new("sh")
        .args(["-c", "which wpm"])
        .output()
        .context("failed to invoke 'which wpm'")?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Ok(PathBuf::from(path_str));
        }
    }

    bail!(
        "wpm binary not found: set WPM_BIN, install wasm4pm to /Users/sac/wasm4pm/target/release/wpm, or add wpm to PATH"
    )
}

// ---------------------------------------------------------------------------
// Stage 1 — PRE-FLIGHT: `wpm doctor`
// ---------------------------------------------------------------------------

/// Runs `wpm doctor` from `project_root`.
///
/// Returns `Ok(())` if the exit code is 0 (all PASS).
/// Any non-zero exit code propagates as a CI failure.
pub fn stage_preflight(wpm: &Path, project_root: &Path) -> Result<ProcessEvent> {
    let output = Command::new(wpm)
        .arg("doctor")
        .current_dir(project_root)
        .output()
        .with_context(|| format!("spawn wpm doctor in {}", project_root.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    let (verdict, details) = if output.status.success() {
        ("PASS".to_string(), None)
    } else {
        (
            "FAIL".to_string(),
            Some(first_fail_line(&combined).unwrap_or_else(|| combined.clone())),
        )
    };

    if verdict == "FAIL" {
        bail!(
            "PRE-FLIGHT FAILED — wpm doctor exited {}\n{}",
            output.status,
            combined
        );
    }

    Ok(ProcessEvent {
        kind: "pre_flight".to_string(),
        timestamp: now_rfc3339(),
        verdict,
        details,
    })
}

// ---------------------------------------------------------------------------
// Stage 2 — CONFORMANCE AUDIT GATE: `wpm audit <xes> --activity-key concept:name`
// ---------------------------------------------------------------------------

/// Conformance verdict after parsing `wpm audit` output.
#[derive(Debug, PartialEq, Eq)]
pub enum ConformanceVerdict {
    /// fitness >= 0.95
    Truthful,
    /// 0.70 <= fitness < 0.95
    Variance,
    /// fitness < 0.70
    Deceptive,
}

/// Parses a fitness score from `wpm audit` stdout.
///
/// The line of interest is:
/// ```text
/// Fitness Score:            0.9712
/// ```
fn parse_fitness(stdout: &str) -> Option<f64> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Fitness Score:") {
            let score_str = trimmed
                .trim_start_matches("Fitness Score:")
                .trim();
            return score_str.parse::<f64>().ok();
        }
    }
    None
}

/// Runs `wpm audit <xes_path> --activity-key concept:name`.
///
/// Gate rules:
/// - fitness >= 0.95 → TRUTHFUL (OK)
/// - 0.70 <= fitness < 0.95 → VARIANCE (warn, CI continues)
/// - fitness < 0.70 → DECEPTIVE (CI failure)
///
/// The XES artifact at `xes_path` must be written by the cargo test harness
/// as a file exchange artifact before this stage runs.
pub fn stage_conformance_audit(wpm: &Path, xes_path: &Path) -> Result<ProcessEvent> {
    if !xes_path.exists() {
        bail!(
            "CONFORMANCE AUDIT GATE FAILED — XES event log not found at {}. \
             The cargo test harness must write this artifact before the audit stage runs.",
            xes_path.display()
        );
    }

    let output = Command::new(wpm)
        .args(["audit", xes_path.to_str().unwrap_or(""), "--activity-key", "concept:name"])
        .output()
        .with_context(|| format!("spawn wpm audit {}", xes_path.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let fitness = parse_fitness(&stdout).unwrap_or(0.0);

    let cv = if fitness >= 0.95 {
        ConformanceVerdict::Truthful
    } else if fitness >= 0.70 {
        ConformanceVerdict::Variance
    } else {
        ConformanceVerdict::Deceptive
    };

    let (verdict, details) = match cv {
        ConformanceVerdict::Truthful => (
            "TRUTHFUL".to_string(),
            Some(format!("fitness={:.4}", fitness)),
        ),
        ConformanceVerdict::Variance => (
            "VARIANCE".to_string(),
            Some(format!(
                "fitness={:.4} — below 0.95 threshold (WARN); stderr={}",
                fitness, stderr.trim()
            )),
        ),
        ConformanceVerdict::Deceptive => {
            let detail = format!(
                "fitness={:.4} — below 0.70 threshold (DECEPTIVE)\nwpm stdout:\n{}\nwpm stderr:\n{}",
                fitness, stdout.trim(), stderr.trim()
            );
            bail!("CONFORMANCE AUDIT GATE FAILED — DECEPTIVE\n{}", detail);
        }
    };

    Ok(ProcessEvent {
        kind: "conformance_audit".to_string(),
        timestamp: now_rfc3339(),
        verdict,
        details,
    })
}

// ---------------------------------------------------------------------------
// Stage 3 — RECEIPT DOCTOR GATE
// ---------------------------------------------------------------------------

/// Minimal subset of the `wpm receipt doctor --format json` response.
#[derive(Debug, Deserialize)]
struct ReceiptDoctorResponse {
    state: String,
    findings: Vec<ReceiptFinding>,
}

#[derive(Debug, Deserialize)]
struct ReceiptFinding {
    severity: String,
    message: String,
    #[allow(dead_code)]
    code: Option<String>,
    #[allow(dead_code)]
    json_path: Option<String>,
}

/// Runs `wpm receipt doctor <receipt_path> --audience ci --format json`.
///
/// Gate rules:
/// - `state == "Refused"` → CI failure
/// - any finding with `severity == "Deny"` → CI failure
/// - `severity == "Warn"` findings → collected into `details`, CI continues
pub fn stage_receipt_doctor(wpm: &Path, receipt_path: &Path) -> Result<ProcessEvent> {
    if !receipt_path.exists() {
        bail!(
            "RECEIPT DOCTOR GATE FAILED — receipt file not found at {}",
            receipt_path.display()
        );
    }

    let output = Command::new(wpm)
        .args([
            "receipt",
            "doctor",
            receipt_path.to_str().unwrap_or(""),
            "--audience",
            "ci",
            "--format",
            "json",
        ])
        .output()
        .with_context(|| format!("spawn wpm receipt doctor {}", receipt_path.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Parse JSON response — strip any trailing non-JSON (wpm prints an error line after JSON on refusal)
    let json_text = extract_json_object(&stdout).unwrap_or(stdout.as_str());
    let response: ReceiptDoctorResponse = serde_json::from_str(json_text)
        .with_context(|| format!("parse wpm receipt doctor JSON; raw output:\n{}", stdout))?;

    // Gate: refused state
    if response.state == "Refused" {
        let deny_messages: Vec<String> = response
            .findings
            .iter()
            .filter(|f| f.severity == "Deny")
            .map(|f| format!("[Deny] {}", f.message))
            .collect();
        bail!(
            "RECEIPT DOCTOR GATE FAILED — state=Refused\n{}",
            deny_messages.join("\n")
        );
    }

    // Gate: any Deny-severity finding
    let deny_findings: Vec<&ReceiptFinding> = response
        .findings
        .iter()
        .filter(|f| f.severity == "Deny")
        .collect();
    if !deny_findings.is_empty() {
        let msgs: Vec<String> = deny_findings
            .iter()
            .map(|f| format!("[Deny] {}", f.message))
            .collect();
        bail!(
            "RECEIPT DOCTOR GATE FAILED — Deny findings present\n{}",
            msgs.join("\n")
        );
    }

    // Collect warnings for build summary attachment
    let warn_messages: Vec<String> = response
        .findings
        .iter()
        .filter(|f| f.severity == "Warn")
        .map(|f| format!("[Warn] {}", f.message))
        .collect();

    let details = if warn_messages.is_empty() {
        Some(format!("state={}", response.state))
    } else {
        Some(format!(
            "state={} — warnings:\n{}",
            response.state,
            warn_messages.join("\n")
        ))
    };

    Ok(ProcessEvent {
        kind: "receipt_doctor".to_string(),
        timestamp: now_rfc3339(),
        verdict: "PASS".to_string(),
        details,
    })
}

// ---------------------------------------------------------------------------
// Stage 4 — TELCO HEALTH: `wpm telco status`
// ---------------------------------------------------------------------------

/// Parses the "Operational State:" line from `wpm telco status` stdout.
fn parse_operational_state(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Operational State:") {
            let state = trimmed
                .trim_start_matches("Operational State:")
                .trim()
                .to_string();
            return Some(state);
        }
    }
    None
}

/// Runs `wpm telco status`.
///
/// Gate rule: "Operational State" must be "ACTIVE".
/// Any other value (e.g. "DEGRADED") propagates as a CI failure.
pub fn stage_telco_health(wpm: &Path) -> Result<ProcessEvent> {
    let output = Command::new(wpm)
        .args(["telco", "status"])
        .output()
        .context("spawn wpm telco status")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let op_state = parse_operational_state(&stdout)
        .unwrap_or_else(|| format!("UNKNOWN (stdout: {}; stderr: {})", stdout.trim(), stderr.trim()));

    if op_state != "ACTIVE" {
        bail!(
            "TELCO HEALTH GATE FAILED — Operational State={}\nwpm telco status output:\n{}",
            op_state,
            stdout
        );
    }

    Ok(ProcessEvent {
        kind: "telco_health".to_string(),
        timestamp: now_rfc3339(),
        verdict: "ACTIVE".to_string(),
        details: Some(format!("Operational State={}", op_state)),
    })
}

// ---------------------------------------------------------------------------
// Pipeline orchestrator
// ---------------------------------------------------------------------------

/// Runs the full four-stage wasm4pm shell-out pipeline.
///
/// Parameters:
/// - `project_root`  — cargo project root (for `wpm doctor` and `exchange_path`)
/// - `xes_path`      — path to the XES event log artifact written by the test harness
///                     (typically `target/process-intelligence/ci-run.xes`)
/// - `receipt_path`  — path to the checkpoint receipt JSON to audit (may be `None`
///                     to skip Stage 3 when no receipt has been emitted yet)
///
/// All events are appended to the JSONL exchange file regardless of pass/fail
/// up to the point of failure.
pub fn run_pipeline(
    project_root: &Path,
    xes_path: &Path,
    receipt_path: Option<&Path>,
) -> Result<Vec<ProcessEvent>> {
    let wpm = resolve_wpm_bin()?;
    let mut events: Vec<ProcessEvent> = Vec::new();

    // Stage 1 — PRE-FLIGHT
    let e1 = stage_preflight(&wpm, project_root)?;
    events.push(e1);
    emit_events_jsonl(&events[events.len() - 1..], Some(project_root))?;

    // Stage 2 — CONFORMANCE AUDIT GATE
    let e2 = stage_conformance_audit(&wpm, xes_path)?;
    let is_variance = e2.verdict == "VARIANCE";
    events.push(e2);
    emit_events_jsonl(&events[events.len() - 1..], Some(project_root))?;
    if is_variance {
        // Print warning but continue — VARIANCE does not fail CI
        eprintln!(
            "WARNING: conformance audit returned VARIANCE — {}",
            events.last().and_then(|e| e.details.as_deref()).unwrap_or("")
        );
    }

    // Stage 3 — RECEIPT DOCTOR GATE (optional — skip if no receipt path provided)
    if let Some(rp) = receipt_path {
        let e3 = stage_receipt_doctor(&wpm, rp)?;
        events.push(e3);
        emit_events_jsonl(&events[events.len() - 1..], Some(project_root))?;
    }

    // Stage 4 — TELCO HEALTH
    let e4 = stage_telco_health(&wpm)?;
    events.push(e4);
    emit_events_jsonl(&events[events.len() - 1..], Some(project_root))?;

    Ok(events)
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn now_rfc3339() -> String {
    // Use a simple system call to avoid pulling in chrono/time as a dependency.
    // Falls back to epoch string on error.
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => "1970-01-01T00:00:00Z".to_string(),
    }
}

/// Extracts the first JSON object `{...}` from a string that may have trailing text.
fn extract_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    // Walk backwards from end to find the matching closing brace
    let end = s.rfind('}')?;
    if end >= start {
        Some(&s[start..=end])
    } else {
        None
    }
}

/// Returns the first line containing "[FAIL]" from the output, if any.
fn first_fail_line(output: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains("[FAIL]"))
        .map(|l| l.trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fitness_truthful() {
        let stdout = "Vision 2030 Conformance Audit Report\n\nAudit Verdict:            TRUTHFUL\nFitness Score:            0.9712\nPrecision Score:          0.8800\n";
        let f = parse_fitness(stdout);
        assert_eq!(f, Some(0.9712));
    }

    #[test]
    fn test_parse_fitness_deceptive() {
        let stdout = "Audit Verdict:            DECEPTIVE\nFitness Score:            0.0000\nPrecision Score:          0.0000\n";
        let f = parse_fitness(stdout);
        assert_eq!(f, Some(0.0));
    }

    #[test]
    fn test_conformance_verdict_thresholds() {
        let verdict_for = |f: f64| -> &'static str {
            if f >= 0.95 {
                "TRUTHFUL"
            } else if f >= 0.70 {
                "VARIANCE"
            } else {
                "DECEPTIVE"
            }
        };
        assert_eq!(verdict_for(0.9712), "TRUTHFUL");
        assert_eq!(verdict_for(0.95), "TRUTHFUL");
        assert_eq!(verdict_for(0.94), "VARIANCE");
        assert_eq!(verdict_for(0.70), "VARIANCE");
        assert_eq!(verdict_for(0.6999), "DECEPTIVE");
        assert_eq!(verdict_for(0.0), "DECEPTIVE");
    }

    #[test]
    fn test_parse_operational_state_active() {
        let stdout = "--- WASM4PM TELCO ROUTER STATUS ---\nOperational State:        ACTIVE\nLoop Latency (Target):    34 ns\n";
        assert_eq!(parse_operational_state(stdout), Some("ACTIVE".to_string()));
    }

    #[test]
    fn test_parse_operational_state_degraded() {
        let stdout = "Operational State:        DEGRADED\n";
        assert_eq!(
            parse_operational_state(stdout),
            Some("DEGRADED".to_string())
        );
    }

    #[test]
    fn test_parse_operational_state_missing() {
        let stdout = "no state line here\n";
        assert_eq!(parse_operational_state(stdout), None);
    }

    #[test]
    fn test_extract_json_object() {
        let s = r#"{"state":"Accepted","findings":[]}\nerror: something"#;
        let j = extract_json_object(s);
        assert!(j.is_some());
        assert!(j.unwrap().starts_with('{'));
        assert!(j.unwrap().ends_with('}'));
    }

    #[test]
    fn test_exchange_path_default() {
        let p = exchange_path(None);
        assert!(p.ends_with("target/cargo-cicd/process/events.jsonl"));
    }

    #[test]
    fn test_process_event_roundtrip() {
        let e = ProcessEvent {
            kind: "telco_health".to_string(),
            timestamp: "2026-06-02T00:00:00Z".to_string(),
            verdict: "ACTIVE".to_string(),
            details: Some("Operational State=ACTIVE".to_string()),
        };
        let json = serde_json::to_string(&e).unwrap();
        let e2: ProcessEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e2.kind, "telco_health");
        assert_eq!(e2.verdict, "ACTIVE");
    }
}
