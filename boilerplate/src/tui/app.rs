//! TUI application lifecycle state.
//!
//! [`App`] holds everything the dashboard needs between frames: which tab is
//! selected, when the engine state was last refreshed, and whether the user has
//! requested quit. It deliberately does **not** hold ratatui types — those are
//! constructed transiently in [`crate::tui::ui::render`].
//!
//! # Refresh lifecycle
//!
//! Every [`crate::tui::event::Event::Tick`] calls [`App::on_tick`].  That
//! method checks whether `refresh_interval_secs` has elapsed since
//! `last_refresh`; if so, it calls [`App::refresh`] which re-runs all
//! adapters via [`EngineState::from_workspace`].  The refresh is synchronous
//! and happens on the main thread — for the typical < 100 ms adapter budget
//! this is imperceptible to the user.

use std::time::{Duration, Instant};

#[cfg(feature = "process-data")]
use crate::engine::EngineState;

/// The number of tabs in the dashboard.
const TAB_COUNT: usize = 3;

/// TUI lifecycle state — everything the dashboard needs across frames.
#[derive(Debug)]
pub struct App {
    /// The most recent engine snapshot, refreshed on the tick interval.
    #[cfg(feature = "process-data")]
    pub engine: EngineState,

    /// How often (in seconds) to re-run the adapters and update the display.
    /// Default: 5 seconds.
    pub refresh_interval_secs: u64,

    /// When was the engine state last populated?
    pub last_refresh: Instant,

    /// Set to `true` by [`App::quit`]; the event loop exits on the next
    /// iteration.
    pub should_quit: bool,

    /// Index of the currently visible tab.
    ///
    /// | Index | Name      |
    /// |-------|-----------|
    /// | 0     | Overview  |
    /// | 1     | Git       |
    /// | 2     | Toolchain |
    pub selected_tab: usize,

    /// Vertical scroll offset for the main panel (used by the Git tab when
    /// the dirty-file list overflows the visible area).
    pub scroll_offset: u16,
}

impl App {
    /// Construct a new [`App`] from an initial engine snapshot.
    ///
    /// The refresh timer starts immediately; `on_tick` will not trigger
    /// another refresh until `refresh_interval_secs` have elapsed.
    #[cfg(feature = "process-data")]
    pub fn new(engine: EngineState) -> Self {
        Self {
            engine,
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        }
    }

    /// Construct a minimal [`App`] for environments where `process-data` is
    /// not compiled in. All engine-derived fields are absent; the UI will
    /// show placeholder text.
    #[cfg(not(feature = "process-data"))]
    pub fn new_minimal() -> Self {
        Self {
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        }
    }

    /// Signal the event loop to exit after the current frame.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Re-run all adapters to get a fresh [`EngineState`] snapshot and reset
    /// the refresh timer.
    #[cfg(feature = "process-data")]
    pub fn refresh(&mut self) {
        self.engine = EngineState::from_workspace();
        self.last_refresh = Instant::now();
        self.scroll_offset = 0; // reset scroll on manual refresh
    }

    /// No-op refresh when `process-data` is not available.
    #[cfg(not(feature = "process-data"))]
    pub fn refresh(&mut self) {
        self.last_refresh = Instant::now();
    }

    /// Advance to the next tab, wrapping around at the end.
    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % TAB_COUNT;
    }

    /// Go back to the previous tab, wrapping around at the beginning.
    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            TAB_COUNT - 1
        } else {
            self.selected_tab - 1
        };
    }

    /// Scroll the main panel down by one line.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll the main panel up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Called on every [`crate::tui::event::Event::Tick`].
    ///
    /// Triggers a background refresh when the configured interval has elapsed.
    pub fn on_tick(&mut self) {
        let elapsed = self.last_refresh.elapsed();
        if elapsed >= Duration::from_secs(self.refresh_interval_secs) {
            self.refresh();
        }
    }

    /// How many seconds remain until the next automatic refresh.
    pub fn secs_until_refresh(&self) -> u64 {
        let elapsed_secs = self.last_refresh.elapsed().as_secs();
        self.refresh_interval_secs.saturating_sub(elapsed_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a minimal App without the process-data feature for unit tests.
    fn make_test_app() -> App {
        App {
            #[cfg(feature = "process-data")]
            engine: {
                // Use Default so tests don't need a real workspace.
                crate::engine::EngineState::default()
            },
            refresh_interval_secs: 5,
            last_refresh: Instant::now(),
            should_quit: false,
            selected_tab: 0,
            scroll_offset: 0,
        }
    }

    #[test]
    fn quit_sets_flag() {
        let mut app = make_test_app();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn next_tab_wraps_around() {
        let mut app = make_test_app();
        app.selected_tab = 0;
        app.next_tab(); // 0 → 1
        assert_eq!(app.selected_tab, 1);
        app.next_tab(); // 1 → 2
        assert_eq!(app.selected_tab, 2);
        app.next_tab(); // 2 → 0 (wrap)
        assert_eq!(app.selected_tab, 0);
    }

    #[test]
    fn prev_tab_wraps_around() {
        let mut app = make_test_app();
        app.selected_tab = 0;
        app.prev_tab(); // 0 → TAB_COUNT-1 = 2
        assert_eq!(app.selected_tab, 2);
        app.prev_tab(); // 2 → 1
        assert_eq!(app.selected_tab, 1);
        app.prev_tab(); // 1 → 0
        assert_eq!(app.selected_tab, 0);
    }

    #[test]
    fn scroll_up_does_not_underflow() {
        let mut app = make_test_app();
        app.scroll_offset = 0;
        app.scroll_up();
        assert_eq!(app.scroll_offset, 0); // saturating_sub — no underflow
    }

    #[test]
    fn scroll_down_increments() {
        let mut app = make_test_app();
        app.scroll_down();
        assert_eq!(app.scroll_offset, 1);
        app.scroll_down();
        assert_eq!(app.scroll_offset, 2);
    }

    #[test]
    fn on_tick_triggers_refresh_after_interval() {
        let mut app = make_test_app();
        // Force last_refresh into the past so the interval has elapsed.
        app.last_refresh = Instant::now() - Duration::from_secs(10);
        app.on_tick();
        // After on_tick the timer should have been reset (elapsed < interval).
        assert!(app.last_refresh.elapsed().as_secs() < app.refresh_interval_secs);
    }

    #[test]
    fn on_tick_does_not_refresh_before_interval() {
        let mut app = make_test_app();
        let before = app.last_refresh;
        app.on_tick(); // interval has not elapsed yet
        // last_refresh should be unchanged (same Instant)
        assert_eq!(before, app.last_refresh);
    }

    #[test]
    fn secs_until_refresh_decreases_over_time() {
        let app = make_test_app();
        let secs = app.secs_until_refresh();
        assert!(secs <= app.refresh_interval_secs);
    }
}
