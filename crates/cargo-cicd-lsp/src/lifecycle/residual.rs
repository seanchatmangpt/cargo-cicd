//! Residual preservation — retain findings as ResidualPreserved after clearing.

use cargo_cicd_core::diagnostics::{CicdFinding, DiagnosticLifecycle};

/// Mark a finding's lifecycle as ResidualPreserved.
/// Call this before re-inserting a finding that should remain visible as a residual record.
pub fn mark_residual(finding: &mut CicdFinding) {
    finding.lifecycle = DiagnosticLifecycle::ResidualPreserved;
}
