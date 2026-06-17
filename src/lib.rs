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

#![allow(dead_code, unused_imports)]

pub mod adapters;
#[cfg(feature = "advanced")]
pub mod advanced;
pub mod autonomic;
pub mod certification;
pub mod cicd_toml;
pub mod code_provenance;
pub mod engine;
pub mod evidence;
pub mod evidence_jsonl;
pub mod evidence_xes_v2;
pub mod integrations;
pub mod nouns;
pub mod ocel;
pub mod oracle_keys;
pub mod policies;
pub mod receipt_validation;
pub mod session;
pub mod state;
pub mod ui;

pub use cicd_toml::CicdToml;
pub use engine::EngineState;

// Re-export the shared domain types from cargo-cicd-core.
pub use cargo_cicd_core as core;
