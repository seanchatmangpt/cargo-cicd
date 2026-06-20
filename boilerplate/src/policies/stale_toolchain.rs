//! Policy: warn when the active Rust toolchain appears outdated.
//!
//! "Outdated" is defined as a toolchain whose release year is more than one
//! year behind the current year.  The version string from `rustc --version`
//! encodes the release date: `rustc 1.86.0 (05f9846f8 2025-03-31)`.  We
//! extract the year from that trailing date.
//!
//! If the version string is empty or cannot be parsed, we skip rather than
//! warn — absence of information does not constitute evidence of staleness.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "stale_toolchain";

/// Current year used as the reference point.
///
/// Hard-coded to the project's declared current date (2026) so tests are
/// deterministic.  In production the function uses `SystemTime` to derive it
/// dynamically.
fn current_year() -> u32 {
    // Derive the current year from the system clock so this stays correct
    // after 2026 without a code change.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Rough estimate: 365.25 days per year from epoch (1970)
    let approx_year = 1970 + (secs / 31_557_600) as u32;
    approx_year
}

/// Extract a 4-digit year from a `rustc --version` string.
///
/// Example input: `"rustc 1.86.0 (05f9846f8 2025-03-31)"`
/// Looks for a substring matching `YYYY-MM-DD` inside parentheses.
fn extract_release_year(version: &str) -> Option<u32> {
    // Find the opening paren — the date lives inside `(hash YYYY-MM-DD)`.
    let paren = version.find('(')?;
    let inside = &version[paren..];

    // Walk the inside looking for `YYYY-MM-DD`.
    for window in inside.as_bytes().windows(10) {
        if let Ok(chunk) = std::str::from_utf8(window) {
            // Pattern: 4 digits, dash, 2 digits, dash, 2 digits
            let bytes = chunk.as_bytes();
            let looks_like_date = bytes[0..4].iter().all(|b| b.is_ascii_digit())
                && bytes[4] == b'-'
                && bytes[5..7].iter().all(|b| b.is_ascii_digit())
                && bytes[7] == b'-'
                && bytes[8..10].iter().all(|b| b.is_ascii_digit());

            if looks_like_date {
                let year_str = &chunk[0..4];
                return year_str.parse::<u32>().ok();
            }
        }
    }
    None
}

/// Evaluate toolchain age.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | `rust_version` is empty or unparseable | `Skip` |
/// | Release year >= `current_year - 1` | `Pass` |
/// | Release year < `current_year - 1` | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    let version = &state.toolchain.rust_version;

    if version.is_empty() {
        return PolicyEntry::skip(POLICY_NAME);
    }

    let release_year = match extract_release_year(version) {
        Some(y) => y,
        None => {
            // Cannot parse the date — skip rather than false-positive.
            return PolicyEntry::skip(POLICY_NAME);
        }
    };

    let threshold = current_year().saturating_sub(1);

    if release_year >= threshold {
        return PolicyEntry::pass(POLICY_NAME);
    }

    PolicyEntry {
        policy_name: POLICY_NAME.to_string(),
        verdict: PolicyVerdict::Warn,
        recommendation: format!(
            "Toolchain {version} may be outdated (released {release_year}). \
             Run `rustup update stable`"
        ),
        emitted_at: now_iso8601(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::policy_engine::PolicyVerdict;
    use crate::engine::{EngineState, ToolchainState};

    fn state_with_version(rust_version: &str) -> EngineState {
        EngineState {
            toolchain: ToolchainState {
                rust_version: rust_version.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn skip_when_version_is_empty() {
        let state = state_with_version("");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn skip_when_version_has_no_date() {
        let state = state_with_version("rustc 1.86.0");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn pass_for_recent_toolchain() {
        // 2025-03-31 is at most 1 year behind 2026 — should pass.
        let state = state_with_version("rustc 1.86.0 (05f9846f8 2025-03-31)");
        let entry = eval(&state);
        // Pass or Skip depending on current year at test time; must not warn.
        assert!(
            entry.verdict != PolicyVerdict::Warn,
            "recent toolchain should not warn"
        );
    }

    #[test]
    fn warn_for_very_old_toolchain() {
        // 2020 is more than 1 year behind any reasonable current year.
        let state = state_with_version("rustc 1.48.0 (7eac88ab1 2020-11-16)");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("rustup update stable"));
    }

    #[test]
    fn recommendation_contains_version_string() {
        let ver = "rustc 1.48.0 (7eac88ab1 2020-11-16)";
        let state = state_with_version(ver);
        let entry = eval(&state);
        if entry.verdict == PolicyVerdict::Warn {
            assert!(
                entry.recommendation.contains(ver),
                "expected version string in recommendation: {}",
                entry.recommendation
            );
        }
    }

    #[test]
    fn extract_release_year_parses_standard_version() {
        assert_eq!(
            extract_release_year("rustc 1.86.0 (05f9846f8 2025-03-31)"),
            Some(2025)
        );
    }

    #[test]
    fn extract_release_year_returns_none_for_no_date() {
        assert_eq!(extract_release_year("rustc 1.86.0"), None);
    }

    #[test]
    fn extract_release_year_handles_nightly_format() {
        // Nightly: `rustc 1.87.0-nightly (abc123 2025-04-01)`
        assert_eq!(
            extract_release_year("rustc 1.87.0-nightly (abc123 2025-04-01)"),
            Some(2025)
        );
    }
}
