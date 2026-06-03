//! Lifecycle clear operations — keyed subtraction on the diagnostic store.

use crate::state::DiagnosticStore;

/// Remove only the finding with the given `code_str` from `uri`.
/// All other findings for that URI are left intact.
pub fn clear_by_code(store: &mut DiagnosticStore, uri: &str, code_str: &str) {
    store.remove_code(uri, code_str);
}
