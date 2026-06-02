//! Process evidence emission layer for cargo-cicd.
//!
//! This module **emits** structured process events to
//! `target/cargo-cicd/evidence/events.jsonl`. It does not adjudicate verdicts.
//! Adjudication belongs exclusively to wasm4pm. Tests must assert only the
//! wasm4pm verdict, never the raw event content.
//!
//! ## Law
//!
//! cargo-cicd emits. wasm4pm adjudicates. Tests assert only the wasm4pm verdict.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic counter for event index within a process lifetime.
static EVENT_INDEX: AtomicU64 = AtomicU64::new(0);

/// A single process event emitted by cargo-cicd.
///
/// This is a structural record only — no engine logic, no discovery, no
/// conformance checking. Those operations graduate to wasm4pm.
///
/// ## Identity
///
/// `event_id` is `evt_{timestamp_ns}_{index}` where `index` is monotonically
/// increasing within the process. This is not a UUID but is unique within a
/// single cargo-cicd invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    /// Unique event identifier: `evt_{timestamp_ns}_{index}`.
    pub event_id: String,
    /// ISO-8601 timestamp of event creation (UTC, second precision).
    pub timestamp_iso: String,
    /// Workspace identifier derived from `Cargo.toml` name and root path hash.
    pub workspace_id: String,
    /// Absolute path to the repository root.
    pub repo_path: String,
    /// The cargo-cicd command that produced this event (e.g. `"status show"`).
    pub command: String,
    /// Structured inputs passed to the command.
    pub inputs: serde_json::Value,
    /// Structured outputs produced by the command.
    pub outputs: serde_json::Value,
    /// The verdict claimed by cargo-cicd: `"PASS"`, `"WARN"`, `"FAIL"`, or `"PARTIAL"`.
    /// wasm4pm may override this verdict during adjudication.
    pub verdict_claimed: String,
    /// Wall-clock duration of the command in milliseconds.
    pub duration_ms: u64,
    /// Paths of files emitted as artifacts by this command.
    pub artifacts: Vec<String>,
}

impl ProcessEvent {
    /// Construct a new `ProcessEvent` with a generated `event_id` and current timestamp.
    ///
    /// ```ignore
    /// // Requires a running process context; see EvidenceEmitter for the standard path.
    /// let evt = ProcessEvent::new("status show", serde_json::json!({}), serde_json::json!({}), "PASS", 42);
    /// ```
    pub fn new(
        command: impl Into<String>,
        inputs: serde_json::Value,
        outputs: serde_json::Value,
        verdict_claimed: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_ns = now.as_nanos() as u64;
        let index = EVENT_INDEX.fetch_add(1, Ordering::Relaxed);
        let event_id = format!("evt_{}_{}", timestamp_ns, index);
        let timestamp_iso = format_timestamp_iso(now.as_secs());
        let repo_path = std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            event_id,
            timestamp_iso,
            workspace_id: workspace_id(),
            repo_path,
            command: command.into(),
            inputs,
            outputs,
            verdict_claimed: verdict_claimed.into(),
            duration_ms,
            artifacts: Vec::new(),
        }
    }

    /// Attach artifact paths to this event.
    ///
    /// ```ignore
    /// let evt = ProcessEvent::new("publish run", serde_json::json!({}), serde_json::json!({}), "PASS", 0)
    ///     .with_artifacts(vec!["target/cargo-cicd/evidence/events.jsonl".to_string()]);
    /// ```
    pub fn with_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.artifacts = artifacts;
        self
    }
}

/// Emits [`ProcessEvent`] records to `target/cargo-cicd/evidence/events.jsonl`.
///
/// Creates the evidence directory on first use. Each call to [`emit`][Self::emit]
/// appends one JSON line to `events.jsonl`. The emitter is stateless across
/// invocations — it does not buffer events in memory.
///
/// ## Directory layout
///
/// ```text
/// target/cargo-cicd/evidence/
///   events.jsonl      ← append-only JSONL event stream
///   receipts/         ← per-command receipt files (from emit_receipt)
/// ```
pub struct EvidenceEmitter {
    /// Absolute path to the evidence directory.
    pub dir: PathBuf,
}

impl EvidenceEmitter {
    /// Construct an `EvidenceEmitter` rooted at the standard evidence directory.
    ///
    /// ```ignore
    /// let emitter = EvidenceEmitter::new();
    /// ```
    pub fn new() -> Self {
        Self { dir: evidence_dir() }
    }

