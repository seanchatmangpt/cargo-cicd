//! `project` — public library surface.
//!
//! Re-exports the types most commonly needed by integration tests and any
//! downstream crates that embed project as a library dependency.
//!
//! # Feature flags
//!
//! | Flag | Purpose |
//! |------|---------|
//! | `process-data` | Enable Level 5 [`engine::EngineState`] and all adapters. |
//! | `autonomic` | Enable read-only policy suggestion layer (implies `process-data`). |
//! | `completions` | Enable shell completion generation via `clap_complete`. |
//! | `advanced` | Enable high-performance scan, cache, and observability extensions. |

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Noun modules are always available so the CLI grammar compiles in all configs.
pub mod nouns;
pub mod ui;

// Shell completions — opt-in so the binary stays lean by default.
#[cfg(feature = "completions")]
pub mod completions;

// Level 5 engine and adapters are opt-in.
#[cfg(feature = "process-data")]
pub mod adapters;
#[cfg(feature = "process-data")]
pub mod engine;

// Autonomic policy layer (read-only suggestions).
#[cfg(feature = "autonomic")]
pub mod autonomic;
#[cfg(feature = "autonomic")]
pub mod policies;

// Interactive TUI dashboard.
#[cfg(feature = "tui")]
pub mod tui;

// Advanced capabilities: parallel scan, fingerprinting, caching, observability.
#[cfg(feature = "advanced")]
pub mod advanced;

// Re-export the most commonly needed types at the crate root.
#[cfg(feature = "process-data")]
pub use engine::EngineState;

// Re-export the core error types.
pub use project_core::{CoreError, Result, Verdict, WorkspaceId};
