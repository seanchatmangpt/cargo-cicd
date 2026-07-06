use anyhow::Result;

mod adapters;
#[cfg(feature = "advanced")]
mod advanced;
mod autonomic;
pub mod barrier;
mod certification;
mod cicd_toml;
pub mod code_provenance;
mod engine;
pub mod evidence;
pub mod evidence_helpers;
mod integrations;
pub mod legacy_nouns;
pub mod nouns;
pub mod ocel;
mod policies;
pub mod session;
mod state;
mod ui;

/// When cargo invokes this binary as a subcommand (`cargo cicd ...`), it
/// prepends the subcommand name ("cicd") as `argv[1]` before forwarding to
/// `cargo-cicd`, per cargo's subcommand convention (look up `cargo-<name>`
/// on PATH, then call it with `<name>` as the first argument). Strip that
/// leading token here so `cargo cicd standing --help` and
/// `cargo-cicd standing --help` reach the exact same clap parse.
fn strip_cargo_subcommand_token(mut args: Vec<String>) -> Vec<String> {
    if args.len() > 1 && args[1] == "cicd" {
        args.remove(1);
    }
    args
}

fn main() -> Result<()> {
    // Report this binary's own package identity (name + version) instead of
    // the clap-noun-verb framework's compiled-in version. Without this call
    // `--version` falls back to the framework's own `CARGO_PKG_VERSION`.
    clap_noun_verb::cli::CommandRegistry::set_app_metadata(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    let args = strip_cargo_subcommand_token(std::env::args().collect());

    let registry = clap_noun_verb::cli::CommandRegistry::get();
    let registry = registry
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock command registry: {}", e))?;
    registry.run(args).map_err(|e| anyhow::anyhow!("{}", e))
}

#[cfg(test)]
mod dispatch_tests {
    use super::strip_cargo_subcommand_token;

    fn v(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn direct_binary_argv_shape_is_untouched() {
        let args = v(&["cargo-cicd", "standing", "--help"]);
        assert_eq!(strip_cargo_subcommand_token(args.clone()), args);
    }

    #[test]
    fn cargo_prepended_argv_shape_strips_leading_cicd_token() {
        let args = v(&["cargo-cicd", "cicd", "standing", "--help"]);
        let stripped = strip_cargo_subcommand_token(args);
        assert_eq!(stripped, v(&["cargo-cicd", "standing", "--help"]));
    }

    #[test]
    fn bare_cicd_token_without_leading_binary_name_is_left_alone() {
        // Sanity: only argv[1] is ever considered, never argv[0].
        let args = v(&["cicd", "standing"]);
        assert_eq!(strip_cargo_subcommand_token(args.clone()), args);
    }

    #[test]
    fn help_only_argv_is_unaffected() {
        let args = v(&["cargo-cicd", "--help"]);
        assert_eq!(strip_cargo_subcommand_token(args.clone()), args);
    }

    #[test]
    fn unrelated_second_token_is_left_alone() {
        let args = v(&["cargo-cicd", "status", "show"]);
        assert_eq!(strip_cargo_subcommand_token(args.clone()), args);
    }
}
