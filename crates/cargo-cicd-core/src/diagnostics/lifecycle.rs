//! Diagnostic lifecycle states.
/// Lifecycle state of a diagnostic.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticLifecycle {
    #[default]
    Raised,
    PendingRepair,
    Routed,
    ResidualPreserved,
    Cleared,
}
