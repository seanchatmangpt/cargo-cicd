pub mod wasm4pm_current;
pub mod wasm4pm_shell;

// Advanced integrations (feature-gated)
#[cfg(feature = "advanced")]
pub mod metrics_collector;

// Cryptographic provenance receipts via the affidavit `affi` CLI (feature-gated
// shell-out oracle, mirroring wasm4pm_shell).
#[cfg(feature = "affidavit")]
pub mod affidavit_shell;

#[cfg(feature = "affidavit")]
pub use affidavit_shell::{
    affidavit_receipt_dir, AffidavitResult, AffidavitShell, AffidavitVerdict,
};

pub use wasm4pm_shell::{discover_wpm_binary, Wasm4pmShell, WpmResult, WpmVerdict};

pub use cargo_cicd_core::diagnostics::CicdCode;
/// Re-exported core types for wpm verdict handling.
pub use cargo_cicd_core::wpm::verdict::WpmVerdict as CoreWpmVerdict;
