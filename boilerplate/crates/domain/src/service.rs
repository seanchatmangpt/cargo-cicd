//! Application service primitives.
//!
//! Application services orchestrate domain objects to fulfil use cases. They
//! receive commands or queries, delegate to entities and repositories, and
//! publish domain events.
//!
//! # Pattern
//!
//! ```text
//! CLI / HTTP adapter
//!     │  Command / Query
//!     ▼
//! CommandHandler / QueryHandler  ← application service
//!     │  aggregates + repositories
//!     ▼
//! Domain entities + port traits
//! ```
//!
//! # Types
//!
//! | Type | Role |
//! |------|------|
//! | [`CommandHandler<C, R>`] | Executes a state-changing command |
//! | [`QueryHandler<Q, R>`] | Executes a read-only query |
//! | [`ServiceContext`] | Infrastructure ports available to all services |
//! | [`ServiceRegistry`] | Dynamic registry of handler instances |

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::port::{Clock, IdGenerator};

// ── CommandHandler ────────────────────────────────────────────────────────────

/// Handler for a state-changing command.
///
/// Commands encode *intent* — they may be rejected if domain invariants would
/// be violated. On success they typically return an id or a summary result.
///
/// # Example
///
/// ```rust
/// use project_domain::service::CommandHandler;
/// use project_domain::error::Result;
/// use async_trait::async_trait;
///
/// struct PlaceOrder { product_id: String, quantity: u32 }
/// struct OrderId(String);
///
/// struct PlaceOrderHandler;
///
/// #[async_trait]
/// impl CommandHandler<PlaceOrder, OrderId> for PlaceOrderHandler {
///     async fn handle(&self, command: PlaceOrder) -> Result<OrderId> {
///         // validate, create aggregate, save, publish events
///         Ok(OrderId("ord-001".to_string()))
///     }
/// }
/// ```
#[async_trait]
pub trait CommandHandler<C, R>: Send + Sync + 'static
where
    C: Send + 'static,
    R: Send + 'static,
{
    /// Execute the command and return the result.
    ///
    /// # Errors
    ///
    /// Returns a [`DomainError`](crate::error::DomainError) if the command
    /// cannot be fulfilled.
    async fn handle(&self, command: C) -> Result<R>;
}

// ── QueryHandler ──────────────────────────────────────────────────────────────

/// Handler for a read-only query.
///
/// Queries must not change domain state. They return view models or DTOs
/// rather than domain aggregates, keeping the read model independent of the
/// write model (CQRS pattern).
///
/// # Example
///
/// ```rust
/// use project_domain::service::QueryHandler;
/// use project_domain::error::Result;
/// use async_trait::async_trait;
///
/// struct GetOrderById { id: String }
/// struct OrderView { id: String, status: String }
///
/// struct GetOrderByIdHandler;
///
/// #[async_trait]
/// impl QueryHandler<GetOrderById, Option<OrderView>> for GetOrderByIdHandler {
///     async fn handle(&self, query: GetOrderById) -> Result<Option<OrderView>> {
///         // read from repository or read model
///         Ok(None)
///     }
/// }
/// ```
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync + 'static
where
    Q: Send + 'static,
    R: Send + 'static,
{
    /// Execute the query and return the result.
    ///
    /// # Errors
    ///
    /// Returns a [`DomainError`](crate::error::DomainError) only for
    /// infrastructure failures (e.g. database unreachable).
    async fn handle(&self, query: Q) -> Result<R>;
}

// ── ServiceContext ────────────────────────────────────────────────────────────

/// Infrastructure dependencies shared by all application services.
///
/// `ServiceContext` is injected into service constructors so they do not
/// need to take individual port parameters. New ports can be added here
/// without changing every service signature.
///
/// Ports are stored as `Arc<dyn Trait>` so they are cheap to clone and can be
/// shared across services.
#[derive(Clone)]
pub struct ServiceContext {
    /// Mockable wall-clock.
    pub clock: Arc<dyn Clock>,
    /// Unique id generator.
    pub id_generator: Arc<dyn IdGenerator>,
}

impl ServiceContext {
    /// Construct a context from the given port implementations.
    pub fn new(clock: Arc<dyn Clock>, id_generator: Arc<dyn IdGenerator>) -> Self {
        Self { clock, id_generator }
    }

    /// Construct a context using the production defaults:
    /// [`SystemClock`](crate::port::SystemClock) and
    /// [`UuidIdGenerator`](crate::port::UuidIdGenerator).
    pub fn production() -> Self {
        use crate::port::{SystemClock, UuidIdGenerator};
        Self {
            clock: Arc::new(SystemClock),
            id_generator: Arc::new(UuidIdGenerator),
        }
    }

    /// Return the current wall-clock time via the injected [`Clock`].
    pub fn now(&self) -> std::time::SystemTime {
        self.clock.now()
    }

    /// Generate a new [`EntityId<T>`](crate::entity::EntityId) via the
    /// injected [`IdGenerator`].
    pub fn new_id<T>(&self) -> crate::entity::EntityId<T> {
        self.id_generator.generate()
    }
}

impl std::fmt::Debug for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceContext")
            .field("clock", &"<dyn Clock>")
            .field("id_generator", &"<dyn IdGenerator>")
            .finish()
    }
}

