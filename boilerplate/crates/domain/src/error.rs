//! Domain error taxonomy.
//!
//! All fallible domain operations return [`Result<T>`], which is
//! `std::result::Result<T, DomainError>`. Using the crate-local alias avoids
//! repetition and makes signatures easier to read.
//!
//! # Variants
//!
//! | Variant | When to use |
//! |---------|-------------|
//! | [`NotFound`](DomainError::NotFound) | Entity lookup returned nothing |
//! | [`ValidationFailed`](DomainError::ValidationFailed) | Input violates a field constraint |
//! | [`Conflict`](DomainError::Conflict) | Optimistic concurrency or duplicate key |
//! | [`Unauthorized`](DomainError::Unauthorized) | Actor lacks permission for an action |
//! | [`BusinessRuleViolated`](DomainError::BusinessRuleViolated) | Invariant enforced by domain logic |

use thiserror::Error;

/// All errors that can originate in the domain layer.
///
/// This enum is `#[non_exhaustive]` so that adding new variants in future
/// minor releases does not break existing match arms in downstream crates.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DomainError {
    /// The requested entity was not found in the repository.
    #[error("{entity_type} with id '{id}' was not found")]
    NotFound {
        /// The unqualified entity type name (e.g. `"Order"`).
        entity_type: &'static str,
        /// The string representation of the missing id.
        id: String,
    },

    /// A field value does not satisfy a domain constraint.
    #[error("validation failed for field '{field}': {reason}")]
    ValidationFailed {
        /// The field whose value is invalid.
        field: &'static str,
        /// Human-readable explanation of why the value is rejected.
        reason: String,
    },

    /// A conflicting state was detected (e.g. duplicate entity, stale version).
    #[error("conflict: {message}")]
    Conflict {
        /// Human-readable description of what conflicted.
        message: String,
    },

    /// The current actor is not permitted to perform an action.
    #[error("unauthorized: actor is not permitted to '{action}'")]
    Unauthorized {
        /// The name of the action that was refused.
        action: &'static str,
    },

    /// A domain invariant or business rule was violated.
    #[error("business rule '{rule}' violated: {details}")]
    BusinessRuleViolated {
        /// A stable machine-readable rule identifier.
        rule: &'static str,
        /// Human-readable explanation of the violation.
        details: String,
    },
}

impl DomainError {
    /// Convenience constructor for a `NotFound` error.
    ///
    /// ```rust
    /// use project_domain::error::DomainError;
    ///
    /// let err = DomainError::not_found("Order", "ord-001");
    /// ```
    pub fn not_found(entity_type: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type,
            id: id.into(),
        }
    }

    /// Convenience constructor for a `ValidationFailed` error.
    pub fn validation(field: &'static str, reason: impl Into<String>) -> Self {
        Self::ValidationFailed {
            field,
            reason: reason.into(),
        }
    }

    /// Convenience constructor for a `Conflict` error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Convenience constructor for a `BusinessRuleViolated` error.
    pub fn rule_violated(rule: &'static str, details: impl Into<String>) -> Self {
        Self::BusinessRuleViolated {
            rule,
            details: details.into(),
        }
    }

    /// Returns `true` if this error represents a "not found" condition.
    ///
    /// Useful for translating domain errors into HTTP 404 responses at the
    /// adapter boundary.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns `true` if this is an authorization failure.
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized { .. })
    }

    /// Returns `true` if this is a validation error.
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationFailed { .. })
    }
}

/// Crate-local result alias.
///
/// All fallible domain operations should use this rather than spelling out
/// `std::result::Result<T, DomainError>`.
pub type Result<T> = std::result::Result<T, DomainError>;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display_includes_entity_type_and_id() {
        let err = DomainError::not_found("Order", "ord-42");
        let msg = err.to_string();
        assert!(msg.contains("Order"), "should mention entity type");
        assert!(msg.contains("ord-42"), "should mention id");
    }

    #[test]
    fn validation_failed_display_includes_field_and_reason() {
        let err = DomainError::validation("email", "must contain @");
        let msg = err.to_string();
        assert!(msg.contains("email"));
        assert!(msg.contains("must contain @"));
    }

    #[test]
    fn conflict_display_includes_message() {
        let err = DomainError::conflict("duplicate order id");
        assert!(err.to_string().contains("duplicate order id"));
    }

    #[test]
    fn business_rule_violated_display() {
        let err = DomainError::rule_violated("max_items", "cart exceeds 50 items");
        let msg = err.to_string();
        assert!(msg.contains("max_items"));
        assert!(msg.contains("cart exceeds 50 items"));
    }

    #[test]
    fn is_not_found_predicate() {
        assert!(DomainError::not_found("X", "1").is_not_found());
        assert!(!DomainError::conflict("x").is_not_found());
    }

    #[test]
    fn is_unauthorized_predicate() {
        let err = DomainError::Unauthorized { action: "delete" };
        assert!(err.is_unauthorized());
        assert!(!err.is_not_found());
    }

    #[test]
    fn is_validation_error_predicate() {
        assert!(DomainError::validation("f", "r").is_validation_error());
        assert!(!DomainError::conflict("c").is_validation_error());
    }

    #[test]
    fn result_alias_works() {
        fn may_fail(fail: bool) -> Result<i32> {
            if fail {
                Err(DomainError::not_found("Thing", "1"))
            } else {
                Ok(42)
            }
        }
        assert_eq!(may_fail(false).unwrap(), 42);
        assert!(may_fail(true).is_err());
    }
}
