pub mod wasm4pm_current;
pub mod wasm4pm_shell;

// Advanced integrations (feature-gated)
#[cfg(feature = "advanced")]
pub mod metrics_collector;

pub use wasm4pm_shell::{Wasm4pmShell, WpmResult, WpmVerdict};
