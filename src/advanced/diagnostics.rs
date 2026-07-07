//! Rich rendered diagnostics for policy violations in `cicd.toml`.
//!
//! Built on [`miette`] + [`thiserror`]. The [`EngineDiagnostic`] enum models the
//! recoverable policy/state violations the engine can surface, each annotated
//! with a stable diagnostic code, severity, and remediation help. Variants that
//! reference a concrete location in `cicd.toml` carry a source span so the
//! offending region can be rendered with a caret underline.

use miette::{
    Diagnostic, GraphicalReportHandler, GraphicalTheme, NamedSource, Severity, SourceSpan,
};
use thiserror::Error;

/// A diagnostic describing a policy or engine-state violation discovered while
/// reading `cicd.toml` or evaluating workspace state.
// This whole module's public API is exercised by
// `examples/03_max_pipeline.rs` (tutorial anchor for
// docs/tutorials/03-full-pipeline.md), a separate cargo target whose usage
// doesn't suppress `cargo build`'s dead_code lint on the library crate.
#[allow(dead_code)]
#[derive(Debug, Error, Diagnostic)]
pub enum EngineDiagnostic {
    /// The pinned toolchain in `cicd.toml` does not match the active toolchain.
    #[error("toolchain mismatch: workspace expects `{expected}`, found `{found}`")]
    #[diagnostic(
        code(cargo_cicd::toolchain_mismatch),
        help("update the active toolchain or adjust `[state].toolchain` in cicd.toml"),
        severity(Error)
    )]
    ToolchainMismatch {
        /// Toolchain declared in `cicd.toml`.
        expected: String,
        /// Toolchain actually detected on the host.
        found: String,
        /// The full `cicd.toml` source under inspection.
        #[source_code]
        src: NamedSource<String>,
        /// Region of the source pinning the expected toolchain.
        #[label("declared here")]
        span: SourceSpan,
    },

    /// A git phase is dirty while the policy requires a clean working tree.
    #[error("git phase `{phase}` is dirty: {dirty_paths} uncommitted path(s)")]
    #[diagnostic(
        code(cargo_cicd::dirty_git_phase),
        help("commit or stash pending changes before advancing the git phase"),
        severity(Error)
    )]
    DirtyGitPhase {
        /// Name of the offending git phase.
        phase: String,
        /// Count of uncommitted paths.
        dirty_paths: usize,
    },

    /// The `target/` directory has grown past the configured pressure budget.
    #[error("target pressure: {size_mb} MiB exceeds budget of {budget_mb} MiB")]
    #[diagnostic(
        code(cargo_cicd::target_pressure),
        help("run a cleanup pass or raise `[target].budget_mb` in cicd.toml"),
        severity(Warning)
    )]
    TargetPressure {
        /// Current measured size in MiB.
        size_mb: u64,
        /// Configured budget in MiB.
        budget_mb: u64,
    },
}

/// Locate `needle` within `source` and build a [`EngineDiagnostic::ToolchainMismatch`]
/// whose label points at that substring.
///
/// `file_name` is used only for display in rendered output. If `needle` is not
/// found, the span collapses to the start of the file so rendering still succeeds.
#[allow(dead_code)] // see examples/03_max_pipeline.rs note above
pub fn toolchain_mismatch_at(
    file_name: impl AsRef<str>,
    source: impl Into<String>,
    needle: &str,
    expected: impl Into<String>,
    found: impl Into<String>,
) -> EngineDiagnostic {
    let source = source.into();
    let span = locate_span(&source, needle);
    EngineDiagnostic::ToolchainMismatch {
        expected: expected.into(),
        found: found.into(),
        src: NamedSource::new(file_name, source),
        span,
    }
}

/// Compute the byte-offset span of `needle` inside `source`. Falls back to a
/// zero-length span at offset 0 when the substring is absent.
#[allow(dead_code)] // helper for toolchain_mismatch_at, see note above
fn locate_span(source: &str, needle: &str) -> SourceSpan {
    match source.find(needle) {
        Some(offset) => SourceSpan::from((offset, needle.len())),
        None => SourceSpan::from((0usize, 0usize)),
    }
}

