//! Route population — attach repair routes to a finding.

use cargo_cicd_core::diagnostics::{CicdFinding, DiagnosticLifecycle, RepairRoute};

/// Populate the first repair route on a finding and advance its lifecycle to `Routed`.
pub fn populate_routes(finding: &mut CicdFinding, routes: Vec<RepairRoute>) {
    finding.route = routes.into_iter().next();
    finding.lifecycle = DiagnosticLifecycle::Routed;
}
