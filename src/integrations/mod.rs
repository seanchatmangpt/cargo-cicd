pub mod wasm4pm_current;
pub mod wasm4pm_shell;

pub use wasm4pm_shell::{Wasm4pmShell, WpmResult, WpmVerdict};

/// Re-exported core types for wpm verdict handling.
pub use cargo_cicd_core::wpm::verdict::WpmVerdict as CoreWpmVerdict;
pub use cargo_cicd_core::diagnostics::CicdCode;
