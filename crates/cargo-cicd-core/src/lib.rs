//! Shared workspace and process-state domain for cargo-cicd tooling.
//!
//! This crate owns workspace snapshots, git models, evidence event models,
//! receipt references, diagnostic codes, finding structures, lifecycle states,
//! and repair routes. It has no knowledge of LSP, editors, or tower-lsp.

pub mod diagnostics;
pub mod evidence;
pub mod git;
pub mod ggen;
pub mod public_boundary;
pub mod publish;
pub mod target;
pub mod tests_changed;
pub mod workspace;
pub mod wpm;
