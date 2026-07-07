pub mod wasm4pm_shell;

// Note: `wasm4pm_current` (an ungated "deferred to v26.6.3+" integration
// stub) and `metrics_collector` (an `advanced`-gated adapter never
// constructed by any noun or the engine) were removed as orphaned
// scaffolding with zero call sites outside their own definitions.

// Cryptographic provenance receipts via the affidavit `affi` CLI (feature-gated
// shell-out oracle, mirroring wasm4pm_shell).
#[cfg(feature = "affidavit")]
pub mod affidavit_shell;

// Note: affidavit_shell's public items are consumed elsewhere via the full
// `crate::integrations::affidavit_shell::*` path (see src/evidence.rs,
// src/legacy_nouns/affidavit.rs, tests/affidavit_integration.rs), not through
// a re-export here, so the `pub use` previously on this line was itself
// unused and has been removed.

pub use wasm4pm_shell::{discover_wpm_binary, Wasm4pmShell, WpmResult, WpmVerdict};
