//! `cargo project completions` — shell completion script generator.
//!
//! Writes a completion script for the requested shell to stdout so the caller
//! can redirect it to the correct location:
//!
//! ```text
//! cargo project completions --shell bash   > ~/.bash_completion.d/cargo-project
//! cargo project completions --shell zsh    > ~/.zsh/completions/_cargo-project
//! cargo project completions --shell fish   > ~/.config/fish/completions/cargo-project.fish
//! ```
//!
//! There is no "default verb" for this noun — `--shell` is required and clap
//! will print a clear error message if it is omitted.

use anyhow::Result;
use clap::Args;

use crate::completions::{generate_completions, Shell};

// ─────────────────────────────────────────────────────────────────────────────
// Clap structures
// ─────────────────────────────────────────────────────────────────────────────

/// Generate shell completion scripts.
///
/// # Examples
///
/// ```text
/// # Bash — write to the per-user completion directory
/// cargo project completions --shell bash > ~/.bash_completion.d/cargo-project
///
/// # Zsh — write to a directory in $fpath
/// cargo project completions --shell zsh > ~/.zsh/completions/_cargo-project
///
/// # Fish — fish picks this up automatically
/// cargo project completions --shell fish > ~/.config/fish/completions/cargo-project.fish
///
/// # PowerShell — add to your $PROFILE
/// cargo project completions --shell powershell >> $PROFILE
///
/// # Elvish
/// cargo project completions --shell elvish > ~/.config/elvish/completions/cargo-project.elv
/// ```
#[derive(Debug, Args)]
#[command(
    after_help = "\
EXAMPLES:
    # Install bash completions for the current user
    cargo project completions --shell bash > ~/.bash_completion.d/cargo-project

    # Install zsh completions (directory must be in $fpath)
    cargo project completions --shell zsh > ~/.zsh/completions/_cargo-project

    # Install fish completions
    cargo project completions --shell fish \\
        > ~/.config/fish/completions/cargo-project.fish

    # Use the install script instead of doing it manually
    ./scripts/install-completions.sh

TIP:
    Run `./scripts/install-completions.sh` to auto-detect your shell and
    install completions in the right place without any manual redirects."
)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    ///
    /// Accepted values: bash, zsh, fish, powershell, elvish
    #[arg(long, value_enum)]
    pub shell: Shell,
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Run the `completions` noun: generate and print the completion script.
///
/// Output goes to `stdout` so the caller can redirect as needed.
pub fn run(args: CompletionsArgs) -> Result<()> {
    generate_completions(args.shell, &mut std::io::stdout());
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_bash_completions_succeeds() {
        let args = CompletionsArgs { shell: Shell::Bash };
        // run() writes to stdout — we just confirm it returns Ok.
        // For output content assertions see completions::tests.
        assert!(run(args).is_ok());
    }

    #[test]
    fn run_zsh_completions_succeeds() {
        let args = CompletionsArgs { shell: Shell::Zsh };
        assert!(run(args).is_ok());
    }

    #[test]
    fn run_fish_completions_succeeds() {
        let args = CompletionsArgs { shell: Shell::Fish };
        assert!(run(args).is_ok());
    }

    #[test]
    fn run_powershell_completions_succeeds() {
        let args = CompletionsArgs { shell: Shell::PowerShell };
        assert!(run(args).is_ok());
    }

    #[test]
    fn run_elvish_completions_succeeds() {
        let args = CompletionsArgs { shell: Shell::Elvish };
        assert!(run(args).is_ok());
    }
}
