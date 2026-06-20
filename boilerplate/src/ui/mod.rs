//! Zero-dependency terminal UI design system.
//!
//! All terminal output in noun modules must go through this module.
//!
//! ## Rules (never break these)
//!
//! 1. **All colour** goes through [`style::Style::paint`] — never write raw
//!    ANSI escape codes inside noun modules.
//! 2. **All glyphs** go through [`symbols`] constants — never hard-code
//!    Unicode box-drawing characters or emoji inline.
//! 3. **Column widths** for tables must use [`text::display_width`], not
//!    `.len()`, to handle multi-byte characters correctly.
//! 4. When stdout is **not a TTY** (piped output), all output must be plain
//!    ASCII with no escape codes.  This is enforced by [`caps`] detection and
//!    verified by tests that capture non-TTY output.
//!
//! ## Module map
//!
//! | Module | Role |
//! |--------|------|
//! | [`caps`] | Detect terminal capabilities (colour, Unicode, TTY). |
//! | [`style`] | `Style::paint` — single entry point for coloured text. |
//! | [`symbols`] | Named glyph constants with ASCII fallbacks. |
//! | [`text`] | String helpers (`display_width`, `truncate`). |
//! | [`badge`] | Inline `[PASS]`/`[FAIL]`/`[WARN]` status badges. |
//! | [`theme`] | Named colour palette. |

pub mod badge;
pub mod caps;
pub mod style;
pub mod symbols;
pub mod text;
pub mod theme;
