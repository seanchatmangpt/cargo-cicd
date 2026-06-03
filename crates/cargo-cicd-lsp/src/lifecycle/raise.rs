//! Raise a new finding into the diagnostic store.

use cargo_cicd_core::diagnostics::CicdFinding;

use crate::state::DiagnosticStore;

/// Insert a new finding for the given URI.
/// The finding lifecycle is `Raised` by construction.
pub fn raise(store: &mut DiagnosticStore, uri: impl Into<String>, finding: CicdFinding) {
    store.insert(uri.into(), finding);
}
