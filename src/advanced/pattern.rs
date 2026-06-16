//! High-throughput multi-pattern scanning via `aho-corasick`.
//!
//! [`MultiPatternScanner`] compiles an arbitrary, caller-supplied set of literal
//! patterns into a single Aho-Corasick automaton, then scans text in a single
//! pass regardless of how many patterns are in play. This is well suited to
//! engine work such as:
//!
//! * matching changed-file paths against a long list of ignore substrings or
//!   glob fragments (e.g. `target/`, `node_modules`, `.git`),
//! * auditing arbitrary text against a configurable deny-list that the caller
//!   provides at runtime,
//! * classifying which of many configured markers appear in a blob of output.
//!
//! Crucially, the patterns are always a **runtime parameter** — this module
//! hard-codes no vocabulary of its own. Callers own the policy; the scanner
//! only provides the search machinery.
//!
//! ```
//! use cargo_cicd::advanced::pattern::MultiPatternScanner;
//!
//! let scanner = MultiPatternScanner::new(&["target", "node_modules"]).unwrap();
//! assert!(scanner.contains_any("crates/foo/target/debug"));
//! assert_eq!(scanner.matched_patterns("a/target/b"), vec!["target".to_string()]);
//! ```

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

/// A single non-overlapping match produced by [`MultiPatternScanner::scan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternMatch {
    /// Index of the matched pattern within the original pattern slice.
    pub pattern_index: usize,
    /// The matched pattern text.
    pub pattern: String,
    /// Byte offset of the match start within the haystack.
    pub start: usize,
    /// Byte offset of the match end (exclusive) within the haystack.
    pub end: usize,
}

/// A compiled multi-pattern scanner over a caller-supplied pattern set.
///
/// Holds the compiled [`AhoCorasick`] automaton alongside the owned pattern
/// strings so matches can be reported back with their original text.
#[derive(Debug, Clone)]
pub struct MultiPatternScanner {
    automaton: AhoCorasick,
    patterns: Vec<String>,
}

impl MultiPatternScanner {
    /// Build a case-sensitive scanner from an arbitrary set of patterns.
    ///
    /// # Errors
    /// Returns the underlying [`aho_corasick::BuildError`] if the automaton
    /// cannot be constructed (for example, on an empty pattern set).
    pub fn new<P>(patterns: &[P]) -> Result<Self, aho_corasick::BuildError>
    where
        P: AsRef<str>,
    {
        let owned: Vec<String> = patterns.iter().map(|p| p.as_ref().to_string()).collect();
        let automaton = AhoCorasick::new(&owned)?;
        Ok(Self {
            automaton,
            patterns: owned,
        })
    }

    /// Build a case-insensitive (ASCII) scanner from an arbitrary set of patterns.
    ///
    /// # Errors
    /// Returns the underlying [`aho_corasick::BuildError`] if the automaton
    /// cannot be constructed.
    pub fn new_case_insensitive<P>(patterns: &[P]) -> Result<Self, aho_corasick::BuildError>
    where
        P: AsRef<str>,
    {
        let owned: Vec<String> = patterns.iter().map(|p| p.as_ref().to_string()).collect();
        let automaton = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&owned)?;
        Ok(Self {
            automaton,
            patterns: owned,
        })
    }

    /// Return all non-overlapping leftmost matches in `haystack`.
    pub fn scan(&self, haystack: &str) -> Vec<PatternMatch> {
        self.automaton
            .find_iter(haystack)
            .map(|m| {
                let idx = m.pattern().as_usize();
                PatternMatch {
                    pattern_index: idx,
                    pattern: self.patterns[idx].clone(),
                    start: m.start(),
                    end: m.end(),
                }
            })
            .collect()
    }

    /// Return `true` if any pattern occurs in `haystack`.
    pub fn contains_any(&self, haystack: &str) -> bool {
        self.automaton.is_match(haystack)
    }

    /// Return the deduplicated set of patterns that occur in `haystack`,
    /// in first-seen order.
    pub fn matched_patterns(&self, haystack: &str) -> Vec<String> {
        let mut seen = vec![false; self.patterns.len()];
        let mut out = Vec::new();
        for m in self.automaton.find_iter(haystack) {
            let idx = m.pattern().as_usize();
            if !seen[idx] {
                seen[idx] = true;
                out.push(self.patterns[idx].clone());
            }
        }
        out
    }

    /// The patterns this scanner was built from, in their original order.
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_reports_multiple_matches_with_offsets() {
        let scanner = MultiPatternScanner::new(&["target", "cache"]).unwrap();
        let haystack = "a/target/b/cache/c";
        let matches = scanner.scan(haystack);

        assert_eq!(matches.len(), 2);

        assert_eq!(matches[0].pattern, "target");
        assert_eq!(matches[0].pattern_index, 0);
        assert_eq!(matches[0].start, 2);
        assert_eq!(matches[0].end, 8);
        assert_eq!(&haystack[matches[0].start..matches[0].end], "target");

        assert_eq!(matches[1].pattern, "cache");
        assert_eq!(matches[1].pattern_index, 1);
        assert_eq!(matches[1].start, 11);
        assert_eq!(matches[1].end, 16);
        assert_eq!(&haystack[matches[1].start..matches[1].end], "cache");
    }

    #[test]
    fn no_match_returns_empty_and_false() {
        let scanner = MultiPatternScanner::new(&["target", "node_modules"]).unwrap();
        assert!(scanner.scan("src/lib.rs").is_empty());
        assert!(!scanner.contains_any("src/lib.rs"));
        assert!(scanner.matched_patterns("src/lib.rs").is_empty());
    }

    #[test]
    fn case_insensitive_matching_works() {
        let sensitive = MultiPatternScanner::new(&["build"]).unwrap();
        assert!(!sensitive.contains_any("Crates/BUILD/out"));

        let insensitive = MultiPatternScanner::new_case_insensitive(&["build"]).unwrap();
        assert!(insensitive.contains_any("Crates/BUILD/out"));
        let matches = insensitive.scan("Crates/BUILD/out");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern, "build");
        assert_eq!(matches[0].start, 7);
        assert_eq!(matches[0].end, 12);
    }

    #[test]
    fn contains_any_true_and_false() {
        let scanner = MultiPatternScanner::new(&["target", "cache"]).unwrap();
        assert!(scanner.contains_any("workspace/target/debug"));
        assert!(!scanner.contains_any("workspace/src/main.rs"));
    }

    #[test]
    fn matched_patterns_dedups_in_first_seen_order() {
        let scanner = MultiPatternScanner::new(&["target", "cache"]).unwrap();
        let hit = scanner.matched_patterns("target/a/cache/b/target/c/cache");
        assert_eq!(hit, vec!["target".to_string(), "cache".to_string()]);
    }

    #[test]
    fn patterns_accessor_preserves_order() {
        let scanner = MultiPatternScanner::new(&["target", "build", "cache"]).unwrap();
        assert_eq!(scanner.patterns(), &["target", "build", "cache"]);
    }
}
