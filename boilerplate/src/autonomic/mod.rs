//! Autonomic policy suggestion layer.
//!
//! This module is only compiled when the `autonomic` feature flag is active.
//! All policies are **read-only**: they inspect [`crate::engine::EngineState`]
//! and emit human-readable recommendations. They never mutate state or invoke
//! external processes.
//!
//! # Usage
//!
//! ```rust,ignore
//! #[cfg(feature = "autonomic")]
//! {
//!     let state = EngineState::from_workspace();
//!     let report = project::autonomic::run_all_policies(&state);
//!     if report.has_warnings() {
//!         report.display();
//!     }
//! }
//! ```

#![cfg(feature = "autonomic")]

pub mod policy_engine;

pub use policy_engine::{run_all_policies, PolicyEntry, PolicyReport, PolicyVerdict};
