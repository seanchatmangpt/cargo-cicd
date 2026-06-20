//! Port trait interfaces — the domain's view of the outside world.
//!
//! In hexagonal architecture, ports are *abstract interfaces* that the domain
//! defines. Adapters (in the infrastructure layer) provide concrete
//! implementations. The domain only depends on these traits, never on concrete
//! types.
//!
//! # Available ports
//!
//! | Trait | Concern |
//! |-------|---------|
//! | [`EventPublisher`] | Publish domain events to a message bus |
//! | [`Cache`] | Temporary key-value storage |
//! | [`IdGenerator`] | Generate new unique entity identifiers |
//! | [`Clock`] | Obtain the current wall-clock time (mockable) |
//! | [`HealthCheck`] | Report service health to infrastructure monitoring |

use std::collections::HashMap;
use std::time::SystemTime;

use async_trait::async_trait;
use uuid::Uuid;

use crate::entity::EntityId;
use crate::error::Result;
use crate::event::DomainEvent;

// ── EventPublisher ────────────────────────────────────────────────────────────

/// Port for publishing domain events to an external message bus or event store.
///
/// Adapters may implement this as a Kafka producer, RabbitMQ publisher,
/// in-memory channel, or any other transport. The domain only cares that
/// events are delivered reliably.
///
/// Implementors must be `Send + Sync` so they can be shared across async tasks.
#[async_trait]
pub trait EventPublisher: Send + Sync + 'static {
    /// Publish a single domain event.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport layer fails. The domain does not
    /// interpret transport errors — callers should decide whether to retry.
    async fn publish<E: DomainEvent>(&self, event: E) -> Result<()>;

    /// Publish a batch of events in order.
    ///
    /// The default implementation publishes each event sequentially.
    /// Adapters may override this for batch optimisations.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered; subsequent events in the batch
    /// are not published after a failure.
    async fn publish_batch<E: DomainEvent>(&self, events: Vec<E>) -> Result<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

// ── Cache ─────────────────────────────────────────────────────────────────────

/// Port for a generic key-value cache.
///
/// Suitable for short-lived data that can be recomputed if missing (Redis,
/// Memcached, or an in-process `DashMap`).
///
/// The key type `K` must be `Send + Sync + Eq + std::hash::Hash`, and the
/// value type `V` must be `Clone + Send + Sync`.
#[async_trait]
pub trait Cache<K, V>: Send + Sync + 'static
where
    K: Send + Sync + Eq + std::hash::Hash + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Retrieve a cached value by key.
    ///
    /// Returns `Ok(None)` for a cache miss; `Err(...)` only on transport
    /// failure.
    async fn get(&self, key: &K) -> Result<Option<V>>;

    /// Insert or overwrite a value.
    async fn set(&self, key: K, value: V) -> Result<()>;

    /// Remove a key from the cache.
    ///
    /// Returns `Ok(())` even if the key was absent (idempotent).
    async fn delete(&self, key: &K) -> Result<()>;

    /// Return `true` if the key exists in the cache.
    ///
    /// The default delegates to [`Cache::get`]; adapters may override with a
    /// cheaper EXISTS call.
    async fn contains(&self, key: &K) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }
}

// ── IdGenerator ───────────────────────────────────────────────────────────────

/// Port for generating new unique entity identifiers.
///
/// Sync because UUID generation is CPU-only and never blocks. Adapters can
/// produce UUIDs, ULIDs, or any other unique id scheme.
pub trait IdGenerator: Send + Sync + 'static {
    /// Generate a new unique identifier for entity type `T`.
    fn generate<T>(&self) -> EntityId<T>;
}

/// A production `IdGenerator` that wraps `Uuid::new_v4`.
#[derive(Debug, Clone, Default)]
pub struct UuidIdGenerator;

impl IdGenerator for UuidIdGenerator {
    fn generate<T>(&self) -> EntityId<T> {
        EntityId::new()
    }
}

// ── Clock ─────────────────────────────────────────────────────────────────────

