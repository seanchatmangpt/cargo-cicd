use proptest::prelude::*;
use project_core::Verdict;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Produce every canonical uppercase verdict string exactly once per shrink
/// boundary, so all four arms are exercised uniformly.
fn verdict_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("PASS".to_owned()),
        Just("WARN".to_owned()),
        Just("FAIL".to_owned()),
        Just("BLOCKED".to_owned()),
    ]
}

/// Produce a `Verdict` value directly.
fn any_verdict() -> impl Strategy<Value = Verdict> {
    prop_oneof![
        Just(Verdict::Pass),
        Just(Verdict::Warn),
        Just(Verdict::Fail),
        Just(Verdict::Blocked),
    ]
}

/// Produce a lowercase version of a verdict string to check case-insensitivity.
fn verdict_string_lower() -> impl Strategy<Value = String> {
    verdict_string().prop_map(|s| s.to_lowercase())
}

/// Produce a mixed-case version (first char upper, rest lower).
fn verdict_string_mixed() -> impl Strategy<Value = String> {
    verdict_string().prop_map(|s| {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let upper: String = first.to_uppercase().collect();
                upper + &chars.as_str().to_lowercase()
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Roundtrip: `parse(display(v)) == v` for all four variants.
    ///
    /// Display must produce a string that FromStr accepts and that round-trips
    /// back to the identical variant.
    #[test]
    fn verdict_display_parse_roundtrip(v in any_verdict()) {
        let displayed = format!("{}", v);
        let parsed: Verdict = displayed.parse()
            .expect("display output must be parseable by FromStr");
        prop_assert_eq!(v, parsed,
            "display->parse roundtrip failed: display={:?}", displayed);
    }

    /// `is_ok()` semantics: Pass, Warn, and Blocked are considered "ok";
    /// Fail is not.
    ///
    /// This test encodes the invariant as a property so that any refactor of
    /// `is_ok` that accidentally inverts a branch is caught immediately.
    #[test]
    fn verdict_is_ok_consistency(v in any_verdict()) {
        let expected_ok = match v {
            Verdict::Pass    => true,
            Verdict::Warn    => true,
            Verdict::Blocked => true,
            Verdict::Fail    => false,
        };
        prop_assert_eq!(v.is_ok(), expected_ok,
            "is_ok() disagrees with expected value for variant {:?}", v);
    }

    /// `label()` returns exactly the same string that `FromStr` accepts
    /// (case-insensitive parse should succeed on the label).
    ///
    /// Specifically: `v.label().parse::<Verdict>()` must succeed and equal v.
    #[test]
    fn verdict_label_parseable(v in any_verdict()) {
        let label = v.label();
        let parsed: Verdict = label.parse()
            .unwrap_or_else(|e| panic!(
                "label() produced a string that FromStr rejects: label={:?}, err={}", label, e
            ));
        prop_assert_eq!(v, parsed,
            "label()->parse roundtrip failed for variant {:?}", v);
    }

    /// Uppercase canonical strings must parse without error.
    #[test]
    fn verdict_uppercase_strings_parse(s in verdict_string()) {
        let result: Result<Verdict, _> = s.parse();
        prop_assert!(result.is_ok(),
            "uppercase canonical string {:?} failed to parse", s);
    }

    /// Lowercase canonical strings must parse without error (case-insensitive).
    #[test]
    fn verdict_lowercase_strings_parse(s in verdict_string_lower()) {
        let result: Result<Verdict, _> = s.parse();
        prop_assert!(result.is_ok(),
            "lowercase canonical string {:?} failed to parse", s);
    }

    /// Mixed-case canonical strings must parse without error.
    #[test]
    fn verdict_mixed_case_strings_parse(s in verdict_string_mixed()) {
        let result: Result<Verdict, _> = s.parse();
        prop_assert!(result.is_ok(),
            "mixed-case canonical string {:?} failed to parse", s);
    }

    /// Two verdicts are equal if and only if their labels are equal
    /// (case-insensitively).
    ///
    /// This ensures the `PartialEq` implementation is consistent with the
    /// string representation.
    #[test]
    fn verdict_equality_via_label(a in any_verdict(), b in any_verdict()) {
        let labels_equal = a.label().eq_ignore_ascii_case(b.label());
        let verdicts_equal = a == b;
        prop_assert_eq!(labels_equal, verdicts_equal,
            "label equality ({}) disagrees with PartialEq ({}) for {:?} vs {:?}",
            labels_equal, verdicts_equal, a, b);
    }

    /// `is_ok()` is the negation of `is_fail()` (if that method exists).
    ///
    /// Even if `is_fail()` is not defined we encode the invariant via `Fail`
    /// being the only non-ok variant.
    #[test]
    fn verdict_only_fail_is_not_ok(v in any_verdict()) {
        if !v.is_ok() {
            prop_assert_eq!(v, Verdict::Fail,
                "only Fail should return is_ok() == false, but got {:?}", v);
        }
    }

    /// Display output for any verdict is non-empty.
    #[test]
    fn verdict_display_is_nonempty(v in any_verdict()) {
        let s = format!("{}", v);
        prop_assert!(!s.is_empty(),
            "Display produced an empty string for {:?}", v);
    }

    /// Clone of a verdict equals the original.
    #[test]
    fn verdict_clone_equals_original(v in any_verdict()) {
        let cloned = v.clone();
        prop_assert_eq!(v, cloned, "clone() broke equality for {:?}", v);
    }

    /// Debug output is non-empty and does not panic.
    #[test]
    fn verdict_debug_is_nonempty(v in any_verdict()) {
        let s = format!("{:?}", v);
        prop_assert!(!s.is_empty(),
            "Debug produced an empty string for {:?}", v);
    }
}

// ---------------------------------------------------------------------------
// Unit sanity checks (complement the properties)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_pass_uppercase() {
        assert_eq!("PASS".parse::<Verdict>().unwrap(), Verdict::Pass);
    }

    #[test]
    fn parse_warn_uppercase() {
        assert_eq!("WARN".parse::<Verdict>().unwrap(), Verdict::Warn);
    }

    #[test]
    fn parse_fail_uppercase() {
        assert_eq!("FAIL".parse::<Verdict>().unwrap(), Verdict::Fail);
    }

    #[test]
    fn parse_blocked_uppercase() {
        assert_eq!("BLOCKED".parse::<Verdict>().unwrap(), Verdict::Blocked);
    }

    #[test]
    fn parse_unknown_string_is_err() {
        assert!("UNKNOWN".parse::<Verdict>().is_err());
        assert!("".parse::<Verdict>().is_err());
        assert!("pass_extra".parse::<Verdict>().is_err());
    }

    #[test]
    fn pass_is_ok() {
        assert!(Verdict::Pass.is_ok());
    }

    #[test]
    fn fail_is_not_ok() {
        assert!(!Verdict::Fail.is_ok());
    }

    #[test]
    fn warn_is_ok() {
        assert!(Verdict::Warn.is_ok());
    }

    #[test]
    fn blocked_is_ok() {
        assert!(Verdict::Blocked.is_ok());
    }
}
