//! Severity levels.
/// Severity of a cicd diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CicdSeverity {
    Error,
    Warning,
    Information,
    Hint,
}