/// Port for obtaining the current wall-clock time.
///
/// Keeping this behind a trait makes domain services fully testable: tests
/// inject a [`FakeClock`] that returns a fixed timestamp.
pub trait Clock: Send + Sync + 'static {
    /// Return the current wall-clock time.
    fn now(&self) -> SystemTime;

    /// Return the current time as a Unix timestamp (seconds since epoch).
    fn unix_timestamp(&self) -> u64 {
        self.now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// A production [`Clock`] that returns `SystemTime::now()`.
#[derive(Debug, Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// A test [`Clock`] that always returns a fixed, user-specified time.
///
/// Useful in unit tests that need deterministic timestamps.
#[derive(Debug, Clone)]
pub struct FakeClock {
    fixed_time: SystemTime,
}

impl FakeClock {
    /// Construct a `FakeClock` that returns `fixed_time`.
    pub fn new(fixed_time: SystemTime) -> Self {
        Self { fixed_time }
    }

    /// Construct a `FakeClock` fixed at the Unix epoch.
    pub fn epoch() -> Self {
        Self {
            fixed_time: SystemTime::UNIX_EPOCH,
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        self.fixed_time
    }
}

// ── HealthCheck ───────────────────────────────────────────────────────────────

/// Status returned by a [`HealthCheck`] probe.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthStatus {
    /// Overall health flag.
    pub is_healthy: bool,
    /// Arbitrary key-value details reported by the component.
    pub details: HashMap<String, String>,
}

impl HealthStatus {
    /// Construct a healthy status with no detail entries.
    pub fn healthy() -> Self {
        Self {
            is_healthy: true,
            details: HashMap::new(),
        }
    }

    /// Construct an unhealthy status with a reason string.
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        details.insert("reason".to_string(), reason.into());
        Self {
            is_healthy: false,
            details,
        }
    }

    /// Add a detail entry and return `self` for chaining.
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Port for reporting readiness and liveness to infrastructure monitoring.
///
/// Adapters typically expose this via an HTTP `/health` endpoint or a
/// Kubernetes probe handler.
#[async_trait]
pub trait HealthCheck: Send + Sync + 'static {
    /// Perform the health check and return the current status.
    ///
    /// Implementors should not panic — return an `Err` if the check itself
    /// could not be completed.
    async fn check(&self) -> Result<HealthStatus>;
}

/// A trivial always-healthy [`HealthCheck`] implementation for tests.
#[derive(Debug, Clone, Default)]
pub struct AlwaysHealthy;

#[async_trait]
impl HealthCheck for AlwaysHealthy {
    async fn check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::healthy())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn uuid_id_generator_produces_unique_ids() {
        let gen = UuidIdGenerator;
        struct Marker;
        let a: EntityId<Marker> = gen.generate();
        let b: EntityId<Marker> = gen.generate();
        assert_ne!(a, b);
    }

    #[test]
    fn system_clock_advances() {
        let clock = SystemClock;
        let t1 = clock.now();
        // SystemTime::now() should be >= the previous call.
        let t2 = clock.now();
        assert!(t2 >= t1);
    }

    #[test]
    fn fake_clock_returns_fixed_time() {
        let fixed = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let clock = FakeClock::new(fixed);
        assert_eq!(clock.now(), fixed);
        assert_eq!(clock.now(), fixed); // stable across calls
    }

    #[test]
    fn fake_clock_epoch() {
        let clock = FakeClock::epoch();
        assert_eq!(clock.now(), SystemTime::UNIX_EPOCH);
        assert_eq!(clock.unix_timestamp(), 0);
    }

    #[test]
    fn health_status_healthy_has_no_details() {
        let status = HealthStatus::healthy();
        assert!(status.is_healthy);
        assert!(status.details.is_empty());
    }

    #[test]
    fn health_status_unhealthy_has_reason() {
        let status = HealthStatus::unhealthy("db connection failed");
        assert!(!status.is_healthy);
        assert_eq!(status.details["reason"], "db connection failed");
    }

    #[test]
    fn health_status_with_detail_builder() {
        let status = HealthStatus::healthy()
            .with_detail("version", "1.0.0")
            .with_detail("region", "us-east-1");
        assert_eq!(status.details["version"], "1.0.0");
        assert_eq!(status.details["region"], "us-east-1");
    }

    #[tokio::test]
    async fn always_healthy_check() {
        let h = AlwaysHealthy;
        let status = h.check().await.unwrap();
        assert!(status.is_healthy);
    }

    // Verify the generate() method accepts generic T
    #[test]
    fn id_generator_generic_over_entity_type() {
        struct User;
        struct Product;
        let gen = UuidIdGenerator;
        let _user_id: EntityId<User> = gen.generate();
        let _prod_id: EntityId<Product> = gen.generate();
    }

    #[test]
    fn uuid_id_generator_generate_returns_unique_uuid() {
        let gen = UuidIdGenerator;
        struct Entity1;
        let ids: Vec<Uuid> = (0..10)
            .map(|_| gen.generate::<Entity1>().into_uuid())
            .collect();
        let unique: std::collections::HashSet<Uuid> = ids.iter().cloned().collect();
        assert_eq!(unique.len(), 10);
    }
}
