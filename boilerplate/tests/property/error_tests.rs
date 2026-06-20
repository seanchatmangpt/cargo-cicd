use proptest::prelude::*;
use project_core::CoreError;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Printable ASCII, no control characters, length 1–64.
fn printable_ascii() -> impl Strategy<Value = String> {
    prop::string::string_regex("[[:print:]]{1,64}")
        .expect("regex is valid")
}

/// Possibly-empty printable ASCII, length 0–64.
fn printable_ascii_or_empty() -> impl Strategy<Value = String> {
    prop::string::string_regex("[[:print:]]{0,64}")
        .expect("regex is valid")
}

/// Arbitrary Unicode, length 0–128 chars.
fn arbitrary_string() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..=128)
        .prop_map(|v| v.into_iter().collect::<String>())
}

/// Exit-code-like i32 in [-1, 255].
fn exit_code() -> impl Strategy<Value = i32> {
    -1_i32..=255_i32
}

/// Generate one of the eight CoreError variants with arbitrary string payloads.
fn any_core_error() -> impl Strategy<Value = CoreError> {
    prop_oneof![
        printable_ascii()
            .prop_map(CoreError::workspace_not_found),
        (printable_ascii(), printable_ascii())
            .prop_map(|(f, r)| CoreError::config_invalid(f, r)),
        (printable_ascii(), exit_code())
            .prop_map(|(cmd, code)| CoreError::process_failed(cmd, code)),
        (printable_ascii(), printable_ascii())
            .prop_map(|(name, details)| CoreError::invariant_violated(name, details)),
        printable_ascii()
            .prop_map(CoreError::io_error),
        printable_ascii()
            .prop_map(CoreError::serialization_failed),
        printable_ascii()
            .prop_map(CoreError::oracle_unavailable),
        printable_ascii()
            .prop_map(CoreError::evidence_invalid),
    ]
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    /// Every CoreError variant displays without panicking.
    ///
    /// The Display impl must never call unwrap/expect internally for any
    /// string payload. This property exercises all eight variants.
    #[test]
    fn all_variants_display_without_panicking(e in any_core_error()) {
        let displayed = format!("{}", e);
        prop_assert!(!displayed.is_empty(),
            "Display produced an empty string for {:?}", e);
    }

    /// `CoreError::workspace_not_found(s)` — the display string contains `s`.
    ///
    /// Users must be able to see the reason why the workspace was not found.
    #[test]
    fn workspace_not_found_display_contains_reason(s in printable_ascii()) {
        let err = CoreError::workspace_not_found(s.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(s.as_str()),
            "workspace_not_found display {:?} does not contain reason {:?}",
            displayed, s
        );
    }

    /// `CoreError::workspace_not_found` with an arbitrary Unicode reason does
    /// not panic on Display.
    #[test]
    fn workspace_not_found_unicode_reason_no_panic(s in arbitrary_string()) {
        let err = CoreError::workspace_not_found(s.clone());
        let displayed = format!("{}", err);
        prop_assert!(!displayed.is_empty());
    }

    /// `CoreError::config_invalid(field, reason)` — display contains both
    /// `field` and `reason`.
    ///
    /// Both pieces of context must be surfaced so a user can diagnose which
    /// config key is wrong and why.
    #[test]
    fn config_invalid_display_contains_field_and_reason(
        field  in printable_ascii(),
        reason in printable_ascii(),
    ) {
        let err = CoreError::config_invalid(field.clone(), reason.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(field.as_str()),
            "config_invalid display {:?} does not contain field {:?}",
            displayed, field
        );
        prop_assert!(
            displayed.contains(reason.as_str()),
            "config_invalid display {:?} does not contain reason {:?}",
            displayed, reason
        );
    }

    /// `CoreError::process_failed(cmd, code)` — display contains `cmd` and `code`.
    ///
    /// Both the command name and the numeric exit code must appear so that
    /// operators can debug pipeline failures.
    #[test]
    fn process_failed_display_contains_cmd_and_code(
        cmd  in printable_ascii(),
        code in exit_code(),
    ) {
        let err = CoreError::process_failed(cmd.clone(), code);
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(cmd.as_str()),
            "process_failed display {:?} does not contain cmd {:?}",
            displayed, cmd
        );
        prop_assert!(
            displayed.contains(&code.to_string()),
            "process_failed display {:?} does not contain code {}",
            displayed, code
        );
    }

    /// `CoreError::invariant_violated(name, details)` — display contains both
    /// `name` and `details`.
    #[test]
    fn invariant_violated_display_contains_name_and_details(
        name    in printable_ascii(),
        details in printable_ascii(),
    ) {
        let err = CoreError::invariant_violated(name.clone(), details.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(name.as_str()),
            "invariant_violated display {:?} does not contain name {:?}",
            displayed, name
        );
        prop_assert!(
            displayed.contains(details.as_str()),
            "invariant_violated display {:?} does not contain details {:?}",
            displayed, details
        );
    }

    /// `CoreError::io_error(msg)` — display contains `msg`.
    #[test]
    fn io_error_display_contains_msg(msg in printable_ascii()) {
        let err = CoreError::io_error(msg.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(msg.as_str()),
            "io_error display {:?} does not contain msg {:?}",
            displayed, msg
        );
    }

    /// `CoreError::serialization_failed(msg)` — display contains `msg`.
    #[test]
    fn serialization_failed_display_contains_msg(msg in printable_ascii()) {
        let err = CoreError::serialization_failed(msg.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(msg.as_str()),
            "serialization_failed display {:?} does not contain msg {:?}",
            displayed, msg
        );
    }

    /// `CoreError::oracle_unavailable(msg)` — display contains `msg`.
    #[test]
    fn oracle_unavailable_display_contains_msg(msg in printable_ascii()) {
        let err = CoreError::oracle_unavailable(msg.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(msg.as_str()),
            "oracle_unavailable display {:?} does not contain msg {:?}",
            displayed, msg
        );
    }

    /// `CoreError::evidence_invalid(msg)` — display contains `msg`.
    #[test]
    fn evidence_invalid_display_contains_msg(msg in printable_ascii()) {
        let err = CoreError::evidence_invalid(msg.clone());
        let displayed = format!("{}", err);
        prop_assert!(
            displayed.contains(msg.as_str()),
            "evidence_invalid display {:?} does not contain msg {:?}",
            displayed, msg
        );
    }

    /// Debug output is non-empty for every variant.
    #[test]
    fn all_variants_debug_nonempty(e in any_core_error()) {
        let debug = format!("{:?}", e);
        prop_assert!(!debug.is_empty(),
            "Debug produced an empty string for a CoreError variant");
    }

    /// Empty payloads do not cause panics (edge case: reason/field/cmd = "").
    #[test]
    fn empty_payloads_do_not_panic(
        s1 in printable_ascii_or_empty(),
        s2 in printable_ascii_or_empty(),
        code in exit_code(),
    ) {
        // Each constructor with empty strings must not panic.
        let _ = format!("{}", CoreError::workspace_not_found(s1.clone()));
        let _ = format!("{}", CoreError::config_invalid(s1.clone(), s2.clone()));
        let _ = format!("{}", CoreError::process_failed(s1.clone(), code));
        let _ = format!("{}", CoreError::invariant_violated(s1.clone(), s2.clone()));
        let _ = format!("{}", CoreError::io_error(s1.clone()));
        let _ = format!("{}", CoreError::serialization_failed(s1.clone()));
        let _ = format!("{}", CoreError::oracle_unavailable(s1.clone()));
        let _ = format!("{}", CoreError::evidence_invalid(s1.clone()));
    }
}

