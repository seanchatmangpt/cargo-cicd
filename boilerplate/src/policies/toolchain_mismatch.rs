//! Policy: warn when a nightly toolchain is active without a pinned date.
//!
//! Nightly toolchains change daily and can introduce breaking changes between
//! runs.  A project that relies on nightly features must pin a specific nightly
//! date in `rust-toolchain.toml` to guarantee reproducible builds.
//!
//! We consider a channel "mismatched" when it is `"nightly"` (the bare rolling
//! nightly) rather than a date-pinned nightly like `"nightly-2025-04-01"`.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "toolchain_mismatch";

/// Evaluate toolchain channel suitability.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | `rust_version` is empty | `Skip` |
/// | `channel` is not `"nightly"` | `Pass` |
/// | `channel` is `"nightly"` (bare, unpinned) | `Warn` |
/// | `channel` starts with `"nightly-"` (date-pinned) | `Pass` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    if state.toolchain.rust_version.is_empty() {
        // No toolchain information available — skip rather than false-positive.
        return PolicyEntry::skip(POLICY_NAME);
    }

    let channel = state.toolchain.channel.as_str();

    // Date-pinned nightly (e.g. "nightly-2025-04-01") is acceptable.
    if channel.starts_with("nightly-") {
        return PolicyEntry::pass(POLICY_NAME);
    }

    // Bare rolling nightly — warn.
    if channel == "nightly" {
        return PolicyEntry {
            policy_name: POLICY_NAME.to_string(),
            verdict: PolicyVerdict::Warn,
            recommendation:
                "Using nightly toolchain — ensure `rust-toolchain.toml` pins a specific \
                 nightly date for reproducibility"
                    .to_string(),
            emitted_at: now_iso8601(),
        };
    }

    // stable, beta, or any other channel — fine.
    PolicyEntry::pass(POLICY_NAME)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::policy_engine::PolicyVerdict;
    use crate::engine::{EngineState, ToolchainState};

    fn state_with_channel(channel: &str, rust_version: &str) -> EngineState {
        EngineState {
            toolchain: ToolchainState {
                channel: channel.to_string(),
                rust_version: rust_version.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn skip_when_version_empty() {
        let state = state_with_channel("nightly", "");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn pass_for_stable_channel() {
        let state = state_with_channel("stable", "rustc 1.86.0 (abc 2025-03-31)");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn pass_for_beta_channel() {
        let state = state_with_channel("beta", "rustc 1.87.0-beta.1 (abc 2025-04-01)");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_for_bare_nightly() {
        let state = state_with_channel("nightly", "rustc 1.87.0-nightly (abc 2025-04-15)");
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
        assert!(entry.recommendation.contains("rust-toolchain.toml"));
    }

    #[test]
    fn pass_for_date_pinned_nightly() {
        let state = state_with_channel(
            "nightly-2025-04-01",
            "rustc 1.87.0-nightly (abc 2025-04-01)",
        );
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_recommendation_mentions_reproducibility() {
        let state = state_with_channel("nightly", "rustc 1.87.0-nightly (abc 2025-04-15)");
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("reproducibility"),
            "expected reproducibility hint: {}",
            entry.recommendation
        );
    }
}