    /// Construct an `EvidenceEmitter` rooted at a custom directory.
    ///
    /// Useful in tests to redirect output to a temporary directory.
    ///
    /// ```ignore
    /// let emitter = EvidenceEmitter::with_dir(tempdir.path().to_path_buf());
    /// ```
    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Append `event` as one JSON line to `events.jsonl`.
    ///
    /// Creates the evidence directory if it does not yet exist.
    /// Returns the path of `events.jsonl`.
    ///
    /// ```ignore
    /// let emitter = EvidenceEmitter::new();
    /// let evt = ProcessEvent::new("status show", serde_json::json!({}), serde_json::json!({}), "PASS", 10);
    /// let path = emitter.emit(evt)?;
    /// ```
    pub fn emit(&self, event: ProcessEvent) -> Result<PathBuf> {
        self.ensure_dir()?;
        let events_path = self.dir.join("events.jsonl");
        let line = serde_json::to_string(&event)
            .context("failed to serialize ProcessEvent to JSON")?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&events_path)
            .with_context(|| format!("failed to open events.jsonl at {}", events_path.display()))?;
        writeln!(file, "{}", line)
            .with_context(|| format!("failed to write event to {}", events_path.display()))?;
        Ok(events_path)
    }

    /// Emit a lightweight receipt file for `command` with a `verdict` and free-form `details`.
    ///
    /// Receipt files are written to `receipts/{command_slug}_{timestamp_ns}.json`
    /// within the evidence directory. The `command_slug` is the command string
    /// with spaces replaced by underscores.
    ///
    /// Returns the path of the written receipt file.
    ///
    /// ```ignore
    /// let emitter = EvidenceEmitter::new();
    /// emitter.emit_receipt("status show", "PASS", "all checks green")?;
    /// ```
    pub fn emit_receipt(&self, command: &str, verdict: &str, details: &str) -> Result<PathBuf> {
        self.ensure_dir()?;
        let receipts_dir = self.dir.join("receipts");
        fs::create_dir_all(&receipts_dir)
            .with_context(|| format!("failed to create receipts dir at {}", receipts_dir.display()))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_ns = now.as_nanos() as u64;
        let slug = command.replace(' ', "_");
        let filename = format!("{}_{}.json", slug, timestamp_ns);
        let receipt_path = receipts_dir.join(&filename);

        let receipt = serde_json::json!({
            "command": command,
            "verdict": verdict,
            "details": details,
            "timestamp_iso": format_timestamp_iso(now.as_secs()),
            "workspace_id": workspace_id(),
        });
        let content = serde_json::to_string_pretty(&receipt)
            .context("failed to serialize receipt")?;
        fs::write(&receipt_path, content)
            .with_context(|| format!("failed to write receipt to {}", receipt_path.display()))?;
        Ok(receipt_path)
    }

    fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("failed to create evidence dir at {}", self.dir.display()))
    }
}

impl Default for EvidenceEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the standard evidence directory: `{cwd}/target/cargo-cicd/evidence/`.
///
/// ```ignore
/// let dir = evidence_dir();
/// assert!(dir.ends_with("target/cargo-cicd/evidence"));
/// ```
pub fn evidence_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("target")
        .join("cargo-cicd")
        .join("evidence")
}

/// Returns a workspace identifier derived from the `Cargo.toml` `[package] name`
/// field and a hash of the current working directory path.
///
/// Falls back to `"unknown"` if `Cargo.toml` cannot be read or parsed.
///
/// ```ignore
/// let id = workspace_id();
/// assert!(!id.is_empty());
/// ```
pub fn workspace_id() -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let name = read_cargo_toml_name(&cwd).unwrap_or_else(|| "unknown".to_string());
    let path_hash = simple_hash(cwd.to_string_lossy().as_bytes());
    format!("{}_{:08x}", name, path_hash)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn read_cargo_toml_name(root: &std::path::Path) -> Option<String> {
    let cargo_toml = root.join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml).ok()?;
    // Minimal extraction: look for `name = "..."` in [package] section.
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start().strip_prefix('=')?.trim();
            let name = rest.trim_matches('"').trim_matches('\'');
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// A simple djb2-style hash for path strings. Not cryptographic.
fn simple_hash(bytes: &[u8]) -> u32 {
    let mut h: u32 = 5381;
    for &b in bytes {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    h
}

/// Format a Unix timestamp (seconds) as ISO-8601 UTC without external crates.
///
/// Output format: `YYYY-MM-DDTHH:MM:SSZ`
fn format_timestamp_iso(secs: u64) -> String {
    // Days since Unix epoch → calendar date via the proleptic Gregorian calendar.
    let s = secs % 86400;
    let days = secs / 86400;

    let hh = s / 3600;
    let mm = (s % 3600) / 60;
    let ss = s % 60;

    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z: i64 = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hh, mm, ss
    )
}
