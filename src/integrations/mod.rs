pub mod wasm4pm_current;
pub mod wasm4pm_shell;

// Advanced integrations (feature-gated)
#[cfg(feature = "advanced")]
pub mod metrics_collector;

pub use wasm4pm_shell::{Wasm4pmShell, WpmResult, WpmVerdict};

pub use cargo_cicd_core::diagnostics::CicdCode;
/// Re-exported core types for wpm verdict handling.
pub use cargo_cicd_core::wpm::verdict::WpmVerdict as CoreWpmVerdict;
