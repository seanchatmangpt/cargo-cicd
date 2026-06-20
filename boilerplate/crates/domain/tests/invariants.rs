//! Property-based invariant tests for the domain layer.
//!
//! Uses `proptest` to verify that domain invariants hold for arbitrary inputs,
//! not just the specific cases covered by unit tests.
//!
//! # What is tested
//!
//! | Module | Invariants |
//! |--------|-----------|
//! | `entity` | EntityId round-trips, uniqueness, display |
//! | `value_object` | EmailAddress normalization, Money arithmetic |
//! | `repository` | Page navigation arithmetic |
//! | `error` | Error display never panics |

use project_domain::entity::EntityId;
use project_domain::error::DomainError;
use project_domain::repository::Page;
use project_domain::value_object::{Currency, Money};

use proptest::prelude::*;

// ── EntityId invariants ───────────────────────────────────────────────────────

proptest! {
    /// Two separately generated EntityIds must never be equal.
    #[test]
    fn entity_id_unique_for_all_generations(_seed in 0u32..1000) {
        struct Marker;
        let a: EntityId<Marker> = EntityId::new();
        let b: EntityId<Marker> = EntityId::new();
        prop_assert_ne!(a, b);
    }

    /// EntityId round-trips through its Display → parse_str path.
    #[test]
    fn entity_id_display_parse_roundtrip(_seed in 0u32..1000) {
        struct Marker;
        let id: EntityId<Marker> = EntityId::new();
        let s = id.to_string();
        let parsed = EntityId::<Marker>::parse_str(&s).expect("valid display string");
        prop_assert_eq!(id, parsed);
    }

    /// EntityId::parse_str rejects all non-UUID strings.
    #[test]
    fn entity_id_rejects_non_uuid_strings(garbage in "[a-z]{1,20}") {
        struct Marker;
        // Only accept strings that happen to be valid UUIDs (extremely unlikely
        // in this alphabet/length range, but guard anyway).
        let result = EntityId::<Marker>::parse_str(&garbage);
        if uuid::Uuid::parse_str(&garbage).is_err() {
            prop_assert!(result.is_err(), "should reject non-uuid '{}'", garbage);
        }
    }
}

// ── Money arithmetic invariants ───────────────────────────────────────────────

fn arb_currency() -> impl Strategy<Value = Currency> {
    prop_oneof![
        Just(Currency::Usd),
        Just(Currency::Eur),
        Just(Currency::Gbp),
        Just(Currency::Jpy),
        Just(Currency::Chf),
        Just(Currency::Cad),
        Just(Currency::Aud),
    ]
}

proptest! {
    /// Money addition is commutative: a + b == b + a.
    #[test]
    fn money_addition_is_commutative(
        a in -1_000_000_i64..1_000_000,
        b in -1_000_000_i64..1_000_000,
        currency in arb_currency(),
    ) {
        let ma = Money::new(a, currency);
        let mb = Money::new(b, currency);
        let ab = ma.add(&mb).unwrap();
        let ba = mb.add(&ma).unwrap();
        prop_assert_eq!(ab, ba);
    }

    /// Money addition is associative: (a + b) + c == a + (b + c).
    #[test]
    fn money_addition_is_associative(
        a in -100_000_i64..100_000,
        b in -100_000_i64..100_000,
        c in -100_000_i64..100_000,
        currency in arb_currency(),
    ) {
        let ma = Money::new(a, currency);
        let mb = Money::new(b, currency);
        let mc = Money::new(c, currency);
        let left = ma.add(&mb).unwrap().add(&mc).unwrap();
        let right = ma.add(&mb.add(&mc).unwrap()).unwrap();
        prop_assert_eq!(left, right);
    }

    /// Money zero is the additive identity: m + 0 == m.
    #[test]
    fn money_zero_is_additive_identity(
        amount in -1_000_000_i64..1_000_000,
        currency in arb_currency(),
    ) {
        let m = Money::new(amount, currency);
        let zero = Money::zero(currency);
        prop_assert_eq!(m.add(&zero).unwrap(), m.clone());
        prop_assert_eq!(zero.add(&m).unwrap(), m);
    }

    /// Subtracting a value and then adding it back gives the original.
    #[test]
    fn money_subtract_then_add_is_identity(
        base in -500_000_i64..500_000,
        delta in -500_000_i64..500_000,
        currency in arb_currency(),
    ) {
        let m = Money::new(base, currency);
        let d = Money::new(delta, currency);
        let result = m.subtract(&d).unwrap().add(&d).unwrap();
        prop_assert_eq!(result, m);
    }

    /// Mixed-currency addition always errors.
    #[test]
    fn money_cross_currency_add_always_errors(
        a in -100_000_i64..100_000,
        b in -100_000_i64..100_000,
    ) {
        let ma = Money::new(a, Currency::Usd);
        let mb = Money::new(b, Currency::Eur);
        prop_assert!(ma.add(&mb).is_err());
    }

    /// Money display never panics for any amount and currency.
    #[test]
    fn money_display_never_panics(
        amount in i64::MIN / 2..i64::MAX / 2,
        currency in arb_currency(),
    ) {
        let m = Money::new(amount, currency);
        let s = m.to_string();
        prop_assert!(!s.is_empty());
    }
}

