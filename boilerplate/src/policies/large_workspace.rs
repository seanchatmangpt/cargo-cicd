//! Policy: warn when the workspace has grown large enough to warrant splitting.
//!
//! Large workspaces with many crate members increase build times and reduce
//! cohesion.  Past 10 members, it is worth evaluating whether independent
//! crates could live in their own repositories.

#![cfg(feature = "autonomic")]

use crate::autonomic::policy_engine::{now_iso8601, PolicyEntry, PolicyVerdict};

const POLICY_NAME: &str = "large_workspace";

/// Threshold: workspaces with more than this many members trigger a warning.
const MEMBER_THRESHOLD: usize = 10;

/// Evaluate workspace size.
///
/// # Verdict
///
/// | Condition | Verdict |
/// |-----------|---------|
/// | `members.len() == 0` (single-crate / unknown) | `Skip` |
/// | `members.len() <= MEMBER_THRESHOLD` | `Pass` |
/// | `members.len() > MEMBER_THRESHOLD` | `Warn` |
pub fn eval(state: &crate::engine::EngineState) -> PolicyEntry {
    let n = state.workspace.members.len();

    if n == 0 {
        // Single-crate project or members not populated — policy is not applicable.
        return PolicyEntry::skip(POLICY_NAME);
    }

    if n <= MEMBER_THRESHOLD {
        return PolicyEntry::pass(POLICY_NAME);
    }

    PolicyEntry {
        policy_name: POLICY_NAME.to_string(),
        verdict: PolicyVerdict::Warn,
        recommendation: format!(
            "Workspace has {n} members — consider splitting into separate repos \
             if modules are independently deployable"
        ),
        emitted_at: now_iso8601(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::policy_engine::PolicyVerdict;
    use crate::engine::{EngineState, WorkspaceState};

    fn state_with_members(members: Vec<String>) -> EngineState {
        EngineState {
            workspace: WorkspaceState {
                members,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn skip_when_no_members() {
        let state = state_with_members(vec![]);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Skip);
    }

    #[test]
    fn pass_when_exactly_at_threshold() {
        let members = (0..MEMBER_THRESHOLD)
            .map(|i| format!("crate-{i}"))
            .collect();
        let state = state_with_members(members);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn pass_when_under_threshold() {
        let state = state_with_members(vec!["a".to_string(), "b".to_string()]);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn warn_when_over_threshold() {
        let members = (0..=MEMBER_THRESHOLD) // MEMBER_THRESHOLD + 1
            .map(|i| format!("crate-{i}"))
            .collect();
        let state = state_with_members(members);
        let entry = eval(&state);
        assert_eq!(entry.verdict, PolicyVerdict::Warn);
    }

    #[test]
    fn recommendation_contains_member_count() {
        let members: Vec<String> = (0..15).map(|i| format!("crate-{i}")).collect();
        let state = state_with_members(members);
        let entry = eval(&state);
        assert!(
            entry.recommendation.contains("15"),
            "expected member count 15 in: {}",
            entry.recommendation
        );
    }
}
