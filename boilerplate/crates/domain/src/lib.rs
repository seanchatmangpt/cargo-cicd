//! Domain layer for the hexagonal architecture template.
//!
//! This crate implements the core domain model following hexagonal (ports & adapters)
//! architecture. It has zero infrastructure dependencies — all external concerns
//! are expressed as port traits that adapters must implement.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   Domain Layer                      │
//! │  ┌──────────┐  ┌──────────┐  ┌───────────────────┐ │
//! │  │ Entities │  │  Value   │  │  Domain Events    │ │
//! │  │          │  │ Objects  │  │                   │ │
//! │  └──────────┘  └──────────┘  └───────────────────┘ │
//! │  ┌──────────────────────────────────────────────┐   │
//! │  │              Port Traits                     │   │
//! │  │  Repository | EventPublisher | Cache | ...   │   │
//! │  └──────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Modules
//!
//! - [`entity`] — Entity and AggregateRoot base traits + `EntityId<T>` newtype
//! - [`value_object`] — Value object trait + `EmailAddress`, `Money`, `Currency`
//! - [`event`] — `DomainEvent` trait, `EventEnvelope`, `EventMetadata`
//! - [`error`] — `DomainError` enum and `Result<T>` alias
//! - [`port`] — Async port trait interfaces (`EventPublisher`, `Cache`, `Clock`, …)
//! - [`repository`] — `Repository<T>`, `PaginatedRepository<T>`, `Page<T>`
//! - [`service`] — `CommandHandler`, `QueryHandler`, `ServiceContext`, `ServiceRegistry`

#![warn(missing_docs)]
#![warn(clippy::all)]
#![forbid(unsafe_code)]

pub mod entity;
pub mod error;
pub mod event;
pub mod port;
pub mod repository;
pub mod service;
pub mod value_object;

#[cfg(test)]
pub mod tests;

// ── Key re-exports ────────────────────────────────────────────────────────────

pub use entity::{AggregateRoot, Entity, EntityId};
pub use error::{DomainError, Result};
pub use event::{DomainEvent, EventEnvelope, EventMetadata};
pub use port::{Cache, Clock, EventPublisher, HealthCheck, HealthStatus, IdGenerator};
pub use repository::{Page, PaginatedRepository, Repository};
pub use service::{CommandHandler, QueryHandler, ServiceContext, ServiceRegistry};
pub use value_object::{Currency, EmailAddress, Money, ValueObject};
