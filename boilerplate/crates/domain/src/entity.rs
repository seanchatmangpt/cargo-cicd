//! Entity and aggregate root primitives.
//!
//! Every domain object that has a continuous identity through time is an [`Entity`].
//! Entities that own a consistency boundary and emit [`DomainEvent`]s are
//! [`AggregateRoot`]s.
//!
//! [`DomainEvent`]: crate::event::DomainEvent

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::time::SystemTime;

use uuid::Uuid;

// ── EntityId ─────────────────────────────────────────────────────────────────

/// A strongly-typed entity identifier wrapping a [`Uuid`].
///
/// The phantom type parameter `T` prevents accidentally passing an id of one
/// entity type where another is expected.
///
/// # Example
///
/// ```rust
/// use project_domain::entity::EntityId;
///
/// struct User;
/// struct Order;
///
/// let user_id: EntityId<User>   = EntityId::new();
/// let order_id: EntityId<Order> = EntityId::new();
/// // EntityId<User> and EntityId<Order> are different types — the compiler
/// // rejects a mix-up at the call site.
/// ```
#[derive(Debug)]
pub struct EntityId<T> {
    value: Uuid,
    _marker: PhantomData<T>,
}

impl<T> EntityId<T> {
    /// Create a new random identifier.
    pub fn new() -> Self {
        Self {
            value: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    /// Wrap an existing [`Uuid`] value.
    pub fn from_uuid(value: Uuid) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Parse a UUID from its hyphenated string representation.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string is not a valid UUID.
    pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            value: Uuid::parse_str(s)?,
            _marker: PhantomData,
        })
    }

    /// The raw underlying [`Uuid`].
    pub fn as_uuid(&self) -> &Uuid {
        &self.value
    }

    /// Consume the id and return the underlying [`Uuid`].
    pub fn into_uuid(self) -> Uuid {
        self.value
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for EntityId<T> {}

impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for EntityId<T> {}

impl<T> Hash for EntityId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> fmt::Display for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for EntityId<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for EntityId<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Uuid::deserialize(d)?;
        Ok(Self {
            value,
            _marker: PhantomData,
        })
    }
}

// ── Entity trait ─────────────────────────────────────────────────────────────

/// Core trait for domain entities.
///
/// An entity has:
/// - A unique identifier whose type is fixed by the associated `Id` type
/// - Lifecycle timestamps (`created_at`, `updated_at`)
///
/// Entities are compared **by identity**, not by field value.
pub trait Entity: Send + Sync + 'static {
    /// The specific identifier type for this entity.
    ///
    /// Usually `EntityId<Self>`.
    type Id: Clone + PartialEq + Eq + Hash + fmt::Display + Send + Sync + 'static;

    /// Return a reference to this entity's identifier.
    fn id(&self) -> &Self::Id;

    /// Timestamp when the entity was first created.
    fn created_at(&self) -> SystemTime;

    /// Timestamp when the entity was last modified.
    fn updated_at(&self) -> SystemTime;
}

// ── AggregateRoot trait ───────────────────────────────────────────────────────

/// Extension of [`Entity`] for aggregate roots.
///
/// An aggregate root is the consistency boundary of a cluster of entities. All
/// mutations must go through the root, which records what happened as a sequence
/// of [`DomainEvent`]s that can be published to the outside world.
///
/// [`DomainEvent`]: crate::event::DomainEvent
pub trait AggregateRoot: Entity {
    /// The domain event type this aggregate emits.
    type Event: crate::event::DomainEvent;

    /// Immutable view of all unpublished domain events.
    fn domain_events(&self) -> &[Self::Event];

    /// Drain and return all pending domain events.
    ///
    /// Callers (application services) invoke this after saving the aggregate,
    /// then publish the returned events.
    fn take_domain_events(&mut self) -> Vec<Self::Event>;

    /// Record a new domain event without publishing it yet.
    fn record_event(&mut self, event: Self::Event);

    /// Return `true` if there are pending unpublished events.
    fn has_pending_events(&self) -> bool {
        !self.domain_events().is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct Marker;

    #[test]
    fn entity_id_new_generates_unique_ids() {
        let ids: HashSet<EntityId<Marker>> = (0..100).map(|_| EntityId::new()).collect();
        assert_eq!(ids.len(), 100, "all ids must be unique");
    }

    #[test]
    fn entity_id_clone_is_equal() {
        let id: EntityId<Marker> = EntityId::new();
        let cloned = id;
        assert_eq!(id, cloned);
    }

    #[test]
    fn entity_id_parse_str_round_trips() {
        let id: EntityId<Marker> = EntityId::new();
        let s = id.to_string();
        let parsed = EntityId::<Marker>::parse_str(&s).expect("valid uuid string");
        assert_eq!(id, parsed);
    }

    #[test]
    fn entity_id_parse_str_rejects_garbage() {
        assert!(EntityId::<Marker>::parse_str("not-a-uuid").is_err());
    }

    #[test]
    fn entity_id_display_is_hyphenated_uuid() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id: EntityId<Marker> = EntityId::from_uuid(uuid);
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn entity_id_hash_consistent_with_eq() {
        use std::collections::HashMap;
        let id: EntityId<Marker> = EntityId::new();
        let mut map = HashMap::new();
        map.insert(id, "value");
        assert_eq!(map[&id], "value");
    }
}
