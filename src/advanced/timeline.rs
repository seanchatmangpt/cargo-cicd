//! High-precision process-event timeline backed by [`jiff`].
//!
//! [`ProcessTimeline`] records an ordered list of labeled, timestamped events
//! using nanosecond-precision [`jiff::Timestamp`] values. It is the timestamp
//! source for the engine's `ProcessEventState`: every recorded event maps to a
//! process event whose `at` field is an exact, serializable instant. Spans
//! between events are computed with signed [`jiff::Span`] arithmetic so callers
//! can measure stage latency or reconstruct the full first-to-last duration.

// This module's public API is exercised by `examples/03_max_pipeline.rs`
// (tutorial anchor for docs/tutorials/03-full-pipeline.md) and by
// `engine::process_event_state::ProcessEventState`'s own test, both compiled
// as separate cargo targets/paths whose usage doesn't suppress `cargo
// build`'s dead_code lint on the library crate.
#![allow(dead_code)]

use jiff::{Span, Timestamp};
use serde::{Deserialize, Serialize};

/// A single labeled event stamped at an exact instant.
///
/// Serializes via the `jiff` serde integration; the `at` field round-trips as
/// an ISO-8601 / RFC3339 string, making it a stable carrier for the engine's
/// `ProcessEventState` records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedEvent {
    /// Human-readable name of the event (e.g. a pipeline stage).
    pub label: String,
    /// Exact instant at which the event was recorded.
    pub at: Timestamp,
}

impl TimedEvent {
    /// Construct an event from a label and a fixed timestamp.
    pub fn new(label: &str, at: Timestamp) -> Self {
        Self {
            label: label.to_string(),
            at,
        }
    }
}

/// An ordered, append-only sequence of [`TimedEvent`] records.
///
/// Events are kept in insertion order. Use [`record`](Self::record) for
/// live real-time stamping and [`record_at`](Self::record_at) for
/// deterministic, fixture-driven timestamps in tests.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessTimeline {
    events: Vec<TimedEvent>,
}

impl ProcessTimeline {
    /// Create an empty timeline.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record an event stamped at the current instant ([`Timestamp::now`]).
    pub fn record(&mut self, label: &str) {
        self.events.push(TimedEvent::new(label, Timestamp::now()));
    }

    /// Record an event at an explicit instant. Useful for deterministic tests.
    pub fn record_at(&mut self, label: &str, ts: Timestamp) {
        self.events.push(TimedEvent::new(label, ts));
    }

    /// All recorded events in insertion order.
    pub fn events(&self) -> &[TimedEvent] {
        &self.events
    }

    /// Number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether no events have been recorded yet.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// First event matching `label`, if any.
    fn find(&self, label: &str) -> Option<&TimedEvent> {
        self.events.iter().find(|e| e.label == label)
    }

    /// Signed span from the first `from_label` event to the first `to_label`
    /// event. Returns `None` if either label is absent.
    ///
    /// The span is positive when `to_label` occurs after `from_label` and
    /// negative when it occurs before.
    pub fn span_between(&self, from_label: &str, to_label: &str) -> Option<Span> {
        let from = self.find(from_label)?;
        let to = self.find(to_label)?;
        to.at.since(from.at).ok()
    }

    /// Span from the earliest-recorded event to the latest-recorded event.
    /// Returns `None` when the timeline holds fewer than two events.
    pub fn total_span(&self) -> Option<Span> {
        if self.events.len() < 2 {
            return None;
        }
        let first = self.events.first()?;
        let last = self.events.last()?;
        last.at.since(first.at).ok()
    }

    /// Each event's label paired with its ISO-8601 / RFC3339 timestamp string.
    pub fn to_iso8601(&self) -> Vec<(String, String)> {
        self.events
            .iter()
            .map(|e| (e.label.clone(), e.at.to_string()))
            .collect()
    }
}

/// Parse an ISO-8601 / RFC3339 instant into a [`jiff::Timestamp`].
pub fn parse_timestamp(s: &str) -> Result<Timestamp, jiff::Error> {
    s.parse::<Timestamp>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: &str = "2026-06-14T10:00:00Z";
    const T1: &str = "2026-06-14T10:00:05Z";
    const T2: &str = "2026-06-14T10:01:00Z";

    fn fixed(s: &str) -> Timestamp {
        parse_timestamp(s).expect("fixture timestamp parses")
    }

    #[test]
    fn records_preserve_insertion_order() {
        let mut tl = ProcessTimeline::new();
        tl.record_at("start", fixed(T0));
        tl.record_at("middle", fixed(T1));
        tl.record_at("end", fixed(T2));

        let labels: Vec<&str> = tl.events().iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels, ["start", "middle", "end"]);
        assert_eq!(tl.len(), 3);
        assert!(!tl.is_empty());
    }

    #[test]
    fn span_between_matches_fixed_duration() {
        let mut tl = ProcessTimeline::new();
        tl.record_at("start", fixed(T0));
        tl.record_at("end", fixed(T1));

        let span = tl
            .span_between("start", "end")
            .expect("both labels present");
        // T0 -> T1 is exactly 5 seconds.
        assert_eq!(span.get_seconds(), 5);

        // Reversed direction yields a negative span.
        let back = tl
            .span_between("end", "start")
            .expect("both labels present");
        assert_eq!(back.get_seconds(), -5);

        // Missing labels yield None.
        assert!(tl.span_between("start", "absent").is_none());
    }

    #[test]
    fn total_span_covers_first_to_last() {
        let mut tl = ProcessTimeline::new();
        assert!(tl.total_span().is_none(), "empty timeline has no span");

        tl.record_at("a", fixed(T0));
        assert!(tl.total_span().is_none(), "single event has no span");

        tl.record_at("b", fixed(T1));
        tl.record_at("c", fixed(T2));
        // T0 -> T2 is exactly 60 seconds.
        let total = tl.total_span().expect("two or more events");
        assert_eq!(total.get_seconds(), 60);
    }

    #[test]
    fn parse_format_round_trip() {
        let ts = fixed(T0);
        let rendered = ts.to_string();
        let reparsed = parse_timestamp(&rendered).expect("round-trip parses");
        assert_eq!(ts, reparsed);

        let mut tl = ProcessTimeline::new();
        tl.record_at("only", ts);
        let iso = tl.to_iso8601();
        assert_eq!(iso.len(), 1);
        assert_eq!(iso[0].0, "only");
        assert_eq!(parse_timestamp(&iso[0].1).expect("iso parses"), ts);

        // Invalid input surfaces an error rather than panicking.
        assert!(parse_timestamp("not-a-timestamp").is_err());
    }

    #[test]
    fn now_recording_does_not_panic_and_keeps_order() {
        let mut tl = ProcessTimeline::new();
        tl.record("first");
        tl.record("second");

        let labels: Vec<&str> = tl.events().iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels, ["first", "second"]);
        // Real-time values are not asserted; only that two events landed.
        assert_eq!(tl.len(), 2);
    }
}
