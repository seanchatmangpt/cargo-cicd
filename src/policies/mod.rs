pub mod git_phase_dirty;
pub mod target_pressure;
pub mod toolchain_mismatch;
pub mod trybuild_changed;

pub use git_phase_dirty::GitPhaseDirtyPolicy;
pub use target_pressure::TargetPressurePolicy;
pub use toolchain_mismatch::ToolchainMismatchPolicy;
pub use trybuild_changed::TrybuildChangedPolicy;

/// Policy mode — only suggest is enabled by default
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyMode {
    Suggest,
    Apply,
}

/// Policy verdict
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyVerdict {
    Pass,
    Warn,
    Alert,
}

/// Common policy interface
pub trait CicdPolicy {
    fn name(&self) -> &'static str;
    fn enabled(&self) -> bool;
    fn mode(&self) -> PolicyMode;
    fn evaluate(&self) -> PolicyResult;
}

#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub name: String,
    pub enabled: bool,
    pub mode: String,
    pub verdict: String,
    pub recommendation: Option<String>,
    pub event_kind: String,
}

#[cfg(feature = "advanced")]
use crate::advanced::diagnostics::EngineDiagnostic;

#[cfg(feature = "advanced")]
/// Render a policy diagnostic using the advanced diagnostics module when available.
///
/// When the `advanced` feature is enabled, delegates to the diagnostics module's
/// rich rendering to produce formatted output with code, severity, and help text.
pub fn render_policy_diagnostic(diag: &EngineDiagnostic) -> String {
    crate::advanced::diagnostics::render(diag)
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;

    #[test]
    fn render_policy_diagnostic_produces_output() {
        use crate::advanced::diagnostics::EngineDiagnostic;

        // Construct a mock diagnostic for target pressure
        let diag = EngineDiagnostic::TargetPressure {
            size_mb: 4096,
            budget_mb: 2048,
        };

        let rendered = render_policy_diagnostic(&diag);

        // Assert that the output contains expected keywords
        assert!(!rendered.is_empty(), "rendered output must not be empty");
        assert!(
            rendered.contains("target pressure") || rendered.contains("4096"),
            "rendered output must contain diagnostic details: {}",
            rendered
        );
        assert!(
            rendered.contains("cargo_cicd::target_pressure"),
            "rendered output must contain diagnostic code: {}",
            rendered
        );
    }
}
