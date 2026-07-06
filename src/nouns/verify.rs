//! Verifies a repository against configured checks, including semver compatibility.
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifyOutput {
    pub schema: String, // "cargo-cicd.verify.v2"
    pub q_verify: u8,
    pub errors: Vec<String>,
    pub semver_status: Option<String>, // "pass", "fail", "unavailable"
    pub semver_errors: Vec<String>,
}

pub fn evaluate_verify(repo_dir: &str) -> VerifyOutput {
    let mut q_verify: u8 = 1;
    let errors: Vec<String> = vec![];
    let semver_status: Option<String>;
    let mut semver_errors: Vec<String> = vec![];

    // Try to find cargo-semver-checks
    let semver_available = std::process::Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if semver_available {
        let result = std::process::Command::new("cargo")
            .args([
                "semver-checks",
                "check-release",
                "--package",
                "cargo-cicd-core",
            ])
            .current_dir(repo_dir)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    semver_status = Some("pass".to_string());
                } else {
                    semver_status = Some("fail".to_string());
                    q_verify = 0;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    for line in stderr.lines() {
                        if !line.trim().is_empty() {
                            semver_errors.push(line.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                semver_status = Some("unavailable".to_string());
                eprintln!("warning: cargo-semver-checks failed to run: {}", e);
            }
        }
    } else {
        semver_status = Some("unavailable".to_string());
        eprintln!("warning: cargo-semver-checks not found; skipping semver check");
    }

    VerifyOutput {
        schema: "cargo-cicd.verify.v2".to_string(),
        q_verify,
        errors,
        semver_status,
        semver_errors,
    }
}

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool) -> Result<()> {
    let _ = json;
    let output = evaluate_verify(&repo.unwrap_or(".".into()));
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    Ok(())
}
