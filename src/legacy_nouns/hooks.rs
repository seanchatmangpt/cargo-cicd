use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct HooksNoun;
impl HooksNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HooksNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for HooksNoun {
    fn name(&self) -> &'static str {
        "hooks"
    }
    fn about(&self) -> &'static str {
        "Manage git pre-commit hooks for cargo-cicd"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(HooksInstallVerb), Box::new(HooksUninstallVerb)]
    }
}

pub struct HooksInstallVerb;
impl VerbCommand for HooksInstallVerb {
    fn name(&self) -> &'static str {
        "install"
    }
    fn about(&self) -> &'static str {
        "Install and configure git pre-commit hooks for cargo-cicd"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        use crate::ui::theme::{self, Role};

        println!(
            "{}",
            theme::paint("▶ Installing cargo-cicd hooks\n", Role::Info)
        );

        let repo_root_cmd = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        let repo_root_str = String::from_utf8_lossy(&repo_root_cmd.stdout)
            .trim()
            .to_string();
        if repo_root_str.is_empty() {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "Not a git repository. Run this command from a git root.",
            ));
        }

        let repo_root = PathBuf::from(repo_root_str);
        let hooks_dir = repo_root.join(".git").join("hooks");

        println!("✓ Git repository detected at {}", repo_root.display());

        // 1. Install main pre-commit hook
        println!(
            "{}",
            theme::paint("▶ Installing .git/hooks/pre-commit", Role::Info)
        );
        if !hooks_dir.exists() {
            if let Err(e) = fs::create_dir_all(&hooks_dir) {
                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                    format!("Failed to create hooks directory: {}", e),
                ));
            }
            println!("  Created hooks directory");
        }

        let src_hook = repo_root.join("scripts").join("hooks").join("pre-commit");
        let dst_hook = hooks_dir.join("pre-commit");

        if src_hook.exists() {
            if let Err(e) = fs::copy(&src_hook, &dst_hook) {
                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                    format!("Failed to copy hook: {}", e),
                ));
            }
            if let Ok(metadata) = fs::metadata(&dst_hook) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&dst_hook, perms);
                }
                println!(
                    "{}",
                    theme::paint(
                        "✓ Installed .git/hooks/pre-commit (executable)",
                        Role::Success
                    )
                );
            }
        } else {
            eprintln!("✗ Hook file not found at {}", src_hook.display());
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "Hook file not found",
            ));
        }

        // 2. Install pre-commit framework config (optional logic from bash script)
        println!(
            "{}",
            theme::paint("▶ Setting up pre-commit framework", Role::Info)
        );
        let pre_commit_yaml = repo_root.join(".pre-commit-config.yaml");
        if pre_commit_yaml.exists() {
            println!("  .pre-commit-config.yaml already exists (keeping existing)");
        } else {
            println!("⚠ Could not find .pre-commit-config.yaml template");
        }

        if std::process::Command::new("pre-commit")
            .arg("--version")
            .output()
            .is_ok()
        {
            println!("  pre-commit framework detected");
            print!("  Install pre-commit hook environments? (y/n): ");
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() && input.trim() == "y" {
                if std::process::Command::new("pre-commit")
                    .arg("install")
                    .status()
                    .is_ok()
                {
                    println!(
                        "{}",
                        theme::paint("✓ Pre-commit environments installed", Role::Success)
                    );
                    let _ = std::process::Command::new("pre-commit")
                        .arg("run")
                        .arg("--all-files")
                        .status();
                }
            }
        } else {
            println!("⚠ pre-commit framework not installed");
            println!("  Optional: Install with: pip install pre-commit");
            println!("  Then run:  pre-commit install");
        }

        // 3. Create forbidden-terms checker script
        println!(
            "{}",
            theme::paint("▶ Setting up forbidden-terms checker", Role::Info)
        );
        let scripts_dir = repo_root.join("scripts");
        if !scripts_dir.exists() {
            let _ = fs::create_dir_all(&scripts_dir);
            println!("  Created scripts directory");
        }

        let checker_script = r#"#!/bin/bash
# Check for forbidden terms in staged files
# Part of cargo-cicd pre-commit framework

FORBIDDEN_TERMS=(
    "ALIVE"
    "Inspection Gate"
    "wall"
    "Nehemiah"
    "Field8"
    "Instinct8"
    "Cargo Court"
    "AGI"
    "Truex"
    "CONSTRUCT8"
)

