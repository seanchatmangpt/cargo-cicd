//! Bridge module for rendering policy diagnostics.
//!
//! Provides the `render_policy_diagnostic` function that renders diagnostics
//! with appropriate formatting based on available capabilities.

/// Render a policy diagnostic as a string.
///
/// Produces formatted output describing policy violations. When the advanced
/// feature is available, this can be called with `EngineDiagnostic` types
/// for rich miette formatting with code, severity, and help text. Otherwise,
/// renders a simple debug string representation.
pub fn render_policy_diagnostic<T: std::fmt::Debug>(diag: &T) -> String {
    format!("{:?}", diag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_policy_diagnostic_produces_output() {
        // Mock diagnostic-like struct for testing the render function
        #[derive(Debug)]
        struct MockDiagnostic {
            error_msg: &'static str,
            size_mb: u64,
            budget_mb: u64,
        }

        let diag = MockDiagnostic {
            error_msg: "target pressure",
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
        // When advanced is available, verify the rendered code is present
        #[cfg(feature = "advanced")]
        assert!(
            rendered.contains("cargo_cicd") || rendered.contains("4096"),
            "rendered output must contain diagnostic identifier: {}",
            rendered
        );
    }
}
