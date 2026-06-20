//! Shell completion generator for `cargo-project`.
//!
//! This module provides:
//! - A [`Shell`] enum covering all shells supported by `clap_complete`.
//! - [`Shell::file_name`] — returns the conventional completion filename for a
//!   given binary name (e.g. `_cargo-project` for Zsh, `cargo-project.fish` for Fish).
//! - [`generate_completions`] — builds the [`Cli`] command tree via
//!   [`clap::CommandFactory`] and delegates to [`clap_complete::generate`].
//!
//! # CLI surface
//!
//! ```text
//! cargo project completions --shell bash   > ~/.bash_completion.d/cargo-project
//! cargo project completions --shell zsh    > ~/.zsh/completions/_cargo-project
//! cargo project completions --shell fish   > ~/.config/fish/completions/cargo-project.fish
//! cargo project completions --shell powershell
//! cargo project completions --shell elvish
//! ```
//!
//! The completions subcommand intentionally writes to stdout so the caller can
//! redirect to any destination.  See `scripts/install-completions.sh` for an
//! automated installer that chooses the right path per shell.

use std::io::Write;

use clap::ValueEnum;
use clap_complete::generate;

// Re-exported so callers can use this crate's Shell without importing clap_complete.
pub use clap_complete::Shell as ClapShell;

// ─────────────────────────────────────────────────────────────────────────────
// Shell enum
// ─────────────────────────────────────────────────────────────────────────────

/// Shell variants supported by the completion generator.
///
/// The `ValueEnum` derive lets clap parse `--shell bash` directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Bourne Again SHell (bash ≥ 3.2).
    Bash,
    /// Z Shell (zsh ≥ 5.0).
    Zsh,
    /// Friendly Interactive Shell (fish ≥ 3.0).
    Fish,
    /// PowerShell (pwsh ≥ 7 or Windows PowerShell ≥ 5.1).
    PowerShell,
    /// Elvish shell (≥ 0.17).
    Elvish,
}

impl Shell {
    /// Returns the conventional file name for completion scripts targeting
    /// `bin_name`.
    ///
    /// | Shell        | Convention                              | Example                            |
    /// |---|---|---|
    /// | Bash         | `<bin_name>`                            | `cargo-project`                    |
    /// | Zsh          | `_<bin_name>`                           | `_cargo-project`                   |
    /// | Fish         | `<bin_name>.fish`                       | `cargo-project.fish`               |
    /// | PowerShell   | `_<bin_name>.ps1`                       | `_cargo-project.ps1`               |
    /// | Elvish       | `<bin_name>.elv`                        | `cargo-project.elv`                |
    pub fn file_name(&self, bin_name: &str) -> String {
        match self {
            Shell::Bash => bin_name.to_owned(),
            Shell::Zsh => format!("_{bin_name}"),
            Shell::Fish => format!("{bin_name}.fish"),
            Shell::PowerShell => format!("_{bin_name}.ps1"),
            Shell::Elvish => format!("{bin_name}.elv"),
        }
    }

    /// Maps this crate's [`Shell`] to the `clap_complete::Shell` variant.
    pub fn to_clap_shell(self) -> ClapShell {
        match self {
            Shell::Bash => ClapShell::Bash,
            Shell::Zsh => ClapShell::Zsh,
            Shell::Fish => ClapShell::Fish,
            Shell::PowerShell => ClapShell::PowerShell,
            Shell::Elvish => ClapShell::Elvish,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Generator
// ─────────────────────────────────────────────────────────────────────────────

/// Generate shell completions and write them to `out`.
///
/// # Arguments
///
/// * `shell`    — Target shell variant.
/// * `out`      — Any [`Write`] sink; in production this is `std::io::stdout()`.
///
/// # Example
///
/// ```no_run
/// use project::completions::{Shell, generate_completions};
///
/// generate_completions(Shell::Bash, &mut std::io::stdout());
/// ```
pub fn generate_completions(shell: Shell, out: &mut dyn Write) {
    use clap::CommandFactory;
    // Import the top-level Cli struct that owns the full command tree.
    // It must implement clap::CommandFactory (automatically derived by #[derive(Parser)]).
    use crate::Cli;

    let mut cmd = Cli::command();
    generate(shell.to_clap_shell(), &mut cmd, "cargo-project", out);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_bash() {
        assert_eq!(Shell::Bash.file_name("cargo-project"), "cargo-project");
    }

    #[test]
    fn file_name_zsh() {
        assert_eq!(Shell::Zsh.file_name("cargo-project"), "_cargo-project");
    }

    #[test]
    fn file_name_fish() {
        assert_eq!(Shell::Fish.file_name("cargo-project"), "cargo-project.fish");
    }

    #[test]
    fn file_name_powershell() {
        assert_eq!(
            Shell::PowerShell.file_name("cargo-project"),
            "_cargo-project.ps1"
        );
    }

    #[test]
    fn file_name_elvish() {
        assert_eq!(
            Shell::Elvish.file_name("cargo-project"),
            "cargo-project.elv"
        );
    }

    #[test]
    fn generate_bash_completions_is_nonempty() {
        let mut buf = Vec::new();
        generate_completions(Shell::Bash, &mut buf);
        assert!(!buf.is_empty(), "bash completions must not be empty");
    }

    #[test]
    fn generate_zsh_completions_is_nonempty() {
        let mut buf = Vec::new();
        generate_completions(Shell::Zsh, &mut buf);
        assert!(!buf.is_empty(), "zsh completions must not be empty");
    }

    #[test]
    fn generate_fish_completions_is_nonempty() {
        let mut buf = Vec::new();
        generate_completions(Shell::Fish, &mut buf);
        assert!(!buf.is_empty(), "fish completions must not be empty");
    }

    #[test]
    fn shell_to_clap_shell_round_trip() {
        use clap_complete::Shell as CS;
        assert_eq!(Shell::Bash.to_clap_shell(), CS::Bash);
        assert_eq!(Shell::Zsh.to_clap_shell(), CS::Zsh);
        assert_eq!(Shell::Fish.to_clap_shell(), CS::Fish);
        assert_eq!(Shell::PowerShell.to_clap_shell(), CS::PowerShell);
        assert_eq!(Shell::Elvish.to_clap_shell(), CS::Elvish);
    }
}
