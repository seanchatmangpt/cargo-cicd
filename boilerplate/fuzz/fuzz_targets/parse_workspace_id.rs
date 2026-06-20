//! Fuzz target: construct `WorkspaceId` from arbitrary byte sequences.
//!
//! `WorkspaceId::new(s)` must never panic for any valid UTF-8 string, and
//! `as_str()` must return a string byte-identical to the input.
//!
//! Run with:
//!   cargo +nightly fuzz run parse_workspace_id

#![no_main]

use libfuzzer_sys::fuzz_target;
use project_core::WorkspaceId;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let id = WorkspaceId::new(s.to_string());

        // Invariant 1: as_str() equals the input — no mutation.
        assert_eq!(
            id.as_str(),
            s,
            "WorkspaceId::as_str() must equal the constructor argument"
        );

        // Invariant 2: Display equals as_str() — consistent string representation.
        assert_eq!(
            format!("{}", id),
            s,
            "WorkspaceId Display must equal as_str()"
        );
    }
});
