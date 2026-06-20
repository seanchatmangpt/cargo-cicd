//! Extended help text and after-help examples for each noun.
//!
//! This module does **not** define any runtime logic.  It exists solely to
//! document the `#[command(after_help = "...")]` strings used on each noun's
//! `Args` struct so they can be reviewed, tested, and updated in one place
//! without hunting through every noun file.
//!
//! # Usage
//!
//! Import the constant that matches a noun, then apply it to the clap Args:
//!
//! ```rust,ignore
//! use crate::nouns::help_extra;
//!
//! #[derive(Debug, Args)]
//! #[command(after_help = help_extra::STATUS_SHOW)]
//! pub struct ShowArgs { /* … */ }
//! ```
//!
//! All strings are `&'static str` so they are zero-cost at runtime.

// ─────────────────────────────────────────────────────────────────────────────
// status show
// ─────────────────────────────────────────────────────────────────────────────

/// After-help examples appended to `cargo project status show --help`.
pub const STATUS_SHOW: &str = "\
EXAMPLES:
    # Default human-readable snapshot
    cargo project status

    # Same as above — 'show' is the default verb
    cargo project status show

    # Machine-readable JSON (useful in CI scripts)
    cargo project status show --json

    # Include per-file details for dirty/staged files
    cargo project status show --verbose

    # Combine JSON and verbose for full diagnostic dump
    cargo project status show --json --verbose

ENVIRONMENT:
    RUST_LOG=debug    Enable trace-level adapter diagnostics.
    APP_ENV=ci        Signal a CI environment (disables colour, enables JSON).

NOTES:
    Exit code 0 — workspace is healthy (PASS or WARN).
    Exit code 1 — workspace is unhealthy (FAIL) or an unexpected error occurred.
    Exit code 2 — bad arguments (clap error).";

// ─────────────────────────────────────────────────────────────────────────────
// workspace doctor
// ─────────────────────────────────────────────────────────────────────────────

/// After-help examples appended to `cargo project workspace doctor --help`.
pub const WORKSPACE_DOCTOR: &str = "\
EXAMPLES:
    # Run all workspace diagnostics
    cargo project workspace

    # Same as above — 'doctor' is the default verb
    cargo project workspace doctor

    # Emit diagnostics as JSON
    cargo project workspace doctor --json

WHAT IS CHECKED:
    - Cargo.toml is present and well-formed
    - All workspace members resolve
    - rust-toolchain.toml (if present) matches the active toolchain
    - No duplicate package names across members

ENVIRONMENT:
    RUST_LOG=debug    Show per-check trace output.
    APP_ENV=ci        Disables colour; suitable for log aggregators.";

// ─────────────────────────────────────────────────────────────────────────────
// completions
// ─────────────────────────────────────────────────────────────────────────────

/// After-help examples appended to `cargo project completions --help`.
pub const COMPLETIONS: &str = "\
EXAMPLES:
    # Bash — write to the per-user completion directory
    cargo project completions --shell bash > ~/.bash_completion.d/cargo-project

    # Then reload your shell or source the file:
    source ~/.bash_completion.d/cargo-project

    # Zsh — write to a directory in $fpath
    mkdir -p ~/.zsh/completions
    cargo project completions --shell zsh > ~/.zsh/completions/_cargo-project

    # Ensure ~/.zsh/completions is in $fpath (add to ~/.zshrc if absent):
    # fpath=(~/.zsh/completions $fpath); autoload -Uz compinit; compinit

    # Fish — fish reads this directory automatically
    cargo project completions --shell fish \\
        > ~/.config/fish/completions/cargo-project.fish

    # PowerShell — append to your profile so it loads on start
    cargo project completions --shell powershell >> $PROFILE

    # Elvish
    mkdir -p ~/.config/elvish/completions
    cargo project completions --shell elvish \\
        > ~/.config/elvish/completions/cargo-project.elv

    # Use the installer script to do all of the above automatically
    ./scripts/install-completions.sh [bash|zsh|fish]";

// ─────────────────────────────────────────────────────────────────────────────
// Top-level CLI
// ─────────────────────────────────────────────────────────────────────────────

/// After-help text appended to the root `cargo project --help` output.
pub const ROOT: &str = "\
QUICK START:
    cargo project status           Show workspace health
    cargo project workspace        Run workspace diagnostics
    cargo project completions \\
        --shell bash > ~/.bash_completion.d/cargo-project

ENVIRONMENT:
    RUST_LOG      Log filter for structured diagnostics (e.g. RUST_LOG=debug).
    APP_ENV       Set to 'ci' to disable colour and enable JSON-friendly output.
    NO_COLOR      Set to any value to disable all ANSI colour output (per
                  <https://no-color.org>).

FURTHER READING:
    docs/USAGE.md           Full usage guide with terminal examples.
    docs/man/cargo-project.1  Manual page (view with: man ./docs/man/cargo-project.1)";

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_contains(text: &str, needle: &str) {
        assert!(
            text.contains(needle),
            "expected {text:?} to contain {needle:?}"
        );
    }

    #[test]
    fn status_show_help_contains_examples_header() {
        assert_contains(STATUS_SHOW, "EXAMPLES:");
    }

    #[test]
    fn status_show_help_contains_json_flag() {
        assert_contains(STATUS_SHOW, "--json");
    }

    #[test]
    fn status_show_help_mentions_exit_codes() {
        assert_contains(STATUS_SHOW, "Exit code 0");
        assert_contains(STATUS_SHOW, "Exit code 1");
    }

    #[test]
    fn workspace_doctor_help_contains_examples_header() {
        assert_contains(WORKSPACE_DOCTOR, "EXAMPLES:");
    }

    #[test]
    fn workspace_doctor_help_mentions_what_is_checked() {
        assert_contains(WORKSPACE_DOCTOR, "WHAT IS CHECKED:");
    }

    #[test]
    fn completions_help_covers_all_shells() {
        for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
            assert_contains(COMPLETIONS, shell);
        }
    }

    #[test]
    fn root_help_mentions_all_nouns() {
        assert_contains(ROOT, "status");
        assert_contains(ROOT, "workspace");
        assert_contains(ROOT, "completions");
    }

    #[test]
    fn root_help_documents_rust_log_env_var() {
        assert_contains(ROOT, "RUST_LOG");
    }

    #[test]
    fn root_help_documents_no_color() {
        assert_contains(ROOT, "NO_COLOR");
    }
}
