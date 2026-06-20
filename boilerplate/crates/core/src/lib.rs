//! `project-core` — shared domain types, error taxonomy, and primitives.
//!
//! This crate is the only place where typed error variants are defined.
//! Downstream crates use `anyhow::Error` for ergonomic propagation, but they
//! are expected to wrap domain errors from here so callers can match on them.
//!
//! # Design principles
//!
//! - **No I/O** — this crate performs zero file system or network operations.
//!   It is a pure value/type library.
//! - **No panic** — every public function returns `Result` or a plain value.
//! - **Stable ABI** — structs used in `cicd.toml` or XES serialization carry
//!   `#[non_exhaustive]` so downstream can add fields without a semver break.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod primitives;

pub use error::{CoreError, Result};
pub use primitives::{Verdict, WorkspaceId};
