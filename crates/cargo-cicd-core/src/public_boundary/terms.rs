//! Forbidden term list — private/internal terms that must not appear in public surfaces.

/// Terms that must not appear in public-facing documentation or help text.
pub const FORBIDDEN_TERMS: &[&str] = &[
    // Internal code names and project handles.
    "ostar",
    "o*",
    "wasm4pm-internal",
    "cicd-internal",
    // Internal infrastructure terms.
    "staging-only",
    "internal-only",
    "not-for-release",
    // Draft / placeholder markers.
    "TODO(internal)",
    "FIXME(internal)",
    "HACK(internal)",
    // Credential / secret patterns caught by term scan.
    "SECRET_KEY",
    "PRIVATE_TOKEN",
    "API_SECRET",
];
