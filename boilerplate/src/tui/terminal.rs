//! Raw terminal setup and teardown.
//!
//! [`setup_terminal`] enables raw mode and the alternate screen buffer so the
//! dashboard does not corrupt the user's scroll-back history. [`restore_terminal`]
//! is the exact inverse and **must** be called on every exit path — including
//! panics. The dashboard noun wraps the main loop in a `panic::catch_unwind` to
//! guarantee this.
//!
//! ## Why alternate screen?
//!
//! The alternate screen is a separate terminal buffer. Switching to it hides the
//! normal terminal contents; switching back restores them exactly. This means the
//! dashboard disappears cleanly when the user quits, leaving the shell prompt in
//! the same state it was before.

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// The concrete terminal type used throughout the TUI.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Switch the terminal into raw mode and open the alternate screen.
///
/// Call [`restore_terminal`] before the process exits to undo these changes.
///
/// # Errors
///
/// Returns an error if `crossterm` is unable to configure the terminal — for
/// example when stdout is not a TTY.
pub fn setup_terminal() -> Result<Tui> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("failed to create ratatui Terminal")
}

/// Restore the terminal to its original state.
///
/// This disables raw mode, leaves the alternate screen, hides the mouse
/// capture, and shows the cursor. Must be called even when the application
/// terminates abnormally — prefer wrapping the main loop with
/// `std::panic::catch_unwind` to guarantee execution.
///
/// # Errors
///
/// Returns an error if any of the restoration steps fail, though in practice
/// crossterm rarely fails here.
pub fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Terminal setup/teardown cannot be tested without a real TTY. We verify
    // the public API compiles and that the type alias resolves correctly.

    #[test]
    fn tui_type_alias_is_usable() {
        // Ensure the Tui type alias compiles — no runtime assertion needed.
        fn _accepts_tui(_: &super::Tui) {}
    }
}
