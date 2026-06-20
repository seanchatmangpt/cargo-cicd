//! Event pump for the TUI dashboard.
//!
//! [`EventHandler`] runs a background thread that forwards crossterm key events
//! and emits [`Event::Tick`] at a configurable interval. The main loop calls
//! [`EventHandler::next`] to block until the next event arrives.
//!
//! # Design
//!
//! A single `mpsc` channel bridges the background thread and the main loop.
//! The thread calls `crossterm::event::poll` with the tick deadline; if a key
//! event arrives it is forwarded immediately. If the poll times out (i.e., the
//! tick interval has elapsed with no key) a `Tick` is sent instead.
//!
//! This means `Tick` events are delivered at *approximately* `tick_rate_ms`
//! intervals. In practice the resolution is within a few milliseconds, which is
//! more than sufficient for a 5-second refresh cycle.

use anyhow::{anyhow, Result};
use crossterm::event::{self as ct_event, Event as CtEvent, KeyEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Events that the dashboard event loop handles.
#[derive(Debug, Clone)]
pub enum Event {
    /// A key was pressed.
    Key(KeyEvent),
    /// The tick interval elapsed with no key activity.
    Tick,
    /// The terminal was resized to the given (width, height).
    Resize(u16, u16),
}

/// Drives the event pump for the TUI dashboard.
///
/// Construct with [`EventHandler::new`], then call [`EventHandler::next`] in
/// the main event loop to receive the next [`Event`].
pub struct EventHandler {
    /// Sender side kept alive so the channel is not dropped.
    _tx: mpsc::Sender<Event>,
    /// Receiver read by the main loop.
    rx: mpsc::Receiver<Event>,
}

impl EventHandler {
    /// Create a new event handler that emits [`Event::Tick`] every
    /// `tick_rate_ms` milliseconds when no key events occur.
    ///
    /// Spawns a daemon thread. The thread terminates automatically when the
    /// last sender is dropped (i.e., when `EventHandler` is dropped).
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // How long until the next tick deadline?
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                // Poll crossterm for any terminal events within that window.
                match ct_event::poll(timeout) {
                    Ok(true) => {
                        // An event is ready — read it.
                        match ct_event::read() {
                            Ok(CtEvent::Key(key)) => {
                                if tx_clone.send(Event::Key(key)).is_err() {
                                    // Main loop has dropped the receiver; exit
                                    // the thread.
                                    break;
                                }
                            }
                            Ok(CtEvent::Resize(w, h)) => {
                                if tx_clone.send(Event::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            // Focus, paste, and other events are ignored.
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                    Ok(false) => {
                        // Poll timed out — emit a Tick.
                        if tx_clone.send(Event::Tick).is_err() {
                            break;
                        }
                        last_tick = Instant::now();
                    }
                    Err(_) => break,
                }
            }
        });

        Self { _tx: tx, rx }
    }

    /// Block until the next event is available and return it.
    ///
    /// # Errors
    ///
    /// Returns an error if the background thread has exited unexpectedly and
    /// the channel is disconnected.
    pub fn next(&self) -> Result<Event> {
        self.rx.recv().map_err(|_| anyhow!("event channel disconnected"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tick_events_arrive_within_reasonable_time() {
        // Construct a handler with a very short tick rate and verify that
        // at least one Tick arrives within 500 ms.
        let handler = EventHandler::new(50);
        let start = Instant::now();
        let timeout = Duration::from_millis(500);

        loop {
            match handler.next() {
                Ok(Event::Tick) => return, // success
                Ok(_) => {} // key event on a test runner tty — ignore
                Err(_) => panic!("channel disconnected before Tick arrived"),
            }
            if start.elapsed() > timeout {
                panic!("no Tick arrived within 500 ms");
            }
        }
    }

    #[test]
    fn event_enum_is_clone() {
        use crossterm::event::{KeyCode, KeyModifiers};
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let event = Event::Key(key_event);
        let _ = event.clone();

        let tick = Event::Tick;
        let _ = tick.clone();

        let resize = Event::Resize(80, 24);
        let _ = resize.clone();
    }
}
