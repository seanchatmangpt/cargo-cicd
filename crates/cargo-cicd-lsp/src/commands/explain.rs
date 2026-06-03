//! Explain command — delegates to cargo_cicd_core diagnostics.

/// Return a prose explanation for the given diagnostic code string.
/// Delegates to [`cargo_cicd_core::diagnostics::code::explain_code`].
pub fn explain_code(code: &str) -> String {
    cargo_cicd_core::diagnostics::code::explain_code(code)
}
