//! Process evidence models and freshness checks.
pub mod case_id;
pub mod event;
pub mod freshness;
pub mod receipt_ref;
pub mod timestamp;
pub use event::EvidenceEvent;
pub use freshness::EvidenceState;