// ---------------------------------------------------------------------------
// Unit sanity checks
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn workspace_not_found_contains_reason() {
        let err = CoreError::workspace_not_found("no Cargo.toml found".to_string());
        let s = format!("{}", err);
        assert!(s.contains("no Cargo.toml found"), "got: {}", s);
    }

    #[test]
    fn config_invalid_contains_both_parts() {
        let err = CoreError::config_invalid(
            "git.base_ref".to_string(),
            "must be a valid branch name".to_string(),
        );
        let s = format!("{}", err);
        assert!(s.contains("git.base_ref"), "got: {}", s);
        assert!(s.contains("must be a valid branch name"), "got: {}", s);
    }

    #[test]
    fn process_failed_contains_cmd_and_code() {
        let err = CoreError::process_failed("cargo build".to_string(), 101);
        let s = format!("{}", err);
        assert!(s.contains("cargo build"), "got: {}", s);
        assert!(s.contains("101"), "got: {}", s);
    }

    #[test]
    fn invariant_violated_contains_both_parts() {
        let err = CoreError::invariant_violated(
            "no_full_trybuild".to_string(),
            "trybuild ran all fixtures".to_string(),
        );
        let s = format!("{}", err);
        assert!(s.contains("no_full_trybuild"), "got: {}", s);
        assert!(s.contains("trybuild ran all fixtures"), "got: {}", s);
    }

    #[test]
    fn io_error_contains_msg() {
        let err = CoreError::io_error("permission denied on /tmp/foo".to_string());
        let s = format!("{}", err);
        assert!(s.contains("permission denied on /tmp/foo"), "got: {}", s);
    }

    #[test]
    fn serialization_failed_contains_msg() {
        let err = CoreError::serialization_failed("TOML parse error at line 3".to_string());
        let s = format!("{}", err);
        assert!(s.contains("TOML parse error at line 3"), "got: {}", s);
    }

    #[test]
    fn oracle_unavailable_contains_msg() {
        let err = CoreError::oracle_unavailable("wpm not found on PATH".to_string());
        let s = format!("{}", err);
        assert!(s.contains("wpm not found on PATH"), "got: {}", s);
    }

    #[test]
    fn evidence_invalid_contains_msg() {
        let err = CoreError::evidence_invalid("missing case_id in XES trace".to_string());
        let s = format!("{}", err);
        assert!(s.contains("missing case_id in XES trace"), "got: {}", s);
    }
}
