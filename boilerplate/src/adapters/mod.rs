//! Adapter registry — stateless translators from external sources to engine state.
//!
//! ## Silence contract
//!
//! Every public function in every adapter **must not panic** and **must not
//! return an error to the caller**.  When an external source is unavailable
//! (binary not on `PATH`, file not found, parse failure) the adapter returns
//! the `Default` value for its output type and optionally emits a
//! `tracing::warn!` so operators can diagnose the missing data.
//!
//! ## Isolation contract
//!
//! Adapters are **stateless** — they hold no fields and do not cache results
//! internally.  [`crate::engine::EngineState`] is the single owner of all
//! runtime data.  Adapters must not call other adapters.
//!
//! ## Performance budget
//!
//! | Tier | Max latency | Examples |
//! |------|-------------|---------|
//! | Fast | < 5 ms | TOML parse, filesystem stat |
//! | Medium | < 100 ms | `git status`, `rustc --version` |
//! | Slow | > 100 ms | recursive `walkdir` over `target/` |
//!
//! Slow adapters must be called last in `EngineState::from_workspace()` so
//! that fast data is ready for early-return exits.

#![cfg(feature = "process-data")]

pub mod git;
pub mod toolchain;
pub mod workspace;
