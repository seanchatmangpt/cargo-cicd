use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A temporary workspace directory for fixture-based integration tests.
///
/// Each constructor creates a `TempDir` and populates it with the described
/// state. The `TempDir` is kept alive by `dir`; drop it to clean up.
pub struct FixtureWorkspace {
    /// The backing temporary directory. Kept alive for the lifetime of the fixture.
    #[allow(dead_code)]
    pub dir: TempDir,
    /// Absolute path to the workspace root (same as `dir.path()`).
    pub root: PathBuf,
}

impl FixtureWorkspace {
    /// Minimal, well-formed workspace: valid `Cargo.toml`, git-initialized and
    /// fully committed, no `target/` directory, no `cicd.toml`.
    ///
    /// Expected verdict: **pass**.
    pub fn clean() -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().to_path_buf();

        write_minimal_cargo_toml(&root);

        // git init + add + commit so the tree is clean
        // Errors are intentionally ignored: some CI environments may not have
        // git in PATH. Tests that depend on a clean git tree must skip if git
        // is unavailable.
        let _ = run_git(&root, &["init"]);
        let _ = run_git(&root, &["config", "user.email", "test@example.com"]);
        let _ = run_git(&root, &["config", "user.name", "Test"]);
        let _ = run_git(&root, &["add", "."]);
        let _ = run_git(&root, &["commit", "-m", "init"]);

        Self { dir, root }
    }

    /// Clean workspace plus one untracked file added after the initial commit.
    ///
    /// Expected verdict: **warn** (git dirty — untracked files).
    pub fn dirty() -> Self {
        let fixture = Self::clean();

        fs::write(fixture.root.join("untracked.txt"), "dirty\n").expect("write untracked file");

        fixture
    }

    /// Empty temporary directory with no `Cargo.toml`.
    ///
    /// Expected verdict: **refuse** (no manifest).
    pub fn missing_manifest() -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().to_path_buf();
        // Deliberately empty — no Cargo.toml, no git init.
        Self { dir, root }
    }

    /// Clean workspace plus a `rust-toolchain.toml` that declares a channel
    /// unlikely to be installed (simulates toolchain mismatch).
    ///
    /// Expected verdict: **warn** (toolchain mismatch).
    pub fn with_toolchain_mismatch() -> Self {
        let fixture = Self::clean();

        fs::write(
            fixture.root.join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.50.0\"\n",
        )
        .expect("write rust-toolchain.toml");

        // Re-stage the new file so the tree stays clean for git checks.
        let _ = run_git(&fixture.root, &["add", "rust-toolchain.toml"]);
        let _ = run_git(&fixture.root, &["commit", "-m", "add toolchain"]);

        fixture
    }

    /// Clean workspace plus a `target/` directory containing a 1 MB fake
    /// artifact, simulating a target dir that exceeds size thresholds.
    ///
    /// Expected verdict: **warn** (target over limit).
    pub fn with_target_over_limit() -> Self {
        let fixture = Self::clean();

        let target = fixture.root.join("target").join("debug");
        fs::create_dir_all(&target).expect("create target/debug");

        // Write a 1 MB placeholder — enough to trigger size-limit warnings in
        // tests that set a low threshold; not so large as to slow CI.
        let one_mb = vec![0u8; 1_048_576];
        fs::write(target.join("placeholder.bin"), &one_mb).expect("write placeholder binary");

        fixture
    }

    /// Clean workspace plus a `cicd.toml` that contains invalid TOML syntax.
    ///
    /// Expected verdict: **fail** or **refuse** (corrupted config cannot be parsed).
    pub fn with_corrupted_cicd_toml() -> Self {
        let fixture = Self::clean();

        fs::write(fixture.root.join("cicd.toml"), "not valid toml [[[\n")
            .expect("write corrupted cicd.toml");

        fixture
    }

    /// Clean workspace plus a `cicd.toml` that declares `dirty = false` while
    /// the workspace actually has untracked files. Simulates stale cached state.
    ///
    /// Expected verdict: **warn** (stale cicd.toml state detected).
    pub fn with_stale_cicd_toml() -> Self {
        let fixture = Self::clean();

        // Write a plausible-but-stale cicd.toml.
        fs::write(fixture.root.join("cicd.toml"), "[state]\ndirty = false\n")
            .expect("write stale cicd.toml");

        // Now make the workspace dirty so the cached state is wrong.
        fs::write(fixture.root.join("untracked.txt"), "dirty\n").expect("write untracked file");

        fixture
    }

    /// Clean workspace plus a `tests/ui/` directory containing one `.rs`
    /// trybuild fixture file (changed) alongside ten placeholder fixtures
    /// (unchanged). Tests that changed-only trybuild selection picks exactly
    /// the one changed fixture.
    ///
    /// Expected verdict: **pass** (only changed fixture is run).
    pub fn with_changed_trybuild_fixture() -> Self {
        let fixture = Self::clean();

        let ui_dir = fixture.root.join("tests").join("ui");
        fs::create_dir_all(&ui_dir).expect("create tests/ui");

        // Ten pre-existing fixtures committed as "unchanged".
        for i in 0..10 {
            fs::write(
                ui_dir.join(format!("existing_{i:02}.rs")),
                "// existing fixture\n",
            )
            .expect("write existing fixture");
        }

        let _ = run_git(&fixture.root, &["add", "tests/"]);
        let _ = run_git(&fixture.root, &["commit", "-m", "add existing fixtures"]);

        // One new fixture that represents the "changed" file not yet committed.
        fs::write(
            ui_dir.join("changed_fixture.rs"),
            "// new changed fixture\n",
        )
        .expect("write changed fixture");

        fixture
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Write a minimal valid `Cargo.toml` into `root`.
fn write_minimal_cargo_toml(root: &std::path::Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
}

/// Run a git sub-command in `cwd`. Returns `Ok(())` on exit-status 0, or
/// `Err(String)` with the combined stderr on failure.
///
/// Callers that can tolerate a missing git binary should ignore the error with
/// `let _ = run_git(...)`.
fn run_git(cwd: &std::path::Path, args: &[&str]) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}
