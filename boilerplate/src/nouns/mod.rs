//! Noun registry — one module per top-level CLI noun.
//!
//! ## Adding a new noun
//!
//! 1. Create `src/nouns/<noun>.rs` implementing the noun's `Args` struct and
//!    a `run(args: NounArgs) -> anyhow::Result<()>` entry point.
//! 2. Register a public `mod <noun>;` below.
//! 3. Add a `NounCommands::<Noun>` variant in `main.rs` and wire the `run()`
//!    call in the `match` arm.
//! 4. Add a default-verb entry in `DEFAULT_VERBS` in `main.rs` if the noun
//!    has a sensible default verb.
//!
//! ## Noun design rules
//!
//! - Nouns are **read-only by default**.  Any destructive action must require
//!   an explicit `--confirm` flag.
//! - Nouns read from [`crate::engine::EngineState`]; they do not perform raw
//!   I/O themselves.
//! - All terminal output goes through [`crate::ui`] — never write raw ANSI
//!   escape codes or hard-coded Unicode glyphs inside a noun module.

pub mod status;
pub mod workspace;
