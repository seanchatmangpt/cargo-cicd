//! Fundamental domain primitives shared across all crates in the workspace.

/// The outcome of an adjudicated process step.
///
/// Mirrors the wasm4pm oracle vocabulary so that evidence consumers and the
/// CLI can use the same type without depending on the oracle crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verdict {
    /// All checks passed; the step is accepted.
    Pass,
    /// The step completed with non-blocking warnings.
    Warn,
    /// A blocking failure occurred.
    Fail,
    /// The oracle was unavailable; the step is neither accepted nor refused.
    Blocked,
}

impl Verdict {
    /// Returns `true` when the verdict allows execution to continue.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Pass | Self::Warn | Self::Blocked)
    }

    /// Returns `true` when the verdict is a hard failure.
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail)
    }

    /// Returns the string label used in log output and evidence files.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Blocked => "BLOCKED",
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl std::str::FromStr for Verdict {
    type Err = crate::error::CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "PASS" => Ok(Self::Pass),
            "WARN" => Ok(Self::Warn),
            "FAIL" => Ok(Self::Fail),
            "BLOCKED" => Ok(Self::Blocked),
            other => Err(crate::error::CoreError::config_invalid(
                "verdict",
                format!("unknown verdict `{other}`; expected PASS, WARN, FAIL, or BLOCKED"),
            )),
        }
    }
}

/// An opaque identifier for a workspace instance.
///
/// Used as a correlation key in process evidence (XES traces).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceId(String);

impl WorkspaceId {
    /// Construct a `WorkspaceId` from any string-like value.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the raw string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn verdict_roundtrip() {
        for (s, expected) in [
            ("PASS", Verdict::Pass),
            ("WARN", Verdict::Warn),
            ("FAIL", Verdict::Fail),
            ("BLOCKED", Verdict::Blocked),
        ] {
            let v = Verdict::from_str(s).expect("should parse");
            assert_eq!(v, expected);
            assert_eq!(v.label(), s);
        }
    }

    #[test]
    fn verdict_is_ok_semantics() {
        assert!(Verdict::Pass.is_ok());
        assert!(Verdict::Warn.is_ok());
        assert!(!Verdict::Fail.is_ok());
        assert!(Verdict::Blocked.is_ok());
    }

    #[test]
    fn workspace_id_display() {
        let id = WorkspaceId::new("my-project");
        assert_eq!(id.to_string(), "my-project");
        assert_eq!(id.as_str(), "my-project");
    }
}
