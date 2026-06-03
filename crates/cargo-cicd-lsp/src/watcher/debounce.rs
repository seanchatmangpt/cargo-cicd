//! Simple 500ms debounce for file events.

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Debounce duration for file watcher events.
pub const DEBOUNCE_MS: u64 = 500;

/// Wrap a receiver so that rapid events are coalesced into a single signal.
/// The returned receiver emits `()` at most once per `DEBOUNCE_MS` window.
pub fn debounce(mut input: mpsc::Receiver<()>) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        while input.recv().await.is_some() {
            // Drain any pending events within the debounce window
            let deadline = sleep(Duration::from_millis(DEBOUNCE_MS));
            tokio::pin!(deadline);
            loop {
                tokio::select! {
                    _ = &mut deadline => break,
                    v = input.recv() => {
                        if v.is_none() { return; }
                        // reset not needed — we just wait out the window
                    }
                }
            }
            let _ = tx.send(()).await;
        }
    });
    rx
}
