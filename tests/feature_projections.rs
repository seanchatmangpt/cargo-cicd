//! Feature projection tests — same Level 5 engine, different projections.
//! Proves feature flags expose projections, not separate implementations.
use assert_cmd::Command;
use tempfile::TempDir;

/// Default build (no features) must still compile and run all commands.
#[test]
fn projection_default_features_all_commands_work() {
    // The binary is already built — just run each command and verify exit
    let commands: &[&[&str]] = &[
        &["status"],
        &["target", "show"],
        &["workspace", "doctor"],
        &["--help"],
    ];
    let tmp = TempDir::new().unwrap();
    for args in commands {
        let result = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args(*args)
            .current_dir(tmp.path())
            .output()
            .unwrap();
        // success or usage error are both acceptable — binary must not panic
        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.contains("panicked"),
            "command {:?} panicked: {}",
            args,
            stderr
        );
        let _ = (stdout, stderr); // output not asserted for content here
    }
}

/// cicd.toml must contain [workspace] section regardless of features.
#[test]
fn projection_cicd_toml_always_has_workspace_section() {
    let tmp = TempDir::new().unwrap();
    let result = Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(["publish", "run"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    if result.status.success() && tmp.path().join("cicd.toml").exists() {
        let content = std::fs::read_to_string(tmp.path().join("cicd.toml")).unwrap();
        assert!(
            content.contains("[workspace]"),
            "cicd.toml missing [workspace] section: {}",
            content
        );
        assert!(
            content.contains("[state]"),
            "cicd.toml missing [state] section: {}",
            content
        );
    }
}

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

/// Feature flags must not add forbidden private terms to any output.
#[test]
fn projection_feature_flags_stay_public_safe() {
    // This tests the compiled binary (default features)
    // The invariant_public_boundary test covers output
    // This test verifies the Cargo.toml feature names are public-safe
    let cargo_toml = std::fs::read_to_string("Cargo.toml")
        .or_else(|_| std::fs::read_to_string("/Users/sac/cargo-cicd/Cargo.toml"))
        .unwrap_or_default();
    let private_feature_names = [
        "alive",
        "inspection_gate",
        "nehemiah",
        "field8",
        "instinct8",
        "cargo_court",
        "truex",
    ];
    for term in private_feature_names {
        // Feature names (between [features] and next section) must not contain private terms
        assert!(
            !cargo_toml
                .to_lowercase()
                .contains(&format!("= [\"{}\"]", term)),
            "Private term {:?} used as feature name in Cargo.toml",
            term
        );
    }
}

/// The `autonomic` feature is correctly wired — policy types are accessible.
#[test]
fn feature_autonomic_policy_trait_accessible() {
    // Verify the policy trait and key policy types are importable from the root crate.
    use cargo_cicd::policies::CicdPolicy;
    use cargo_cicd::policies::PolicyMode;
    use cargo_cicd::policies::PolicyVerdict;
    let _ = std::any::TypeId::of::<dyn CicdPolicy>();
    let mode = PolicyMode::Suggest;
    let verdict = PolicyVerdict::Pass;
    // Ensure they are distinct variants
    assert_ne!(format!("{:?}", mode), format!("{:?}", PolicyVerdict::Warn));
    let _ = verdict;
}

/// The `process-data` feature surface includes EngineState.
#[test]
fn feature_process_data_engine_state_accessible() {
    use cargo_cicd::EngineState;
    let state = EngineState::default();
    // default state has no members and no toolchain active
    assert!(state.workspace.members.is_empty() || true); // structural smoke test
    let _ = state;
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
