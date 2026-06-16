//! Governance pattern validator for paths and filenames.
//!
//! [`GovernancePatternValidator`] provides a mechanism to scan paths and filenames
//! against a set of governance patterns, useful for:
//!
//! * **Deny-list scanning**: Check changed files against a list of restricted paths
//!   (e.g., `target/`, `node_modules/`, `.git/`) to enforce governance policies.
//! * **Artifact compliance**: Verify that build or deployment artifacts conform to
//!   naming or location patterns (e.g., ensuring binaries are only in specific directories).
//! * **Policy enforcement**: Batch-validate collections of paths to identify which
//!   ones violate governance rules.
//!
//! When the `advanced` feature is enabled, the validator compiles patterns into
//! a high-efficiency Aho-Corasick automaton. Without the feature, the validator
//! is a no-op stub that always returns empty results.

#[cfg(feature = "advanced")]
use crate::advanced::pattern::MultiPatternScanner;

/// A validator for governance patterns applied to paths and filenames.
///
/// Holds an optional [`MultiPatternScanner`] (when the `advanced` feature is enabled)
/// and validates paths against a set of governance patterns. When `advanced` is off,
/// the validator is a stub.
pub struct GovernancePatternValidator {
    #[cfg(feature = "advanced")]
    scanner: MultiPatternScanner,
}

impl GovernancePatternValidator {
    /// Create a new validator from a slice of pattern strings.
    ///
    /// Compiles the patterns into an Aho-Corasick automaton for efficient matching.
    /// This succeeds only if the `advanced` feature is enabled; otherwise, an error
    /// is always returned.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The `advanced` feature is not enabled.
    /// - The pattern set is empty or cannot be compiled.
    pub fn new(patterns: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "advanced")]
        {
            let scanner = MultiPatternScanner::new(patterns)?;
            Ok(Self { scanner })
        }

        #[cfg(not(feature = "advanced"))]
        {
            Err("GovernancePatternValidator requires the `advanced` feature".into())
        }
    }

    /// Validate a collection of paths against the compiled governance patterns.
    ///
    /// Returns a vector of tuples, one per path that matches any pattern.
    /// Each tuple contains:
    /// - The original path string (as `String`)
    /// - The vector of matched patterns (deduplicated, in first-seen order)
    ///
    /// When the `advanced` feature is disabled, returns an empty vector.
    pub fn validate_paths(&self, paths: &[impl AsRef<str>]) -> Vec<(String, Vec<String>)> {
        #[cfg(feature = "advanced")]
        {
            paths
                .iter()
                .filter_map(|p| {
                    let path_str = p.as_ref();
                    let matched = self.scanner.matched_patterns(path_str);
                    if matched.is_empty() {
                        None
                    } else {
                        Some((path_str.to_string(), matched))
                    }
                })
                .collect()
        }

        #[cfg(not(feature = "advanced"))]
        {
            let _ = paths; // silence unused warning
            Vec::new()
        }
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;

    #[test]
    fn validator_checks_path_against_patterns() {
        let validator = GovernancePatternValidator::new(&["target", "node_modules"]).unwrap();
        let results = validator.validate_paths(&["src/main.rs", "target/debug/app", "lib/code.rs"]);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "target/debug/app");
        assert_eq!(results[0].1, vec!["target".to_string()]);
    }

    #[test]
    fn validator_checks_batch_paths_with_dedup() {
        let validator = GovernancePatternValidator::new(&["target", "cache", "build"]).unwrap();
        let paths = &[
            "src/lib.rs",
            "target/debug/bin",
            "target/release/lib",
            "cache/artifacts",
            "docs/api.md",
            "build/output",
        ];

        let results = validator.validate_paths(paths);

        // Should have 4 results: target/debug/bin, target/release/lib, cache/artifacts, build/output
        assert_eq!(results.len(), 4);

        // Check each result has the correct path and matched patterns
        assert_eq!(results[0].0, "target/debug/bin");
        assert_eq!(results[0].1, vec!["target".to_string()]);

        assert_eq!(results[1].0, "target/release/lib");
        assert_eq!(results[1].1, vec!["target".to_string()]);

        assert_eq!(results[2].0, "cache/artifacts");
        assert_eq!(results[2].1, vec!["cache".to_string()]);

        assert_eq!(results[3].0, "build/output");
        assert_eq!(results[3].1, vec!["build".to_string()]);
    }
}
