use proptest::prelude::*;
use project_core::WorkspaceId;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate an arbitrary Unicode string (may be empty) for WorkspaceId.
fn arbitrary_string() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..=128)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

/// Generate a non-empty alphanumeric/symbol string — realistic workspace IDs.
fn realistic_id_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_\\-\\.]{1,64}")
        .expect("regex is valid")
}

/// Generate an empty string explicitly (edge case).
fn empty_string() -> impl Strategy<Value = String> {
    Just(String::new())
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// `new(s).as_str() == s` for any string, including empty.
    ///
    /// WorkspaceId must faithfully preserve the string it was constructed with;
    /// no normalisation, truncation, or escaping.
    #[test]
    fn workspace_id_new_as_str_roundtrip(s in arbitrary_string()) {
        let id = WorkspaceId::new(s.clone());
        prop_assert_eq!(id.as_str(), s.as_str(),
            "as_str() did not match constructor input for {:?}", s);
    }

    /// `format!("{}", id) == id.as_str()`.
    ///
    /// Display must be identical to `as_str()` — no extra formatting.
    #[test]
    fn workspace_id_display_equals_as_str(s in arbitrary_string()) {
        let id = WorkspaceId::new(s.clone());
        let displayed = format!("{}", id);
        prop_assert_eq!(displayed.as_str(), id.as_str(),
            "Display != as_str() for input {:?}", s);
    }

    /// Two `WorkspaceId` values created from the same string are equal by
    /// value (PartialEq), even though they are distinct heap allocations.
    #[test]
    fn workspace_id_same_string_equal_by_value(s in arbitrary_string()) {
        let id_a = WorkspaceId::new(s.clone());
        let id_b = WorkspaceId::new(s.clone());
        prop_assert_eq!(id_a, id_b,
            "Two WorkspaceIds from the same string are not equal: {:?}", s);
    }

    /// Two `WorkspaceId` values created from *different* strings are not equal.
    ///
    /// We construct pairs where the strings differ by at least one char.
    #[test]
    fn workspace_id_different_strings_not_equal(
        a in realistic_id_string(),
        b in realistic_id_string(),
    ) {
        // Only assert inequality when the strings actually differ.
        prop_assume!(a != b);
        let id_a = WorkspaceId::new(a.clone());
        let id_b = WorkspaceId::new(b.clone());
        prop_assert_ne!(id_a, id_b,
            "WorkspaceIds from different strings should not be equal: {:?} vs {:?}", a, b);
    }

    /// `WorkspaceId::new("")` does not panic; an empty workspace ID is valid.
    #[test]
    fn workspace_id_empty_string_no_panic(_s in empty_string()) {
        // Construction must not panic.
        let id = WorkspaceId::new(String::new());
        // as_str() on an empty ID returns "".
        prop_assert_eq!(id.as_str(), "",
            "empty WorkspaceId should return empty as_str()");
    }

    /// Clone of a WorkspaceId equals the original.
    #[test]
    fn workspace_id_clone_equals_original(s in arbitrary_string()) {
        let id = WorkspaceId::new(s.clone());
        let cloned = id.clone();
        prop_assert_eq!(id, cloned,
            "clone() broke equality for WorkspaceId({:?})", s);
    }

    /// Debug output is non-empty and does not panic.
    #[test]
    fn workspace_id_debug_nonempty(s in arbitrary_string()) {
        let id = WorkspaceId::new(s);
        let debug = format!("{:?}", id);
        prop_assert!(!debug.is_empty(),
            "Debug produced an empty string for WorkspaceId");
    }

    /// `as_str()` length matches the source string length (no hidden bytes added).
    #[test]
    fn workspace_id_as_str_length_matches(s in arbitrary_string()) {
        let id = WorkspaceId::new(s.clone());
        prop_assert_eq!(id.as_str().len(), s.len(),
            "as_str() byte length changed for input {:?}", s);
    }

    /// Realistic IDs (alphanumeric, hyphens, dots) produce non-empty as_str().
    #[test]
    fn workspace_id_realistic_nonempty_as_str(s in realistic_id_string()) {
        let id = WorkspaceId::new(s.clone());
        prop_assert!(!id.as_str().is_empty(),
            "non-empty input should yield non-empty as_str(), got {:?}", s);
    }

    /// Hash consistency: two equal WorkspaceIds must hash to the same value.
    ///
    /// This exercises the relationship between PartialEq and Hash (Rust's
    /// standard contract).
    #[test]
    fn workspace_id_hash_consistent_with_eq(s in arbitrary_string()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id_a = WorkspaceId::new(s.clone());
        let id_b = WorkspaceId::new(s.clone());

        prop_assert_eq!(id_a, id_b); // sanity

        let mut h_a = DefaultHasher::new();
        let mut h_b = DefaultHasher::new();
        id_a.hash(&mut h_a);
        id_b.hash(&mut h_b);

        prop_assert_eq!(h_a.finish(), h_b.finish(),
            "equal WorkspaceIds produced different hash values for input {:?}", s);
    }
}

// ---------------------------------------------------------------------------
// Unit sanity checks
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn new_and_as_str() {
        let id = WorkspaceId::new("my-workspace".to_string());
        assert_eq!(id.as_str(), "my-workspace");
    }

    #[test]
    fn display_matches_input() {
        let id = WorkspaceId::new("cargo-cicd".to_string());
        assert_eq!(format!("{}", id), "cargo-cicd");
    }

    #[test]
    fn two_ids_from_same_string_are_equal() {
        let a = WorkspaceId::new("workspace".to_string());
        let b = WorkspaceId::new("workspace".to_string());
        assert_eq!(a, b);
    }

    #[test]
    fn empty_workspace_id_is_valid() {
        let id = WorkspaceId::new(String::new());
        assert_eq!(id.as_str(), "");
    }

    #[test]
    fn unicode_workspace_id_preserved() {
        let s = "arbeitsbereich-αβγ".to_string();
        let id = WorkspaceId::new(s.clone());
        assert_eq!(id.as_str(), s);
    }
}
