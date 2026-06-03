use crate::state::target::TargetVerdict;
use crate::state::workspace::WorkspaceState;

/// Derive autonomic signals from the current workspace state.
pub fn derive_signals(workspace: &WorkspaceState, target_verdict: &TargetVerdict) -> Vec<String> {
    let mut signals = Vec::new();

    if workspace.dirty {
        signals.push("workspace has uncommitted changes".to_string());
    }

    if workspace.target_size_gb > 15.0 {
        signals.push(format!(
            "target dir {:.1} GB exceeds 15 GB warning threshold",
            workspace.target_size_gb
        ));
    }

    if matches!(target_verdict, TargetVerdict::Fail) {
        signals.push("target dir exceeds max_size_gb — clean recommended".to_string());
    }

    if workspace.changed_trybuild_fixtures > 0 {
        signals.push(format!(
            "{} trybuild fixture(s) changed — review snapshots",
            workspace.changed_trybuild_fixtures
        ));
    }

    signals
}
