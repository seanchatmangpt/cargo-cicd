//! Interactive live-refresh TUI dashboard for `cargo project status dashboard`.
//!
//! All submodules are gated behind the `tui` feature flag so the default
//! build remains lean. Enable with `--features tui`.
//!
//! # Entry point
//!
//! The `dashboard` noun (`src/nouns/dashboard.rs`) sets up the raw terminal,
//! constructs an [`app::App`], and runs the event loop. Rendering is delegated
//! to [`ui::render`].
//!
//! # Architecture
//!
//! ```text
//! EventHandler  ──tick/key──▶  App  ──engine snapshot──▶  EngineState
//!                               │
//!                               ▼
//!                           ui::render   (ratatui Frame)
//! ```
//!
//! - [`terminal`] — raw-mode setup and teardown.
//! - [`event`]    — thread-based event pump: key events + periodic ticks.
//! - [`app`]      — TUI lifecycle state (selected tab, refresh timer, etc.).
//! - [`ui`]       — ratatui widget rendering.

#[cfg(feature = "tui")]
pub mod app;
#[cfg(feature = "tui")]
pub mod event;
#[cfg(feature = "tui")]
pub mod terminal;
#[cfg(feature = "tui")]
pub mod ui;
