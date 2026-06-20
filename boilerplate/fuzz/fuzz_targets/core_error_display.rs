//! Fuzz target: construct every `CoreError` variant from arbitrary byte
//! sequences and call Display on each.
//!
//! Invariant: `format!("{}", err)` must never panic for any UTF-8 payload.
//!
//! Run with:
//!   cargo +nightly fuzz run core_error_display

#![no_main]

use libfuzzer_sys::fuzz_target;
use project_core::CoreError;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Exercise all 8 variants.  We reuse the same string `s` as the
        // payload so the fuzzer can focus on content rather than combinator
        // selection.
        let variants: &[Box<dyn std::fmt::Display>] = &[
            Box::new(CoreError::workspace_not_found(s.to_string())),
            Box::new(CoreError::config_invalid(s.to_string(), s.to_string())),
            Box::new(CoreError::process_failed(s.to_string(), 1)),
            Box::new(CoreError::invariant_violated(s.to_string(), s.to_string())),
            Box::new(CoreError::io_error(s.to_string())),
            Box::new(CoreError::serialization_failed(s.to_string())),
            Box::new(CoreError::oracle_unavailable(s.to_string())),
            Box::new(CoreError::evidence_invalid(s.to_string())),
        ];

        for variant in variants {
            // Must not panic.
            let displayed = format!("{}", variant);
            // Must not be empty.
            assert!(!displayed.is_empty(), "CoreError Display must not be empty");
        }
    }
});
