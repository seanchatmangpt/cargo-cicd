//! Fuzz target: parse arbitrary byte sequences as `Verdict`.
//!
//! The invariant under test: `<str>.parse::<Verdict>()` must **never panic**
//! for any valid UTF-8 input — it should return `Ok(v)` or `Err(e)`, never
//! abort the process.
//!
//! To run this fuzz target (requires cargo-fuzz and a nightly toolchain):
//!
//!   cargo +nightly fuzz run parse_verdict
//!
//! Or with a corpus of known-interesting inputs:
//!
//!   cargo +nightly fuzz run parse_verdict fuzz/corpus/parse_verdict/
//!
//! A starter corpus is included in `fuzz/corpus/parse_verdict/`.
//! LibFuzzer will extend the corpus automatically via coverage feedback.
//!
//! To inspect a crash:
//!
//!   cargo +nightly fuzz fmt parse_verdict fuzz/artifacts/parse_verdict/crash-*

#![no_main]

use libfuzzer_sys::fuzz_target;
use project_core::Verdict;

fuzz_target!(|data: &[u8]| {
    // Only feed valid UTF-8 to the parser — malformed UTF-8 is a property of
    // the byte layer, not the string parser.  We skip non-UTF-8 inputs rather
    // than testing the UTF-8 decoder itself.
    if let Ok(s) = std::str::from_utf8(data) {
        // The ONLY invariant: must not panic.
        // It may return Ok or Err — both are correct.
        let _ = s.parse::<Verdict>();
    }
});
