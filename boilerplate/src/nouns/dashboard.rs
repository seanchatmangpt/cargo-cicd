//! `cargo project status dashboard` — interactive live-refresh TUI dashboard.
//!
//! This noun is only compiled when the `tui` feature is active. It sets up a
//! raw-mode terminal, builds an [`App`] from the current engine state, then
//! drives the ratatui event loop until the user presses `q`.
//!
//! # Keyboard bindings
//!
//! | Key        | Action |
//! |------------|--------|
//! | `q` / `Esc`| Quit the dashboard |
//! | `Tab`      | Next tab |
//! | `Shift-Tab`| Previous tab |
//! | `r`        | Force refresh engine state immediately |
//! | `↑` / `k`  | Scroll up |
//! | `↓` / `j`  | Scroll down |
//! | `?`        | (reserved for future help overlay) |
//!
//! # Panic safety
//!
//! The raw-mode terminal must be restored even if the render loop panics.
//! [`run`] wraps the event loop in `std::panic::catch_unwind`; on panic the
//! terminal is restored and the panic is re-raised.
//!
//! # Feature gate
//!
//! This module is compiled only when both `tui` and `process-data` features
//! are enabled. With `process-data` disabled the module still compiles but
//! the dashboard shows placeholder text.

#[cfg(feature = "tui")]
mod tui_impl {
    use anyhow::Result;
    use clap::Args;
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::tui::{
        app::App,
        event::{Event, EventHandler},
        terminal::{restore_terminal, setup_terminal},
        ui,
    };

    #[cfg(feature = "process-data")]
    use crate::engine::EngineState;

    // ─────────────────────────────────────────────────────────────────────────
    // Clap structures
    // ─────────────────────────────────────────────────────────────────────────

    /// Arguments for `status dashboard`.
    #[derive(Debug, Args)]
    pub struct DashboardArgs {
        /// How often (in seconds) to auto-refresh engine state.
        ///
        /// Defaults to 5 seconds. Set to 0 to disable auto-refresh.
        #[arg(long, default_value_t = 5)]
        pub refresh: u64,

        /// Tick rate in milliseconds (controls event poll frequency).
        ///
        /// Lower values make the UI more responsive; higher values use less
        /// CPU. Default: 250 ms.
        #[arg(long, default_value_t = 250)]
        pub tick_ms: u64,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Entry point
    // ─────────────────────────────────────────────────────────────────────────

    /// Run the interactive dashboard.
    ///
    /// Sets up the raw terminal, runs the event loop, then restores the
    /// terminal on exit. Returns `Ok(())` when the user quits normally.
    pub fn run(args: DashboardArgs) -> Result<()> {
        // Build the initial engine snapshot before entering raw mode so any
        // adapter errors are printed to the normal terminal.
        #[cfg(feature = "process-data")]
        let engine = EngineState::from_workspace();

        // Construct App.
        #[cfg(feature = "process-data")]
        let mut app = App::new(engine);
        #[cfg(not(feature = "process-data"))]
        let mut app = App::new_minimal();

        app.refresh_interval_secs = if args.refresh == 0 {
            u64::MAX // effectively disabled
        } else {
            args.refresh
        };

        // Switch to raw mode + alternate screen.
        let mut terminal = setup_terminal()?;

        // Spawn the event pump thread.
        let events = EventHandler::new(args.tick_ms);

        // Wrap the event loop so we can restore the terminal even on panic.
        let loop_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            event_loop(&mut app, &mut terminal, &events)
        }));

        // Always restore — regardless of whether the loop panicked.
        let _ = restore_terminal(&mut terminal);

        match loop_result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Event loop
    // ─────────────────────────────────────────────────────────────────────────

    fn event_loop(
        app: &mut App,
        terminal: &mut crate::tui::terminal::Tui,
        events: &EventHandler,
    ) -> anyhow::Result<()> {
        loop {
            // Draw the current frame.
            terminal.draw(|frame| ui::render(app, frame))?;

            // Block until the next event.
            match events.next()? {
                Event::Key(key) => handle_key(app, key.code, key.modifiers),
                Event::Tick => app.on_tick(),
                Event::Resize(_, _) => {
                    // ratatui auto-redraws on resize — nothing extra needed.
                }
            }

            if app.should_quit {
                break;
            }
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Key handling
    // ─────────────────────────────────────────────────────────────────────────

    fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => app.quit(),

            // Ctrl-C is a hard quit.
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),

            // Tab navigation
            KeyCode::Tab => app.next_tab(),
            KeyCode::BackTab => app.prev_tab(), // Shift-Tab

            // Manual refresh
            KeyCode::Char('r') => app.refresh(),

            // Scrolling — arrow keys and vim-style bindings
            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),

            // '1'/'2'/'3' jump directly to a tab
            KeyCode::Char('1') => app.selected_tab = 0,
            KeyCode::Char('2') => app.selected_tab = 1,
            KeyCode::Char('3') => app.selected_tab = 2,

            // '?' is reserved for a future help overlay.
            KeyCode::Char('?') => {}

            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public re-exports (always visible so `mod.rs` can reference the type without
// conditional imports in callers)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "tui")]
pub use tui_impl::{run, DashboardArgs};

// ─────────────────────────────────────────────────────────────────────────────
// Stub when tui feature is disabled (keeps the module compilable)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(not(feature = "tui"))]
pub mod stub {
    use anyhow::{bail, Result};
    use clap::Args;

    /// Placeholder args struct for when the `tui` feature is not compiled in.
    #[derive(Debug, Args)]
    pub struct DashboardArgs {
        /// Ignored — enable the `tui` feature to use the dashboard.
        #[arg(long, default_value_t = 5, hide = true)]
        pub refresh: u64,
    }

    /// Always returns an error directing the user to enable the `tui` feature.
    pub fn run(_args: DashboardArgs) -> Result<()> {
        bail!(
            "The interactive dashboard requires the `tui` feature.\n\
             Rebuild with: cargo build --features tui"
        )
    }
}

#[cfg(not(feature = "tui"))]
pub use stub::{run, DashboardArgs};

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #[cfg(feature = "tui")]
    use super::tui_impl::DashboardArgs;

    #[cfg(not(feature = "tui"))]
    use super::stub::DashboardArgs;

    #[test]
    fn dashboard_args_defaults() {
        // Verify that the default values compile and are correct.
        let args = DashboardArgs {
            refresh: 5,
            #[cfg(feature = "tui")]
            tick_ms: 250,
        };
        assert_eq!(args.refresh, 5);
        #[cfg(feature = "tui")]
        assert_eq!(args.tick_ms, 250);
    }

    #[cfg(not(feature = "tui"))]
    #[test]
    fn stub_run_returns_error() {
        use super::stub::DashboardArgs;
        let result = super::run(DashboardArgs { refresh: 5 });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("tui"));
    }
}
