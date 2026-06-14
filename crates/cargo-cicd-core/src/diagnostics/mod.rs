//! Diagnostic codes, finding structures, severity, lifecycle, and route.
pub mod code;
pub mod finding;
pub mod lifecycle;
pub mod route;
pub mod severity;
pub use code::explain_code;
pub use code::CicdCode;
pub use finding::{CicdFinding, RepairRoute};
pub use lifecycle::DiagnosticLifecycle;
pub use severity::CicdSeverity;