// ── ServiceRegistry ───────────────────────────────────────────────────────────

/// A type-erased registry that maps handler types to their `Arc<H>` instances.
///
/// Handlers are registered once at application startup and retrieved by their
/// concrete type at call sites.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use project_domain::service::ServiceRegistry;
///
/// # use project_domain::service::CommandHandler;
/// # use project_domain::error::Result;
/// # use async_trait::async_trait;
/// #
/// # struct Ping;
/// # struct Pong;
/// # struct PingHandler;
/// # #[async_trait]
/// # impl CommandHandler<Ping, Pong> for PingHandler {
/// #     async fn handle(&self, _cmd: Ping) -> Result<Pong> { Ok(Pong) }
/// # }
/// #
/// let mut registry = ServiceRegistry::new();
/// registry.register(PingHandler);
///
/// let handler: Arc<PingHandler> = registry.get::<PingHandler>().unwrap();
/// ```
#[derive(Clone)]
pub struct ServiceRegistry {
    handlers: Arc<std::sync::Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Register a handler.
    ///
    /// If a handler of the same concrete type was already registered, it is
    /// replaced.
    pub fn register<H: Any + Send + Sync + 'static>(&mut self, handler: H) {
        let mut map = self.handlers.lock().expect("registry lock poisoned");
        map.insert(TypeId::of::<H>(), Arc::new(handler));
    }

    /// Look up a handler by its concrete type.
    ///
    /// Returns `None` if no handler of type `H` has been registered.
    pub fn get<H: Any + Send + Sync + 'static>(&self) -> Option<Arc<H>> {
        let map = self.handlers.lock().expect("registry lock poisoned");
        map.get(&TypeId::of::<H>())
            .cloned()
            .and_then(|arc| arc.downcast::<H>().ok())
    }

    /// Return the number of registered handlers.
    pub fn len(&self) -> usize {
        let map = self.handlers.lock().expect("registry lock poisoned");
        map.len()
    }

    /// Return `true` if no handlers have been registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ServiceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.len();
        f.debug_struct("ServiceRegistry")
            .field("handler_count", &count)
            .finish()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::{FakeClock, UuidIdGenerator};
    use std::time::SystemTime;

    // Minimal command/query types for tests
    struct NoopCommand;
    struct NoopQuery;
    struct NoopResult;

    struct NoopCommandHandler;

    #[async_trait]
    impl CommandHandler<NoopCommand, NoopResult> for NoopCommandHandler {
        async fn handle(&self, _cmd: NoopCommand) -> Result<NoopResult> {
            Ok(NoopResult)
        }
    }

    struct NoopQueryHandler;

    #[async_trait]
    impl QueryHandler<NoopQuery, NoopResult> for NoopQueryHandler {
        async fn handle(&self, _query: NoopQuery) -> Result<NoopResult> {
            Ok(NoopResult)
        }
    }

    #[tokio::test]
    async fn command_handler_returns_ok() {
        let h = NoopCommandHandler;
        assert!(h.handle(NoopCommand).await.is_ok());
    }

    #[tokio::test]
    async fn query_handler_returns_ok() {
        let h = NoopQueryHandler;
        assert!(h.handle(NoopQuery).await.is_ok());
    }

    #[test]
    fn service_context_production_creates_ok() {
        let ctx = ServiceContext::production();
        // Just verify it constructs without panic.
        let _ = ctx.now();
    }

    #[test]
    fn service_context_new_id_produces_unique_ids() {
        let ctx = ServiceContext::production();
        struct Marker;
        let a = ctx.new_id::<Marker>();
        let b = ctx.new_id::<Marker>();
        assert_ne!(a, b);
    }

    #[test]
    fn service_context_fake_clock() {
        let fixed = SystemTime::UNIX_EPOCH;
        let ctx = ServiceContext::new(
            Arc::new(FakeClock::new(fixed)),
            Arc::new(UuidIdGenerator),
        );
        assert_eq!(ctx.now(), fixed);
    }

    #[test]
    fn service_registry_register_and_get() {
        let mut reg = ServiceRegistry::new();
        reg.register(NoopCommandHandler);
        let handler: Arc<NoopCommandHandler> = reg.get::<NoopCommandHandler>().unwrap();
        // Verify we can call the handler through the Arc
        let _ = Arc::clone(&handler);
    }

    #[test]
    fn service_registry_get_unregistered_returns_none() {
        let reg = ServiceRegistry::new();
        assert!(reg.get::<NoopCommandHandler>().is_none());
    }

    #[test]
    fn service_registry_len_and_is_empty() {
        let mut reg = ServiceRegistry::new();
        assert!(reg.is_empty());
        reg.register(NoopCommandHandler);
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());
    }

    #[test]
    fn service_registry_register_replaces_existing() {
        let mut reg = ServiceRegistry::new();
        reg.register(NoopCommandHandler);
        reg.register(NoopCommandHandler); // second registration
        assert_eq!(reg.len(), 1, "duplicate registration should not grow the map");
    }

    #[test]
    fn service_registry_debug_format() {
        let mut reg = ServiceRegistry::new();
        reg.register(NoopCommandHandler);
        let s = format!("{:?}", reg);
        assert!(s.contains("ServiceRegistry"));
    }
}