/// Render a diagnostic to a `String` using a deterministic (no-color, unicode)
/// graphical report handler, suitable for logs, receipts, and stable tests.
#[allow(dead_code)] // see examples/03_max_pipeline.rs note above
pub fn render(diag: &EngineDiagnostic) -> String {
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
    let mut out = String::new();
    // Rendering into a pre-allocated String is infallible for the graphical
    // handler; on the off chance it errors, surface a readable fallback.
    if handler.render_report(&mut out, diag).is_err() {
        out.push_str(&diag.to_string());
    }
    out
}

/// Convenience: the severity advertised by a diagnostic, defaulting to `Error`
/// when a variant declares none.
#[allow(dead_code)] // see examples/03_max_pipeline.rs note above
pub fn severity_of(diag: &EngineDiagnostic) -> Severity {
    diag.severity().unwrap_or(Severity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = "\
[workspace]
name = \"cargo-cicd\"

[state]
toolchain = \"1.79.0\"
";

    fn sample_toolchain_diag() -> EngineDiagnostic {
        toolchain_mismatch_at("cicd.toml", SAMPLE_TOML, "1.79.0", "1.79.0", "1.81.0")
    }

    #[test]
    fn code_is_present_and_correct() {
        let diag = sample_toolchain_diag();
        let code = diag.code().expect("diagnostic must expose a code");
        assert_eq!(code.to_string(), "cargo_cicd::toolchain_mismatch");

        let pressure = EngineDiagnostic::TargetPressure {
            size_mb: 4096,
            budget_mb: 2048,
        };
        assert_eq!(
            pressure.code().expect("code").to_string(),
            "cargo_cicd::target_pressure"
        );

        let dirty = EngineDiagnostic::DirtyGitPhase {
            phase: "publish".into(),
            dirty_paths: 3,
        };
        assert_eq!(
            dirty.code().expect("code").to_string(),
            "cargo_cicd::dirty_git_phase"
        );
    }

    #[test]
    fn help_text_is_present() {
        let diag = sample_toolchain_diag();
        let help = diag.help().expect("help text must be present").to_string();
        assert!(
            help.contains("toolchain") && help.contains("cicd.toml"),
            "unexpected help text: {help}"
        );

        let pressure = EngineDiagnostic::TargetPressure {
            size_mb: 4096,
            budget_mb: 2048,
        };
        assert!(pressure
            .help()
            .expect("help")
            .to_string()
            .contains("budget_mb"));
    }

    #[test]
    fn render_contains_code_and_labeled_snippet() {
        let diag = sample_toolchain_diag();
        let rendered = render(&diag);

        // The stable code must appear in the rendered report.
        assert!(
            rendered.contains("cargo_cicd::toolchain_mismatch"),
            "render missing code:\n{rendered}"
        );
        // The labeled snippet from cicd.toml must be rendered.
        assert!(
            rendered.contains("1.79.0"),
            "render missing labeled snippet:\n{rendered}"
        );
        // The label text accompanies the underlined region.
        assert!(
            rendered.contains("declared here"),
            "render missing label text:\n{rendered}"
        );
    }

    #[test]
    fn severity_reflects_variant() {
        assert_eq!(severity_of(&sample_toolchain_diag()), Severity::Error);
        assert_eq!(
            severity_of(&EngineDiagnostic::TargetPressure {
                size_mb: 1,
                budget_mb: 0
            }),
            Severity::Warning
        );
    }

    #[test]
    fn missing_needle_yields_safe_span() {
        let diag = toolchain_mismatch_at("cicd.toml", SAMPLE_TOML, "no-such-substring", "a", "b");
        // Rendering must still succeed and include the code.
        let rendered = render(&diag);
        assert!(rendered.contains("cargo_cicd::toolchain_mismatch"));
    }
}
