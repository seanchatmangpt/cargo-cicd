use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policies::{run_all_policies, GitState, PolicyVerdict, WorkspaceInfo};
use crate::evidence::ProcessEvent;
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::{panel, symbols};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct WorkspaceNoun;
impl WorkspaceNoun {
    pub fn new() -> Self {
        Self
    }
    pub fn run_doctor() -> anyhow::Result<()> {
        WorkspaceDoctorVerb
            .run(&VerbArgs::new(clap::ArgMatches::default()))
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
impl Default for WorkspaceNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for WorkspaceNoun {
    fn name(&self) -> &'static str {
        "workspace"
    }
    fn about(&self) -> &'static str {
        "Workspace diagnostics"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(WorkspaceDoctorVerb)]
    }
}

/// Render one diagnostic check row as `<badge> <label>`. The verdict tag carries
/// the semantic color (auto-dropped off-TTY) while the `label` stays a
/// contiguous plain-text token — the public-boundary tests grep these labels
/// (e.g. "Cargo.toml"), so the badge is placed before, never inside, the label.
fn check_row(tag: &str, label: &str) -> String {
    format!(
        "{} {}",
        badge::tag(Verdict::from_tag(tag)),
        theme::paint(label, Role::Value)
    )
}

pub struct WorkspaceDoctorVerb;
impl VerbCommand for WorkspaceDoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn about(&self) -> &'static str {
        "Diagnose workspace health"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        println!("{}", panel::header("workspace doctor"));

        let has_cargo = std::path::Path::new("Cargo.toml").exists();
        println!(
            "{}",
            check_row(if has_cargo { "OK" } else { "FAIL" }, "Cargo.toml")
        );

        let toolchain = ToolchainDetector::active_toolchain();
        println!(
            "{}",
            check_row("OK", &format!("toolchain: {}", toolchain))
        );

        let has_toolchain_file = std::path::Path::new("rust-toolchain.toml").exists()
            || std::path::Path::new("rust-toolchain").exists();
        println!(
            "{}",
            check_row(
                if has_toolchain_file { "OK" } else { "WARN" },
                "rust-toolchain file"
            )
        );

        let has_git = std::path::Path::new(".git").exists();
        println!(
            "{}",
            check_row(if has_git { "OK" } else { "FAIL" }, "git repository")
        );

        let has_cicd = std::path::Path::new("cicd.toml").exists();
        println!(
            "{}",
            check_row(
                if has_cicd { "OK" } else { "WARN" },
                "cicd.toml (run 'cargo cicd publish' to generate)"
            )
        );

        // ── autonomic policy checks ──────────────────────────────────────────
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let pinned_toolchain = read_pinned_toolchain();
        let git_dirty = GitStatusAdapter::query()
            .map(|r| r.dirty_files.len())
            .unwrap_or(0);

        let workspace_info = WorkspaceInfo {
            target_gb,
            max_gb: 20.0,
            active_toolchain: toolchain.clone(),
            pinned_toolchain,
            changed_trybuild_fixtures: 0,
        };
        let git_state = GitState {
            dirty_count: git_dirty,
        };

        let results = run_all_policies(&workspace_info, &git_state);

        if !results.is_empty() {
            println!();
            println!("{}", panel::header("autonomic policy results"));
            for r in &results {
                let tag = match r.verdict {
                    PolicyVerdict::Pass => "PASS",
                    PolicyVerdict::Warn => "WARN",
                    PolicyVerdict::Suggest => "SUGGEST",
                };
                if r.recommendation.is_empty() {
                    println!("{}", check_row(tag, &r.name));
                } else {
                    println!(
                        "{} {}",
                        check_row(tag, &r.name),
                        theme::paint(
                            &format!("{} {}", symbols::arrow(), r.recommendation),
                            Role::Muted
                        )
                    );
                }
            }
        }

        println!();
        let verdict = if !has_cargo || !has_git {
            println!(
                "{} {}",
                badge::tag(Verdict::Fail),
                theme::paint("workspace has critical issues", Role::Danger)
            );
            "FAIL"
        } else {
            println!(
                "{} {}",
                badge::tag(Verdict::Pass),
                theme::paint("workspace is healthy", Role::Success)
            );
            "PASS"
        };

        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let mut event = ProcessEvent::new("workspace:doctor", verdict);
        event.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[event], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

/// Read the channel from `rust-toolchain.toml` if it exists.
fn read_pinned_toolchain() -> Option<String> {
    if std::path::Path::new("rust-toolchain.toml").exists() {
        std::fs::read_to_string("rust-toolchain.toml")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.contains("channel"))
                    .and_then(|l| l.split('"').nth(1))
                    .map(|s| s.to_string())
            })
    } else {
        None
    }
}
