use crate::adapters::{
    ChangedFileDetector, GitStatusAdapter, TargetScannerAdapter, ToolchainDetector,
};
use crate::cicd_toml::CicdToml;
use crate::evidence::ProcessEvent;
use crate::legacy_nouns::evidence_helpers::init_evidence;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};
use std::path::Path;

pub struct PublishNoun;
impl PublishNoun {
    pub fn new() -> Self {
        Self
    }
    pub fn run_direct() -> anyhow::Result<()> {
        let matches = clap::Command::new("publish").get_matches_from(vec!["publish"]);
        let args = clap_noun_verb::VerbArgs::new(matches);
        PublishRunVerb
            .run(&args)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
impl Default for PublishNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for PublishNoun {
    fn name(&self) -> &'static str {
        "publish"
    }
    fn about(&self) -> &'static str {
        "Publish cicd.toml with current workspace state"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(PublishRunVerb),
            Box::new(PublishCheckVerb),
            Box::new(PublishValidateVerb),
        ]
    }
}

/// ── Doc-drift gate ──────────────────────────────────────────────────────────
/// Gate law: if any doc file (`*.md`, `*.txt`, `docs/**`) changed since the last
/// release tag, the workspace's `[docs] verify_cmd` (cicd.toml) must pass before
/// publish proceeds. No doc changes → gate skipped. Missing config or failing
/// command → typed refusal DOC_DRIFT_UNVERIFIED.
fn last_release_tag_in(dir: &Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn docs_changed_since_in(dir: &Path, tag: &str) -> Vec<String> {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["diff", "--name-only", &format!("{tag}..HEAD"), "--"])
        .output();
    String::from_utf8_lossy(&out.map(|o| o.stdout).unwrap_or_default())
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| {
            l.starts_with("docs/")
                || l.ends_with(".md")
                || l.ends_with(".txt")
        })
        .map(str::to_string)
        .collect()
}

