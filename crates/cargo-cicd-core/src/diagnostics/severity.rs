//! Severity levels.
/// Severity of a cicd diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CicdSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl CicdSeverity {
    /// Returns true if this severity level is blocking (Error), false otherwise.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Error)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_is_blocking_only_for_error() {
        assert!(CicdSeverity::Error.is_blocking());
        assert!(!CicdSeverity::Warning.is_blocking());
        assert!(!CicdSeverity::Information.is_blocking());
        assert!(!CicdSeverity::Hint.is_blocking());
    }
}
