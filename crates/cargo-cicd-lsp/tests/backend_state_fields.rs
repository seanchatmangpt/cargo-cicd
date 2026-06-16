//! backend_state_fields — verifies that all Backend state types compile and
//! default-construct correctly.

use cargo_cicd_lsp::state::{CapabilityCache, DiagnosticStore, ReceiptIndex, WorkspaceState};

#[test]
fn backend_state_fields_are_initialized() {
    // just verify the types compile and default-construct
    let _cache = CapabilityCache::new();
    let _store = DiagnosticStore::new();
    let _idx = ReceiptIndex::new();
    let _ws = WorkspaceState::new();
}
