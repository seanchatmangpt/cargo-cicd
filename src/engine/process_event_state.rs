use serde::{Deserialize, Serialize};

#[cfg(feature = "advanced")]
use crate::advanced::timeline::ProcessTimeline;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProcessEventState {
    pub events: Vec<ProcessEvent>,
    #[cfg(feature = "advanced")]
    pub timeline: Option<ProcessTimeline>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessEvent {
    pub kind: String,
    pub verdict: String,
    pub timestamp: String,
    pub details: Option<String>,
}

impl ProcessEventState {
    /// Record a labeled event to the timeline (if advanced feature is enabled and timeline is initialized).
    // Exercised by this module's own `#[cfg(test)]` suite below (see
    // `test_record_timeline_event`); `cargo build` doesn't compile test code,
    // so the lib crate sees no call site.
    #[cfg(feature = "advanced")]
    #[allow(dead_code)]
    pub fn record_timeline_event(&mut self, label: &str) {
        if let Some(timeline) = &mut self.timeline {
            timeline.record(label);
        }
    }

    /// Access the timeline, if available (requires advanced feature).
    #[cfg(feature = "advanced")]
    #[allow(dead_code)] // see record_timeline_event note above
    pub fn timeline(&self) -> Option<&ProcessTimeline> {
        self.timeline.as_ref()
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;

    #[test]
    fn test_record_timeline_event() {
        let mut state = ProcessEventState {
            events: Vec::new(),
            timeline: Some(ProcessTimeline::new()),
        };

        state.record_timeline_event("test_event");

        let timeline = state.timeline().expect("timeline should exist");
        assert_eq!(timeline.len(), 1, "timeline should have exactly 1 event");
    }
}
