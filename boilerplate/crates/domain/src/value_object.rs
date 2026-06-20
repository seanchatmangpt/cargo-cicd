//! Value object primitives.
//!
//! Value objects have **no identity** — two value objects with the same fields
//! are considered identical. They are immutable; any mutation produces a new
//! instance. The [`ValueObject`] marker trait expresses this contract.
//!
//! # Provided value objects
//!
//! | Type | Invariants |
//! |------|-----------|
//! | [`EmailAddress`] | RFC 5321 local-part + `@` + domain; no whitespace |
//! | [`Money`] | Amount in minor units (e.g. cents) + ISO 4217 currency |
//! | [`Currency`] | Closed enum of supported ISO 4217 codes |

use std::fmt;
use std::str::FromStr;

use crate::error::{DomainError, Result};

// ── ValueObject marker trait ──────────────────────────────────────────────────

/// Marker trait for value objects.
///
/// Implementors must satisfy:
/// - `PartialEq + Eq` — equality is structural (value-based)
/// - `Hash` — usable as map keys
/// - `Clone` — cheaply copyable
/// - `Send + Sync + 'static` — safe to share across threads
pub trait ValueObject: Clone + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static {}

// ── EmailAddress ──────────────────────────────────────────────────────────────

/// A validated e-mail address.
///
/// Validation rules applied on construction:
/// - Must contain exactly one `@` character
/// - Local part (left of `@`) must not be empty
/// - Domain part (right of `@`) must contain at least one `.` and no whitespace
/// - Total length must not exceed 254 characters (RFC 5321)
///
/// # Example
///
/// ```rust
/// use std::str::FromStr;
/// use project_domain::value_object::EmailAddress;
///
/// let email = EmailAddress::from_str("alice@example.com").unwrap();
/// assert_eq!(email.as_str(), "alice@example.com");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Construct a new `EmailAddress`, validating the input.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationFailed`] if the string is not a valid
    /// e-mail address.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let value = raw.into().trim().to_lowercase();
        validate_email(&value)?;
        Ok(Self(value))
    }

    /// The normalized (lowercased, trimmed) e-mail string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The local part (left of `@`).
    pub fn local(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    /// The domain part (right of `@`).
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

fn validate_email(value: &str) -> Result<()> {
    if value.len() > 254 {
        return Err(DomainError::ValidationFailed {
            field: "email",
            reason: "exceeds maximum length of 254 characters".to_string(),
        });
    }
    let parts: Vec<&str> = value.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(DomainError::ValidationFailed {
            field: "email",
            reason: "must contain exactly one '@' character".to_string(),
        });
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() {
        return Err(DomainError::ValidationFailed {
            field: "email",
            reason: "local part must not be empty".to_string(),
        });
    }
    if domain.is_empty() || !domain.contains('.') || domain.contains(char::is_whitespace) {
        return Err(DomainError::ValidationFailed {
            field: "email",
            reason: "domain part must contain at least one '.' and no whitespace".to_string(),
        });
    }
    if value.contains(char::is_whitespace) {
        return Err(DomainError::ValidationFailed {
            field: "email",
            reason: "must not contain whitespace".to_string(),
        });
    }
    Ok(())
}

impl FromStr for EmailAddress {
    type Err = DomainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl ValueObject for EmailAddress {}

// ── Currency ──────────────────────────────────────────────────────────────────

/// ISO 4217 currency codes supported by the domain.
///
/// Extend this enum as additional currencies are needed. All variants must
/// be kept in sync with [`Currency::decimal_places`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Currency {
    /// United States Dollar (2 decimal places).
    Usd,
    /// Euro (2 decimal places).
    Eur,
    /// British Pound Sterling (2 decimal places).
    Gbp,
    /// Japanese Yen (0 decimal places).
    Jpy,
    /// Swiss Franc (2 decimal places).
    Chf,
    /// Canadian Dollar (2 decimal places).
    Cad,
    /// Australian Dollar (2 decimal places).
    Aud,
}

impl Currency {
    /// Number of decimal places in the minor unit for this currency.
    ///
    /// For example, USD uses cents (2 decimal places), while JPY has no
    /// sub-unit (0 decimal places).
    pub fn decimal_places(self) -> u8 {
        match self {
            Currency::Jpy => 0,
            _ => 2,
        }
    }

    /// The ISO 4217 alphabetic code string.
    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Jpy => "JPY",
            Currency::Chf => "CHF",
            Currency::Cad => "CAD",
            Currency::Aud => "AUD",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl FromStr for Currency {
    type Err = DomainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::Usd),
            "EUR" => Ok(Currency::Eur),
            "GBP" => Ok(Currency::Gbp),
            "JPY" => Ok(Currency::Jpy),
            "CHF" => Ok(Currency::Chf),
            "CAD" => Ok(Currency::Cad),
            "AUD" => Ok(Currency::Aud),
            other => Err(DomainError::ValidationFailed {
                field: "currency",
                reason: format!("unsupported currency code '{}'", other),
            }),
        }
    }
}

