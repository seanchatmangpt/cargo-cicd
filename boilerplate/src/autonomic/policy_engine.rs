//! Core policy engine types and the top-level policy runner.
//!
//! ## Design
//!
//! - [`PolicyVerdict`] — the outcome of a single policy evaluation.
//! - [`PolicyEntry`] — one evaluated policy with its verdict, recommendation,
//!   and an ISO-8601 timestamp marking when it was evaluated.
//! - [`PolicyReport`] — the aggregate of all evaluated policies for a single
//!   command invocation.
//! - [`run_all_policies`] — calls every registered policy module and collects
//!   results into a [`PolicyReport`].
//!
//! ## Invariant
//!
//! Policies are **suggest-only**.  They must never mutate `EngineState` or
//! invoke external processes.  All `eval()` functions take a shared reference
//! to [`EngineState`].

#![cfg(feature = "autonomic")]

use crate::policies;

// ── Verdict ─────────────────────────────────────────────────────────────────

/// The outcome of evaluating a single policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyVerdict {
    /// All conditions satisfied — no action required.
    Pass,
    /// A condition was detected that warrants attention.  The associated
    /// [`PolicyEntry::recommendation`] describes the suggested remediation.
    Warn,
    /// The policy is inapplicable in this context (e.g., single-crate workspace
    /// for a multi-member policy).  No recommendation is emitted.
    Skip,
}

impl std::fmt::Display for PolicyVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyVerdict::Pass => write!(f, "PASS"),
            PolicyVerdict::Warn => write!(f, "WARN"),
            PolicyVerdict::Skip => write!(f, "SKIP"),
        }
    }
}

// ── PolicyEntry ──────────────────────────────────────────────────────────────

/// One evaluated policy result.
#[derive(Debug, Clone)]
pub struct PolicyEntry {
    /// Machine-readable name identifying the policy (e.g. `"git_phase_dirty"`).
    pub policy_name: String,
    /// Outcome of evaluation.
    pub verdict: PolicyVerdict,
    /// Human-readable recommendation.  Empty string when verdict is `Pass` or
    /// `Skip`.
    pub recommendation: String,
    /// ISO-8601 UTC timestamp of evaluation (e.g. `"2026-06-20T14:32:00.000Z"`).
    pub emitted_at: String,
}

impl PolicyEntry {
    /// Construct a `Pass` entry with no recommendation.
    pub fn pass(policy_name: impl Into<String>) -> Self {
        Self {
            policy_name: policy_name.into(),
            verdict: PolicyVerdict::Pass,
            recommendation: String::new(),
            emitted_at: now_iso8601(),
        }
    }

    /// Construct a `Warn` entry with a recommendation.
    pub fn warn(policy_name: impl Into<String>, recommendation: impl Into<String>) -> Self {
        Self {
            policy_name: policy_name.into(),
            verdict: PolicyVerdict::Warn,
            recommendation: recommendation.into(),
            emitted_at: now_iso8601(),
        }
    }

    /// Construct a `Skip` entry.
    pub fn skip(policy_name: impl Into<String>) -> Self {
        Self {
            policy_name: policy_name.into(),
            verdict: PolicyVerdict::Skip,
            recommendation: String::new(),
            emitted_at: now_iso8601(),
        }
    }
}

// ── PolicyReport ─────────────────────────────────────────────────────────────

/// Aggregate of all policy evaluations for one command invocation.
#[derive(Debug, Clone, Default)]
pub struct PolicyReport {
    /// All evaluated entries, in evaluation order.
    pub entries: Vec<PolicyEntry>,
}

impl PolicyReport {
    /// Returns `true` when at least one policy produced a `Warn` verdict.
    pub fn has_warnings(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.verdict == PolicyVerdict::Warn)
    }

    /// Returns an iterator over entries whose verdict is `Warn`.
    pub fn warnings(&self) -> impl Iterator<Item = &PolicyEntry> {
        self.entries
            .iter()
            .filter(|e| e.verdict == PolicyVerdict::Warn)
    }

    /// Print a formatted policy report to stdout.
    ///
    /// Entries with `Pass` or `Skip` are shown as single-line status lines.
    /// `Warn` entries include the recommendation on a second indented line.
    pub fn display(&self) {
        let width = 60;
        println!("{}", "─".repeat(width));
        println!("  Autonomic Policy Report");
        println!("{}", "─".repeat(width));

        if self.entries.is_empty() {
            println!("  (no policies evaluated)");
        } else {
            for entry in &self.entries {
                let badge = match entry.verdict {
                    PolicyVerdict::Pass => "[PASS]",
                    PolicyVerdict::Warn => "[WARN]",
                    PolicyVerdict::Skip => "[SKIP]",
                };
                println!("  {badge:<6}  {}", entry.policy_name);
                if entry.verdict == PolicyVerdict::Warn && !entry.recommendation.is_empty() {
                    println!("           → {}", entry.recommendation);
                }
            }
        }

        let warn_count = self.entries.iter().filter(|e| e.verdict == PolicyVerdict::Warn).count();
        let pass_count = self.entries.iter().filter(|e| e.verdict == PolicyVerdict::Pass).count();
        let skip_count = self.entries.iter().filter(|e| e.verdict == PolicyVerdict::Skip).count();

        println!("{}", "─".repeat(width));
        println!(
            "  {} pass  {} warn  {} skip",
            pass_count, warn_count, skip_count
        );
        println!("{}", "─".repeat(width));
    }
}

