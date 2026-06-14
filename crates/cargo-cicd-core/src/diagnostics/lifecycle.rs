//! Diagnostic lifecycle states.
/// Lifecycle state of a diagnostic.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DiagnosticLifecycle {
    #[default]
    Raised,
    PendingRepair,
    Routed,
    ResidualPreserved,
    Cleared,
}
