//! Git phase and status models.
pub mod base_ref;
pub mod head;
pub mod phase;
pub mod status;
pub use base_ref::{resolve_base_ref, BaseRefError, DEFAULT_BASE_CHAIN};
pub use status::{GitStatus, GitStatusSummary, GitStatusError};
