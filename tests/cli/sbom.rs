//! sbom noun — Chicago subprocess tests.
//!
//! `sbom show` is a real filesystem read over `sbom.json` in the working
//! directory. `sbom generate` shells out to `cargo cyclonedx`; where that tool
//! is unavailable the verb is fail-closed: a typed Unavailable verdict and a
//! non-zero exit — never a silent pass.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

fn run_cargo_cicd(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should build and run")
}

#[test]
fn sbom_show_with_seeded_sbom_json_exits_zero_and_reports_present() {
    let repo = TempDir::new().expect("temp repo dir");
    std::fs::write(
        repo.path().join("sbom.json"),
        "{\n  \"bomFormat\": \"CycloneDX\",\n  \"specVersion\": \"1.5\"\n}\n",
    )
    .expect("seed sbom.json");

    let out = run_cargo_cicd(repo.path(), &["sbom", "show"]);
    assert!(
        out.status.success(),
        "sbom show with a present sbom.json must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[OK] CycloneDX SBOM present"),
        "sbom show must report the SBOM as present; got:\n{stdout}"
    );
    assert!(
        stdout.contains("sbom.json"),
        "output must name the sbom.json path; got:\n{stdout}"
    );
    // The preview really reads the seeded file.
    assert!(
        stdout.contains("CycloneDX"),
        "preview must include content from the seeded sbom.json; got:\n{stdout}"
    );
}

#[test]
fn sbom_show_without_sbom_json_exits_zero_with_warn() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["sbom", "show"]);
    assert!(
        out.status.success(),
        "sbom show without sbom.json must still exit 0 (WARN, not fail); got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[WARN]"),
        "missing sbom.json must be reported as WARN; got:\n{stdout}"
    );
    assert!(
        stdout.contains("sbom generate"),
        "WARN must point the operator at `sbom generate`; got:\n{stdout}"
    );
}

#[test]
fn sbom_generate_without_cyclonedx_is_fail_closed_unavailable_nonzero() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(repo.path(), &["sbom", "generate"]);
    assert!(
        !out.status.success(),
        "new contract: sbom generate with the cyclonedx oracle unavailable must be \
         fail-closed non-zero; got exit {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("UNAVAILABLE"),
        "typed Unavailable verdict must be visible in output; got:\n{combined}"
    );
}
