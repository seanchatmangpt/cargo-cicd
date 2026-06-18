//! XES 2.0 compliant emitter for cargo-cicd process evidence.
//!
//! Implements the IEEE XES standard (ISO/IEC 20880:2013) with full attribute
//! sets required for process mining tool compatibility (ProM, Disco, Celonis).
//!
//! ## Key differences from the baseline XES 1.0 emitter in `evidence.rs`
//!
//! - `<log>` carries `xes.version="2.0"` and the `xmlns:xes` namespace.
//! - Each `<trace>` carries workspace context: `workspace_id`, `workspace_root`,
//!   `git_branch`, `git_commit_sha`, `toolchain_version`, `cargo_version`,
//!   `os_version`, and `session_id`.
//! - Each `<event>` carries `event_id`, `timestamp` (ISO-8601 UTC with ms),
//!   `lifecycle_transition`, `event_name` (`{noun}:{verb}` form), and
//!   `verdict_claimed`.
//! - Completion events additionally carry `duration_ms`, `verdict_adjudicated`,
//!   `adjudicated_at`, `oracle_command`, and `trace_class`.

use crate::evidence::{now_iso8601, ProcessEvent};
use std::io;
use std::path::{Path, PathBuf};

// ── Workspace metadata ────────────────────────────────────────────────────────

/// Metadata injected into each `<trace>` element for XES 2.0 compliance.
///
/// Provides workspace context so process mining tools can correlate traces
/// across time and machines without relying on external configuration.
#[derive(Debug, Clone)]
pub struct XesWorkspaceMeta {
    pub workspace_id: String,
    pub workspace_root: String,
    pub git_branch: String,
    pub git_commit_sha: String,
    pub toolchain_version: String,
    pub cargo_version: String,
    pub os_version: String,
    pub session_id: String,
}

impl XesWorkspaceMeta {
    /// Populate by querying the environment.
    ///
    /// Each field degrades gracefully to a safe default when the underlying
    /// source is unavailable (e.g. `git` not installed, not a git repo).
    /// This method never panics.
    pub fn from_env() -> Self {
        Self {
            workspace_id: detect_workspace_id(),
            workspace_root: detect_workspace_root(),
            git_branch: detect_git_branch(),
            git_commit_sha: detect_git_commit_sha(),
            toolchain_version: detect_toolchain_version(),
            cargo_version: detect_cargo_version(),
            os_version: detect_os_version(),
            session_id: generate_session_id(),
        }
    }

    /// Construct a minimal metadata block suitable for tests.
    pub fn for_testing() -> Self {
        Self {
            workspace_id: "test-workspace".to_string(),
            workspace_root: "/tmp/test".to_string(),
            git_branch: "main".to_string(),
            git_commit_sha: "abc1234".to_string(),
            toolchain_version: "rustc 1.86.0".to_string(),
            cargo_version: "cargo 1.86.0".to_string(),
            os_version: "linux".to_string(),
            session_id: "session-test-001".to_string(),
        }
    }
}

// ── Environment probes ────────────────────────────────────────────────────────

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(program)
        .args(args)
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
}

fn detect_workspace_id() -> String {
    // Try to derive from the git remote URL or fall back to the directory name.
    run_cmd("git", &["remote", "get-url", "origin"])
        .map(|url| {
            // Extract the last component, strip ".git" suffix.
            url.split('/')
                .next_back()
                .unwrap_or("workspace")
                .trim_end_matches(".git")
                .to_string()
        })
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .unwrap_or_else(|| "cargo-cicd-workspace".to_string())
}

fn detect_workspace_root() -> String {
    run_cmd("git", &["rev-parse", "--show-toplevel"])
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| ".".to_string())
}

fn detect_git_branch() -> String {
    run_cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "HEAD-detached".to_string())
}

