//! Verifies a repository against configured checks, including semver compatibility.
use clap_noun_verb::error::NounVerbError;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifyOutput {
    pub schema: String, // "cargo-cicd.verify.v2"
    pub q_verify: u8,
    pub errors: Vec<String>,
    pub semver_status: Option<String>, // "pass", "fail"
    pub semver_errors: Vec<String>,
}

/// Typed Unavailable error: the verification tool itself is missing. This is
/// distinct from "the tool ran and found issues" and must exit non-zero.
fn semver_unavailable() -> NounVerbError {
    NounVerbError::execution_error(
        "TOOL_UNAVAILABLE: cargo-semver-checks not found \
         (run `cargo install cargo-semver-checks`)",
    )
}

pub fn evaluate_verify(repo_dir: &str, package: &str) -> std::result::Result<VerifyOutput, NounVerbError> {
    let errors: Vec<String> = vec![];
    let mut q_verify: u8 = 1;
    let semver_status: String;
    let mut semver_errors: Vec<String> = vec![];

    // Try to find cargo-semver-checks — missing tool is a typed, fail-closed
    // Unavailable error, never a silent skip.
    let semver_available = std::process::Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !semver_available {
        return Err(semver_unavailable());
    }

    let result = std::process::Command::new("cargo")
        .args([
            "semver-checks",
            "check-release",
            "--package",
            package,
        ])
        .current_dir(repo_dir)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                semver_status = "pass".to_string();
            } else {
                semver_status = "fail".to_string();
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
            return Err(NounVerbError::execution_error(format!(
                "cargo-semver-checks failed to run: {}",
                e
            )));
        }
    }

    Ok(VerifyOutput {
        schema: "cargo-cicd.verify.v2".to_string(),
        q_verify,
        errors,
        semver_status: Some(semver_status),
        semver_errors,
    })
}

/// Thin verb: delegates to `verify_repo_impl` (CLI layer stays simple).
#[verb("repo")]
pub fn cmd_repo(
    repo: Option<String>,
    json: bool,
    package: Option<String>,
) -> Result<()> {
    verify_repo_impl(repo, json, package)
}

fn verify_repo_impl(
    repo: Option<String>,
    json: bool,
    package: Option<String>,
) -> Result<()> {
    let package = package.unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());
    let output = match evaluate_verify(&repo.unwrap_or(".".into()), &package) {
        Ok(o) => o,
        // Typed Unavailable / spawn failures propagate as non-zero exits.
        Err(e) => return Err(e),
    };
    render_and_adjudicate(&output, &package, json)
}

fn render_and_adjudicate(output: &VerifyOutput, package: &str, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(output).unwrap());
    } else {
        println!("verify (schema {})", output.schema);
        println!("  package: {}", package);
        println!("  semver:  {}", output.semver_status.as_deref().unwrap_or("unknown"));
        println!("  q_verify: {}", output.q_verify);
        for e in &output.semver_errors {
            println!("  error: {}", e);
        }
    }
    if output.q_verify == 0 {
        // Fail-closed: a FAIL verdict exits non-zero even though the report printed.
        return Err(NounVerbError::execution_error(
            "verify: semver checks failed".to_string(),
        ));
    }
    Ok(())
}
