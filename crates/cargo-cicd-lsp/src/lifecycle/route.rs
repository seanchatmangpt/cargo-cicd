//! Route population — attach repair routes to a finding.

use cargo_cicd_core::diagnostics::route::RepairRoute;
use cargo_cicd_core::diagnostics::{CicdFinding, DiagnosticLifecycle};

/// Populate repair routes on a finding and advance its lifecycle to `Routed`.
///
/// Existing routes are replaced by the provided `routes` list.
pub fn populate_routes(finding: &mut CicdFinding, routes: Vec<RepairRoute>) {
    finding.routes = routes.clone();
    finding.route_commands = routes.iter().map(|r| r.command.clone()).collect();
    finding.lifecycle = DiagnosticLifecycle::Routed;
}