/// `[docs] verify_cmd` from cicd.toml (text scan: `verify_cmd = "..."` inside the
/// `[docs]` section).
fn docs_verify_cmd_in(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("cicd.toml")).ok()?;
    let mut in_docs = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_docs = t == "[docs]";
            continue;
        }
        if in_docs {
            if let Some(rest) = t.strip_prefix("verify_cmd") {
                let rest = rest.trim_start();
                let rest = rest.strip_prefix('=')?.trim();
                let v = rest.trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn emit_doc_gate_fail(
    verb: &str,
    evidence_dir: &std::path::Path,
    t0: std::time::Instant,
    reason: &str,
) -> clap_noun_verb::error::NounVerbError {
    eprintln!("AndonPull: {reason} — publish blocked (DOC_DRIFT_UNVERIFIED)");
    let mut complete_evt = ProcessEvent::completed(verb, t0, "FAIL");
    complete_evt.case_id = Some(crate::session::read_or_create_session_id(evidence_dir));
    if let Err(e) = crate::evidence::append_events(&[complete_evt], evidence_dir) {
        eprintln!("warning: evidence emission failed: {e}");
    }
    clap_noun_verb::error::NounVerbError::execution_error(reason.to_string())
}

/// Run the doc-drift gate. Returns `Ok(Some(changed_docs))` if the gate passed with
/// docs changed, `Ok(None)` if skipped (no doc changes), `Err` on refusal.
fn run_doc_gate(
    dir: &Path,
    verb: &str,
    evidence_dir: &std::path::Path,
    t0: std::time::Instant,
) -> clap_noun_verb::error::Result<Option<Vec<String>>> {
    let Some(tag) = last_release_tag_in(dir) else {
        return Ok(None);
    };
    let docs = docs_changed_since_in(dir, &tag);
    if docs.is_empty() {
        println!("  docs: unchanged since {tag} (gate skipped)");
        return Ok(None);
    }
    println!("  docs: {} file(s) changed since {tag}", docs.len());
    let Some(cmd) = docs_verify_cmd_in(dir) else {
        return Err(emit_doc_gate_fail(
            verb,
            evidence_dir,
            t0,
            "docs changed since last release but cicd.toml has no [docs] verify_cmd",
        ));
    };
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .current_dir(dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !status {
        return Err(emit_doc_gate_fail(
            verb,
            evidence_dir,
            t0,
            "[docs] verify_cmd failed — doc claims do not match code",
        ));
    }
    println!("  docs: PASS ({cmd})");
    Ok(Some(docs))
}

pub struct PublishRunVerb;
impl VerbCommand for PublishRunVerb {
    fn name(&self) -> &'static str {
        "run"
    }
    fn about(&self) -> &'static str {
        "Emit cicd.toml with current workspace state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("publish:run");

        let mut cicd = CicdToml::from_current_workspace();
        let target_gb = TargetScannerAdapter::total_size_gb(&cicd.workspace.target_dir);
        cicd.state.target_size_gb = (target_gb * 100.0).round() / 100.0;
        let git = GitStatusAdapter::query().unwrap_or_default();
        cicd.state.dirty = !git.dirty_files.is_empty() || !git.untracked_files.is_empty();
        let changed = ChangedFileDetector::changed_rs_files(&cicd.test.changed.base);
        cicd.state.changed_files = changed.len();
        cicd.state.changed_tests = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_test_file(f))
            .count();
        cicd.state.changed_trybuild_fixtures = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_trybuild_fixture(f))
            .count();
        cicd.workspace.toolchain = ToolchainDetector::active_toolchain();
        // ── Adjudicated publish gate (receipt doctor) ─────────────────────────
        // Gate law: publish_ready = true only after wpm receipt doctor accepts
        // the latest runtime receipt. wpm audit (XES) is secondary only.
        //
        // Gate outcomes:
        //   RECEIPT_DOCTOR:accepted   — oracle admitted the receipt; proceed.
        //   WARN:oracle_unavailable   — wpm not found; proceed with a warning.
        //   (bail!)                   — oracle refused receipt; publish is blocked.
        let publish_readiness = match crate::evidence::ReceiptDoctor::discover() {
            None => {
                eprintln!(
                    "warning: wasm4pm oracle unavailable — publish proceeding without \
                     receipt adjudication (BLOCKED:oracle_unavailable)"
                );
                "BLOCKED:oracle_unavailable"
            }
            Some(doctor) => {
                use crate::evidence::ReceiptDoctorVerdict;

                // Load accumulated events to build receipt.
                let jsonl_path = evidence_dir.join("events.jsonl");
                let events: Vec<crate::evidence::ProcessEvent> = {
                    let content = std::fs::read_to_string(&jsonl_path).unwrap_or_default();
                    content
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .filter_map(|l| serde_json::from_str(l).ok())
                        .collect()
                };

                let (_receipt_path, verdict) =
                    doctor.emit_and_adjudicate(&events, &evidence_dir, "cargo cicd publish run");
                match verdict {
                    ReceiptDoctorVerdict::Accepted { ref stdout_json } => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout_json) {
                            let state = v
                                .get("state")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Admitted");
                            println!("  receipt doctor: {} (wpm adjudicated)", state);
                        }
                        "RECEIPT_DOCTOR:accepted"
                    }
                    ReceiptDoctorVerdict::Refused { ref stdout, .. } => {
                        eprintln!("AndonPull: receipt doctor refused — publish blocked");
                        eprint!("{}", stdout);
                        let mut complete_evt = ProcessEvent::completed("publish:run", t0, "FAIL");
                        complete_evt.case_id = Some(case_id);
                        if let Err(e) = crate::evidence::append_events(
                            &[start_evt, complete_evt],
                            &evidence_dir,
                        ) {
                            eprintln!("warning: evidence emission failed: {}", e);
                        }
                        return Err(clap_noun_verb::error::NounVerbError::execution_error(
                            "wpm receipt doctor refused admission — publish blocked".to_string(),
                        ));
                    }
                    ReceiptDoctorVerdict::Blocked { ref reason } => {
                        eprintln!(
                            "warning: wasm4pm blocked — publish proceeding without \
                             receipt adjudication (reason: {})",
                            reason
                        );
                        "BLOCKED:oracle_unavailable"
                    }
                }
            }
        };
        // ── Doc-drift gate ─────────────────────────────────────────────────────
        // Docs changed since the last release tag must be re-verified before the
        // artifact (which embeds the README) ships.
        run_doc_gate(Path::new("."), "publish:run", &evidence_dir, t0)?;

        // Always write cicd.toml to capture current state, regardless of gate outcome.
        cicd.write_default()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("published cicd.toml");

        println!("  adjudication: {}", publish_readiness);

        // Run cargo publish --dry-run as final gate (after state capture).
        // This confirms the crate would actually publish.
        if publish_readiness == "RECEIPT_DOCTOR:accepted" {
            let dry_run_status = std::process::Command::new("cargo")
                .args(["publish", "--dry-run", "--allow-dirty"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !dry_run_status {
                eprintln!("AndonPull: cargo publish --dry-run failed — publish blocked");
                let mut complete_evt = ProcessEvent::completed("publish:run", t0, "FAIL");
                complete_evt.case_id = Some(case_id);
                if let Err(e) =
                    crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir)
                {
                    eprintln!("warning: evidence emission failed: {}", e);
                }
                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                    "cargo publish --dry-run failed — crate is not publishable".to_string(),
                ));
            }
            println!("  dry-run: PASS (cargo publish --dry-run succeeded)");
        }
        println!("  workspace:    {}", cicd.workspace.name);
        println!("  toolchain:    {}", cicd.workspace.toolchain);
        println!("  target:       {:.2} GB", cicd.state.target_size_gb);
        println!("  dirty:        {}", cicd.state.dirty);
        println!("  changed:      {}", cicd.state.changed_files);

        let mut complete_evt = ProcessEvent::completed("publish:run", t0, "PASS");
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct PublishCheckVerb;
impl VerbCommand for PublishCheckVerb {
    fn name(&self) -> &'static str {
        "check"
    }
    fn about(&self) -> &'static str {
        "Run cargo publish --dry-run to verify publish readiness"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("publish:check");
        start_evt.case_id = Some(case_id.clone());

        println!("publish check (dry-run)");
        println!("=======================");

        // Doc-drift gate: report-only here (check never hard-fails), same law as run.
        match run_doc_gate(Path::new("."), "publish:check", &evidence_dir, t0) {
            Ok(_) => {}
            Err(e) => println!("[FAIL] doc gate: {}", e),
        }

        let output = std::process::Command::new("cargo")
            .args(["publish", "--dry-run", "--allow-dirty"])
            .output();

        let verdict = match output {
            Ok(ref out) if out.status.success() => {
                println!("{}", String::from_utf8_lossy(&out.stdout));
                println!("[PASS] cargo publish --dry-run succeeded");
                "PASS"
            }
            Ok(ref out) => {
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                println!("[FAIL] cargo publish --dry-run failed");
                "FAIL"
            }
            Err(e) => {
                eprintln!("error running cargo publish --dry-run: {}", e);
                "FAIL"
            }
        };

        let mut complete_evt = ProcessEvent::completed("publish:check", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct PublishValidateVerb;
impl VerbCommand for PublishValidateVerb {
    fn name(&self) -> &'static str {
        "validate"
    }
    fn about(&self) -> &'static str {
        "Check all publish preconditions without running cargo"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("publish:validate");
        start_evt.case_id = Some(case_id.clone());

        println!("publish validate");
        println!("================");

        let mut overall = "PASS";

        // cicd.toml exists?
        let has_cicd = std::path::Path::new("cicd.toml").exists();
        println!(
            "[{}] cicd.toml exists",
            if has_cicd { "PASS" } else { "WARN" }
        );
        if !has_cicd {
            overall = "WARN";
        }

        // Cargo.toml has required metadata?
        let cargo_content = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
        let has_name = cargo_content
            .lines()
            .any(|l| l.trim().starts_with("name ="));
        let has_version = cargo_content
            .lines()
            .any(|l| l.trim().starts_with("version ="));
        let has_description = cargo_content
            .lines()
            .any(|l| l.trim().starts_with("description ="));
        let has_license = cargo_content
            .lines()
            .any(|l| l.trim().starts_with("license ="));

        println!(
            "[{}] Cargo.toml: name",
            if has_name { "PASS" } else { "FAIL" }
        );
        println!(
            "[{}] Cargo.toml: version",
            if has_version { "PASS" } else { "FAIL" }
        );
        println!(
            "[{}] Cargo.toml: description",
            if has_description { "PASS" } else { "WARN" }
        );
        println!(
            "[{}] Cargo.toml: license",
            if has_license { "PASS" } else { "WARN" }
        );

        if !has_name || !has_version {
            overall = "FAIL";
        } else if (!has_description || !has_license) && overall == "PASS" {
            overall = "WARN";
        }

        // README.md exists?
        let has_readme = std::path::Path::new("README.md").exists();
        println!(
            "[{}] README.md exists",
            if has_readme { "PASS" } else { "WARN" }
        );
        if !has_readme && overall == "PASS" {
            overall = "WARN";
        }

        // LICENSE file exists?
        let has_license_mit = std::path::Path::new("LICENSE-MIT").exists();
        let has_license_apache = std::path::Path::new("LICENSE-APACHE").exists();
        let has_any_license = has_license_mit || has_license_apache;
        println!(
            "[{}] LICENSE-MIT or LICENSE-APACHE exists",
            if has_any_license { "PASS" } else { "WARN" }
        );
        if !has_any_license && overall == "PASS" {
            overall = "WARN";
        }

        println!();
        println!("validate verdict: {}", overall);

        let mut complete_evt = ProcessEvent::completed("publish:validate", t0, overall);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

#[cfg(test)]
mod doc_gate_tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
        let st = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .expect("git available");
        assert!(st.success(), "git {args:?} failed");
    }

    fn init_repo() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().to_path_buf();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["config", "user.email", "t@t"]);
        git(&dir, &["config", "user.name", "t"]);
        std::fs::write(dir.join("README.md"), "v1\n").unwrap();
        std::fs::write(dir.join("src.rs"), "fn main() {}\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "init"]);
        (tmp, dir)
    }

    #[test]
    fn no_docs_changed_gate_is_skipped() {
        let (_tmp, dir) = init_repo();
        git(&dir, &["tag", "v1.0.0"]);
        // Change only code, no docs.
        std::fs::write(dir.join("src.rs"), "fn main() { /* v2 */ }\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "code only"]);

        assert_eq!(last_release_tag_in(&dir).as_deref(), Some("v1.0.0"));
        assert!(docs_changed_since_in(&dir, "v1.0.0").is_empty());
    }

    #[test]
    fn docs_changed_without_verify_cmd_refuses() {
        let (_tmp, dir) = init_repo();
        git(&dir, &["tag", "v1.0.0"]);
        std::fs::write(dir.join("README.md"), "v2\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "docs change"]);

        let docs = docs_changed_since_in(&dir, "v1.0.0");
        assert_eq!(docs, vec!["README.md".to_string()]);
        assert_eq!(docs_verify_cmd_in(&dir), None);
    }

    #[test]
    fn docs_changed_with_passing_verify_cmd_is_admitted() {
        let (_tmp, dir) = init_repo();
        git(&dir, &["tag", "v1.0.0"]);
        std::fs::write(dir.join("README.md"), "v2\n").unwrap();
        std::fs::write(
            dir.join("cicd.toml"),
            "[docs]\nverify_cmd = \"true\"\n",
        ).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "docs change with gate config"]);

        let gate = run_doc_gate(
            &dir,
            "publish:run",
            &dir.join("evidence"),
            std::time::Instant::now(),
        );
        assert!(gate.is_ok());
    }

    #[test]
    fn failing_verify_cmd_refuses() {
        let (_tmp, dir) = init_repo();
        git(&dir, &["tag", "v1.0.1"]);
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        std::fs::write(dir.join("docs/x.md"), "x").unwrap();
        std::fs::write(
            dir.join("cicd.toml"),
            "[docs]\nverify_cmd = \"false\"\n",
        ).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "failing gate config"]);

        let gate = run_doc_gate(
            &dir,
            "publish:run",
            &dir.join("evidence"),
            std::time::Instant::now(),
        );
        let err = gate.unwrap_err();
        assert!(err.to_string().contains("verify_cmd failed"));
    }
}
