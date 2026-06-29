//! DEPRECATED: Use `crate::evidence_helpers` instead.
//! This module re-exports from the canonical location and will be deleted
//! once all callers are migrated. See CCICD-106.

#[deprecated(
    since = "0.0.0",
    note = "Use crate::evidence_helpers::init_evidence instead"
)]
pub use crate::evidence_helpers::init_evidence;

#[deprecated(
    since = "0.0.0",
    note = "Use crate::evidence_helpers::finish_evidence instead"
)]
pub use crate::evidence_helpers::finish_evidence;
