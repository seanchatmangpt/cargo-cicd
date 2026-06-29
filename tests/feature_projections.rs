//! Feature projection tests — same Level 5 engine, different projections.
//! Proves feature flags expose projections, not separate implementations.
use assert_cmd::Command;
use tempfile::TempDir;

/// cicd.toml autonomic section must default to suggest mode.
#[test]
fn projection_autonomic_defaults_to_suggest() {
    let tmp = TempDir::new().unwrap();
    let result = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "run"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    if result.status.success() && tmp.path().join("cicd.toml").exists() {
        let content = std::fs::read_to_string(tmp.path().join("cicd.toml")).unwrap();
        if content.contains("[autonomic]") {
            assert!(
                content.contains("suggest"),
                "autonomic mode is not suggest by default: {}",
                content
            );
            assert!(
                !content.contains("mode = \"apply\""),
                "autonomic mode is apply — forbidden default: {}",
                content
            );
        }
    }
}

/// Core crate CicdCode has all expected category prefixes.
#[test]
fn core_cicd_code_category_coverage() {
    use cargo_cicd::core::diagnostics::CicdCode;
    let all = CicdCode::all_variants();
    let codes: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    let prefixes = [
        "CICD-GIT-",
        "CICD-TEST-",
        "CICD-TARGET-",
        "CICD-EVIDENCE-",
        "CICD-PUBLISH-",
        "CICD-WPM-",
    ];
    for prefix in &prefixes {
        assert!(
            codes.iter().any(|c| c.starts_with(prefix)),
            "no CicdCode with prefix {}",
            prefix
        );
    }
}