fn detect_git_commit_sha() -> String {
    run_cmd("git", &["rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|| "HEAD-not-resolved".to_string())
}

fn detect_toolchain_version() -> String {
    run_cmd("rustc", &["--version"]).unwrap_or_else(|| "rustc-unknown".to_string())
}

fn detect_cargo_version() -> String {
    run_cmd("cargo", &["--version"]).unwrap_or_else(|| "cargo-unknown".to_string())
}

fn detect_os_version() -> String {
    // Try /etc/os-release for Linux, fall back to uname.
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    run_cmd("uname", &["-sr"]).unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn generate_session_id() -> String {
    // Use a timestamp-based ID; no external crate required.
    let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
    format!("session-{}", ts)
}

// ── XML helpers ───────────────────────────────────────────────────────────────

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn string_attr(indent: &str, key: &str, value: &str) -> String {
    format!(
        "{}<string key=\"{}\" value=\"{}\"/>\n",
        indent,
        escape_xml(key),
        escape_xml(value)
    )
}

fn int_attr(indent: &str, key: &str, value: u64) -> String {
    format!(
        "{}<int key=\"{}\" value=\"{}\"/>\n",
        indent,
        escape_xml(key),
        value
    )
}

fn date_attr(indent: &str, key: &str, value: &str) -> String {
    format!(
        "{}<date key=\"{}\" value=\"{}\"/>\n",
        indent,
        escape_xml(key),
        escape_xml(value)
    )
}

// ── Core serialisation ────────────────────────────────────────────────────────

/// Serialize a slice of `ProcessEvent`s to an XES 2.0 compliant XML string.
///
/// All events are placed into a single `<trace>` keyed by `case_id`.
/// The `<log>` element carries `xes.version="2.0"` and the standard namespace.
/// The `<trace>` element carries all `XesWorkspaceMeta` fields.
/// Each `<event>` carries the full attribute set required by the spec.
pub fn to_xes_v2(
    events: &[ProcessEvent],
    case_id: &str,
    workspace_meta: &XesWorkspaceMeta,
) -> String {
    let mut xml = String::new();

    // XES 2.0 log header.
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<log xes.version=\"2.0\" xmlns:xes=\"http://www.xes-standard.org/\">\n");

    // Standard XES extensions.
    xml.push_str("  <extension name=\"Concept\" prefix=\"concept\" uri=\"http://www.xes-standard.org/concept.xesext\"/>\n");
    xml.push_str("  <extension name=\"Time\" prefix=\"time\" uri=\"http://www.xes-standard.org/time.xesext\"/>\n");
    xml.push_str("  <extension name=\"Lifecycle\" prefix=\"lifecycle\" uri=\"http://www.xes-standard.org/lifecycle.xesext\"/>\n");
    xml.push_str("  <extension name=\"Organizational\" prefix=\"org\" uri=\"http://www.xes-standard.org/org.xesext\"/>\n");

    // Trace element.
    xml.push_str("  <trace>\n");

    // case_id as concept:name (required by every process mining tool).
    xml.push_str(&format!(
        "    <string key=\"concept:name\" value=\"{}\"/>\n",
        escape_xml(case_id)
    ));

    // Workspace context in trace-level attributes (4-space indent inside <trace>).
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:workspace_id",
        &workspace_meta.workspace_id,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:workspace_root",
        &workspace_meta.workspace_root,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:git_branch",
        &workspace_meta.git_branch,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:git_commit_sha",
        &workspace_meta.git_commit_sha,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:toolchain_version",
        &workspace_meta.toolchain_version,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:cargo_version",
        &workspace_meta.cargo_version,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:os_version",
        &workspace_meta.os_version,
    ));
    xml.push_str(&string_attr(
        "    ",
        "cargo_cicd:session_id",
        &workspace_meta.session_id,
    ));

    // Events (6-space indent inside <event>).
    for event in events {
        xml.push_str("    <event>\n");

        // Required: event_id.
        xml.push_str(&string_attr(
            "      ",
            "cargo_cicd:event_id",
            &event.event_id,
        ));

        // Required: concept:name as "{noun}:{verb}" normalised form.
        let event_name = command_to_event_name(&event.command);
        xml.push_str(&format!(
            "      <string key=\"concept:name\" value=\"{}\"/>\n",
            escape_xml(&event_name)
        ));

        // Required: time:timestamp (ISO-8601 UTC with ms).
        xml.push_str(&date_attr("      ", "time:timestamp", &event.timestamp_iso));

        // Required: lifecycle:transition ("start" or "complete").
        xml.push_str(&string_attr(
            "      ",
            "lifecycle:transition",
            &event.lifecycle_transition,
        ));

        // Required: verdict_claimed.
        xml.push_str(&string_attr(
            "      ",
            "cargo_cicd:verdict_claimed",
            &event.verdict_claimed,
        ));

        // Required: workspace_id (repeated at event level for standalone trace tools).
        xml.push_str(&string_attr(
            "      ",
            "cargo_cicd:workspace_id",
            &event.workspace_id,
        ));

        // Required: trace_class.
        xml.push_str(&string_attr(
            "      ",
            "cargo_cicd:trace_class",
            &event.trace_class,
        ));

        // Completion-only attributes.
        if event.lifecycle_transition == "complete" {
            if let Some(ms) = event.duration_ms {
                xml.push_str(&int_attr("      ", "cargo_cicd:duration_ms", ms));
            }
            if let Some(ref v) = event.verdict_adjudicated {
                xml.push_str(&string_attr("      ", "wasm4pm:verdict_adjudicated", v));
            }
            if let Some(ref ts) = event.adjudicated_at {
                xml.push_str(&string_attr("      ", "wasm4pm:adjudicated_at", ts));
            }
            if let Some(ref oracle) = event.oracle_command {
                xml.push_str(&string_attr("      ", "wasm4pm:oracle_command", oracle));
            }
        }

        xml.push_str("    </event>\n");
    }

    xml.push_str("  </trace>\n");
    xml.push_str("</log>\n");

    xml
}