// ── Policy runner ────────────────────────────────────────────────────────────

/// Evaluate all registered policies against `state` and return a
/// [`PolicyReport`].
///
/// Policies are called in a deterministic order.  The order is intentionally
/// stable across releases so that users can read reports consistently.
pub fn run_all_policies(state: &crate::engine::EngineState) -> PolicyReport {
    let entries = vec![
        policies::git_phase_dirty::eval(state),
        policies::branch_behind::eval(state),
        policies::uncommitted_changes::eval(state),
        policies::untracked_files::eval(state),
        policies::large_workspace::eval(state),
        policies::stale_toolchain::eval(state),
        policies::toolchain_mismatch::eval(state),
    ];

    PolicyReport { entries }
}

// ── Shared helper ────────────────────────────────────────────────────────────

/// Return the current UTC time as an ISO-8601 string with millisecond
/// precision (e.g. `"2026-06-20T14:32:00.123Z"`).
///
/// Uses only `std::time` — no external date crate required.
pub fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Manual ISO-8601 formatting from a Unix millisecond timestamp.
    let total_seconds = millis / 1_000;
    let ms_part = millis % 1_000;

    // Calendar decomposition (Gregorian, UTC).
    let (year, month, day, hour, minute, second) = seconds_to_ymdhms(total_seconds as u64);

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{ms_part:03}Z"
    )
}

/// Decompose a Unix timestamp (seconds) into `(year, month, day, hour, min, sec)`.
///
/// Algorithm: civil-calendar computation from Fliegel & Van Flandern (1968),
/// adapted for integer arithmetic.
fn seconds_to_ymdhms(ts: u64) -> (u32, u32, u32, u32, u32, u32) {
    let second = (ts % 60) as u32;
    let minutes_total = ts / 60;
    let minute = (minutes_total % 60) as u32;
    let hours_total = minutes_total / 60;
    let hour = (hours_total % 24) as u32;
    let days_since_epoch = hours_total / 24; // days since 1970-01-01

    // Convert days since Unix epoch to Gregorian calendar date.
    // Using the algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // day-of-era [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // year-of-era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day-of-year [0, 365]
    let mp = (5 * doy + 2) / 153; // month part
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (y + if month <= 2 { 1 } else { 0 }) as u32;

    (year, month, day, hour, minute, second)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineState;

    #[test]
    fn policy_report_empty_has_no_warnings() {
        let report = PolicyReport::default();
        assert!(!report.has_warnings());
        assert_eq!(report.warnings().count(), 0);
    }

    #[test]
    fn policy_report_with_warn_has_warnings() {
        let report = PolicyReport {
            entries: vec![PolicyEntry::warn("test_policy", "do the thing")],
        };
        assert!(report.has_warnings());
        assert_eq!(report.warnings().count(), 1);
    }

    #[test]
    fn policy_report_pass_not_counted_as_warning() {
        let report = PolicyReport {
            entries: vec![
                PolicyEntry::pass("a"),
                PolicyEntry::skip("b"),
            ],
        };
        assert!(!report.has_warnings());
    }

    #[test]
    fn policy_entry_constructors_set_correct_verdict() {
        let p = PolicyEntry::pass("p");
        assert_eq!(p.verdict, PolicyVerdict::Pass);
        assert!(p.recommendation.is_empty());

        let w = PolicyEntry::warn("w", "fix it");
        assert_eq!(w.verdict, PolicyVerdict::Warn);
        assert_eq!(w.recommendation, "fix it");

        let s = PolicyEntry::skip("s");
        assert_eq!(s.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn now_iso8601_has_expected_format() {
        let ts = now_iso8601();
        // e.g. "2026-06-20T14:32:00.123Z"
        assert!(ts.ends_with('Z'), "timestamp must end with Z: {ts}");
        assert_eq!(ts.len(), 24, "expected 24-char timestamp, got: {ts}");
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
    }

    #[test]
    fn run_all_policies_clean_state_returns_all_pass_or_skip() {
        let state = EngineState::default();
        let report = run_all_policies(&state);
        // A default state (empty strings, empty vecs, zeros) should produce
        // no warnings except possibly stale_toolchain (empty version string).
        for entry in &report.entries {
            // Only stale_toolchain and toolchain_mismatch may warn on empty state;
            // all others must pass or skip.
            if entry.policy_name != "stale_toolchain" && entry.policy_name != "toolchain_mismatch" {
                assert!(
                    entry.verdict != PolicyVerdict::Warn,
                    "policy '{}' should not warn on clean default state, but got WARN: {}",
                    entry.policy_name,
                    entry.recommendation
                );
            }
        }
    }

    #[test]
    fn policy_verdict_display() {
        assert_eq!(PolicyVerdict::Pass.to_string(), "PASS");
        assert_eq!(PolicyVerdict::Warn.to_string(), "WARN");
        assert_eq!(PolicyVerdict::Skip.to_string(), "SKIP");
    }
}