impl ValueObject for Currency {}

// ── Money ─────────────────────────────────────────────────────────────────────

/// A monetary amount expressed in minor units (e.g. cents for USD).
///
/// Amounts are stored as `i64` minor units to avoid floating-point rounding
/// errors. Negative values represent debits or refunds.
///
/// # Example
///
/// ```rust
/// use project_domain::value_object::{Money, Currency};
///
/// // $12.50 USD — stored as 1250 cents
/// let price = Money::new(1250, Currency::Usd);
/// assert_eq!(price.to_string(), "12.50 USD");
///
/// // ¥500 JPY — no decimal places
/// let yen = Money::new(500, Currency::Jpy);
/// assert_eq!(yen.to_string(), "500 JPY");
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Money {
    /// Amount in minor units (e.g. cents for USD).
    pub amount: i64,
    /// The currency this amount is denominated in.
    pub currency: Currency,
}

impl Money {
    /// Construct a `Money` value from minor units and a currency.
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Zero amount for the given currency.
    pub fn zero(currency: Currency) -> Self {
        Self::new(0, currency)
    }

    /// Return `true` if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Return `true` if the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }

    /// Add two `Money` values of the same currency.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationFailed`] if the currencies differ.
    /// Returns [`DomainError::BusinessRuleViolated`] on arithmetic overflow.
    pub fn add(&self, other: &Self) -> Result<Self> {
        self.assert_same_currency(other)?;
        let amount = self.amount.checked_add(other.amount).ok_or_else(|| {
            DomainError::BusinessRuleViolated {
                rule: "money_no_overflow",
                details: "monetary addition would overflow i64".to_string(),
            }
        })?;
        Ok(Self::new(amount, self.currency))
    }

    /// Subtract `other` from `self`, both in the same currency.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationFailed`] if the currencies differ.
    /// Returns [`DomainError::BusinessRuleViolated`] on arithmetic overflow.
    pub fn subtract(&self, other: &Self) -> Result<Self> {
        self.assert_same_currency(other)?;
        let amount = self.amount.checked_sub(other.amount).ok_or_else(|| {
            DomainError::BusinessRuleViolated {
                rule: "money_no_overflow",
                details: "monetary subtraction would overflow i64".to_string(),
            }
        })?;
        Ok(Self::new(amount, self.currency))
    }

    fn assert_same_currency(&self, other: &Self) -> Result<()> {
        if self.currency != other.currency {
            return Err(DomainError::ValidationFailed {
                field: "currency",
                reason: format!(
                    "cannot mix currencies: {} vs {}",
                    self.currency, other.currency
                ),
            });
        }
        Ok(())
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let places = self.currency.decimal_places();
        if places == 0 {
            write!(f, "{} {}", self.amount, self.currency)
        } else {
            let divisor = 10_i64.pow(places as u32);
            let major = self.amount / divisor;
            let minor = (self.amount % divisor).unsigned_abs();
            write!(f, "{}.{:0>width$} {}", major, minor, self.currency, width = places as usize)
        }
    }
}

