//! Severity levels.
/// Severity of a cicd diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CicdSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl std::fmt::Display for CicdSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Information => write!(f, "info"),
            Self::Hint => write!(f, "hint"),
        }
    }
}
