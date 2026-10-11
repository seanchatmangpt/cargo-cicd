//! Typed external-tool discovery.
//!
//! Discovery returns [`Result`] with [`ToolUnavailable`] — never a bare bool
//! or a warn-and-continue — so noun-level callers can fail closed with a
//! actionable hint instead of failing mysteriously downstream.

use std::path::PathBuf;

/// A required external tool could not be located.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("tool `{tool}` unavailable: {hint}")]
pub struct ToolUnavailable {
    /// Canonical tool name that was searched for.
    pub tool: &'static str,
    /// What the operator can do about it (env var override, install cmd).
    pub hint: String,
}

/// Env var checked before PATH for a tool named `name` (e.g. `wpm` → `WPM_BIN`).
fn env_override_var(tool: &str) -> String {
    let mut upper: String = tool
        .chars()
        .map(|c| if c == '-' { '_' } else { c.to_ascii_uppercase() })
        .collect();
    if !upper.ends_with("_BIN") {
        upper.push_str("_BIN");
    }
    upper
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Locate `tool`: `<TOOL>_BIN` env override first, then PATH.
pub fn discover_tool(tool: &'static str, bin_names: &[&str]) -> Result<PathBuf, ToolUnavailable> {
    let var = env_override_var(tool);
    if let Ok(bin) = std::env::var(&var) {
        let p = PathBuf::from(&bin);
        if p.is_file() {
            return Ok(p);
        }
    }
    for name in bin_names {
        if let Some(p) = which_on_path(name) {
            return Ok(p);
        }
    }
    Err(ToolUnavailable {
        tool,
        hint: format!(
            "set {var} to the binary path, or install it and ensure `{}` is on PATH",
            bin_names.first().unwrap_or(&tool)
        ),
    })
}

/// Convenience: the tool set cargo-cicd drives externally.
pub mod known {
    /// wpm / wasm4pm runner.
    pub const WPM: &str = "wpm";
    /// SBOM generator.
    pub const CYCLONEDX: &str = "cyclonedx";
    /// ggen generator.
    pub const GGEN: &str = "ggen";
    /// act (GitHub Actions local runner).
    pub const ACT: &str = "act";
    /// GitHub CLI.
    pub const GH: &str = "gh";
}

/// Discover wpm (accepts `wpm` or `wasm4pm` binary names).
pub fn discover_wpm() -> Result<PathBuf, ToolUnavailable> {
    discover_tool(known::WPM, &["wpm", "wasm4pm"])
}

/// Discover the GitHub CLI.
pub fn discover_gh() -> Result<PathBuf, ToolUnavailable> {
    discover_tool(known::GH, &["gh"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_returns_typed_error_with_hint() {
        // A tool name chosen so no real binary and no _BIN override matches.
        let err = discover_tool("cargocicd-nonexistent-probe", &["cargocicd-nonexistent-probe-bin"])
            .unwrap_err();
        assert_eq!(err.tool, "cargocicd-nonexistent-probe");
        assert!(err.hint.contains("CARGOCICD_NONEXISTENT_PROBE_BIN"));
    }

    #[test]
    fn env_override_wins_when_set_to_real_file() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("fake-tool");
        std::fs::write(&fake, "#!/bin/sh\n").unwrap();
        // SAFETY of test isolation: unique var name per tool string.
        std::env::set_var("CARGOCICD_PROBE2_TOOL_BIN", &fake);
        let found =
            discover_tool("cargocicd-probe2-tool", &["cargocicd-probe2-tool-never-on-path"])
                .unwrap();
        assert_eq!(found, fake);
        std::env::remove_var("CARGOCICD_PROBE2_TOOL_BIN");
    }

    #[test]
    fn gh_discovery_matches_env_or_path_state() {
        // Chicago: assert against the real machine, both outcomes valid —
        // the contract is the type, not presence.
        let result = discover_gh();
        match std::env::var("GH_BIN") {
            Ok(p) if PathBuf::from(&p).is_file() => assert!(result.is_ok()),
            _ => {}
        }
    }
}
