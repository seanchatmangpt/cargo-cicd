//! Pending repair — mark a finding as awaiting repair evidence.

use cargo_cicd_core::diagnostics::{CicdFinding, DiagnosticLifecycle};

/// Advance a finding's lifecycle to `PendingRepair`.
///
/// Call this after routing a finding when repair has been initiated but evidence is not yet
/// present.
pub fn mark_pending(finding: &mut CicdFinding) {
    finding.lifecycle = DiagnosticLifecycle::PendingRepair;
}
