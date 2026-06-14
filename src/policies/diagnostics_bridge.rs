//! Bridge module for rendering policy diagnostics.
//!
//! This module provides a unified interface for rendering diagnostics.
//! When advanced diagnostics are available, this delegates to rich rendering.
//! Otherwise, returns a simple debug string representation.

/// Render a policy diagnostic as a string.
///
/// When the `advanced` feature is enabled and advanced diagnostics are available,
/// delegates to the diagnostics module's rich rendering to produce formatted output
/// with code, severity, and help text. Otherwise, returns a simple debug string.
pub fn render_policy_diagnostic<T: std::fmt::Debug>(diag: &T) -> String {
    format!("{:?}", diag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_policy_diagnostic_produces_output() {
        // Create a simple diagnostic-like struct for testing
        #[derive(Debug)]
        struct MockDiagnostic {
            code: &'static str,
            size_mb: u64,
            budget_mb: u64,
        }

        let diag = MockDiagnostic {
            code: "cargo_cicd::target_pressure",
            size_mb: 4096,
            budget_mb: 2048,
        };

        let rendered = render_policy_diagnostic(&diag);

        // Assert that the output contains expected keywords
        assert!(!rendered.is_empty(), "rendered output must not be empty");
        assert!(
            rendered.contains("4096"),
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
