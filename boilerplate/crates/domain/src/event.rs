//! Domain event primitives.
//!
//! Domain events are facts about things that happened in the domain. They are
//! immutable records of state transitions that other parts of the system may
//! react to.
//!
//! # Pattern
//!
//! 1. An [`AggregateRoot`] records events via [`AggregateRoot::record_event`].
//! 2. The application service calls [`AggregateRoot::take_domain_events`] after
//!    persisting the aggregate.
//! 3. Events are wrapped in [`EventEnvelope`] and handed to an
//!    [`EventPublisher`] port.
//!
//! [`AggregateRoot`]: crate::entity::AggregateRoot
//! [`EventPublisher`]: crate::port::EventPublisher

use std::time::SystemTime;

use uuid::Uuid;

// ── DomainEvent trait ─────────────────────────────────────────────────────────

/// Trait for all domain events.
///
/// Every concrete event must carry:
/// - A unique [`event_id`](DomainEvent::event_id) for idempotent processing
/// - The [`aggregate_id`](DomainEvent::aggregate_id) of the entity it relates to
/// - The [`occurred_at`](DomainEvent::occurred_at) timestamp
/// - A stable [`event_type`](DomainEvent::event_type) string for routing/storage
pub trait DomainEvent: Clone + Send + Sync + 'static {
    /// A unique identifier for this specific event occurrence.
    fn event_id(&self) -> Uuid;

    /// The string representation of the aggregate's id this event belongs to.
    fn aggregate_id(&self) -> &str;

    /// Wall-clock time when the event occurred.
    fn occurred_at(&self) -> SystemTime;

    /// A stable, human-readable event type name (e.g. `"order.placed"`).
    ///
    /// This value is used for routing and storage — it must not change once
    /// events have been persisted.
    fn event_type(&self) -> &'static str;
}

// ── EventMetadata ─────────────────────────────────────────────────────────────

/// Cross-cutting metadata attached to every published event.
///
/// Metadata carries observability and causality information that is not part
/// of the domain model itself.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventMetadata {
    /// Groups a chain of events into a single logical operation.
    ///
    /// All events caused by a single user action share the same
    /// `correlation_id`, even if they span multiple aggregates.
    pub correlation_id: Option<Uuid>,

    /// The event id that directly caused this event to be emitted.
    ///
    /// Enables exact causal ordering (a tree, not just a group).
    pub causation_id: Option<Uuid>,

    /// The identity of the user or system actor who triggered the originating
    /// command.
    pub user_id: Option<String>,
}

impl EventMetadata {
    /// Construct metadata with all fields set to `None`.
    pub fn empty() -> Self {
        Self {
            correlation_id: None,
            causation_id: None,
            user_id: None,
        }
    }

    /// Construct metadata with a fresh correlation id and optional user id.
    pub fn new(user_id: impl Into<Option<String>>) -> Self {
        Self {
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            user_id: user_id.into(),
        }
    }

    /// Return a copy of this metadata where `causation_id` is set to the given
    /// event's id — useful when building a causal chain.
    pub fn caused_by(mut self, causing_event_id: Uuid) -> Self {
        self.causation_id = Some(causing_event_id);
        self
    }

    /// Return `true` if a correlation id is present.
    pub fn is_correlated(&self) -> bool {
        self.correlation_id.is_some()
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::empty()
    }
}

// ── EventEnvelope ─────────────────────────────────────────────────────────────

/// A domain event bundled with its cross-cutting [`EventMetadata`].
///
/// Publishers receive `EventEnvelope<E>` values; consumers unpack them to
/// inspect both the event payload and the metadata.
///
/// # Example
///
/// ```rust
/// use project_domain::event::{EventEnvelope, EventMetadata};
///
/// // Suppose `OrderPlaced` implements `DomainEvent`.
/// // let envelope = EventEnvelope::new(order_placed_event, metadata);
/// // publisher.publish(envelope).await?;
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventEnvelope<E: DomainEvent> {
    /// The domain event payload.
    pub event: E,
    /// Observability and causality metadata.
    pub metadata: EventMetadata,
}

impl<E: DomainEvent> EventEnvelope<E> {
    /// Wrap an event with the given metadata.
    pub fn new(event: E, metadata: EventMetadata) -> Self {
        Self { event, metadata }
    }

    /// Wrap an event with empty metadata.
    pub fn bare(event: E) -> Self {
        Self {
            event,
            metadata: EventMetadata::empty(),
        }
    }

    /// Convenient passthrough to [`DomainEvent::event_id`].
    pub fn event_id(&self) -> Uuid {
        self.event.event_id()
    }

    /// Convenient passthrough to [`DomainEvent::event_type`].
    pub fn event_type(&self) -> &'static str {
        self.event.event_type()
    }

    /// Convenient passthrough to [`DomainEvent::aggregate_id`].
    pub fn aggregate_id(&self) -> &str {
        self.event.aggregate_id()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    /// Minimal concrete event for testing.
    #[derive(Clone, Debug)]
    struct TestEvent {
        id: Uuid,
        aggregate_id: String,
        occurred_at: SystemTime,
    }

    impl TestEvent {
        fn new(aggregate_id: impl Into<String>) -> Self {
            Self {
                id: Uuid::new_v4(),
                aggregate_id: aggregate_id.into(),
                occurred_at: SystemTime::now(),
            }
        }
    }

    impl DomainEvent for TestEvent {
        fn event_id(&self) -> Uuid {
            self.id
        }
        fn aggregate_id(&self) -> &str {
            &self.aggregate_id
        }
        fn occurred_at(&self) -> SystemTime {
            self.occurred_at
        }
        fn event_type(&self) -> &'static str {
            "test.event"
        }
    }

    #[test]
    fn event_metadata_new_has_correlation_id() {
        let meta = EventMetadata::new(Some("user-1".to_string()));
        assert!(meta.correlation_id.is_some());
        assert!(meta.is_correlated());
        assert_eq!(meta.user_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn event_metadata_empty_has_no_fields() {
        let meta = EventMetadata::empty();
        assert!(meta.correlation_id.is_none());
        assert!(meta.causation_id.is_none());
        assert!(meta.user_id.is_none());
        assert!(!meta.is_correlated());
    }

    #[test]
    fn event_metadata_caused_by_sets_causation_id() {
        let cause_id = Uuid::new_v4();
        let meta = EventMetadata::new(None).caused_by(cause_id);
        assert_eq!(meta.causation_id, Some(cause_id));
    }

    #[test]
    fn event_envelope_passthrough_methods() {
        let event = TestEvent::new("agg-123");
        let expected_id = event.id;
        let envelope = EventEnvelope::bare(event);

        assert_eq!(envelope.event_id(), expected_id);
        assert_eq!(envelope.event_type(), "test.event");
        assert_eq!(envelope.aggregate_id(), "agg-123");
    }

    #[test]
    fn event_envelope_new_stores_metadata() {
        let event = TestEvent::new("agg-456");
        let meta = EventMetadata::new(Some("admin".to_string()));
        let corr = meta.correlation_id;
        let envelope = EventEnvelope::new(event, meta);
        assert_eq!(envelope.metadata.correlation_id, corr);
    }
}
