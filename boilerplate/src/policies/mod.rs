//! Individual policy implementations for the autonomic suggestion layer.
//!
//! Each sub-module exposes a single `pub fn eval(state: &EngineState) -> PolicyEntry`
//! function.  All policies are **read-only** — they never mutate state or call
//! external processes.
//!
//! Policies are assembled by [`crate::autonomic::policy_engine::run_all_policies`].

#![cfg(feature = "autonomic")]

pub mod branch_behind;
pub mod git_phase_dirty;
pub mod large_workspace;
pub mod stale_toolchain;
pub mod toolchain_mismatch;
pub mod uncommitted_changes;
pub mod untracked_files;
