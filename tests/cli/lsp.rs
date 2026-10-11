//! lsp noun — Chicago subprocess tests.
//!
//! The lsp noun is entirely behind `#[cfg(feature = "anti-llm-cheat")]`
//! (`src/nouns/lsp.rs`), so these tests are compiled only when that feature is
//! enabled; with default features the file contributes no tests. The test body
//! runs the real binary — no mocks.

#[cfg(feature = "anti-llm-cheat")]
mod anti_llm_cheat {
    use std::process::Command;

    use tempfile::TempDir;

    fn bin() -> std::ffi::OsString {
        let bin_name =
            std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
        std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
            panic!(
                "CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?"
            )
        })
    }

    #[test]
    fn lsp_check_help_is_registered() {
        let repo = TempDir::new().expect("temp repo dir");
        let out = Command::new(bin())
            .args(["lsp", "check", "--help"])
            .current_dir(repo.path())
            .output()
            .expect("binary should build and run");
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.status.success() || combined.contains("Usage"),
            "lsp check must be a registered verb when anti-llm-cheat is on; got {:?}:\n{combined}",
            out.status.code()
        );
    }
}
