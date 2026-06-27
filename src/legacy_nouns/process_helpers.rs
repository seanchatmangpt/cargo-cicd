use clap_noun_verb::error::NounVerbError;

pub fn run_cargo(args: &[&str]) -> &'static str {
    match std::process::Command::new("cargo").args(args).output() {
        Ok(ref out) if out.status.success() => {
            print!("{}", String::from_utf8_lossy(&out.stdout));
            "PASS"
        }
        Ok(ref out) => {
            eprint!("{}", String::from_utf8_lossy(&out.stderr));
            "FAIL"
        }
        Err(e) => {
            eprintln!("cargo error: {}", e);
            "FAIL"
        }
    }
}

pub fn run_git(args: &[&str]) -> Result<std::process::Output, NounVerbError> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| NounVerbError::execution_error(e.to_string()))
}

pub fn aggregate_verdict(verdicts: &[&str]) -> &'static str {
    if verdicts.contains(&"FAIL") {
        "FAIL"
    } else if verdicts.contains(&"WARN") {
        "WARN"
    } else {
        "PASS"
    }
}
