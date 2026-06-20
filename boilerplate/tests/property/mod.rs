pub mod verdict_tests;
pub mod workspace_id_tests;
pub mod error_tests;

use proptest::prelude::*;

/// Shared strategy: generate one of the four canonical verdict strings.
pub fn verdict_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("PASS".to_owned()),
        Just("WARN".to_owned()),
        Just("FAIL".to_owned()),
        Just("BLOCKED".to_owned()),
    ]
}

/// Shared strategy: generate a non-empty ASCII identifier string suitable for
/// workspace IDs and similar newtypes.
pub fn nonempty_string_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_\\-\\.]{1,64}")
        .expect("regex is valid")
}

/// Shared strategy: generate an arbitrary (possibly empty) Unicode string
/// for fuzz-style property checks.
pub fn arbitrary_string_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<char>(), 0..=128)
        .prop_map(|chars| chars.into_iter().collect())
}

/// Shared strategy: generate a short printable ASCII string (no control
/// characters) that is safe to embed in display/error messages.
pub fn printable_ascii_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[[:print:]]{0,64}")
        .expect("regex is valid")
}

/// Shared strategy: generate a non-empty printable ASCII string.
pub fn nonempty_printable_ascii_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[[:print:]]{1,64}")
        .expect("regex is valid")
}

/// Shared strategy: generate an i32 exit code in the realistic range [-1, 255].
pub fn exit_code_strategy() -> impl Strategy<Value = i32> {
    -1_i32..=255_i32
}
