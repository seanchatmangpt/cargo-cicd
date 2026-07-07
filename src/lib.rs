//! # cargo-cicd
//!
//! Keep Rust workspaces clean, fast, and push-ready.
//!
//! cargo-cicd is a local-first CI/CD helper that manufactures a noun-verb CLI
//! grammar from an ontology, emits process evidence in OCEL 2.0 format, and
//! delegates all verdict adjudication to the external wasm4pm oracle.
//!
//! ## Documentation
//!
//! | Quadrant | When to use it |
//! |----------|----------------|
//! | **Tutorials** — `docs/tutorials/` | Learning from scratch: follow a guided journey to one concrete outcome |
//! | **How-to guides** — `docs/how-to/` | Solve a specific problem: assumes you already know the basics |
//! | **Reference** — `cargo doc` (here) | Look up a type, function, or flag: exhaustive, accurate, machine-generated |
//! | **Explanation** — `docs/explanation/` | Understand the design: read ADRs and architectural rationale |
//!
//! ## Quick start (library)
//!
//! Query the current workspace state — reads Cargo.toml, git, and the target dir:
//!
//! ```no_run
//! use cargo_cicd::EngineState;
//! let state = EngineState::from_workspace();
//! println!("workspace: {}", state.workspace.name);
//! println!("branch:    {}", state.git_phase.branch);
//! let dirty = state.git_phase.dirty_files.len();
//! if dirty == 0 {
//!     println!("status: CLEAN");
//! } else {
//!     println!("status: DIRTY — {} file(s) with uncommitted changes", dirty);
//! }
//! ```
//!
//! Construct a default state for unit tests (no filesystem access):
//!
//! ```
//! use cargo_cicd::EngineState;
//! let state = EngineState::default();
//! assert!(state.git_phase.dirty_files.is_empty());
//! assert_eq!(state.git_phase.ahead, 0);
//! assert!(!state.trybuild.run_all_by_default, "conservative mode is the default");
//! ```
//!
//! ## Stability
//!
//! Only the types re-exported at the crate root (currently [`EngineState`],
//! [`CicdToml`], and the `core` re-export of `cargo_cicd_core`) carry a
//! compatibility guarantee. All other modules in this crate are internal
//! implementation detail: they are technically reachable (marked `pub` for
//! internal cross-crate use within the cargo-cicd workspace) but are hidden
//! from generated documentation and may change or be removed without notice
//! in any release, including patch releases. Do not depend on their paths,
//! types, or signatures.

#[doc(hidden)]
pub mod adapters;
#[cfg(feature = "advanced")]
#[doc(hidden)]
pub mod advanced;
#[doc(hidden)]
pub mod autonomic;
#[doc(hidden)]
pub mod barrier;
#[doc(hidden)]
pub mod certification;
pub mod cicd_toml;
#[doc(hidden)]
pub mod code_provenance;
pub mod engine;
#[doc(hidden)]
pub mod evidence;
#[doc(hidden)]
pub mod evidence_helpers;
#[doc(hidden)]
pub mod evidence_jsonl;
#[doc(hidden)]
pub mod evidence_manifest;
#[doc(hidden)]
pub mod evidence_sarif;
#[doc(hidden)]
pub mod evidence_xes_v2;
#[doc(hidden)]
pub mod integrations;
#[doc(hidden)]
pub mod legacy_nouns;
#[doc(hidden)]
pub mod nouns;

#[doc(hidden)]
pub mod ocel;
#[doc(hidden)]
pub mod oracle_keys;
#[doc(hidden)]
pub mod policies;
#[doc(hidden)]
pub mod receipt_validation;
#[doc(hidden)]
pub mod session;
#[doc(hidden)]
pub mod ui;

pub use cicd_toml::CicdToml;
pub use engine::EngineState;

// Re-export the shared domain types from cargo-cicd-core.
pub use cargo_cicd_core as core;