/// Parse from `"1250 USD"` or `"1250USD"` format (amount in minor units).
impl FromStr for Money {
    type Err = DomainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        // Accept both "1250 USD" and "1250USD"
        let split_pos = s
            .find(|c: char| c.is_alphabetic())
            .ok_or_else(|| DomainError::ValidationFailed {
                field: "money",
                reason: format!("expected '<amount> <CURRENCY>', got '{}'", s),
            })?;
        let amount_str = s[..split_pos].trim();
        let currency_str = s[split_pos..].trim();
        let amount: i64 = amount_str.parse().map_err(|_| DomainError::ValidationFailed {
            field: "money",
            reason: format!("invalid amount '{}'", amount_str),
        })?;
        let currency = Currency::from_str(currency_str)?;
        Ok(Self::new(amount, currency))
    }
}

impl ValueObject for Money {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // EmailAddress tests
    #[test]
    fn email_valid_addresses_accepted() {
        let cases = [
            "alice@example.com",
            "bob+tag@sub.domain.org",
            "UPPER@Example.COM", // normalized to lowercase
        ];
        for addr in cases {
            assert!(EmailAddress::new(addr).is_ok(), "should accept: {}", addr);
        }
    }

    #[test]
    fn email_normalized_to_lowercase() {
        let email = EmailAddress::new("Alice@Example.COM").unwrap();
        assert_eq!(email.as_str(), "alice@example.com");
    }

    #[test]
    fn email_local_and_domain_parts() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.local(), "user");
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn email_rejects_missing_at() {
        assert!(EmailAddress::new("nodomain.com").is_err());
    }

    #[test]
    fn email_rejects_empty_local() {
        assert!(EmailAddress::new("@example.com").is_err());
    }

    #[test]
    fn email_rejects_domain_without_dot() {
        assert!(EmailAddress::new("user@localhost").is_err());
    }

    #[test]
    fn email_rejects_whitespace() {
        assert!(EmailAddress::new("user @example.com").is_err());
    }

    #[test]
    fn email_from_str_roundtrip() {
        let email: EmailAddress = "dev@rust-lang.org".parse().unwrap();
        assert_eq!(email.to_string(), "dev@rust-lang.org");
    }

    // Currency tests
    #[test]
    fn currency_decimal_places() {
        assert_eq!(Currency::Usd.decimal_places(), 2);
        assert_eq!(Currency::Eur.decimal_places(), 2);
        assert_eq!(Currency::Jpy.decimal_places(), 0);
    }

    #[test]
    fn currency_from_str_case_insensitive() {
        assert_eq!("usd".parse::<Currency>().unwrap(), Currency::Usd);
        assert_eq!("EUR".parse::<Currency>().unwrap(), Currency::Eur);
    }

    #[test]
    fn currency_from_str_rejects_unknown() {
        assert!("XYZ".parse::<Currency>().is_err());
    }

    // Money tests
    #[test]
    fn money_display_usd_two_decimal_places() {
        assert_eq!(Money::new(1250, Currency::Usd).to_string(), "12.50 USD");
        assert_eq!(Money::new(100, Currency::Usd).to_string(), "1.00 USD");
        assert_eq!(Money::new(5, Currency::Usd).to_string(), "0.05 USD");
    }

    #[test]
    fn money_display_jpy_no_decimal_places() {
        assert_eq!(Money::new(500, Currency::Jpy).to_string(), "500 JPY");
    }

    #[test]
    fn money_zero() {
        let m = Money::zero(Currency::Eur);
        assert!(m.is_zero());
        assert!(!m.is_negative());
    }

    #[test]
    fn money_add_same_currency() {
        let a = Money::new(100, Currency::Usd);
        let b = Money::new(50, Currency::Usd);
        assert_eq!(a.add(&b).unwrap(), Money::new(150, Currency::Usd));
    }

    #[test]
    fn money_add_mixed_currency_errors() {
        let a = Money::new(100, Currency::Usd);
        let b = Money::new(100, Currency::Eur);
        assert!(a.add(&b).is_err());
    }

    #[test]
    fn money_subtract() {
        let a = Money::new(200, Currency::Gbp);
        let b = Money::new(50, Currency::Gbp);
        assert_eq!(a.subtract(&b).unwrap(), Money::new(150, Currency::Gbp));
    }

    #[test]
    fn money_from_str() {
        let m: Money = "1250 USD".parse().unwrap();
        assert_eq!(m, Money::new(1250, Currency::Usd));
    }
}
