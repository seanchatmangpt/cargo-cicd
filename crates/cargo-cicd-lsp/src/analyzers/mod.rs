//! Read-only workspace analyzers. Each takes a WorkspaceSnapshot and returns findings.
//!
//! No analyzer may commit, prune, publish, write receipts, or mutate the workspace.

use cargo_cicd_core::diagnostics::CicdFinding;
use cargo_cicd_core::workspace::WorkspaceSnapshot;

pub mod changed_tests;
pub mod close_readiness;
pub mod evidence;
pub mod git_phase;
pub mod public_boundary;
pub mod publish;
pub mod rendered_surface;
pub mod runtime_court;
pub mod target_hygiene;

/// A read-only workspace analyzer.
pub trait CicdAnalyzer: Send + Sync {
    /// Run the analyzer against a workspace snapshot.
    /// Returns zero or more findings. Never mutates the workspace.
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding>;
    /// Human-readable name for this analyzer.
    fn name(&self) -> &'static str;
}

/// Run all analyzers against a snapshot and collect findings.
pub fn run_all(snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
    let analyzers: Vec<Box<dyn CicdAnalyzer>> = vec![
        Box::new(git_phase::GitPhaseAnalyzer),
        Box::new(evidence::EvidenceAnalyzer),
        Box::new(publish::PublishAnalyzer),
        Box::new(public_boundary::PublicBoundaryAnalyzer),
        Box::new(runtime_court::RuntimeCourtAnalyzer),
        Box::new(runtime_court::VerdictKeyMismatchAnalyzer),
        Box::new(rendered_surface::RenderedSurfaceAnalyzer),
        Box::new(close_readiness::CloseReadinessAnalyzer),
        Box::new(target_hygiene::TargetHygieneAnalyzer),
        Box::new(changed_tests::ChangedTestsAnalyzer),
    ];
    analyzers.iter().flat_map(|a| a.analyze(snapshot)).collect()
}