// ── Page invariants ───────────────────────────────────────────────────────────

proptest! {
    /// total_pages * size >= total (enough pages to cover all items).
    #[test]
    fn page_total_pages_covers_total(
        total in 0u64..10_000,
        size in 1u32..100,
    ) {
        let page: Page<()> = Page::new(vec![], total, 0, size);
        let pages = page.total_pages();
        prop_assert!(
            pages * size as u64 >= total,
            "pages={} * size={} must cover total={}",
            pages, size, total
        );
    }

    /// First page never has_previous.
    #[test]
    fn page_first_never_has_previous(
        total in 0u64..10_000,
        size in 1u32..100,
    ) {
        let count = (total as usize).min(size as usize);
        let items: Vec<u64> = (0..count as u64).collect();
        let page = Page::new(items, total, 0, size);
        prop_assert!(!page.has_previous());
    }

    /// A page beyond the last page has no items and no next.
    #[test]
    fn page_beyond_end_is_empty(
        total in 0u64..100,
        size in 1u32..10,
    ) {
        // Request a very high page number — way past the end.
        let far_page = 1_000u32;
        let page: Page<u64> = Page::new(vec![], total, far_page, size);
        prop_assert!(page.is_empty());
        prop_assert!(!page.has_next());
    }

    /// Page::empty always has is_empty() == true.
    #[test]
    fn page_empty_is_always_empty(size in 1u32..100) {
        let page: Page<u64> = Page::empty(size);
        prop_assert!(page.is_empty());
        prop_assert_eq!(page.len(), 0);
    }
}

// ── DomainError display invariants ───────────────────────────────────────────

proptest! {
    /// DomainError::not_found display contains both entity type and id.
    #[test]
    fn domain_error_not_found_display_contains_id(
        id in "[a-z0-9\\-]{1,40}",
    ) {
        let err = DomainError::not_found("Order", id.clone());
        let msg = err.to_string();
        prop_assert!(
            msg.contains(&id),
            "display '{}' should contain id '{}'", msg, id
        );
        prop_assert!(msg.contains("Order"));
    }

    /// DomainError::ValidationFailed display contains field and reason.
    #[test]
    fn domain_error_validation_display_contains_field_and_reason(
        reason in "[a-zA-Z0-9 ]{1,80}",
    ) {
        let err = DomainError::validation("email", reason.clone());
        let msg = err.to_string();
        prop_assert!(msg.contains("email"));
        prop_assert!(msg.contains(&reason));
    }

    /// DomainError::Conflict display contains the message.
    #[test]
    fn domain_error_conflict_display_contains_message(
        message in "[a-zA-Z0-9 ]{1,80}",
    ) {
        let err = DomainError::conflict(message.clone());
        prop_assert!(err.to_string().contains(&message));
    }
}

// ── EmailAddress invariants ───────────────────────────────────────────────────

proptest! {
    /// Valid email addresses are always lowercased after construction.
    #[test]
    fn email_always_lowercased(
        local in "[a-zA-Z0-9]{1,20}",
        domain in "[a-zA-Z0-9]{1,10}",
        tld in "[a-zA-Z]{2,5}",
    ) {
        use project_domain::value_object::EmailAddress;
        use std::str::FromStr;

        let raw = format!("{}@{}.{}", local, domain, tld);
        match EmailAddress::from_str(&raw) {
            Ok(email) => {
                prop_assert_eq!(
                    email.as_str(),
                    raw.to_lowercase(),
                    "email should be lowercased"
                );
            }
            Err(_) => {
                // Some generated strings may still be rejected — that's fine,
                // the invariant only applies to accepted values.
            }
        }
    }

    /// Emails without '@' are always rejected.
    #[test]
    fn email_without_at_always_rejected(s in "[a-zA-Z0-9\\.]{1,50}") {
        use project_domain::value_object::EmailAddress;
        use std::str::FromStr;

        if !s.contains('@') {
            prop_assert!(
                EmailAddress::from_str(&s).is_err(),
                "'{}' has no '@' so must be rejected",
                s
            );
        }
    }
}