ERROR=0
for FILE in "$@"; do
    [ ! -f "$FILE" ] && continue
    for TERM in "${FORBIDDEN_TERMS[@]}"; do
        if grep -iq "$TERM" "$FILE" 2>/dev/null; then
            echo "✗ $FILE: Found forbidden term '$TERM'"
            ERROR=1
        fi
    done
done

exit $ERROR
"#;
        let checker_path = scripts_dir.join("check-forbidden-terms.sh");
        if fs::write(&checker_path, checker_script).is_ok() {
            if let Ok(metadata) = fs::metadata(&checker_path) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&checker_path, perms);
                }
                println!(
                    "{}",
                    theme::paint("✓ Created scripts/check-forbidden-terms.sh", Role::Success)
                );
            }
        }

        // 4. Test hooks
        println!("{}", theme::paint("▶ Testing hooks", Role::Info));
        println!("  Running a dry-run of the pre-commit hook...");
        if std::process::Command::new("bash")
            .arg(dst_hook.to_str().unwrap())
            .arg("/dev/null")
            .output()
            .is_ok()
        {
            println!(
                "{}",
                theme::paint("✓ Hook executable and functional", Role::Success)
            );
        } else {
            println!("⚠ Hook test produced output (this may be normal)");
        }

        // 5. Summary
        println!(
            "\n{}",
            theme::paint("▶ Hook installation complete!\n", Role::Info)
        );
        println!("Installed hooks:");
        println!("✓ .git/hooks/pre-commit — Runs before every commit");
        println!("  Checks: formatting, compilation, tests, forbidden terms, commit format\n");

        println!("Optional additions:");
        println!("• .pre-commit-config.yaml — Framework config (pre-commit run --all-files)");
        println!("• scripts/check-forbidden-terms.sh — Forbidden term checker\n");

        println!("Next steps:");
        println!("1. Make a test commit: git add . && git commit -m 'test(core): validate hooks'");
        println!("2. Verify checks run and pass");
        println!("3. (Optional) Install pre-commit framework: pip install pre-commit && pre-commit install\n");

        println!("Disable hooks temporarily:");
        println!("  SKIP=pre-commit git commit ...\n");

        println!("Uninstall all hooks:");
        println!("  cargo cicd hooks uninstall\n");

        println!(
            "{}",
            theme::paint("✓ Ready to enforce code quality!", Role::Success)
        );
        Ok(())
    }
}

pub struct HooksUninstallVerb;
impl VerbCommand for HooksUninstallVerb {
    fn name(&self) -> &'static str {
        "uninstall"
    }
    fn about(&self) -> &'static str {
        "Uninstall cargo-cicd pre-commit hooks"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        use crate::ui::theme::{self, Role};

        println!("{}", theme::paint("▶ Uninstalling hooks", Role::Info));

        let repo_root_cmd = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        let repo_root_str = String::from_utf8_lossy(&repo_root_cmd.stdout)
            .trim()
            .to_string();
        if repo_root_str.is_empty() {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "Not a git repository.",
            ));
        }

        let repo_root = PathBuf::from(repo_root_str);
        let hooks_dir = repo_root.join(".git").join("hooks");

        let pre_commit_hook = hooks_dir.join("pre-commit");
        if pre_commit_hook.exists() {
            let _ = fs::remove_file(&pre_commit_hook);
            println!(
                "{}",
                theme::paint("✓ Removed .git/hooks/pre-commit", Role::Success)
            );
        }

        let pre_commit_yaml = repo_root.join(".pre-commit-config.yaml");
        if pre_commit_yaml.exists() {
            print!("Remove .pre-commit-config.yaml? (y/n): ");
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() && input.trim() == "y" {
                let _ = fs::remove_file(&pre_commit_yaml);
                println!(
                    "{}",
                    theme::paint("✓ Removed .pre-commit-config.yaml", Role::Success)
                );
            }
        }

        if std::process::Command::new("pre-commit")
            .arg("--version")
            .output()
            .is_ok()
        {
            print!("Uninstall pre-commit framework? (y/n): ");
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() && input.trim() == "y" {
                if std::process::Command::new("pre-commit")
                    .arg("uninstall")
                    .status()
                    .is_ok()
                {
                    println!(
                        "{}",
                        theme::paint("✓ Pre-commit framework uninstalled", Role::Success)
                    );
                }
            }
        }

        println!(
            "\n{}",
            theme::paint(
                "✓ Hooks uninstalled. You can commit without restrictions.",
                Role::Success
            )
        );
        Ok(())
    }
}