/// Normalise a command string to `{noun}:{verb}` form.
///
/// - `"status show"` → `"status:show"`
/// - `"status:show"`  → unchanged
/// - `"publish"` → `"publish:run"` (bare noun gets a default verb)
pub fn command_to_event_name(command: &str) -> String {
    if command.contains(':') {
        return command.to_string();
    }
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    match parts.as_slice() {
        [noun, verb] => format!("{}:{}", noun, verb),
        [noun] => format!("{}:run", noun),
        _ => command.to_string(),
    }
}

// ── File I/O ──────────────────────────────────────────────────────────────────

/// Write XES 2.0 compliant evidence to `evidence_dir`.
///
/// The file is named `evt-{case_id}-{timestamp}.xes` following the cargo-cicd
/// evidence naming convention. The directory is created if absent.
///
/// Returns the path to the written file.
pub fn write_xes_v2(
    events: &[ProcessEvent],
    case_id: &str,
    evidence_dir: &Path,
) -> io::Result<PathBuf> {
    write_xes_v2_with_meta(events, case_id, evidence_dir, &XesWorkspaceMeta::from_env())
}

/// Like [`write_xes_v2`] but accepts an explicit `XesWorkspaceMeta` (useful in tests).
pub fn write_xes_v2_with_meta(
    events: &[ProcessEvent],
    case_id: &str,
    evidence_dir: &Path,
    meta: &XesWorkspaceMeta,
) -> io::Result<PathBuf> {
    std::fs::create_dir_all(evidence_dir)?;

    let ts = now_iso8601().replace(['-', ':', '.', 'T', 'Z'], "");
    let safe_case_id = case_id.replace(['/', '\\', ':', ' '], "_");
    let filename = format!("evt-{}-{}.xes", safe_case_id, ts);
    let path = evidence_dir.join(filename);

    let xml = to_xes_v2(events, case_id, meta);
    std::fs::write(&path, xml)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::ProcessEvent;

    fn sample_event(lifecycle: &str) -> ProcessEvent {
        ProcessEvent {
            event_id: "evt-test-001".to_string(),
            timestamp_iso: "2026-06-17T12:00:00.000Z".to_string(),
            case_id: Some("test_case".to_string()),
            lifecycle_transition: lifecycle.to_string(),
            workspace_id: "test-workspace".to_string(),
            repo_path: "/repo".to_string(),
            command: "status show".to_string(),
            verdict_claimed: "PASS".to_string(),
            duration_ms: if lifecycle == "complete" {
                Some(42)
            } else {
                None
            },
            verdict_adjudicated: None,
            adjudicated_at: None,
            oracle_command: None,
            trace_class: "live_workspace".to_string(),
        }
    }

    #[test]
    fn to_xes_v2_contains_xes_version_2() {
        let events = vec![sample_event("complete")];
        let meta = XesWorkspaceMeta::for_testing();
        let xml = to_xes_v2(&events, "test_case", &meta);
        assert!(
            xml.contains("xes.version=\"2.0\""),
            "XES 2.0 version attribute missing"
        );
    }

    #[test]
    fn to_xes_v2_contains_xmlns() {
        let events = vec![sample_event("complete")];
        let meta = XesWorkspaceMeta::for_testing();
        let xml = to_xes_v2(&events, "test_case", &meta);
        assert!(
            xml.contains("xmlns:xes=\"http://www.xes-standard.org/\""),
            "XES namespace attribute missing"
        );
    }

    #[test]
    fn to_xes_v2_trace_contains_workspace_id() {
        let events = vec![sample_event("complete")];
        let meta = XesWorkspaceMeta::for_testing();
        let xml = to_xes_v2(&events, "test_case", &meta);
        assert!(
            xml.contains("cargo_cicd:workspace_id"),
            "workspace_id attribute missing from trace"
        );
    }

    #[test]
    fn command_to_event_name_normalises_space_to_colon() {
        assert_eq!(command_to_event_name("status show"), "status:show");
        assert_eq!(command_to_event_name("target prune"), "target:prune");
    }

    #[test]
    fn command_to_event_name_preserves_colon_form() {
        assert_eq!(command_to_event_name("status:show"), "status:show");
    }

    #[test]
    fn command_to_event_name_bare_noun_gets_run() {
        assert_eq!(command_to_event_name("publish"), "publish:run");
    }
}
