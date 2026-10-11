//! cargo-cicd terminal design system (zero-dependency, std-only).
//!
//! A self-contained UI toolkit for the CLI: ANSI styling, glyphs, panels,
//! badges, and charts. Every component degrades gracefully:
//!
//! * color auto-disables when stdout is not a TTY, when `NO_COLOR` is set, or
//!   when `--no-color` was passed (see [`caps`]);
//! * Unicode glyphs fall back to ASCII when the locale is not UTF-8 or
//!   `CICD_ASCII` is set (see [`symbols`]).
//!
//! Because color is suppressed on non-terminals, piped/captured output is plain
//! text — which keeps the public-boundary substring contracts stable.

// ── foundation (stable, owned by the core) ──────────────────────────────────
pub mod caps;
pub mod style;
pub mod symbols;
pub mod text;

// ── components (each module is an independent unit) ──────────────────────────
pub mod badge;
pub mod chart;
pub mod panel;
pub mod theme;
